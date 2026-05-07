use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use eventsource_client::{Client as _, ClientBuilder as SseClientBuilder, ReconnectOptionsBuilder, SSE};
use futures::TryStreamExt;
use launchdarkly_sdk_transport::HyperTransport;
use reqwest::Client;
use tokio::sync::{oneshot, Mutex};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::error::AppError;
use crate::mcp::types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

/// Pending request senders, keyed by stringified JSON-RPC id.
type PendingMap = Arc<Mutex<HashMap<String, oneshot::Sender<JsonRpcResponse>>>>;

/// HTTP transport for remote MCP servers.
///
/// Supports two modes:
/// - **Streamable HTTP**: POST JSON-RPC to a single endpoint (preferred).
///   The server returns the response directly in the HTTP response body.
/// - **Legacy SSE**: GET an SSE endpoint that returns an `endpoint` event,
///   then POST to that URL. Responses arrive on the SSE stream, not in
///   the POST response body. Auto-reconnects via `eventsource-client`;
///   the post URL is refreshed whenever the server sends a new `endpoint`
///   event after a reconnect.
pub struct HttpTransport {
    next_id: AtomicU64,
    client: Client,
    /// The URL to POST JSON-RPC requests to. Wrapped in a Mutex so the
    /// legacy-SSE reader task can update it on reconnect; for streamable
    /// HTTP this never changes after construction.
    post_url: Arc<Mutex<String>>,
    /// Extra headers to include on every request (e.g. Authorization).
    headers: HashMap<String, String>,
    /// Session ID returned by the server, sent on subsequent requests.
    session_id: Arc<Mutex<Option<String>>>,
    /// OAuth access token, injected as Bearer header when present.
    access_token: Arc<Mutex<Option<String>>>,
    /// Whether this transport uses legacy SSE mode.
    legacy_sse: bool,
    /// For legacy SSE: pending request senders keyed by JSON-RPC id.
    pending: PendingMap,
    /// Background SSE reader task handle (legacy SSE only).
    sse_reader: Option<JoinHandle<()>>,
}

impl HttpTransport {
    /// Connect to a remote MCP server via HTTP.
    ///
    /// If the URL path ends with `/sse`, connects in legacy SSE mode (GET for endpoint
    /// discovery, then POST to discovered URL). Otherwise, assumes streamable HTTP
    /// and POSTs directly to the given URL.
    pub async fn connect(
        url: &str,
        headers: HashMap<String, String>,
        access_token: Option<String>,
    ) -> Result<Self, AppError> {
        let client = Client::new();
        let token = Arc::new(Mutex::new(access_token));

        if url.ends_with("/sse") {
            info!("URL ends with /sse, using legacy SSE transport for {url}");
            return Self::connect_legacy_sse(url, headers, client, token).await;
        }

        info!("Using streamable HTTP transport for {url}");

        Ok(Self {
            next_id: AtomicU64::new(1),
            client,
            post_url: Arc::new(Mutex::new(url.to_string())),
            headers,
            session_id: Arc::new(Mutex::new(None)),
            access_token: token,
            legacy_sse: false,
            pending: Arc::new(Mutex::new(HashMap::new())),
            sse_reader: None,
        })
    }

    /// Legacy SSE connection. Opens an `eventsource-client` stream, waits for
    /// the first `endpoint` event to discover the POST URL, then spawns a
    /// persistent reader that dispatches responses and refreshes the POST
    /// URL on reconnect. The library handles reconnection with exponential
    /// backoff, so a transient network drop (e.g. macOS sleep/wake) recovers
    /// without user intervention.
    async fn connect_legacy_sse(
        url: &str,
        headers: HashMap<String, String>,
        client: Client,
        access_token: Arc<Mutex<Option<String>>>,
    ) -> Result<Self, AppError> {
        let mut builder = SseClientBuilder::for_url(url)
            .map_err(|e| AppError::Transport(format!("Invalid SSE URL: {e}")))?
            .reconnect(
                ReconnectOptionsBuilder::new(true)
                    .retry_initial(true)
                    .delay(Duration::from_secs(1))
                    .backoff_factor(2)
                    .delay_max(Duration::from_secs(60))
                    .build(),
            );

        for (k, v) in &headers {
            builder = builder
                .header(k.as_str(), v.as_str())
                .map_err(|e| AppError::Transport(format!("Invalid header {k}: {e}")))?;
        }

        {
            let tok = access_token.lock().await;
            if let Some(ref token) = *tok {
                builder = builder
                    .header("Authorization", &format!("Bearer {token}"))
                    .map_err(|e| AppError::Transport(format!("Invalid auth header: {e}")))?;
            }
        }

        let transport = HyperTransport::builder()
            .connect_timeout(Duration::from_secs(10))
            .read_timeout(Duration::from_secs(60))
            .build_https()
            .map_err(|e| AppError::Transport(format!("Failed to build SSE transport: {e}")))?;

        let es_client = builder.build_with_transport(transport);
        let mut stream = es_client.stream();
        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));

        let initial_endpoint = wait_for_initial_endpoint(&mut stream, url).await?;
        info!("Legacy SSE: discovered POST endpoint {initial_endpoint}");

        let post_url = Arc::new(Mutex::new(initial_endpoint));
        let post_url_clone = post_url.clone();
        let pending_clone = pending.clone();
        let url_str = url.to_string();

        let sse_reader = tokio::spawn(async move {
            loop {
                match stream.try_next().await {
                    Ok(Some(SSE::Connected(_))) => {
                        info!("Legacy SSE (re)connected to {url_str}");
                        drain_pending_with_error(
                            &pending_clone,
                            "SSE stream reconnected; request lost",
                        )
                        .await;
                    }
                    Ok(Some(SSE::Comment(_))) => {}
                    Ok(Some(SSE::Event(event))) => {
                        if event.event_type == "endpoint" {
                            match resolve_endpoint(event.data.trim(), &url_str) {
                                Ok(new_url) => {
                                    let mut p = post_url_clone.lock().await;
                                    if *p != new_url {
                                        info!("Legacy SSE: endpoint updated to {new_url}");
                                        *p = new_url;
                                    }
                                }
                                Err(e) => warn!("Legacy SSE: failed to resolve endpoint: {e}"),
                            }
                        } else if event.event_type.is_empty() || event.event_type == "message" {
                            match serde_json::from_str::<JsonRpcResponse>(&event.data) {
                                Ok(rpc) => dispatch_response(&pending_clone, rpc).await,
                                Err(e) => warn!(
                                    "Legacy SSE: failed to parse JSON-RPC: {e} — raw: {}",
                                    event.data
                                ),
                            }
                        } else {
                            debug!("Legacy SSE: ignoring event type={}", event.event_type);
                        }
                    }
                    Ok(None) => {
                        info!("Legacy SSE stream ended — reader task exiting");
                        break;
                    }
                    Err(e) => {
                        warn!("Legacy SSE stream error (auto-reconnect pending): {e}");
                    }
                }
            }
            drain_pending_with_error(&pending_clone, "SSE stream closed").await;
        });

        Ok(Self {
            next_id: AtomicU64::new(1),
            client,
            post_url,
            headers,
            session_id: Arc::new(Mutex::new(None)),
            access_token,
            legacy_sse: true,
            pending,
            sse_reader: Some(sse_reader),
        })
    }

    /// Send a JSON-RPC request and return the response.
    pub async fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<JsonRpcResponse, AppError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(id.into())),
            method: method.to_string(),
            params,
        };

        let body = serde_json::to_value(&request)
            .map_err(|e| AppError::Transport(format!("Failed to serialize request: {e}")))?;

        let post_url = self.post_url.lock().await.clone();
        debug!("HTTP send_request id={id} method={method} -> {post_url}");

        if self.legacy_sse {
            return self
                .send_request_legacy_sse(id, &body, method, &post_url)
                .await;
        }

        let mut req = self
            .client
            .post(&post_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream");

        for (k, v) in &self.headers {
            req = req.header(k.as_str(), v.as_str());
        }

        {
            let tok = self.access_token.lock().await;
            if let Some(ref token) = *tok {
                req = req.header("Authorization", format!("Bearer {token}"));
            }
        }

        {
            let sid = self.session_id.lock().await;
            if let Some(ref s) = *sid {
                req = req.header("Mcp-Session-Id", s.as_str());
            }
        }

        let response = req
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Transport(format!("HTTP request failed: {e}")))?;

        if let Some(new_sid) = response
            .headers()
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
        {
            let mut sid = self.session_id.lock().await;
            *sid = Some(new_sid.to_string());
        }

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AppError::AuthRequired(post_url));
        }

        if !response.status().is_success() {
            return Err(AppError::Transport(format!(
                "HTTP request for {method} returned status {}",
                response.status()
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let response_text = response
            .text()
            .await
            .map_err(|e| AppError::Transport(format!("Failed to read HTTP response: {e}")))?;

        let json_text = if content_type.contains("text/event-stream") {
            extract_json_from_sse(&response_text)?
        } else {
            response_text
        };

        let rpc_response: JsonRpcResponse = serde_json::from_str(&json_text).map_err(|e| {
            AppError::Protocol(format!(
                "Failed to parse JSON-RPC response: {e} — raw: {json_text}"
            ))
        })?;

        if let Some(err) = &rpc_response.error {
            return Err(AppError::Protocol(format!("{}: {}", err.code, err.message)));
        }

        Ok(rpc_response)
    }

    /// Legacy SSE: POST the request and wait for the response on the SSE stream.
    async fn send_request_legacy_sse(
        &self,
        id: u64,
        body: &serde_json::Value,
        method: &str,
        post_url: &str,
    ) -> Result<JsonRpcResponse, AppError> {
        let id_str = id.to_string();

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id_str.clone(), tx);
        }

        let mut req = self
            .client
            .post(post_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream");

        for (k, v) in &self.headers {
            req = req.header(k.as_str(), v.as_str());
        }

        {
            let tok = self.access_token.lock().await;
            if let Some(ref token) = *tok {
                req = req.header("Authorization", format!("Bearer {token}"));
            }
        }

        {
            let sid = self.session_id.lock().await;
            if let Some(ref s) = *sid {
                req = req.header("Mcp-Session-Id", s.as_str());
            }
        }

        let response = req.json(body).send().await.map_err(|e| {
            let pending = self.pending.clone();
            let id_str = id_str.clone();
            tokio::spawn(async move {
                pending.lock().await.remove(&id_str);
            });
            AppError::Transport(format!("HTTP request failed: {e}"))
        })?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            self.pending.lock().await.remove(&id_str);
            return Err(AppError::AuthRequired(post_url.to_string()));
        }

        if !response.status().is_success() {
            self.pending.lock().await.remove(&id_str);
            return Err(AppError::Transport(format!(
                "HTTP request for {method} returned status {}",
                response.status()
            )));
        }

        let timeout = Duration::from_secs(60);
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(rpc_response)) => {
                if let Some(err) = &rpc_response.error {
                    return Err(AppError::Protocol(format!("{}: {}", err.code, err.message)));
                }
                Ok(rpc_response)
            }
            Ok(Err(_)) => Err(AppError::Transport(
                "SSE stream closed while waiting for response".to_string(),
            )),
            Err(_) => {
                self.pending.lock().await.remove(&id_str);
                Err(AppError::Transport(format!(
                    "Timeout waiting for SSE response to {method} (id={id})"
                )))
            }
        }
    }

    /// Send a JSON-RPC notification (no response expected).
    pub async fn send_notification(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<(), AppError> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: method.to_string(),
            params,
        };

        let body = serde_json::to_value(&request)
            .map_err(|e| AppError::Transport(format!("Failed to serialize notification: {e}")))?;

        let post_url = self.post_url.lock().await.clone();
        debug!("HTTP send_notification method={method} -> {post_url}");

        let mut req = self
            .client
            .post(&post_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream");

        for (k, v) in &self.headers {
            req = req.header(k.as_str(), v.as_str());
        }

        {
            let tok = self.access_token.lock().await;
            if let Some(ref token) = *tok {
                req = req.header("Authorization", format!("Bearer {token}"));
            }
        }

        {
            let sid = self.session_id.lock().await;
            if let Some(ref s) = *sid {
                req = req.header("Mcp-Session-Id", s.as_str());
            }
        }

        let response = req
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Transport(format!("HTTP notification failed: {e}")))?;

        if let Some(new_sid) = response
            .headers()
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
        {
            let mut sid = self.session_id.lock().await;
            *sid = Some(new_sid.to_string());
        }

        if !response.status().is_success() {
            warn!(
                "HTTP notification {method} returned status {}",
                response.status()
            );
        }

        Ok(())
    }
}

impl Drop for HttpTransport {
    fn drop(&mut self) {
        if let Some(ref handle) = self.sse_reader {
            handle.abort();
        }
    }
}

/// Consume the SSE stream until the first `endpoint` event arrives, returning
/// the resolved POST URL. Bounded by an outer timeout so a totally-down server
/// fails fast on initial connect even though the library would keep retrying.
async fn wait_for_initial_endpoint(
    stream: &mut (impl futures::stream::TryStream<Ok = SSE, Error = eventsource_client::Error> + Unpin),
    base_url: &str,
) -> Result<String, AppError> {
    let timeout = Duration::from_secs(15);
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        match tokio::time::timeout_at(deadline, stream.try_next()).await {
            Ok(Ok(Some(SSE::Event(event)))) if event.event_type == "endpoint" => {
                return resolve_endpoint(event.data.trim(), base_url);
            }
            Ok(Ok(Some(_))) => continue,
            Ok(Ok(None)) => {
                return Err(AppError::Transport(
                    "SSE stream ended before endpoint event".to_string(),
                ));
            }
            Ok(Err(e)) => {
                return Err(AppError::Transport(format!("SSE error: {e}")));
            }
            Err(_) => {
                return Err(AppError::Transport(
                    "Timed out waiting for endpoint event".to_string(),
                ));
            }
        }
    }
}

/// Resolve an `endpoint` event's data — which may be absolute or relative —
/// against the original SSE URL's origin.
fn resolve_endpoint(endpoint: &str, base_url: &str) -> Result<String, AppError> {
    if endpoint.is_empty() {
        return Err(AppError::Transport("Empty endpoint event data".to_string()));
    }
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        return Ok(endpoint.to_string());
    }
    let origin_end = base_url
        .find("://")
        .map(|i| {
            base_url[i + 3..]
                .find('/')
                .map(|j| i + 3 + j)
                .unwrap_or(base_url.len())
        })
        .unwrap_or(base_url.len());
    let origin = &base_url[..origin_end];
    let path = if endpoint.starts_with('/') {
        endpoint.to_string()
    } else {
        format!("/{endpoint}")
    };
    Ok(format!("{origin}{path}"))
}

async fn dispatch_response(pending: &PendingMap, rpc: JsonRpcResponse) {
    let id_str = match &rpc.id {
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => {
            debug!("Legacy SSE: response with no/unexpected id");
            return;
        }
    };
    let mut map = pending.lock().await;
    if let Some(tx) = map.remove(&id_str) {
        debug!("Legacy SSE: dispatching response for id={id_str}");
        let _ = tx.send(rpc);
    } else {
        debug!("Legacy SSE: no waiter for id={id_str}");
    }
}

async fn drain_pending_with_error(pending: &PendingMap, msg: &str) {
    let mut map = pending.lock().await;
    for (id, tx) in map.drain() {
        let _ = tx.send(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::String(id)),
            result: None,
            error: Some(JsonRpcError {
                code: -1,
                message: msg.to_string(),
                data: None,
            }),
        });
    }
}

/// Extract JSON-RPC response data from an SSE response body (streamable HTTP mode).
/// SSE responses contain `data:` lines with JSON fragments.
fn extract_json_from_sse(body: &str) -> Result<String, AppError> {
    let mut json_parts = Vec::new();
    let mut current_event = String::new();

    for line in body.lines() {
        if let Some(event_type) = line.strip_prefix("event:") {
            current_event = event_type.trim().to_string();
        } else if let Some(data) = line.strip_prefix("data:") {
            if current_event.is_empty() || current_event == "message" {
                json_parts.push(data.trim().to_string());
            }
        }
    }

    if json_parts.is_empty() {
        return Err(AppError::Transport(
            "No JSON data found in SSE response".to_string(),
        ));
    }

    Ok(json_parts.last().expect("non-empty after guard").clone())
}

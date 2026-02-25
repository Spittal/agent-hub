//! Shared helpers for MCP Streamable HTTP endpoints.
//!
//! Used by both the per-server proxy (`proxy.rs`) and discovery endpoints
//! (`discovery.rs`) to enforce consistent protocol behaviour: version
//! negotiation, session management, origin validation, and response formatting.

use axum::http::{HeaderMap, HeaderValue, StatusCode};
use serde_json::Value;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Protocol version negotiation
// ---------------------------------------------------------------------------

/// MCP protocol versions this proxy advertises, newest first.
pub(crate) const SUPPORTED_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

/// If the client requests a version we support, echo it back.
/// Otherwise fall back to our latest version.
pub(crate) fn negotiate_version(client_version: &str) -> &'static str {
    SUPPORTED_VERSIONS
        .iter()
        .find(|&&v| v == client_version)
        .copied()
        .unwrap_or(SUPPORTED_VERSIONS[0])
}

// ---------------------------------------------------------------------------
// Session ID generation
// ---------------------------------------------------------------------------

/// Generate a new random session identifier (UUID v4).
pub(crate) fn new_session_id() -> String {
    Uuid::new_v4().to_string()
}

// ---------------------------------------------------------------------------
// Origin validation
// ---------------------------------------------------------------------------

/// Returns `true` if `origin` is a localhost variant we allow through the
/// CORS gate (plain localhost, 127.0.0.1, or [::1] — with or without a port).
fn is_localhost_origin(origin: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "http://localhost",
        "http://127.0.0.1",
        "http://[::1]",
    ];
    for prefix in PREFIXES {
        if origin == *prefix {
            return true;
        }
        // Allow any port: "http://localhost:3000", "http://127.0.0.1:8080", etc.
        if let Some(rest) = origin.strip_prefix(prefix) {
            if rest.starts_with(':') {
                return true;
            }
        }
    }
    false
}

/// Validate the `Origin` header according to MCP Streamable HTTP rules.
///
/// - No Origin header → allow (non-browser client).
/// - Localhost variant → allow.
/// - `tauri://` or `https://tauri.` scheme → allow (Tauri webview).
/// - Anything else → 403 Forbidden.
pub(crate) fn validate_origin(headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
    let origin = match headers.get("origin") {
        Some(v) => v.to_str().unwrap_or(""),
        None => return Ok(()), // non-browser client
    };

    if origin.is_empty() {
        return Ok(());
    }

    if is_localhost_origin(origin) {
        return Ok(());
    }

    if origin.starts_with("tauri://") || origin.starts_with("https://tauri.") {
        return Ok(());
    }

    Err((
        StatusCode::FORBIDDEN,
        format!("Origin not allowed: {origin}"),
    ))
}

// ---------------------------------------------------------------------------
// Accept header parsing
// ---------------------------------------------------------------------------

/// Returns `true` when the client's `Accept` header includes
/// `text/event-stream`, indicating it can consume an SSE response.
pub(crate) fn client_accepts_sse(headers: &HeaderMap) -> bool {
    headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|accept| accept.contains("text/event-stream"))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Response builders
// ---------------------------------------------------------------------------

/// Attach the `Mcp-Session-Id` header when a session ID is present.
fn attach_session_id(headers: &mut HeaderMap, session_id: Option<&str>) {
    if let Some(id) = session_id {
        if let Ok(val) = HeaderValue::from_str(id) {
            headers.insert("mcp-session-id", val);
        }
    }
}

/// Build a JSON response (`application/json`).
pub(crate) fn json_response(
    body: &Value,
    session_id: Option<&str>,
) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert(
        "content-type",
        HeaderValue::from_static("application/json"),
    );
    attach_session_id(&mut headers, session_id);
    (StatusCode::OK, headers, body.to_string())
}

/// Build an SSE response (`text/event-stream`).
///
/// The body is formatted as a single `message` event:
/// ```text
/// event: message
/// data: {"jsonrpc":"2.0",...}
///
/// ```
pub(crate) fn sse_response(
    body: &Value,
    session_id: Option<&str>,
) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert(
        "content-type",
        HeaderValue::from_static("text/event-stream"),
    );
    headers.insert(
        "cache-control",
        HeaderValue::from_static("no-cache"),
    );
    attach_session_id(&mut headers, session_id);

    let sse_body = format!("event: message\ndata: {}\n\n", body);
    (StatusCode::OK, headers, sse_body)
}

/// Build either a JSON or SSE response depending on `use_sse`.
pub(crate) fn mcp_response(
    body: &Value,
    session_id: Option<&str>,
    use_sse: bool,
) -> (StatusCode, HeaderMap, String) {
    if use_sse {
        sse_response(body, session_id)
    } else {
        json_response(body, session_id)
    }
}

/// Build a 202 Accepted response with an empty body.
pub(crate) fn accepted_response(
    session_id: Option<&str>,
) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    attach_session_id(&mut headers, session_id);
    (StatusCode::ACCEPTED, headers, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- negotiate_version --------------------------------------------------

    #[test]
    fn negotiate_known_version() {
        assert_eq!(negotiate_version("2025-03-26"), "2025-03-26");
        assert_eq!(negotiate_version("2024-11-05"), "2024-11-05");
    }

    #[test]
    fn negotiate_unknown_version_returns_latest() {
        assert_eq!(negotiate_version("2019-01-01"), SUPPORTED_VERSIONS[0]);
        assert_eq!(negotiate_version(""), SUPPORTED_VERSIONS[0]);
    }

    // -- validate_origin ----------------------------------------------------

    #[test]
    fn no_origin_allowed() {
        let headers = HeaderMap::new();
        assert!(validate_origin(&headers).is_ok());
    }

    #[test]
    fn localhost_origins_allowed() {
        for origin in &[
            "http://localhost",
            "http://localhost:3000",
            "http://127.0.0.1",
            "http://127.0.0.1:8080",
            "http://[::1]",
            "http://[::1]:4000",
        ] {
            let mut headers = HeaderMap::new();
            headers.insert("origin", HeaderValue::from_str(origin).expect("valid header"));
            assert!(
                validate_origin(&headers).is_ok(),
                "expected {origin} to be allowed"
            );
        }
    }

    #[test]
    fn tauri_origins_allowed() {
        for origin in &["tauri://localhost", "https://tauri.localhost"] {
            let mut headers = HeaderMap::new();
            headers.insert("origin", HeaderValue::from_str(origin).expect("valid header"));
            assert!(
                validate_origin(&headers).is_ok(),
                "expected {origin} to be allowed"
            );
        }
    }

    #[test]
    fn foreign_origin_rejected() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "origin",
            HeaderValue::from_static("https://evil.example.com"),
        );
        let err = validate_origin(&headers).unwrap_err();
        assert_eq!(err.0, StatusCode::FORBIDDEN);
    }

    // -- client_accepts_sse -------------------------------------------------

    #[test]
    fn accepts_sse_when_present() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept",
            HeaderValue::from_static("text/event-stream, application/json"),
        );
        assert!(client_accepts_sse(&headers));
    }

    #[test]
    fn no_sse_when_only_json() {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("application/json"));
        assert!(!client_accepts_sse(&headers));
    }

    #[test]
    fn no_sse_when_no_accept() {
        let headers = HeaderMap::new();
        assert!(!client_accepts_sse(&headers));
    }

    // -- response builders --------------------------------------------------

    #[test]
    fn json_response_shape() {
        let body = serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": {}});
        let (status, headers, text) = json_response(&body, Some("sess-123"));
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            headers.get("content-type").expect("content-type"),
            "application/json"
        );
        assert_eq!(
            headers.get("mcp-session-id").expect("session id"),
            "sess-123"
        );
        assert_eq!(text, body.to_string());
    }

    #[test]
    fn sse_response_shape() {
        let body = serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": {}});
        let (status, headers, text) = sse_response(&body, None);
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            headers.get("content-type").expect("content-type"),
            "text/event-stream"
        );
        assert_eq!(
            headers.get("cache-control").expect("cache-control"),
            "no-cache"
        );
        assert!(headers.get("mcp-session-id").is_none());
        assert!(text.starts_with("event: message\ndata: "));
        assert!(text.ends_with("\n\n"));
    }

    #[test]
    fn accepted_response_shape() {
        let (status, headers, text) = accepted_response(Some("sess-456"));
        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(
            headers.get("mcp-session-id").expect("session id"),
            "sess-456"
        );
        assert!(text.is_empty());
    }

    #[test]
    fn mcp_response_delegates_correctly() {
        let body = serde_json::json!({"ok": true});
        let (_, headers_json, _) = mcp_response(&body, None, false);
        assert_eq!(
            headers_json.get("content-type").expect("content-type"),
            "application/json"
        );

        let (_, headers_sse, _) = mcp_response(&body, None, true);
        assert_eq!(
            headers_sse.get("content-type").expect("content-type"),
            "text/event-stream"
        );
    }

    #[test]
    fn new_session_id_is_valid_uuid() {
        let id = new_session_id();
        assert!(Uuid::parse_str(&id).is_ok(), "expected valid UUID, got {id}");
    }
}

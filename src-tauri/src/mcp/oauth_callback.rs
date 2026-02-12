use std::sync::Arc;

use axum::{extract::Query, extract::State as AxumState, response::Html, routing::get, Router};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tracing::{debug, info};

use crate::error::AppError;

/// The result captured from the OAuth callback redirect.
#[derive(Debug)]
pub struct CallbackResult {
    pub code: String,
    pub state: String,
}

#[derive(serde::Deserialize)]
struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

struct CallbackState {
    tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<Result<CallbackResult, AppError>>>>>,
}

/// Start a temporary localhost HTTP server to capture the OAuth callback.
/// Returns (port, receiver) — the receiver will yield the callback result.
/// The server auto-shuts down after the first request or a 2-minute timeout.
pub async fn start_callback_server(
) -> Result<(u16, oneshot::Receiver<Result<CallbackResult, AppError>>), AppError> {
    let (tx, rx) = oneshot::channel();

    let state = Arc::new(CallbackState {
        tx: Arc::new(tokio::sync::Mutex::new(Some(tx))),
    });

    let app = Router::new()
        .route("/oauth/callback", get(handle_callback))
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| AppError::OAuth(format!("Failed to bind callback server: {e}")))?;

    let port = listener
        .local_addr()
        .map_err(|e| AppError::OAuth(format!("Failed to get callback server address: {e}")))?
        .port();

    info!("OAuth callback server listening on http://127.0.0.1:{port}/oauth/callback");

    // Spawn the server with a 2-minute timeout
    tokio::spawn(async move {
        let server = axum::serve(listener, app);
        let timeout = tokio::time::sleep(std::time::Duration::from_secs(120));

        tokio::select! {
            result = server => {
                if let Err(e) = result {
                    debug!("OAuth callback server error: {e}");
                }
            }
            _ = timeout => {
                debug!("OAuth callback server timed out after 2 minutes");
                // Send timeout error if nobody has claimed the sender yet
                let mut guard = state.tx.lock().await;
                if let Some(tx) = guard.take() {
                    let _ = tx.send(Err(AppError::OAuth(
                        "OAuth callback timed out — no response received within 2 minutes".into(),
                    )));
                }
            }
        }
    });

    Ok((port, rx))
}

async fn handle_callback(
    AxumState(state): AxumState<Arc<CallbackState>>,
    Query(params): Query<CallbackParams>,
) -> Html<&'static str> {
    let result = if let Some(error) = params.error {
        let desc = params.error_description.unwrap_or_default();
        Err(AppError::OAuth(format!(
            "Authorization denied: {error} — {desc}"
        )))
    } else {
        match (params.code, params.state) {
            (Some(code), Some(state_param)) => Ok(CallbackResult {
                code,
                state: state_param,
            }),
            _ => Err(AppError::OAuth(
                "Missing code or state in OAuth callback".into(),
            )),
        }
    };

    // Send the result through the oneshot channel
    let mut guard = state.tx.lock().await;
    if let Some(tx) = guard.take() {
        let _ = tx.send(result);
    }

    Html(
        r#"<!DOCTYPE html>
<html>
<head><title>MCP Manager</title></head>
<body style="font-family: system-ui, sans-serif; display: flex; justify-content: center; align-items: center; min-height: 100vh; margin: 0; background: #1a1a2e; color: #e0e0e0;">
<div style="text-align: center;">
<h1 style="font-size: 1.5rem; margin-bottom: 0.5rem;">Authorization Complete</h1>
<p>You can close this tab and return to MCP Manager.</p>
</div>
</body>
</html>"#,
    )
}

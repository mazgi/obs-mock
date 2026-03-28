mod auth;
mod handler;
mod protocol;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::handshake::server::{
    Request as WsRequest, Response as WsResponse,
};
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::auth::AuthConfig;
use crate::handler::handle_request;
use crate::protocol::*;
use crate::state::ObsState;

const DEFAULT_PORT: u16 = 4455;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let port: u16 = std::env::var("OBS_MOCK_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let password = std::env::var("OBS_MOCK_PASSWORD").ok();

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    info!("OBS WebSocket Mock Server listening on ws://{}", addr);
    if password.is_some() {
        info!("Authentication is enabled");
    } else {
        info!("Authentication is disabled (set OBS_MOCK_PASSWORD to enable)");
    }

    while let Ok((stream, peer)) = listener.accept().await {
        let password = password.clone();
        tokio::spawn(async move {
            info!("New connection from: {}", peer);
            if let Err(e) = handle_connection(stream, password).await {
                error!("Connection error from {}: {}", peer, e);
            }
            info!("Connection closed: {}", peer);
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    password: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_stream = tokio_tungstenite::accept_hdr_async(stream, |req: &WsRequest, mut response: WsResponse| {
        // Negotiate the obswebsocket subprotocol as required by OBS WebSocket v5.x clients
        if let Some(protocols) = req.headers().get("sec-websocket-protocol") {
            let protocols_str = protocols.to_str().unwrap_or("");
            if protocols_str.contains("obswebsocket.json") {
                response.headers_mut().insert(
                    "sec-websocket-protocol",
                    HeaderValue::from_static("obswebsocket.json"),
                );
            } else if protocols_str.contains("obswebsocket.msgpack") {
                response.headers_mut().insert(
                    "sec-websocket-protocol",
                    HeaderValue::from_static("obswebsocket.msgpack"),
                );
            }
        }
        Ok(response)
    })
    .await?;

    let (mut write, mut read) = ws_stream.split();

    let auth_config = AuthConfig::new(password);

    // Send Hello (OpCode 0)
    let hello = protocol::Hello {
        obs_web_socket_version: "5.5.4".to_string(),
        rpc_version: RPC_VERSION,
        authentication: if auth_config.requires_auth() {
            Some(protocol::AuthChallenge {
                challenge: auth_config.challenge.clone(),
                salt: auth_config.salt.clone(),
            })
        } else {
            None
        },
    };

    let hello_msg = protocol::Message::new(OP_HELLO, &hello);
    write
        .send(Message::Text(serde_json::to_string(&hello_msg)?.into()))
        .await?;
    info!("Sent Hello");

    // Wait for Identify (OpCode 1)
    let identify = loop {
        match read.next().await {
            Some(Ok(Message::Text(text))) => {
                let msg: protocol::Message = serde_json::from_str(&text)?;
                if msg.op == OP_IDENTIFY {
                    let identify: Identify = serde_json::from_value(msg.d)?;
                    break identify;
                }
                warn!("Expected Identify, got op: {}", msg.op);
            }
            Some(Ok(Message::Close(_))) | None => return Ok(()),
            Some(Ok(_)) => continue,
            Some(Err(e)) => return Err(e.into()),
        }
    };

    // Verify authentication
    if auth_config.requires_auth() {
        let auth_string = identify.authentication.as_deref().unwrap_or("");
        if !auth_config.verify(auth_string) {
            warn!("Authentication failed");
            write.close().await?;
            return Ok(());
        }
    }

    // Send Identified (OpCode 2)
    let identified = protocol::Identified {
        negotiated_rpc_version: identify.rpc_version.min(RPC_VERSION),
    };
    let identified_msg = protocol::Message::new(OP_IDENTIFIED, &identified);
    write
        .send(Message::Text(
            serde_json::to_string(&identified_msg)?.into(),
        ))
        .await?;
    info!("Client identified (rpcVersion: {})", identify.rpc_version);

    // Process requests
    let state = Arc::new(Mutex::new(ObsState::new()));

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let parsed: protocol::Message = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        warn!("Failed to parse message: {}", e);
                        continue;
                    }
                };

                match parsed.op {
                    OP_REQUEST => {
                        let request: Request = serde_json::from_value(parsed.d)?;
                        info!("Request: {} ({})", request.request_type, request.request_id);

                        let response = {
                            let mut state = state.lock().await;
                            handle_request(
                                &mut state,
                                &request.request_type,
                                &request.request_id,
                                request.request_data.as_ref(),
                            )
                        };

                        let response_msg = protocol::Message::new(OP_REQUEST_RESPONSE, &response);
                        write
                            .send(Message::Text(serde_json::to_string(&response_msg)?.into()))
                            .await?;
                    }

                    OP_REQUEST_BATCH => {
                        let batch: RequestBatch = serde_json::from_value(parsed.d)?;
                        info!(
                            "RequestBatch: {} ({} requests)",
                            batch.request_id,
                            batch.requests.len()
                        );

                        let halt_on_failure = batch.halt_on_failure.unwrap_or(false);
                        let mut results = Vec::new();

                        {
                            let mut state = state.lock().await;
                            for request in &batch.requests {
                                let response = handle_request(
                                    &mut state,
                                    &request.request_type,
                                    &request.request_id,
                                    request.request_data.as_ref(),
                                );
                                let failed = !response.request_status.result;
                                results.push(response);
                                if halt_on_failure && failed {
                                    break;
                                }
                            }
                        }

                        let batch_response = RequestBatchResponse {
                            request_id: batch.request_id,
                            results,
                        };
                        let response_msg =
                            protocol::Message::new(OP_REQUEST_BATCH_RESPONSE, &batch_response);
                        write
                            .send(Message::Text(serde_json::to_string(&response_msg)?.into()))
                            .await?;
                    }

                    OP_REIDENTIFY => {
                        info!("Client reidentified");
                        let identified = protocol::Identified {
                            negotiated_rpc_version: RPC_VERSION,
                        };
                        let msg = protocol::Message::new(OP_IDENTIFIED, &identified);
                        write
                            .send(Message::Text(serde_json::to_string(&msg)?.into()))
                            .await?;
                    }

                    _ => {
                        warn!("Unexpected op code: {}", parsed.op);
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(Message::Ping(data)) => {
                write.send(Message::Pong(data)).await?;
            }
            Ok(_) => {}
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

mod auth;
mod handler;
mod protocol;
mod state;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex as StdMutex};

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

#[derive(Clone, Copy, PartialEq)]
enum WireFormat {
    Json,
    MsgPack,
}

fn encode_msg(format: WireFormat, value: &impl serde::Serialize) -> Result<Message, Box<dyn std::error::Error + Send + Sync>> {
    match format {
        WireFormat::Json => Ok(Message::Text(serde_json::to_string(value)?.into())),
        WireFormat::MsgPack => Ok(Message::Binary(rmp_serde::to_vec_named(value)?.into())),
    }
}

fn decode_msg(format: WireFormat, msg: &Message) -> Result<protocol::Message, Box<dyn std::error::Error + Send + Sync>> {
    match (format, msg) {
        (WireFormat::Json, Message::Text(text)) => Ok(serde_json::from_str(text)?),
        (WireFormat::MsgPack, Message::Binary(data)) => Ok(rmp_serde::from_slice(data)?),
        // Also accept JSON text even in msgpack mode (some clients mix)
        (WireFormat::MsgPack, Message::Text(text)) => Ok(serde_json::from_str(text)?),
        _ => Err("unexpected message type".into()),
    }
}

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

    info!("OBS WebSocket Mock Server v{} listening on ws://{}", env!("CARGO_PKG_VERSION"), addr);
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
    let format = Arc::new(StdMutex::new(WireFormat::Json));
    let format_clone = format.clone();

    let ws_stream = tokio_tungstenite::accept_hdr_async(stream, move |req: &WsRequest, mut response: WsResponse| {
        if let Some(protocols) = req.headers().get("sec-websocket-protocol") {
            let protocols_str = protocols.to_str().unwrap_or("");
            info!("Client requested subprotocols: {}", protocols_str);
            if protocols_str.contains("obswebsocket.json") {
                response.headers_mut().insert(
                    "sec-websocket-protocol",
                    HeaderValue::from_static("obswebsocket.json"),
                );
            } else if protocols_str.contains("obswebsocket.msgpack") {
                *format_clone.lock().unwrap() = WireFormat::MsgPack;
                response.headers_mut().insert(
                    "sec-websocket-protocol",
                    HeaderValue::from_static("obswebsocket.msgpack"),
                );
            }
        } else {
            response.headers_mut().insert(
                "sec-websocket-protocol",
                HeaderValue::from_static("obswebsocket.json"),
            );
        }
        Ok(response)
    })
    .await?;

    let wire = *format.lock().unwrap();
    info!("Using {} format", if wire == WireFormat::MsgPack { "msgpack" } else { "json" });

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
    write.send(encode_msg(wire, &hello_msg)?).await?;
    info!("Sent Hello");

    // Wait for Identify (OpCode 1)
    let identify = loop {
        match read.next().await {
            Some(Ok(ref msg @ (Message::Text(_) | Message::Binary(_)))) => {
                let parsed = match decode_msg(wire, msg) {
                    Ok(m) => m,
                    Err(e) => {
                        warn!("Failed to parse message: {}", e);
                        continue;
                    }
                };
                if parsed.op == OP_IDENTIFY {
                    let identify: Identify = match serde_json::from_value(parsed.d) {
                        Ok(i) => i,
                        Err(e) => {
                            warn!("Failed to parse Identify data: {}", e);
                            continue;
                        }
                    };
                    break identify;
                }
                warn!("Expected Identify, got op: {}", parsed.op);
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
    write.send(encode_msg(wire, &identified_msg)?).await?;
    info!("Client identified (rpcVersion: {})", identify.rpc_version);

    // Process requests
    let state = Arc::new(Mutex::new(ObsState::new()));

    while let Some(msg) = read.next().await {
        match msg {
            Ok(ref msg @ (Message::Text(_) | Message::Binary(_))) => {
                let parsed = match decode_msg(wire, msg) {
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
                        write.send(encode_msg(wire, &response_msg)?).await?;
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
                        write.send(encode_msg(wire, &response_msg)?).await?;
                    }

                    OP_REIDENTIFY => {
                        info!("Client reidentified");
                        let identified = protocol::Identified {
                            negotiated_rpc_version: RPC_VERSION,
                        };
                        let msg = protocol::Message::new(OP_IDENTIFIED, &identified);
                        write.send(encode_msg(wire, &msg)?).await?;
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

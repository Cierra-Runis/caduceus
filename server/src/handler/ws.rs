use std::{collections::HashMap, sync::Arc, time::Duration};

use actix_web::rt;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use bson::oid::ObjectId;

// Type alias for project id to allow easy switching later
pub type ProjectId = String;

// Type alias for connection sender to reduce type complexity
type ConnectionSender = mpsc::UnboundedSender<String>;

// Type alias for project sessions map
type ProjectSessions = HashMap<String, ConnectionSender>;

use actix_web::http::StatusCode;
use actix_web::ResponseError;
use derive_more::Display;
use futures_util::StreamExt as _;
use tokio::{
    sync::{mpsc, RwLock},
    time::{interval, Instant},
};
use tracing::{debug, info};

use crate::models::response::ApiResponse;
use crate::models::ws::Message;

#[derive(Debug, Display)]
pub enum WebSocketError {
    #[display("User not Found")]
    UserNotFound,
    #[display("Project not Found")]
    ProjectNotFound,
    #[display("Handshake Failed: {_0}")]
    HandshakeFailed(actix_web::Error),
}

impl ResponseError for WebSocketError {
    fn error_response(&self) -> HttpResponse {
        let response = ApiResponse::error(&self.to_string());
        HttpResponse::build(self.status_code()).json(response)
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            WebSocketError::UserNotFound | WebSocketError::ProjectNotFound => StatusCode::NOT_FOUND,
            WebSocketError::HandshakeFailed(_) => StatusCode::BAD_REQUEST,
        }
    }
}

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Handshake and start WebSocket handler with heartbeats.
pub async fn ws(
    id: web::Path<String>,
    req: HttpRequest,
    stream: web::Payload,
    data: actix_web::web::Data<crate::AppState>,
    project_server: web::Data<ProjectServer>,
) -> Result<HttpResponse, WebSocketError> {
    let id = ObjectId::parse_str(id.into_inner()).map_err(|_| WebSocketError::ProjectNotFound)?;

    let project_id = match data.project_service.find_by_id(id).await {
        Ok(project) => project.id,
        Err(_) => return Err(WebSocketError::ProjectNotFound),
    };

    let (res, session, stream) = match actix_ws::handle(&req, stream) {
        Ok(tuple) => tuple,
        Err(e) => return Err(WebSocketError::HandshakeFailed(e)),
    };

    // spawn websocket handler (and don't await it) so that the response is returned immediately
    // the handler will register with the ProjectServer and forward messages
    // spawn the websocket handler on the actix runtime
    rt::spawn(handle_ws(
        project_server.as_ref().clone(),
        project_id,
        session,
        stream,
    ));

    Ok(res)
}

/// Echo text & binary messages received from the client, respond to ping messages, and monitor
/// connection health to detect network issues and free up resources.
async fn handle_ws(
    chat_server: ProjectServer,
    project_id: ProjectId,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) {
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    // Register this connection with the ProjectServer. The ProjectServer will
    // keep the mpsc sender and can use it to push messages to this WebSocket.
    // We store the returned string id to allow later disconnect.
    let conn_id = chat_server.connect(project_id.clone(), conn_tx).await;
    info!("WS handler: registered connection {}", conn_id);

    let mut msg_stream = msg_stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    // The main loop for this WebSocket session.
    // We listen to two independent message sources in parallel using `tokio::select!`:
    // 1) `msg_stream.next()` - messages coming from the WebSocket client (incoming frames)
    // 2) `conn_rx.recv()`   - messages pushed from the ProjectServer (server -> this client)
    // The loop exits when a Close frame is received, when heartbeats time out, or when both
    // streams are closed.
    let close_reason = loop {
        tokio::select! {
            Some(Ok(msg)) = msg_stream.next() => {
                match msg {
                    AggregatedMessage::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        if let Err(e) = session.pong(&bytes).await {
                            debug!("AggregatedMessage::Ping {}: failed to send Pong: {:#?}", conn_id, e);
                            break None;
                        }
                        debug!("AggregatedMessage::Ping {}: received Ping, sent Pong", conn_id);
                    }
                    AggregatedMessage::Pong(_) => {
                        last_heartbeat = Instant::now();
                        debug!("AggregatedMessage::Pong {}: received Pong", conn_id);
                    }
                    AggregatedMessage::Text(text) => {
                        let message: Result<Message, serde_json::Error> = serde_json::from_str(text.to_string().as_str());

                        match message {
                            Ok(message) => {
                                debug!("AggregatedMessage::Text {}: received Text message: {:?}", conn_id, message);
                                // Broadcast the message to all sessions in the same project
                                chat_server.broadcast(&project_id, &message).await;
                            }
                            Err(e) => {
                                debug!("AggregatedMessage::Text {}: failed to parse JSON message: {:#?}", conn_id, e);
                                // Optionally, you could choose to close the connection here
                                // break Some(actix_ws::CloseReason {
                                //     code: actix_ws::CloseCode::Invalid,
                                //     description: Some("Invalid JSON".to_string()),
                                // });
                            }

                      }
                    }
                    AggregatedMessage::Binary(bin) => {
                        debug!("AggregatedMessage::Binary {}: received Binary message ({} bytes)", conn_id, bin.len());
                        if let Err(e) = session.binary(bin).await {
                            debug!("AggregatedMessage::Binary {}: failed to send Binary echo: {:#?}", conn_id, e);
                            break None;
                        }
                    }
                    AggregatedMessage::Close(reason) => {
                        debug!("AggregatedMessage::Close {}: received Close message: {:?}", conn_id, reason);
                        break reason;
                    },
                }
            }

            // Message pushed from ProjectServer (server -> this connection)
            // We only call `conn_rx.recv()` once here and handle both Some/None inside to
            // avoid multiple mutable borrows of `conn_rx` in the same `select!`.
            chat_msg = conn_rx.recv() => {
                match chat_msg {
                    Some(chat_msg) => {
                        debug!("WS conn_rx.recv {}: sending message from ProjectServer: {}", conn_id, chat_msg);
                        if let Err(e) = session.text(chat_msg).await {
                            debug!("WS conn_rx.recv {}: failed to send server-pushed text: {:#?}", conn_id, e);
                            break None;
                        }
                    }
                    None => {
                        // The sender side (`conn_tx`) was dropped; no more server messages.
                        debug!("WS conn_rx.recv {}: conn_rx closed (conn_tx dropped).", conn_id);
                        break None;
                    }
                }
            }

            _ = interval.tick() => {
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    break None;
                }
                let _ = session.ping(b"").await;
            }

            else => {
                // This branch triggers when all selected futures are completed / cancelled.
                debug!("WS {}: select! else branch triggered (stream ended).", conn_id);
                break None;
            }
        }
    };

    // Unregister from ProjectServer
    chat_server
        .disconnect(project_id.clone(), conn_id.clone())
        .await;
    info!("WS handler: disconnected {}", conn_id);

    // attempt to close connection gracefully
    let _ = session.close(close_reason).await;
}

/// ProjectServer manages WebSocket connections without using Actix actors.
///
/// Responsibilities:
/// - Maintain mapping of connection id -> session sender
/// - Handle connection/disconnection registration
/// - Broadcast messages between sessions in the same project
#[derive(Debug, Clone, Default)]
pub struct ProjectServer {
    // map project_id -> (map conn_id -> sender)
    sessions: Arc<RwLock<HashMap<ProjectId, ProjectSessions>>>,
}

impl ProjectServer {
    /// Register a new connection and return its unique ID
    pub async fn connect(&self, project_id: ProjectId, conn_tx: ConnectionSender) -> String {
        let id = ObjectId::new().to_string();
        info!("ProjectServer: registering session {}", id);

        let mut sessions = self.sessions.write().await;
        let project_sessions = sessions.entry(project_id.clone()).or_default();
        project_sessions.insert(id.clone(), conn_tx);

        id
    }

    /// Unregister a connection
    pub async fn disconnect(&self, project_id: ProjectId, id: String) {
        info!(
            "ProjectServer: disconnecting session {} from project {}",
            id, project_id
        );

        let mut sessions = self.sessions.write().await;
        if let Some(project_sessions) = sessions.get_mut(&project_id) {
            project_sessions.remove(&id);
            if project_sessions.is_empty() {
                sessions.remove(&project_id);
            }
        }
    }

    /// Broadcast a textual message to all connected sessions in a project
    pub async fn broadcast(&self, project_id: &ProjectId, text: &Message) {
        let sessions = self.sessions.read().await;

        if let Some(project_sessions) = sessions.get(project_id) {
            let message_str = serde_json::to_string(text).unwrap();
            for (id, tx) in project_sessions {
                let _ = tx.send(message_str.clone());
                debug!("Broadcast to project {}: queued to {}", project_id, id);
            }
        } else {
            debug!("Broadcast: no sessions for project {}", project_id);
        }
    }
}

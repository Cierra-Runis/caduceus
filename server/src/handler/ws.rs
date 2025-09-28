use std::{collections::HashMap, time::Duration};

use actix::{prelude::*, Addr};
use actix_web::rt;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use bson::oid::ObjectId;

// Type alias for project id to allow easy switching later
pub type ProjectId = String;
use actix_web::http::StatusCode;
use actix_web::ResponseError;
use derive_more::Display;
use futures_util::StreamExt as _;
use tokio::{
    sync::mpsc,
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
    project_server: web::Data<ProjectServerHandle>,
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
    // the handler will register with the ProjectServer actor and forward messages
    // spawn the websocket handler on the actix runtime
    rt::spawn(handle_ws(
        (**project_server).clone(),
        project_id,
        session,
        stream,
    ));

    Ok(res)
}

/// Echo text & binary messages received from the client, respond to ping messages, and monitor
/// connection health to detect network issues and free up resources.
async fn handle_ws(
    chat_server: ProjectServerHandle,
    project_id: ProjectId,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) {
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    // Register this connection with the ProjectServer actor. The actor will
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
    // 2) `conn_rx.recv()`   - messages pushed from the `ProjectServer` actor (server -> this client)
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
                                chat_server.addr.do_send(BroadcastText {
                                    project_id: project_id.clone(),
                                    text: message,
                                });
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

    // Unregister from ProjectServer actor
    chat_server.disconnect(project_id.clone(), conn_id.clone());
    info!("WS handler: disconnected {}", conn_id);

    // attempt to close connection gracefully
    let _ = session.close(close_reason).await;
}

/// ProjectServer is implemented as an Actix actor.
///
/// Responsibilities:
/// - Maintain mapping of connection id -> session recipient
/// - Handle Connect / Disconnect messages from WebSocket session actors
/// - Broadcast messages between sessions if needed (extension point)
#[derive(Debug)]
pub struct ProjectServer {
    // map project_id -> (map conn_id -> sender)
    sessions: HashMap<ProjectId, HashMap<String, mpsc::UnboundedSender<String>>>,
}

impl ProjectServer {
    pub fn new() -> Self {
        ProjectServer {
            sessions: HashMap::new(),
        }
    }
}

impl Default for ProjectServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Message sent from session actor to ProjectServer to register itself.
#[derive(Message)]
#[rtype(result = "String")]
pub struct Connect {
    pub project_id: ProjectId,
    pub conn_tx: mpsc::UnboundedSender<String>,
}

/// Message sent to unregister a session.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub project_id: ProjectId,
    pub id: String,
}

impl Actor for ProjectServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ProjectServer {
    type Result = String;

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        let id = ObjectId::new().to_string();
        info!("ProjectServer: registering session {}", id);
        let project_sessions = self.sessions.entry(msg.project_id.clone()).or_default();
        project_sessions.insert(id.clone(), msg.conn_tx);
        id
    }
}

impl Handler<Disconnect> for ProjectServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        info!(
            "ProjectServer: disconnecting session {} from project {}",
            msg.id, msg.project_id
        );
        if let Some(sessions) = self.sessions.get_mut(&msg.project_id) {
            sessions.remove(&msg.id);
            if sessions.is_empty() {
                self.sessions.remove(&msg.project_id);
            }
        }
    }
}

impl ProjectServer {
    /// Broadcast a textual message to all connected sessions (simple implementation)
    pub fn broadcast(&self, project_id: &ProjectId, text: &Message) {
        if let Some(sessions) = self.sessions.get(project_id) {
            for (id, tx) in sessions {
                let _ = tx.send(serde_json::to_string(text).unwrap());
                debug!("Broadcast to project {}: queued to {}", project_id, id);
            }
        } else {
            debug!("Broadcast: no sessions for project {}", project_id);
        }
    }
}

/// Message to ask ProjectServer to broadcast a text message to all sessions.
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastText {
    pub project_id: ProjectId,
    pub text: Message,
}

impl Handler<BroadcastText> for ProjectServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastText, _ctx: &mut Self::Context) -> Self::Result {
        self.broadcast(&msg.project_id, &msg.text);
    }
}

/// Handle wrapper exposed to actix-web for storing Addr<ProjectServer> in app data.
#[derive(Clone)]
pub struct ProjectServerHandle {
    pub addr: Addr<ProjectServer>,
}

impl ProjectServerHandle {
    pub fn new(addr: Addr<ProjectServer>) -> Self {
        Self { addr }
    }

    /// Connect helper that asks the actor to register with specific project_id and returns the assigned string id.
    pub async fn connect(
        &self,
        project_id: ProjectId,
        conn_tx: mpsc::UnboundedSender<String>,
    ) -> String {
        self.addr
            .send(Connect {
                project_id,
                conn_tx,
            })
            .await
            .unwrap()
    }

    /// Disconnect helper that includes project id
    pub fn disconnect(&self, project_id: ProjectId, id: String) {
        self.addr.do_send(Disconnect { project_id, id });
    }
}

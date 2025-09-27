use std::{collections::HashMap, time::Duration};

use actix::{prelude::*, Addr};
use actix_web::rt as actix_rt;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use bson::oid::ObjectId;
use futures_util::StreamExt as _;
use tokio::{
    sync::mpsc,
    time::{interval, Instant},
};
use tracing::{debug, info};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Handshake and start WebSocket handler with heartbeats.
pub async fn ws(
    req: HttpRequest,
    stream: web::Payload,
    project_server: web::Data<ProjectServerHandle>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // spawn websocket handler (and don't await it) so that the response is returned immediately
    // the handler will register with the ProjectServer actor and forward messages
    // spawn the websocket handler on the actix runtime
    actix_rt::spawn(handle_ws((**project_server).clone(), session, msg_stream));

    Ok(res)
}

/// Echo text & binary messages received from the client, respond to ping messages, and monitor
/// connection health to detect network issues and free up resources.
async fn handle_ws(
    chat_server: ProjectServerHandle,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) {
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    // Register this connection with the ProjectServer actor. The actor will
    // keep the mpsc sender and can use it to push messages to this WebSocket.
    // We store the returned string id to allow later disconnect.
    let conn_id = chat_server.connect(conn_tx).await;
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
                            debug!("WS {}: failed to send Pong: {:#?}", conn_id, e);
                            break None;
                        }
                        debug!("WS {}: received Ping, sent Pong", conn_id);
                    }
                    AggregatedMessage::Pong(_) => {
                        last_heartbeat = Instant::now();
                        debug!("WS {}: received Pong", conn_id);
                    }
                    AggregatedMessage::Text(text) => {
                        // Convert possibly borrowed bytes to owned String for logging/broadcast
                        let txt = text.to_string();
                        debug!("WS {}: received Text message: {}", conn_id, txt);
                        // Echo back to sender
                        if let Err(e) = session.text(txt.clone()).await {
                            debug!("WS {}: failed to send Text echo: {:#?}", conn_id, e);
                            break None;
                        }
                        // Broadcast to other sessions via ProjectServer actor
                        chat_server.addr.do_send(BroadcastText(txt));
                    }
                    AggregatedMessage::Binary(bin) => {
                        debug!("WS {}: received Binary message ({} bytes)", conn_id, bin.len());
                        if let Err(e) = session.binary(bin).await {
                            debug!("WS {}: failed to send Binary echo: {:#?}", conn_id, e);
                            break None;
                        }
                    }
                    AggregatedMessage::Close(reason) => break reason,
                }
            }

            // Message pushed from ProjectServer (server -> this connection)
            // We only call `conn_rx.recv()` once here and handle both Some/None inside to
            // avoid multiple mutable borrows of `conn_rx` in the same `select!`.
            chat_msg = conn_rx.recv() => {
                match chat_msg {
                    Some(chat_msg) => {
                        debug!("WS {}: sending message from ProjectServer: {}", conn_id, chat_msg);
                        if let Err(e) = session.text(chat_msg).await {
                            debug!("WS {}: failed to send server-pushed text: {:#?}", conn_id, e);
                            break None;
                        }
                    }
                    None => {
                        // The sender side (`conn_tx`) was dropped; no more server messages.
                        debug!("WS {}: conn_rx closed (conn_tx dropped).", conn_id);
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
    chat_server.disconnect(conn_id.clone());
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
    sessions: HashMap<String, mpsc::UnboundedSender<String>>,
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
    pub conn_tx: mpsc::UnboundedSender<String>,
}

/// Message sent to unregister a session.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
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
        self.sessions.insert(id.clone(), msg.conn_tx);
        id
    }
}

impl Handler<Disconnect> for ProjectServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        info!("ProjectServer: disconnecting session {}", msg.id);
        self.sessions.remove(&msg.id);
    }
}

impl ProjectServer {
    /// Broadcast a textual message to all connected sessions (simple implementation)
    pub fn broadcast(&self, text: &str) {
        for (id, tx) in &self.sessions {
            let _ = tx.send(text.to_string());
            debug!("Broadcast: queued to {}", id);
        }
    }
}

/// Message to ask ProjectServer to broadcast a text message to all sessions.
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastText(pub String);

impl Handler<BroadcastText> for ProjectServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastText, _ctx: &mut Self::Context) -> Self::Result {
        self.broadcast(&msg.0);
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

    /// Connect helper that asks the actor to register and returns the assigned string id.
    pub async fn connect(&self, conn_tx: mpsc::UnboundedSender<String>) -> String {
        self.addr.send(Connect { conn_tx }).await.unwrap()
    }

    /// Disconnect helper
    pub fn disconnect(&self, id: String) {
        self.addr.do_send(Disconnect { id });
    }
}

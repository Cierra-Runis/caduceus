use std::{collections::HashMap, io, time::Duration};

use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use bson::oid::ObjectId;
use futures_util::StreamExt as _;
use tokio::{
    sync::{mpsc, oneshot},
    task::spawn_local,
    time::{interval, Instant},
};

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
    spawn_local(handle_ws((**project_server).clone(), session, msg_stream));

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

    // unwrap: chat server is not dropped before the HTTP server
    let conn_id = chat_server.connect(conn_tx).await;

    let mut msg_stream = msg_stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    let close_reason = loop {
        tokio::select! {
            Some(Ok(msg)) = msg_stream.next() => {
                match msg {
                    AggregatedMessage::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        session.pong(&bytes).await.unwrap();
                    }
                    AggregatedMessage::Pong(_) => {
                        last_heartbeat = Instant::now();
                    }
                    AggregatedMessage::Text(text) => {
                        session.text(text).await.unwrap();
                    }
                    AggregatedMessage::Binary(bin) => {
                        session.binary(bin).await.unwrap();
                    }
                    AggregatedMessage::Close(reason) => break reason,
                }
            }

            Some(chat_msg) = conn_rx.recv() => {
                session.text(chat_msg).await.unwrap();
            }

            _ = interval.tick() => {
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    break None;
                }
                let _ = session.ping(b"").await;
            }

            else => {
                break None;
            }
        }
    };

    chat_server.disconnect(conn_id);

    // attempt to close connection gracefully
    let _ = session.close(close_reason).await;
}

/// Handle and command sender for chat server.
///
/// Reduces boilerplate of setting up response channels in WebSocket handlers.
#[derive(Debug, Clone)]
pub struct ProjectServerHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
}

/// A command received by the [`ChatServer`].
#[derive(Debug)]
enum Command {
    Connect {
        conn_tx: mpsc::UnboundedSender<String>,
        res_tx: oneshot::Sender<ObjectId>,
    },

    Disconnect {
        conn: ObjectId,
    },
}

impl ProjectServerHandle {
    /// Register client message sender and obtain connection ID.
    pub async fn connect(&self, conn_tx: mpsc::UnboundedSender<String>) -> ObjectId {
        let (res_tx, res_rx) = oneshot::channel();

        // unwrap: chat server should not have been dropped
        self.cmd_tx
            .send(Command::Connect { conn_tx, res_tx })
            .unwrap();

        // unwrap: chat server does not drop out response channel
        res_rx.await.unwrap()
    }

    /// Unregister message sender and broadcast disconnection message to current room.
    pub fn disconnect(&self, conn: ObjectId) {
        // unwrap: chat server should not have been dropped
        self.cmd_tx.send(Command::Disconnect { conn }).unwrap();
    }
}

/// A multi-room chat server.
///
/// Contains the logic of how connections chat with each other plus room management.
///
/// Call and spawn [`run`](Self::run) to start processing commands.
#[derive(Debug)]
pub struct ProjectServer {
    /// Map of connection IDs to their message receivers.
    sessions: HashMap<ObjectId, mpsc::UnboundedSender<String>>,

    /// Command receiver.
    cmd_rx: mpsc::UnboundedReceiver<Command>,
}

impl ProjectServer {
    pub fn new() -> (Self, ProjectServerHandle) {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            Self {
                sessions: HashMap::new(),
                cmd_rx,
            },
            ProjectServerHandle { cmd_tx },
        )
    }

    /// Register new session and assign unique ID to this session
    async fn connect(&mut self, tx: mpsc::UnboundedSender<String>) -> ObjectId {
        // register session with random connection ID
        let id = ObjectId::new();
        self.sessions.insert(id, tx);

        // send id back
        id
    }

    /// Unregister connection from room map and broadcast disconnection message.
    async fn disconnect(&mut self, conn_id: ObjectId) {
        // remove sender
        if self.sessions.remove(&conn_id).is_some() {}
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect { conn_tx, res_tx } => {
                    let conn_id = self.connect(conn_tx).await;
                    let _ = res_tx.send(conn_id);
                }

                Command::Disconnect { conn } => {
                    self.disconnect(conn).await;
                }
            }
        }

        Ok(())
    }
}

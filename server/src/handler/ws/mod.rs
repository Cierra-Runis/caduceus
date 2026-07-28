use std::{
    collections::HashSet,
    thread,
    time::Duration,
};

use actix_web::ResponseError;
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, rt, web};
use actix_ws::AggregatedMessage;
use bson::oid::ObjectId;
use derive_more::Display;
use futures_util::StreamExt as _;
use tokio::{
    sync::mpsc::{self, UnboundedSender},
    sync::oneshot,
    task::LocalSet,
    time::{Instant, interval},
};
use tracing::{debug, warn};
use yrs::{
    ClientID, Doc,
    sync::{Awareness, Message as YMessage},
    updates::encoder::Encode,
};

use crate::config::{LspConfig, WsConfig};
use crate::lsp::room::RoomWorker;
use crate::models::response::ApiResponse;
use crate::models::tree::{Node, ProjectTree};
use crate::models::user::UserClaims;
use crate::repo::project::{MongoProjectRepo, ProjectRepo};
use crate::storage::{Blob, ProjectStore};

mod lsp;
mod room;
use room::room_manager;

#[derive(Debug, Display)]
pub enum WebSocketError {
    #[display("User not Found")]
    UserNotFound,
    #[display("Project not Found")]
    ProjectNotFound,
    #[display("Handshake Failed: {_0}")]
    HandshakeFailed(actix_web::Error),
    #[display("Unauthorized: {_0}")]
    Unauthorized(String),
    #[display("Forbidden: You don't have access to this project")]
    Forbidden,
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
            WebSocketError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            WebSocketError::Forbidden => StatusCode::FORBIDDEN,
        }
    }
}

/// y-protocol message-type tag for awareness frames (sync is `0`). The tag is a
/// single lib0 varint byte for values < 128, so the first byte identifies it.
pub(super) const MSG_AWARENESS: u8 = 1;

/// Handshake and start WebSocket handler with heartbeats.
// Actix injects each dependency as its own extractor argument; bundling them
// into a struct just to satisfy the lint would obscure the handler.
#[allow(clippy::too_many_arguments)]
pub async fn ws(
    id: web::Path<String>,
    req: HttpRequest,
    stream: web::Payload,
    data: actix_web::web::Data<crate::AppState>,
    project_server: web::Data<ProjectServer>,
    ws_config: web::Data<WsConfig>,
    store: web::Data<ProjectStore>,
    lsp_config: web::Data<Option<LspConfig>>,
    user: UserClaims,
) -> Result<HttpResponse, WebSocketError> {
    let project_id =
        ObjectId::parse_str(id.into_inner()).map_err(|_| WebSocketError::ProjectNotFound)?;

    // Check if user has access to this project
    match data.project_service.accessible(project_id, user.sub).await {
        Ok(true) => {}
        Ok(false) => return Err(WebSocketError::Forbidden),
        Err(_) => return Err(WebSocketError::ProjectNotFound),
    };

    let project = match data
        .project_service
        .project_repo
        .find_by_id(project_id)
        .await
    {
        Ok(Some(project)) => project,
        Ok(None) => return Err(WebSocketError::ProjectNotFound),
        Err(_) => return Err(WebSocketError::ProjectNotFound),
    };

    // Only the *first* connection to a project hydrates the room; later joiners
    // sync against the already-live document. Prefer restoring from the last
    // Y.Doc snapshot; otherwise cold-start from the stored tree (structure), and
    // the room rematerializes each file's text from its blob after building.
    // Blobs already exist (create seeds them, edits flush them) — nothing is
    // uploaded here.
    let store: &ProjectStore = store.get_ref();
    let project_hex = project_id.to_hex();
    let snapshot = store.get_snapshot(&project_hex).await.ok().flatten();

    // How to spawn this project's tinymist worker (if LSP is configured and the
    // pinned version has a binary). Resolved here where the `Project` is in hand;
    // computed before `project.tree` is consumed below.
    let lsp = lsp_config.get_ref().as_ref().and_then(|cfg| {
        let version = project.pinned_version.as_ref().map(|v| v.to_string());
        let binary = cfg.binary_for(version.as_deref())?.to_path_buf();
        let entry_path = project
            .entry
            .and_then(|id| project.tree.get(&id.to_hex()).map(|e| e.path.clone()));
        Some(LspSpawnInfo {
            binary,
            root: cfg.workspace_root.join(&project_hex),
            entry_path,
        })
    });

    let tree = ProjectTree::from_nodes(project.tree.into_iter().map(|(id, entry)| Node {
        id,
        parent: entry.parent,
        name: entry.name,
        content: entry.content,
    }));

    let (res, session, stream) = match actix_ws::handle(&req, stream) {
        Ok(tuple) => tuple,
        Err(e) => return Err(WebSocketError::HandshakeFailed(e)),
    };

    rt::spawn(handle_ws(
        project_server.as_ref().clone(),
        project_id,
        snapshot,
        tree,
        session,
        stream,
        ws_config.as_ref().clone(),
        lsp,
    ));

    Ok(res)
}

/// `GET /api/admin/rooms` — a read-only snapshot of every live collaboration
/// room's state (connection counts, dirty/idle flags, node counts). Aggregate
/// data only, no document content. Behind the same JWT auth as the rest of
/// `/api`; `_user` proves the caller is authenticated. Meant for operator
/// introspection — turning "what are the rooms doing?" into a live query
/// instead of a recompile-with-a-print.
pub async fn rooms(
    project_server: web::Data<ProjectServer>,
    _user: UserClaims,
) -> HttpResponse {
    let rooms = project_server.inspect().await;
    HttpResponse::Ok().json(ApiResponse::success("Live rooms", rooms))
}

/// `GET /ws/project/{id}/lsp` — a per-browser LSP session bridged to the
/// project's tinymist worker. Frames are vscode-ws-jsonrpc style (one bare JSON
/// message per websocket text frame). Diagnostics are pushed to the browser;
/// its requests are forwarded to the worker; its document-sync notifications are
/// dropped (the room owns document sync).
pub async fn ws_lsp(
    id: web::Path<String>,
    req: HttpRequest,
    stream: web::Payload,
    data: actix_web::web::Data<crate::AppState>,
    project_server: web::Data<ProjectServer>,
    user: UserClaims,
) -> Result<HttpResponse, WebSocketError> {
    let project_id =
        ObjectId::parse_str(id.into_inner()).map_err(|_| WebSocketError::ProjectNotFound)?;
    match data.project_service.accessible(project_id, user.sub).await {
        Ok(true) => {}
        Ok(false) => return Err(WebSocketError::Forbidden),
        Err(_) => return Err(WebSocketError::ProjectNotFound),
    };
    let (res, session, stream) =
        actix_ws::handle(&req, stream).map_err(WebSocketError::HandshakeFailed)?;
    rt::spawn(handle_lsp_ws(
        project_server.as_ref().clone(),
        project_id,
        session,
        stream,
    ));
    Ok(res)
}

/// Per-connection LSP bridge loop: browser frames become [`Command::LspData`],
/// and frames the manager routes back (diagnostics, query responses) arrive on
/// `out_rx` and are written to the socket as text.
async fn handle_lsp_ws(
    project_server: ProjectServer,
    project_id: ObjectId,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) {
    let conn_id = ObjectId::new();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    project_server.lsp_connect(project_id, conn_id, out_tx);
    debug!(project = %project_id.to_hex(), conn = %conn_id.to_hex(), "lsp session opened");

    let mut msg_stream = msg_stream
        .max_frame_size(1024 * 1024)
        .aggregate_continuations()
        .max_continuation_size(8 * 1024 * 1024);

    loop {
        tokio::select! {
            Some(Ok(msg)) = msg_stream.next() => {
                match msg {
                    AggregatedMessage::Text(text) => {
                        project_server.lsp_data(project_id, conn_id, text.as_bytes().to_vec());
                    }
                    AggregatedMessage::Binary(bin) => {
                        project_server.lsp_data(project_id, conn_id, bin.to_vec());
                    }
                    AggregatedMessage::Ping(bytes) => {
                        if session.pong(&bytes).await.is_err() { break; }
                    }
                    AggregatedMessage::Close(_) => break,
                    _ => {}
                }
            }
            msg = out_rx.recv() => {
                match msg {
                    // vscode-ws-jsonrpc expects one JSON message per text frame.
                    Some(bytes) => match String::from_utf8(bytes) {
                        Ok(text) => { if session.text(text).await.is_err() { break; } }
                        Err(_) => continue,
                    },
                    None => break,
                }
            }
            else => break,
        }
    }

    project_server.lsp_leave(project_id, conn_id);
    debug!(project = %project_id.to_hex(), conn = %conn_id.to_hex(), "lsp session closed");
    let _ = session.close(None).await;
}

/// Per-connection loop. Bridges this WebSocket to the single-threaded room
/// manager: client frames are forwarded as [`Command::Data`], and messages the
/// manager routes back (initial sync, peers' updates, awareness) arrive on
/// `out_rx` and are written to the socket.
#[allow(clippy::too_many_arguments)]
async fn handle_ws(
    project_server: ProjectServer,
    project_id: ObjectId,
    snapshot: Option<Vec<u8>>,
    tree: ProjectTree,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
    ws_config: WsConfig,
    lsp: Option<LspSpawnInfo>,
) {
    let heartbeat_interval = Duration::from_secs(ws_config.heartbeat_interval_secs);
    let client_timeout = Duration::from_secs(ws_config.client_timeout_secs);
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(heartbeat_interval);
    // Application-level keepalive, sent on the heartbeat cadence (see the tick
    // branch). Built once; empty only if encoding ever fails, in which case it
    // is never sent.
    let keepalive = keepalive_frame();

    let conn_id = ObjectId::new();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    project_server.join(project_id, snapshot, tree, conn_id, out_tx, lsp);
    debug!(project = %project_id.to_hex(), conn = %conn_id.to_hex(), "ws connection opened");

    let mut msg_stream = msg_stream
        .max_frame_size(1024 * 1024)
        .aggregate_continuations()
        .max_continuation_size(8 * 1024 * 1024);

    let close_reason = loop {
        tokio::select! {
            Some(Ok(msg)) = msg_stream.next() => {
                match msg {
                    AggregatedMessage::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        if session.pong(&bytes).await.is_err() { break None; }
                    }
                    AggregatedMessage::Pong(_) => {
                        last_heartbeat = Instant::now();
                    }
                    AggregatedMessage::Binary(bin) => {
                        last_heartbeat = Instant::now();
                        project_server.data(project_id, conn_id, bin.to_vec());
                    }
                    AggregatedMessage::Text(_) => {
                        // The collaboration protocol is binary; ignore text.
                    }
                    AggregatedMessage::Close(reason) => break reason,
                }
            }

            msg = out_rx.recv() => {
                match msg {
                    Some(bytes) => {
                        if session.binary(bytes).await.is_err() { break None; }
                    }
                    None => break None,
                }
            }

            _ = interval.tick() => {
                if Instant::now().duration_since(last_heartbeat) > client_timeout {
                    break None;
                }
                // Keep the client's y-websocket alive with an application-level
                // frame *before* the protocol ping. The browser answers a WS
                // ping/pong itself, at the protocol layer, so those frames never
                // surface to y-websocket's `onmessage` and never reset its
                // `messageReconnectTimeout` (30s of no *message* ⇒ it force-closes
                // and reconnects). A lone client that neither receives peers'
                // updates (broadcasts skip the sender) nor a server push within
                // that window would otherwise reconnect every ~30s — dropping the
                // room to zero connections and back. The keepalive is a no-op
                // awareness frame, so it refreshes that clock without touching the
                // document or presence. The protocol ping still runs, driving the
                // server-side dead-client check above via the client's pong.
                if !keepalive.is_empty() && session.binary(keepalive.clone()).await.is_err() {
                    break None;
                }
                let _ = session.ping(b"").await;
            }

            else => break None,
        }
    };

    project_server.leave(project_id, conn_id);
    debug!(project = %project_id.to_hex(), conn = %conn_id.to_hex(), "ws connection closed");
    let _ = session.close(close_reason).await;
}

/// A no-op y-awareness frame the server sends on the heartbeat cadence to keep a
/// client's y-websocket `messageReconnectTimeout` from firing (see the tick
/// branch in [`handle_ws`]). It carries *zero* awareness clients, so the client
/// decodes it, resets its "last message received" clock, and changes nothing —
/// no presence added, updated, or removed, and the document is untouched. Built
/// from a throwaway awareness because `handle_ws` runs off the room-manager
/// thread and so has no access to the live doc; the frame is content-free, so a
/// throwaway is equivalent. Returns an empty vec only if encoding fails (not
/// expected for an empty client set), in which case no keepalive is sent.
pub(super) fn keepalive_frame() -> Vec<u8> {
    let awareness = Awareness::new(Doc::new());
    match awareness.update_with_clients(Vec::<ClientID>::new()) {
        Ok(update) => YMessage::Awareness(update).encode_v1(),
        Err(e) => {
            warn!("failed to encode keepalive frame: {e:?}");
            Vec::new()
        }
    }
}

/// Commands sent from connection handlers (any worker thread) to the
/// single-threaded room manager. Everything here is `Send`; the `yrs` document
/// itself never leaves the manager thread.
/// Everything the room manager needs to spawn a tinymist worker for a project,
/// resolved at handshake time (where the `Project` and `LspConfig` are in hand).
/// `None` when LSP is unconfigured or the pinned version has no binary.
#[derive(Clone)]
pub struct LspSpawnInfo {
    /// The version-pinned tinymist binary for this project.
    pub binary: std::path::PathBuf,
    /// This project's staging root (`workspace_root/<project_hex>`).
    pub root: std::path::PathBuf,
    /// The compile entry's tree path, if any (for `tinymist.pinMain`).
    pub entry_path: Option<String>,
}

pub(super) enum Command {
    Join {
        project_id: ObjectId,
        /// Prior Y.Doc snapshot bytes, if any — restores the room directly.
        snapshot: Option<Vec<u8>>,
        /// The stored tree (structure), used to cold-start when there is no
        /// snapshot; the room then rematerializes text from blobs.
        tree: ProjectTree,
        conn_id: ObjectId,
        out: UnboundedSender<Vec<u8>>,
        /// How to spawn this project's tinymist worker, if LSP is enabled.
        lsp: Option<LspSpawnInfo>,
    },
    Data {
        project_id: ObjectId,
        conn_id: ObjectId,
        data: Vec<u8>,
    },
    Leave {
        project_id: ObjectId,
        conn_id: ObjectId,
    },
    /// A client asked to save now (its `files.autoSave` policy fired). Force a
    /// blob flush for the room so the current text is materialized, regardless
    /// of the settle cadence. Missing room = nothing to flush.
    FlushRoom {
        project_id: ObjectId,
    },
    /// A persist cycle uploaded changed file text as blobs (async, off the room
    /// thread); this brings the result back so the room can update each file
    /// node's `blob` (sha256 / size) in the Y.Doc and broadcast it — keeping the
    /// node's blob reference current with its edited text.
    FlushBlobs {
        project_id: ObjectId,
        blobs: Vec<(String, Blob)>,
    },
    /// Text fetched from blobs (async) to refill a room rebuilt from a *stripped*
    /// resting snapshot — one whose text overlays were emptied on eviction so the
    /// bytes live only in blobs. Applied to the empty overlays and broadcast.
    ApplyRemat {
        project_id: ObjectId,
        texts: Vec<(String, String)>,
    },
    /// The result of a GC sweep (async): the blobs found orphaned this pass. The
    /// manager stores them so the *next* sweep only deletes blobs orphaned twice
    /// in a row — a grace window so a blob uploaded between passes is never
    /// swept before its `FlushBlobs` records it on a node.
    GcSwept {
        project_id: ObjectId,
        orphans: HashSet<String>,
    },
    /// A read-only snapshot of every live room's state, for operator
    /// introspection (`GET /api/admin/rooms`). The manager fills `reply` with a
    /// [`RoomInfo`] per room; sent over a oneshot so the async handler can await
    /// it off the manager thread.
    Inspect {
        reply: oneshot::Sender<Vec<RoomInfo>>,
    },
    /// A browser opened an LSP session (`/ws/project/{id}/lsp`). Register its out
    /// channel so the room's worker diagnostics fan out to it, priming it with
    /// the latest per-file diagnostics.
    LspConnect {
        project_id: ObjectId,
        conn_id: ObjectId,
        out: UnboundedSender<Vec<u8>>,
    },
    /// A raw LSP frame (bare JSON, vscode-ws-jsonrpc style) from a browser. A
    /// request is forwarded to the room's worker and its response routed back to
    /// this connection; the browser's document-sync notifications are dropped —
    /// the room owns document sync.
    LspData {
        project_id: ObjectId,
        conn_id: ObjectId,
        data: Vec<u8>,
    },
    /// A browser LSP session closed.
    LspLeave {
        project_id: ObjectId,
        conn_id: ObjectId,
    },
    /// The room's worker finished starting (async); store it on the room and
    /// begin forwarding its diagnostics.
    LspReady {
        project_id: ObjectId,
        worker: RoomWorker,
    },
    /// Diagnostics from a room's worker (forwarded off its broadcast), to fan out
    /// to that room's LSP connections as a `publishDiagnostics` notification.
    LspDiagnostics {
        project_id: ObjectId,
        path: String,
        diagnostics: serde_json::Value,
    },
}

/// A read-only view of one live room, safe to send off the manager thread
/// (`Send`, no `yrs` handles). Aggregate state only — ids and counts, never
/// document content — so it is cheap to expose to an authenticated operator.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoomInfo {
    /// Project id (hex) this room serves.
    pub project_id: String,
    /// Live WebSocket connections.
    pub conns: usize,
    /// Whether the Y.Doc changed since the last snapshot.
    pub dirty: bool,
    /// Whether some file's text has drifted from its recorded blob.
    pub blobs_pending: bool,
    /// Seconds the room has sat with no connections, or `null` while occupied.
    pub empty_secs: Option<u64>,
    /// Nodes (files + folders) in the tree.
    pub nodes: usize,
    /// Files that currently carry a non-empty text overlay in the doc.
    pub text_overlays: usize,
    /// Whether a tinymist LSP worker is running for this room.
    pub lsp: bool,
    /// Browser LSP sessions currently connected.
    pub lsp_conns: usize,
}

/// Handle to the collaboration subsystem, stored in actix app data. Cheap to
/// clone and `Send + Sync` (it is just a channel sender), unlike the `yrs`
/// types it fronts.
#[derive(Clone)]
pub struct ProjectServer {
    cmd_tx: UnboundedSender<Command>,
}

impl ProjectServer {
    pub fn new(
        project_repo: MongoProjectRepo,
        ws_config: WsConfig,
        store: ProjectStore,
    ) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        // The room manager owns all `yrs` state on a dedicated thread running a
        // current-thread runtime + LocalSet, so the `!Send` documents never have
        // to cross threads. It keeps a `cmd_tx` clone so a persist task can send
        // itself the blob-flush result once the async upload finishes.
        let manager_tx = cmd_tx.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("build room-manager runtime");
            let local = LocalSet::new();
            local.block_on(
                &rt,
                room_manager(cmd_rx, manager_tx, project_repo, ws_config, store),
            );
        });
        ProjectServer { cmd_tx }
    }

    fn join(
        &self,
        project_id: ObjectId,
        snapshot: Option<Vec<u8>>,
        tree: ProjectTree,
        conn_id: ObjectId,
        out: UnboundedSender<Vec<u8>>,
        lsp: Option<LspSpawnInfo>,
    ) {
        let _ = self.cmd_tx.send(Command::Join {
            project_id,
            snapshot,
            tree,
            conn_id,
            out,
            lsp,
        });
    }

    fn lsp_connect(&self, project_id: ObjectId, conn_id: ObjectId, out: UnboundedSender<Vec<u8>>) {
        let _ = self.cmd_tx.send(Command::LspConnect {
            project_id,
            conn_id,
            out,
        });
    }

    fn lsp_data(&self, project_id: ObjectId, conn_id: ObjectId, data: Vec<u8>) {
        let _ = self.cmd_tx.send(Command::LspData {
            project_id,
            conn_id,
            data,
        });
    }

    fn lsp_leave(&self, project_id: ObjectId, conn_id: ObjectId) {
        let _ = self.cmd_tx.send(Command::LspLeave {
            project_id,
            conn_id,
        });
    }

    fn data(&self, project_id: ObjectId, conn_id: ObjectId, data: Vec<u8>) {
        let _ = self.cmd_tx.send(Command::Data {
            project_id,
            conn_id,
            data,
        });
    }

    fn leave(&self, project_id: ObjectId, conn_id: ObjectId) {
        let _ = self.cmd_tx.send(Command::Leave {
            project_id,
            conn_id,
        });
    }

    /// Request an immediate blob flush for a room (a client's auto-save fired).
    /// Fire-and-forget: the room manager force-flushes on its own thread.
    pub fn flush(&self, project_id: ObjectId) {
        let _ = self.cmd_tx.send(Command::FlushRoom { project_id });
    }

    /// A read-only snapshot of every live room's state. Returns an empty list if
    /// the manager thread has gone away (send fails or the reply is dropped).
    pub async fn inspect(&self) -> Vec<RoomInfo> {
        let (reply, rx) = oneshot::channel();
        if self.cmd_tx.send(Command::Inspect { reply }).is_err() {
            return Vec::new();
        }
        rx.await.unwrap_or_default()
    }
}

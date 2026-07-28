use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
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
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    sync::oneshot,
    task::LocalSet,
    time::{Instant, interval},
};
use tracing::{debug, info, warn};
use yrs::{
    Any, ClientID, Doc, GetString, Map, Out, ReadTxn, Text, Transact, Update,
    sync::{Awareness, DefaultProtocol, Message as YMessage, Protocol, SyncMessage},
    updates::decoder::Decode as _,
    updates::encoder::{Encode, Encoder, EncoderV1},
};

use crate::config::{LspConfig, WsConfig};
use crate::crdt::snapshot::encode_doc;
use crate::crdt::{nodes_map, read_tree, write_tree};
use crate::lsp::LspError;
use crate::lsp::room::RoomWorker;
use crate::models::response::ApiResponse;
use crate::models::tree::{Node, ProjectTree};
use crate::models::user::UserClaims;
use crate::repo::project::{MongoProjectRepo, ProjectRepo};
use crate::storage::{Blob, ProjectStore, sha256_hex};

mod lsp;
use lsp::{LspDispatch, classify_lsp_message, publish_diagnostics_frame, rewrite_uris};

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
const MSG_AWARENESS: u8 = 1;

/// Handshake and start WebSocket handler with heartbeats.
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
fn keepalive_frame() -> Vec<u8> {
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

enum Command {
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

/// sha256 of empty content — a file whose blob is this has no bytes to
/// rematerialize. Matches the client's `EMPTY_SHA256`.
const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// One live collaboration room: the shared CRDT document plus its connections.
/// Lives entirely on the room-manager thread.
struct RoomState {
    awareness: Awareness,
    conns: HashMap<ObjectId, UnboundedSender<Vec<u8>>>,
    /// Which connection last reported each awareness client id, so a
    /// connection's cursor/presence can be retracted when it leaves instead
    /// of lingering as a ghost participant (see `handle_data`/`Leave`).
    client_owner: HashMap<ClientID, ObjectId>,
    /// Whether the Y.Doc changed since the last snapshot, so persist can skip
    /// re-snapshotting an unchanged room.
    dirty: bool,
    /// Whether some file's text has drifted from its recorded blob since the
    /// last blob flush, so a forced flush (client auto-save, or leave) still has
    /// blobs to upload. Set on each text edit, cleared once the blobs flush. The
    /// periodic persist tick never flushes on its own — the client's
    /// `files.autoSave` policy decides *when* via `ProjectServer::flush`.
    blobs_pending: bool,
    /// When the room fell to zero connections, or `None` while occupied. A room
    /// idle past `room_idle_secs` is evicted from memory (freeing RAM); the next
    /// joiner rebuilds it verbatim from the snapshot.
    empty_since: Option<Instant>,
    /// This room's tinymist worker, once it has finished starting (`None` while
    /// LSP is unconfigured or the worker is still spawning). The room is the
    /// worker's sole document owner — it mirrors edits in via `did_change`.
    lsp: Option<RoomWorker>,
    /// The worker's workspace root, for translating between the worker's
    /// `file://<root>/<path>` URIs and the browser's root-relative `file:///…`.
    lsp_root: Option<std::path::PathBuf>,
    /// Browser LSP sessions (`/ws/project/{id}/lsp`), for fanning out diagnostics
    /// and routing query responses back to the right connection.
    lsp_conns: HashMap<ObjectId, UnboundedSender<Vec<u8>>>,
    /// Whether a worker spawn is in flight, so a second joiner doesn't start a
    /// duplicate.
    lsp_starting: bool,
    /// Last text mirrored to the worker per file, so an edit only re-`did_change`s
    /// files whose text actually changed.
    lsp_mirror: HashMap<String, String>,
}

impl RoomState {
    /// Restore a room from a prior Y.Doc snapshot (nodes + text already inside).
    fn from_snapshot(bytes: &[u8]) -> RoomState {
        let doc = Doc::new();
        match Update::decode_v1(bytes) {
            Ok(update) => {
                if let Err(e) = doc.transact_mut().apply_update(update) {
                    warn!("snapshot apply failed: {e:?}");
                }
            }
            Err(e) => warn!("snapshot decode failed: {e:?}"),
        }
        RoomState {
            awareness: Awareness::new(doc),
            conns: HashMap::new(),
            client_owner: HashMap::new(),
            dirty: false, // the snapshot we loaded is already durable
            blobs_pending: false,
            empty_since: Some(Instant::now()),
            lsp: None,
            lsp_root: None,
            lsp_conns: HashMap::new(),
            lsp_starting: false,
            lsp_mirror: HashMap::new(),
        }
    }

    /// Cold-start a room's Y.Doc from the stored tree (structure only): the
    /// `nodes` map plus an empty text root per file. The text is refilled from
    /// blobs by [`rematerialize`] once the room is built. Used when a project has
    /// no snapshot; clients connect empty and sync against this.
    fn from_tree(tree: &ProjectTree) -> RoomState {
        let doc = Doc::new();
        // Empty text root per file — `get_or_insert_text` opens its own txn, so
        // it must precede the write txn below.
        for node in tree.iter().filter(|n| n.is_file()) {
            doc.get_or_insert_text(node.id.as_str());
        }
        let nodes = nodes_map(&doc);
        {
            let mut txn = doc.transact_mut();
            write_tree(&mut txn, &nodes, tree);
        }
        RoomState {
            awareness: Awareness::new(doc),
            conns: HashMap::new(),
            client_owner: HashMap::new(),
            dirty: true, // a fresh cold-start needs an initial snapshot
            blobs_pending: false,
            empty_since: Some(Instant::now()),
            lsp: None,
            lsp_root: None,
            lsp_conns: HashMap::new(),
            lsp_starting: false,
            lsp_mirror: HashMap::new(),
        }
    }
}

/// Retract a leaving connection's orphaned awareness state (cursor, presence)
/// so peers drop it immediately instead of it lingering as a ghost
/// participant. Returns the encoded awareness update to broadcast, or `None`
/// if the connection didn't own any awareness client ids.
fn retract_connection(room: &mut RoomState, conn_id: ObjectId) -> Option<Vec<u8>> {
    let orphaned: Vec<_> = room
        .client_owner
        .iter()
        .filter(|(_, owner)| **owner == conn_id)
        .map(|(client_id, _)| *client_id)
        .collect();
    if orphaned.is_empty() {
        return None;
    }
    for client_id in &orphaned {
        room.client_owner.remove(client_id);
        room.awareness.remove_state(*client_id);
    }
    room.awareness
        .update_with_clients(orphaned)
        .ok()
        .map(|update| YMessage::Awareness(update).encode_v1())
}

/// Derive a [`RoomInfo`] from a live room. Reads the doc once for the node and
/// text-overlay counts (`nodes_map` before opening the read txn, per the yrs
/// borrow rules).
fn room_info(project_id: ObjectId, room: &RoomState) -> RoomInfo {
    let doc = room.awareness.doc();
    let nodes = nodes_map(doc);
    let txn = doc.transact();
    let (node_count, text_overlays) = match read_tree(&txn, &nodes) {
        Ok(tree) => {
            let count = tree.iter().count();
            let overlays = tree
                .iter()
                .filter(|n| n.is_file())
                .filter(|n| {
                    txn.get_text(n.id.as_str())
                        .is_some_and(|t| t.len(&txn) > 0)
                })
                .count();
            (count, overlays)
        }
        Err(_) => (0, 0),
    };
    RoomInfo {
        project_id: project_id.to_hex(),
        conns: room.conns.len(),
        dirty: room.dirty,
        blobs_pending: room.blobs_pending,
        empty_secs: room.empty_since.map(|since| since.elapsed().as_secs()),
        nodes: node_count,
        text_overlays,
        lsp: room.lsp.is_some(),
        lsp_conns: room.lsp_conns.len(),
    }
}

/// The room's current text files as `(tree path, text)`, for mirroring into the
/// worker. Binary files (no text overlay) are skipped. `nodes_map` before the
/// read txn, per the yrs borrow rules.
fn room_text_files(room: &RoomState) -> Vec<(String, String)> {
    let doc = room.awareness.doc();
    let nodes = nodes_map(doc);
    let txn = doc.transact();
    let Ok(tree) = read_tree(&txn, &nodes) else {
        return Vec::new();
    };
    tree.iter()
        .filter(|n| n.is_file())
        .filter_map(|n| {
            let text = txn.get_text(n.id.as_str())?.get_string(&txn);
            let path = tree.path_of(&n.id).ok()?;
            Some((path, text))
        })
        .collect()
}

/// Start this room's tinymist worker once, off the manager thread: snapshot the
/// current text files to mirror, spawn [`RoomWorker::start`], forward its
/// diagnostics back as [`Command::LspDiagnostics`], and hand the ready worker to
/// the manager via [`Command::LspReady`]. A no-op when LSP is unconfigured for
/// the project or a worker is already running/starting.
fn ensure_worker(
    project_id: ObjectId,
    room: &mut RoomState,
    lsp: Option<LspSpawnInfo>,
    cmd_tx: &UnboundedSender<Command>,
) {
    let Some(info) = lsp else {
        return;
    };
    if room.lsp.is_some() || room.lsp_starting {
        return;
    }
    let files = room_text_files(room);
    // Seed the mirror cache so later edits only re-send files that changed.
    room.lsp_mirror = files.iter().cloned().collect();
    room.lsp_root = Some(info.root.clone());
    room.lsp_starting = true;

    let cmd_tx = cmd_tx.clone();
    tokio::task::spawn_local(async move {
        match RoomWorker::start(&info.binary, info.root, &files, info.entry_path.as_deref()).await {
            Ok(worker) => {
                // Pump the worker's diagnostics into the manager for fan-out.
                let mut diagnostics = worker.subscribe();
                let diag_tx = cmd_tx.clone();
                tokio::task::spawn_local(async move {
                    while let Ok(fd) = diagnostics.recv().await {
                        let sent = diag_tx.send(Command::LspDiagnostics {
                            project_id,
                            path: fd.path,
                            diagnostics: fd.diagnostics,
                        });
                        if sent.is_err() {
                            break;
                        }
                    }
                });
                let _ = cmd_tx.send(Command::LspReady { project_id, worker });
            }
            Err(e) => warn!("lsp worker start failed for {}: {e}", project_id.to_hex()),
        }
    });
}

/// Mirror the room's current text into its worker after a doc-changing frame:
/// `did_change` only the files whose text differs from what was last sent
/// (tracked in `lsp_mirror`), so an edit to one file doesn't re-push them all.
fn mirror_text_to_worker(room: &mut RoomState) {
    if room.lsp.is_none() {
        return;
    }
    for (path, text) in room_text_files(room) {
        if room.lsp_mirror.get(&path) == Some(&text) {
            continue; // unchanged since last mirror
        }
        if let Some(worker) = &room.lsp {
            worker.did_change(&path, &text);
        }
        room.lsp_mirror.insert(path, text);
    }
}

/// One browser LSP frame (bare JSON, vscode-ws-jsonrpc). Bridge lifecycle
/// requests are answered locally; genuine queries are forwarded to the room's
/// worker with URIs translated between the browser's root-relative `file:///…`
/// and the worker's `file://<root>/…`, and the response routed back.
fn handle_lsp_data(room: &RoomState, conn_id: ObjectId, data: Vec<u8>) {
    let (Some(worker), Some(root)) = (&room.lsp, &room.lsp_root) else {
        return;
    };
    let Ok(msg) = serde_json::from_slice::<serde_json::Value>(&data) else {
        return;
    };
    let Some(out) = room.lsp_conns.get(&conn_id).cloned() else {
        return;
    };

    match classify_lsp_message(&msg) {
        LspDispatch::Reply(response) => {
            if let Ok(bytes) = serde_json::to_vec(&response) {
                let _ = out.send(bytes);
            }
        }
        LspDispatch::Drop => {}
        LspDispatch::Forward { id, method, params } => {
            let worker_prefix = format!("file://{}/", root.display());
            let params = rewrite_uris(params, "file:///", &worker_prefix);
            let client = worker.client();
            tokio::task::spawn_local(async move {
                let response = match client.request(&method, params).await {
                    Ok(result) => serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": rewrite_uris(result, &worker_prefix, "file:///"),
                    }),
                    Err(LspError::Rpc(error)) => serde_json::json!({
                        "jsonrpc": "2.0", "id": id, "error": error,
                    }),
                    Err(LspError::Closed) => serde_json::json!({
                        "jsonrpc": "2.0", "id": id,
                        "error": { "code": -32000, "message": "lsp worker unavailable" },
                    }),
                };
                if let Ok(bytes) = serde_json::to_vec(&response) {
                    let _ = out.send(bytes);
                }
            });
        }
    }
}

/// Single-threaded owner of every room. Serves commands and periodically
/// persists each room (Y.Doc snapshot to MinIO, tree projection to Mongo),
/// sweeps orphaned blobs, and evicts idle rooms.
async fn room_manager(
    mut cmd_rx: UnboundedReceiver<Command>,
    cmd_tx: UnboundedSender<Command>,
    repo: MongoProjectRepo,
    ws_config: WsConfig,
    store: ProjectStore,
) {
    let mut rooms: HashMap<ObjectId, RoomState> = HashMap::new();
    // Blobs seen orphaned in the previous GC sweep, per project — deleted next
    // sweep only if still orphaned (a two-pass grace window).
    let mut gc_pending: HashMap<ObjectId, HashSet<String>> = HashMap::new();
    let mut persist_tick = interval(Duration::from_secs(ws_config.persist_interval_secs));
    let mut gc_tick = interval(Duration::from_secs(ws_config.gc_interval_secs));
    let room_idle = Duration::from_secs(ws_config.room_idle_secs);
    // Check for idle, evictable rooms on the GC cadence (both are lazy space/RAM
    // reclamation, so they can share a slow tick).
    let mut evict_tick = interval(Duration::from_secs(ws_config.gc_interval_secs));

    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(Command::Join { project_id, snapshot, tree, conn_id, out, lsp }) => {
                        // Track whether this join *builds* the room, so a room
                        // built from a stripped snapshot or cold-started from the
                        // tree refills its text from blobs once.
                        let fresh = !rooms.contains_key(&project_id);
                        // Only meaningful when `fresh` — how the room was built.
                        let source = if snapshot.is_some() { "snapshot" } else { "cold-tree" };
                        let room = rooms.entry(project_id).or_insert_with(|| match &snapshot {
                            Some(bytes) => RoomState::from_snapshot(bytes),
                            None => RoomState::from_tree(&tree),
                        });
                        // Send the initial sync step 1 + awareness state.
                        let mut encoder = EncoderV1::new();
                        if DefaultProtocol.start(&room.awareness, &mut encoder).is_ok() {
                            let _ = out.send(encoder.to_vec());
                        }
                        room.conns.insert(conn_id, out);
                        room.empty_since = None; // occupied again
                        info!(
                            project = %project_id.to_hex(),
                            conn = %conn_id.to_hex(),
                            conns = room.conns.len(),
                            source = if fresh { source } else { "existing" },
                            "room join",
                        );
                        if fresh {
                            rematerialize(project_id, room, &store, &cmd_tx);
                        }
                        // Lazily start this room's tinymist worker on first join
                        // (idempotent: skipped if one is running or starting).
                        ensure_worker(project_id, room, lsp, &cmd_tx);
                    }
                    Some(Command::Data { project_id, conn_id, data }) => {
                        if let Some(room) = rooms.get_mut(&project_id) {
                            handle_data(room, conn_id, data);
                        }
                    }
                    Some(Command::Leave { project_id, conn_id }) => {
                        if let Some(room) = rooms.get_mut(&project_id) {
                            room.conns.remove(&conn_id);

                            // Retract this connection's awareness state (cursor,
                            // presence) so peers drop it immediately, rather than
                            // leaving a ghost participant until the process
                            // restarts (the room itself is kept alive with no
                            // connections, see below).
                            if let Some(msg) = retract_connection(room, conn_id) {
                                broadcast(room, conn_id, &msg);
                            }

                            if room.conns.is_empty() {
                                // The room stays in memory for now; only after it
                                // sits idle past `room_idle_secs` is it evicted
                                // (see the evict tick). Until then a reconnecting
                                // client re-syncs against the SAME live document —
                                // re-deriving a doc from text would re-insert the
                                // same characters and the CRDT would merge them
                                // into DUPLICATED content. Mark the idle clock and
                                // persist now, forcing a blob flush (no later
                                // settle will catch the final edits).
                                room.empty_since = Some(Instant::now());
                                persist_room(project_id, room, &repo, &store, &cmd_tx, true);
                                info!(
                                    project = %project_id.to_hex(),
                                    conn = %conn_id.to_hex(),
                                    idle_threshold_secs = room_idle.as_secs(),
                                    "room emptied; idle eviction clock started",
                                );
                            } else {
                                debug!(
                                    project = %project_id.to_hex(),
                                    conn = %conn_id.to_hex(),
                                    conns = room.conns.len(),
                                    "room leave",
                                );
                            }
                        }
                    }
                    Some(Command::FlushRoom { project_id }) => {
                        if let Some(room) = rooms.get_mut(&project_id) {
                            persist_room(project_id, room, &repo, &store, &cmd_tx, true);
                        }
                    }
                    Some(Command::FlushBlobs { project_id, blobs }) => {
                        if let Some(room) = rooms.get_mut(&project_id) {
                            apply_blobs(room, blobs);
                        }
                    }
                    Some(Command::ApplyRemat { project_id, texts }) => {
                        if let Some(room) = rooms.get_mut(&project_id) {
                            apply_remat(room, texts);
                        }
                    }
                    Some(Command::GcSwept { project_id, orphans }) => {
                        if orphans.is_empty() {
                            gc_pending.remove(&project_id);
                        } else {
                            gc_pending.insert(project_id, orphans);
                        }
                    }
                    Some(Command::Inspect { reply }) => {
                        let infos = rooms
                            .iter()
                            .map(|(id, room)| room_info(*id, room))
                            .collect();
                        let _ = reply.send(infos);
                    }
                    Some(Command::LspReady { project_id, worker }) => {
                        if let Some(room) = rooms.get_mut(&project_id) {
                            room.lsp_starting = false;
                            // The room may have emptied while the worker started;
                            // if so, drop it right back (shutdown off-thread).
                            if room.conns.is_empty() {
                                tokio::task::spawn_local(worker.shutdown());
                            } else {
                                room.lsp = Some(worker);
                                info!(project = %project_id.to_hex(), "lsp worker ready");
                            }
                        } else {
                            tokio::task::spawn_local(worker.shutdown());
                        }
                    }
                    Some(Command::LspConnect { project_id, conn_id, out }) => {
                        if let Some(room) = rooms.get_mut(&project_id) {
                            // Prime the new session with the current diagnostics
                            // before streaming live ones.
                            if let Some(worker) = &room.lsp {
                                for fd in worker.latest() {
                                    let _ = out.send(publish_diagnostics_frame(&fd.path, &fd.diagnostics));
                                }
                            }
                            room.lsp_conns.insert(conn_id, out);
                        }
                    }
                    Some(Command::LspData { project_id, conn_id, data }) => {
                        if let Some(room) = rooms.get_mut(&project_id) {
                            handle_lsp_data(room, conn_id, data);
                        }
                    }
                    Some(Command::LspLeave { project_id, conn_id }) => {
                        if let Some(room) = rooms.get_mut(&project_id) {
                            room.lsp_conns.remove(&conn_id);
                        }
                    }
                    Some(Command::LspDiagnostics { project_id, path, diagnostics }) => {
                        if let Some(room) = rooms.get(&project_id) {
                            let frame = publish_diagnostics_frame(&path, &diagnostics);
                            for out in room.lsp_conns.values() {
                                let _ = out.send(frame.clone());
                            }
                        }
                    }
                    None => break,
                }
            }
            _ = persist_tick.tick() => {
                for (project_id, room) in rooms.iter_mut() {
                    persist_room(*project_id, room, &repo, &store, &cmd_tx, false);
                }
            }
            _ = gc_tick.tick() => {
                for (project_id, room) in rooms.iter() {
                    let prev = gc_pending.get(project_id).cloned().unwrap_or_default();
                    gc_room(*project_id, room, &store, &cmd_tx, prev);
                }
            }
            _ = evict_tick.tick() => {
                // Drop rooms idle (no connections) past the threshold, reclaiming
                // their in-memory document. Before dropping, strip each text
                // overlay whose bytes already live in its blob, so the resting
                // snapshot no longer stores the text twice; the next joiner
                // rematerializes it from the blobs. A reconnecting client learns
                // the strip's deletion (a tombstone in the snapshot) and so never
                // duplicates the rematerialized text.
                let stale: Vec<ObjectId> = rooms
                    .iter()
                    .filter(|(_, room)| {
                        room.empty_since
                            .is_some_and(|since| since.elapsed() >= room_idle)
                    })
                    .map(|(id, _)| *id)
                    .collect();
                for project_id in stale {
                    if let Some(room) = rooms.get_mut(&project_id) {
                        // Idle seconds as a number, not the raw monotonic
                        // `Instant` (whose Debug is an opaque clock base).
                        let idle_secs = room
                            .empty_since
                            .map(|since| since.elapsed().as_secs())
                            .unwrap_or(0);
                        strip_text(room);
                        // Reap this room's tinymist worker along with its doc.
                        if let Some(worker) = room.lsp.take() {
                            tokio::task::spawn_local(worker.shutdown());
                        }
                        let bytes = encode_doc(room.awareness.doc());
                        let store = store.clone();
                        let pid = project_id.to_hex();
                        info!(project = %pid, idle_secs, snapshot_bytes = bytes.len(), "evicting idle room");
                        tokio::task::spawn_local(async move {
                            if let Err(e) = store.put_snapshot(&pid, &bytes).await {
                                warn!("stripped snapshot save failed in {pid}: {e:?}");
                            }
                        });
                    }
                    rooms.remove(&project_id);
                    gc_pending.remove(&project_id);
                }
            }
        }
    }
}

/// Run `f` against the room's [`Awareness`], capturing every Y.Doc update it
/// produces so the caller can relay them. Returns `f`'s result alongside the
/// captured updates (encoded, ready to broadcast); the observer is torn down
/// before returning, so the doc is free to be touched again.
///
/// We capture via `observe_update_v1` rather than diffing the state vector
/// before/after: a deletion only adds tombstones and does *not* advance the
/// state vector, so an SV diff silently drops deletes (they would reach peers
/// only piggy-backed on a later insertion). The observer fires for inserts and
/// deletes alike, and only when a transaction actually changed something, so a
/// redundant update stays a no-op. The `Arc<Mutex<_>>` satisfies the observer's
/// `Send + Sync` bound; everything here runs on the single room-manager thread,
/// so it never contends.
fn capture_doc_updates<R>(
    awareness: &mut Awareness,
    f: impl FnOnce(&mut Awareness) -> R,
) -> (R, Vec<Vec<u8>>) {
    let applied: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = applied.clone();
    let subscription = awareness.doc().observe_update_v1(move |_txn, event| {
        if let Ok(mut updates) = sink.lock() {
            updates.push(event.update.clone());
        }
    });
    if let Err(e) = &subscription {
        warn!("failed to observe doc updates: {e:?}");
    }
    let result = f(awareness);
    drop(subscription); // stop observing before the doc is touched again
    let updates = std::mem::take(&mut *applied.lock().unwrap());
    (result, updates)
}

/// Apply one client frame to the room's document and fan the result out.
fn handle_data(room: &mut RoomState, conn_id: ObjectId, data: Vec<u8>) {
    let is_awareness = data.first() == Some(&MSG_AWARENESS);

    // Run the protocol against the doc, capturing whatever it changes so the
    // result can be relayed verbatim (see `capture_doc_updates`).
    let (replies, updates) = capture_doc_updates(&mut room.awareness, |awareness| {
        DefaultProtocol.handle(awareness, &data)
    });

    // Sync replies (e.g. the sync step 2 carrying current content) go back to
    // the sender only.
    match replies {
        Ok(replies) => {
            if let Some(origin) = room.conns.get(&conn_id) {
                for reply in replies {
                    let _ = origin.send(reply.encode_v1());
                }
            }
        }
        Err(e) => debug!("WS protocol error: {:?}", e),
    }

    // Applied document changes (inserts and deletes) and awareness frames go to
    // everyone else.
    if !updates.is_empty() {
        room.dirty = true;
        // Mark that some file may have drifted from its blob, so the next forced
        // flush (client auto-save, or leave) re-uploads it. This over-
        // approximates: a pure structural edit (no text change) sets it too, but
        // the flush then finds nothing stale and clears it — cheaper than
        // distinguishing text from structure here.
        room.blobs_pending = true;
        for update in updates {
            let msg = YMessage::Sync(SyncMessage::Update(update)).encode_v1();
            broadcast(room, conn_id, &msg);
        }
        // The authority never lets the shared tree rest in an illegal state —
        // a name clash, a cycle, or a dangling parent that concurrent edits can
        // produce. NOTE: reads the tree on every doc-changing frame, text edits
        // included; it is a cheap map scan at current scale — gate it on a
        // nodes-map observer if that ever shows up in a profile.
        reconcile_tree(room);
        // Mirror changed text into the tinymist worker (no-op without one).
        mirror_text_to_worker(room);
    }
    if is_awareness {
        // Track which connection last reported each awareness client id, so
        // it can be retracted if this connection disconnects without
        // reporting a `null` state itself (e.g. a crash or dropped socket).
        if let Ok(YMessage::Awareness(update)) = YMessage::decode_v1(&data) {
            for client_id in update.clients.keys() {
                room.client_owner.insert(*client_id, conn_id);
            }
        }
        broadcast(room, conn_id, &data);
    }
}

/// Send a frame to every connection in the room except `origin`.
fn broadcast(room: &RoomState, origin: ObjectId, msg: &[u8]) {
    for (conn_id, tx) in &room.conns {
        if *conn_id != origin {
            let _ = tx.send(msg.to_vec());
        }
    }
}

/// Send a frame to every connection in the room, no exceptions. Used for
/// authority corrections, which must reach the connection that caused the
/// clash too so its view snaps to the corrected state.
fn broadcast_all(room: &RoomState, msg: &[u8]) {
    for tx in room.conns.values() {
        let _ = tx.send(msg.to_vec());
    }
}

/// Server-authoritative repair of the shared tree, in two passes:
///
/// 1. **Structural** — reparent cycle / dangling-parent victims to the root
///    (the invariants a concurrent *move* can break, see
///    [`ProjectTree::structural_repairs`]).
/// 2. **Naming** — dedupe siblings that share a `(parent, name)`, on the now
///    structurally-sound tree (see [`ProjectTree::dedupe_sibling_names`]).
///
/// Both passes mutate the Y.Doc; the resulting updates are captured and
/// broadcast to *every* connection, so the client that caused the clash also
/// snaps to the corrected state.
fn reconcile_tree(room: &mut RoomState) {
    let nodes = nodes_map(room.awareness.doc());

    // One observer spans both passes, capturing whatever they change to relay.
    let (_, updates) = capture_doc_updates(&mut room.awareness, |awareness| {
        let doc = awareness.doc();

        // Pass 1: reparent structural victims to the root (drop their `parent`).
        let structural = {
            let txn = doc.transact();
            // A node half-written mid-sync — skip; a later frame retries.
            read_tree(&txn, &nodes)
                .map(|tree| tree.structural_repairs())
                .unwrap_or_default()
        };
        if !structural.is_empty() {
            let mut txn = doc.transact_mut();
            for id in structural {
                if let Some(Out::YMap(node)) = nodes.get(&txn, &id) {
                    node.remove(&mut txn, "parent");
                }
            }
        }

        // Pass 2: dedupe sibling names on the repaired tree.
        let renames = {
            let txn = doc.transact();
            read_tree(&txn, &nodes)
                .map(|tree| tree.dedupe_sibling_names())
                .unwrap_or_default()
        };
        if !renames.is_empty() {
            let mut txn = doc.transact_mut();
            for (id, name) in renames {
                if let Some(Out::YMap(node)) = nodes.get(&txn, &id) {
                    node.insert(&mut txn, "name", name);
                }
            }
        }
    });

    if !updates.is_empty() {
        room.dirty = true;
        for update in updates {
            let msg = YMessage::Sync(SyncMessage::Update(update)).encode_v1();
            broadcast_all(room, &msg);
        }
    }
}

/// Persist the room's Y.Doc if it changed since the last snapshot: the whole
/// doc (nodes + text) goes to a MinIO snapshot (the CRDT authority) and the
/// derived tree projection to Mongo (the listing cache). There is no inline
/// text written to Mongo — bytes live once, in content-addressed blobs.
///
/// Content-addressed blobs are materialized only on a **forced flush**
/// (`force_flush`): a client's `files.autoSave` policy firing (via
/// `ProjectServer::flush`), or the room emptying on leave. The periodic tick
/// never mints a blob on its own — uploading a fresh blob on every tick while
/// someone is mid-edit would spray a new MinIO object per keystroke-burst, each
/// superseded moments later and left for GC. The client owns the *when* (it
/// alone knows about editor / window focus and keystroke timing).
///
/// All the `!Send` doc work happens synchronously up front; only the IO is
/// spawned onto this thread's LocalSet.
fn persist_room(
    project_id: ObjectId,
    room: &mut RoomState,
    repo: &MongoProjectRepo,
    store: &ProjectStore,
    cmd_tx: &UnboundedSender<Command>,
    force_flush: bool,
) {
    // Only a forced flush materializes blobs; the plain tick just snapshots.
    let do_flush = force_flush && room.blobs_pending;

    // Nothing to snapshot and no blobs to flush — skip entirely.
    if !room.dirty && !do_flush {
        return;
    }
    let snapshot_dirty = room.dirty;
    room.dirty = false;
    if do_flush {
        room.blobs_pending = false;
    }

    // Phase 1 (sync, holds the doc): encode the snapshot, derive the projection,
    // and — only when flushing — the files whose text has drifted from their
    // recorded blob. Outputs are owned/`Send`.
    let (snapshot_bytes, projection, blob_stale) = {
        let doc = room.awareness.doc();
        let snapshot_bytes = encode_doc(doc);
        let nodes = nodes_map(doc);
        let txn = doc.transact();
        let tree = read_tree(&txn, &nodes).ok();
        let projection = tree.as_ref().and_then(|t| t.projection().ok());

        let mut blob_stale: Vec<(String, String)> = Vec::new();
        if do_flush {
            if let Some(tree) = tree.as_ref() {
                for node in tree.iter().filter(|n| n.is_file()) {
                    // A binary file has no text overlay; skip it (its blob is set
                    // at upload and must never be flushed over with empty text).
                    let Some(text) = txn
                        .get_text(node.id.as_str())
                        .map(|txt| txt.get_string(&txn))
                    else {
                        continue;
                    };
                    // A file whose current text hashes to something other than
                    // its recorded blob sha needs re-uploading.
                    let fresh = sha256_hex(text.as_bytes());
                    let recorded = node.blob().map(|b| b.sha256.as_str());
                    if recorded != Some(fresh.as_str()) {
                        blob_stale.push((node.id.clone(), text));
                    }
                }
            }
        }
        (snapshot_bytes, projection, blob_stale)
    };

    // Phase 2 (async, no doc borrow): write to the durable stores. The snapshot
    // and projection go only when the doc actually changed; blobs are uploaded
    // (write-before-reference) and their hashes sent back as `FlushBlobs` so each
    // node's blob reference catches up.
    let repo = repo.clone();
    let store = store.clone();
    let cmd_tx = cmd_tx.clone();
    tokio::task::spawn_local(async move {
        let pid = project_id.to_hex();
        if snapshot_dirty {
            if let Err(e) = store.put_snapshot(&pid, &snapshot_bytes).await {
                warn!("snapshot save failed in {pid}: {e:?}");
            }
            if let Some(projection) = projection {
                if let Err(e) = repo.update_tree(project_id, projection).await {
                    warn!("projection update failed in {pid}: {e:?}");
                }
            }
        }
        let mut flushed: Vec<(String, Blob)> = Vec::new();
        for (id_hex, text) in blob_stale {
            match store.put_blob(&pid, text.as_bytes()).await {
                Ok(blob) => flushed.push((id_hex, blob)),
                Err(e) => warn!("blob flush failed in {pid}: {e:?}"),
            }
        }
        if !flushed.is_empty() {
            let _ = cmd_tx.send(Command::FlushBlobs {
                project_id,
                blobs: flushed,
            });
        }
    });
}

/// Update each named file node's `blob` (sha256 / size) in the Y.Doc to the
/// freshly-flushed value, so the node reference tracks its edited text, and
/// broadcast the change to every connection. A node whose blob already matches
/// is left untouched (no spurious update). Runs on the room thread in response
/// to a [`Command::FlushBlobs`].
fn apply_blobs(room: &mut RoomState, blobs: Vec<(String, Blob)>) {
    let nodes = nodes_map(room.awareness.doc());

    let (_, updates) = capture_doc_updates(&mut room.awareness, |awareness| {
        let mut txn = awareness.doc().transact_mut();
        for (id, blob) in blobs {
            let Some(Out::YMap(node)) = nodes.get(&txn, &id) else {
                continue;
            };
            // Skip if the node already carries this blob — avoids re-broadcasting
            // the converged state on later persist cycles.
            let unchanged = matches!(
                node.get(&txn, "sha256"),
                Some(Out::Any(Any::String(s))) if s.as_ref() == blob.sha256.as_str()
            );
            if unchanged {
                continue;
            }
            node.insert(&mut txn, "sha256", blob.sha256);
            node.insert(&mut txn, "size", blob.size as i64);
        }
    });

    if !updates.is_empty() {
        room.dirty = true;
        for update in updates {
            let msg = YMessage::Sync(SyncMessage::Update(update)).encode_v1();
            broadcast_all(room, &msg);
        }
    }
}

/// Strip each text overlay whose bytes are safely in its blob (the file's text
/// hashes to its recorded blob sha), by **deleting** the overlay's content. The
/// deletion is a CRDT operation, so the emptied overlay carries a tombstone into
/// the resting snapshot: the text bytes no longer sit in the snapshot (they live
/// once, in the blob), yet a client that reconnects across the eviction learns
/// the deletion and drops its own copy instead of merging it with the
/// rematerialized text — no duplication. A file whose text hasn't settled to its
/// blob is left intact (it keeps its bytes in the snapshot this cycle).
fn strip_text(room: &mut RoomState) {
    let doc = room.awareness.doc();
    let nodes = nodes_map(doc);

    let to_strip: Vec<String> = {
        let txn = doc.transact();
        let Ok(tree) = read_tree(&txn, &nodes) else {
            return;
        };
        tree.iter()
            .filter(|n| n.is_file())
            .filter_map(|n| {
                let content = txn.get_text(n.id.as_str())?.get_string(&txn);
                if content.is_empty() {
                    return None;
                }
                let backed = n.blob().map(|b| b.sha256.as_str())
                    == Some(sha256_hex(content.as_bytes()).as_str());
                backed.then(|| n.id.clone())
            })
            .collect()
    };
    if to_strip.is_empty() {
        return;
    }

    let mut txn = doc.transact_mut();
    for id in to_strip {
        if let Some(text) = txn.get_text(id.as_str()) {
            let len = text.len(&txn);
            if len > 0 {
                text.remove_range(&mut txn, 0, len);
            }
        }
    }
}

/// After a room is (re)built, refill any file whose text overlay is empty but
/// whose blob is non-empty — the resting snapshot was stripped of those bytes.
/// Fetches the blobs off-thread and applies them via [`Command::ApplyRemat`]. A
/// cold room (full snapshot, or freshly seeded) has no empty overlays, so this
/// is a no-op there.
fn rematerialize(
    project_id: ObjectId,
    room: &RoomState,
    store: &ProjectStore,
    cmd_tx: &UnboundedSender<Command>,
) {
    // Sync (holds the doc): (id, blob sha) for each empty-overlay file.
    let needed: Vec<(String, String)> = {
        let doc = room.awareness.doc();
        let nodes = nodes_map(doc);
        let txn = doc.transact();
        let Ok(tree) = read_tree(&txn, &nodes) else {
            return;
        };
        tree.iter()
            .filter(|n| n.is_file())
            .filter_map(|n| {
                let empty = txn
                    .get_text(n.id.as_str())
                    .is_none_or(|t| t.len(&txn) == 0);
                let blob = n.blob()?;
                (empty && blob.sha256 != EMPTY_SHA256)
                    .then(|| (n.id.clone(), blob.sha256.clone()))
            })
            .collect()
    };
    if needed.is_empty() {
        return;
    }

    let store = store.clone();
    let cmd_tx = cmd_tx.clone();
    tokio::task::spawn_local(async move {
        let pid = project_id.to_hex();
        let mut texts: Vec<(String, String)> = Vec::new();
        for (id, sha) in needed {
            match store.get_blob(&pid, &sha).await {
                Ok(Some(bytes)) => match String::from_utf8(bytes) {
                    Ok(text) => texts.push((id, text)),
                    Err(_) => warn!("remat: blob {sha} in {pid} is not UTF-8"),
                },
                Ok(None) => warn!("remat: blob {sha} missing in {pid}"),
                Err(e) => warn!("remat: fetch {sha} in {pid} failed: {e:?}"),
            }
        }
        if !texts.is_empty() {
            let _ = cmd_tx.send(Command::ApplyRemat { project_id, texts });
        }
    });
}

/// Insert rematerialized text into still-empty overlays (see [`rematerialize`])
/// and broadcast, so every connection gets the bytes the stripped snapshot
/// omitted. Skips an overlay that is no longer empty (a peer already typed, or a
/// duplicate apply). Does **not** mark the room dirty — the content came from a
/// blob and is already durable, so re-snapshotting it would just re-bloat the
/// resting snapshot.
fn apply_remat(room: &mut RoomState, texts: Vec<(String, String)>) {
    let (_, updates) = capture_doc_updates(&mut room.awareness, |awareness| {
        let mut txn = awareness.doc().transact_mut();
        for (id, text) in texts {
            let Some(root) = txn.get_text(id.as_str()) else {
                continue;
            };
            if root.len(&txn) != 0 {
                continue;
            }
            root.insert(&mut txn, 0, &text);
        }
    });

    for update in updates {
        let msg = YMessage::Sync(SyncMessage::Update(update)).encode_v1();
        broadcast_all(room, &msg);
    }
}

/// Sweep orphaned blobs for one project. The reference set is every file node's
/// current blob sha *plus* the sha of every file's current text — the latter
/// covers the window between a text upload and the [`Command::FlushBlobs`] that
/// records its hash on the node, so an in-flight blob is never mistaken for an
/// orphan. Combined with the two-pass grace (`prev`), a blob is deleted only
/// when it was orphaned across two consecutive sweeps. The sweep result is
/// reported back via [`Command::GcSwept`].
fn gc_room(
    project_id: ObjectId,
    room: &RoomState,
    store: &ProjectStore,
    cmd_tx: &UnboundedSender<Command>,
    prev: HashSet<String>,
) {
    // Sync (holds the doc): the shas the live doc references right now.
    let referenced: HashSet<String> = {
        let doc = room.awareness.doc();
        let nodes = nodes_map(doc);
        let txn = doc.transact();
        let Ok(tree) = read_tree(&txn, &nodes) else {
            return;
        };
        let mut set: HashSet<String> = tree
            .iter()
            .filter_map(|n| n.blob().map(|b| b.sha256.clone()))
            .collect();
        for node in tree.iter().filter(|n| n.is_file()) {
            if let Some(text) = txn.get_text(node.id.as_str()) {
                set.insert(sha256_hex(text.get_string(&txn).as_bytes()));
            }
        }
        set
    };

    // Async: list stored blobs, delete those orphaned two sweeps running.
    let store = store.clone();
    let cmd_tx = cmd_tx.clone();
    tokio::task::spawn_local(async move {
        let pid = project_id.to_hex();
        let stored = match store.list_blobs(&pid).await {
            Ok(stored) => stored,
            Err(e) => {
                warn!("gc list failed in {pid}: {e:?}");
                return;
            }
        };
        let orphans: HashSet<String> = stored
            .into_iter()
            .filter(|sha| !referenced.contains(sha))
            .collect();
        let mut deleted = 0usize;
        for sha in orphans.intersection(&prev) {
            match store.delete_blob(&pid, sha).await {
                Ok(()) => deleted += 1,
                Err(e) => warn!("gc delete failed in {pid}: {e:?}"),
            }
        }
        if deleted > 0 || !orphans.is_empty() {
            debug!(
                project = %pid,
                orphans = orphans.len(),
                deleted,
                "gc sweep",
            );
        }
        let _ = cmd_tx.send(Command::GcSwept {
            project_id,
            orphans,
        });
    });
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn insert_conn(room: &mut RoomState) -> (ObjectId, UnboundedReceiver<Vec<u8>>) {
        let conn_id = ObjectId::new();
        let (tx, rx) = mpsc::unbounded_channel();
        room.conns.insert(conn_id, tx);
        (conn_id, rx)
    }

    /// Encode a `Sync(Update(..))` frame as if it came from an independent
    /// client doc that inserted `text` into `path` from an empty state.
    fn doc_update_frame(path: &str, text: &str) -> Vec<u8> {
        let doc = Doc::new();
        let root = doc.get_or_insert_text(path);
        {
            let mut txn = doc.transact_mut();
            root.insert(&mut txn, 0, text);
        }
        let update = doc
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default());
        YMessage::Sync(SyncMessage::Update(update)).encode_v1()
    }

    /// Encode an awareness frame as if it came from an independent client
    /// reporting `state` as its local awareness JSON. Returns the client id
    /// that frame carries alongside the encoded bytes.
    fn awareness_frame(state: &str) -> (ClientID, Vec<u8>) {
        let mut awareness = Awareness::new(Doc::new());
        awareness.set_local_state_raw(state);
        let client_id = awareness.client_id();
        let update = awareness.update().expect("awareness update");
        (client_id, YMessage::Awareness(update).encode_v1())
    }

    fn blob() -> Blob {
        Blob {
            sha256: "a".repeat(64),
            size: 0,
        }
    }

    /// Build a populated room for tests from flat root files
    /// `(id, name, text, blob)`: each becomes a file node at the tree root with
    /// its text overlay seeded. Equivalent to a cold-start (`from_tree`) plus the
    /// rematerialization that would refill the overlays from blobs, collapsed
    /// into one call. Text roots are keyed by the file **id** (as in production),
    /// not the name.
    fn seed_room(files: Vec<(ObjectId, String, String, Blob)>) -> RoomState {
        let nodes = files
            .iter()
            .map(|(id, name, _text, blob)| Node {
                id: id.to_hex(),
                parent: None,
                name: name.clone(),
                content: crate::models::tree::NodeContent::File { blob: blob.clone() },
            })
            .collect::<Vec<_>>();
        let room = RoomState::from_tree(&ProjectTree::from_nodes(nodes));
        {
            let doc = room.awareness.doc();
            let mut txn = doc.transact_mut();
            for (id, _name, text, _blob) in &files {
                if !text.is_empty() {
                    if let Some(root) = txn.get_text(id.to_hex().as_str()) {
                        root.insert(&mut txn, 0, text);
                    }
                }
            }
        }
        room
    }

    #[test]
    fn keepalive_frame_is_a_no_op_awareness_update() {
        let frame = keepalive_frame();
        assert!(!frame.is_empty(), "keepalive frame must encode");
        // It must be a well-formed awareness message carrying no clients, so a
        // client decodes it, refreshes its reconnect clock, and changes nothing.
        match YMessage::decode_v1(&frame) {
            Ok(YMessage::Awareness(update)) => assert!(
                update.clients.is_empty(),
                "keepalive must carry zero awareness clients"
            ),
            other => panic!("expected an empty Awareness frame, got {other:?}"),
        }
        // Applying it to a peer's awareness adds no participant (the no-op).
        let mut peer = Awareness::new(Doc::new());
        if let Ok(YMessage::Awareness(update)) = YMessage::decode_v1(&frame) {
            peer.apply_update(update).expect("apply keepalive update");
        }
        assert_eq!(
            peer.iter().count(),
            0,
            "keepalive must not register any awareness client on a peer"
        );
    }

    #[test]
    fn room_info_reports_aggregate_state() {
        let mut room = seed_room(vec![
            (ObjectId::new(), "main.typ".to_string(), "hello".to_string(), blob()),
            // A file with no text bytes: its overlay exists but is empty, so it
            // must not be counted as a live text overlay.
            (ObjectId::new(), "empty.typ".to_string(), String::new(), blob()),
        ]);
        let (_conn, _rx) = insert_conn(&mut room);
        room.empty_since = None; // occupied

        let info = room_info(ObjectId::new(), &room);
        assert_eq!(info.conns, 1);
        assert_eq!(info.nodes, 2, "two root files, no folders");
        assert_eq!(info.text_overlays, 1, "only main.typ carries text");
        assert!(info.empty_secs.is_none(), "occupied room has no idle clock");
        assert!(room.dirty, "a freshly seeded room needs an initial snapshot");
    }

    #[test]
    fn room_info_reports_idle_seconds_when_empty() {
        let room = seed_room(vec![]);
        // Freshly built, unoccupied: empty_since is set, so empty_secs is Some.
        let info = room_info(ObjectId::new(), &room);
        assert_eq!(info.conns, 0);
        assert!(info.empty_secs.is_some());
        assert!(!info.lsp, "no worker without an LSP config");
    }

    #[test]
    fn seed_room_keys_text_by_id_and_builds_nodes() {
        let id_a = ObjectId::new();
        let id_b = ObjectId::new();
        let room = seed_room(vec![
            (id_a, "main.typ".to_string(), "hello".to_string(), blob()),
            (id_b, "intro.typ".to_string(), String::new(), blob()),
        ]);

        let nodes = nodes_map(room.awareness.doc());
        let txn = room.awareness.doc().transact();

        // Text roots are keyed by the file id (hex), not the name.
        assert_eq!(
            txn.get_text(id_a.to_hex().as_str()).unwrap().get_string(&txn),
            "hello"
        );
        // An empty seed still declares the root type, but inserts nothing.
        assert_eq!(
            txn.get_text(id_b.to_hex().as_str()).unwrap().get_string(&txn),
            ""
        );

        let tree = read_tree(&txn, &nodes).unwrap();
        tree.validate().unwrap();
        assert_eq!(tree.path_of(&id_a.to_hex()).unwrap(), "main.typ");
        assert_eq!(tree.path_of(&id_b.to_hex()).unwrap(), "intro.typ");
    }

    #[test]
    fn test_handle_data_broadcasts_doc_update_to_others_not_sender() {
        let mut room = seed_room(vec![]);
        let (conn_a, mut rx_a) = insert_conn(&mut room);
        let (_conn_b, mut rx_b) = insert_conn(&mut room);

        let frame = doc_update_frame("a.typ", "hello");
        handle_data(&mut room, conn_a, frame);

        assert!(rx_a.try_recv().is_err());
        let received = rx_b.try_recv().expect("broadcast to other connection");
        match YMessage::decode_v1(&received) {
            Ok(YMessage::Sync(SyncMessage::Update(_))) => {}
            other => panic!("expected Sync(Update(..)), got {:?}", other),
        }

        let txn = room.awareness.doc().transact();
        assert_eq!(txn.get_text("a.typ").unwrap().get_string(&txn), "hello");
    }

    #[test]
    fn test_handle_data_sync_reply_goes_to_sender_only() {
        let mut room =
            seed_room(vec![(ObjectId::new(), "a.typ".to_string(), "hi".to_string(), blob())]);
        let (conn_a, mut rx_a) = insert_conn(&mut room);
        let (_conn_b, mut rx_b) = insert_conn(&mut room);

        let frame =
            YMessage::Sync(SyncMessage::SyncStep1(yrs::StateVector::default())).encode_v1();
        handle_data(&mut room, conn_a, frame);

        let reply = rx_a.try_recv().expect("sync reply to sender");
        match YMessage::decode_v1(&reply) {
            Ok(YMessage::Sync(SyncMessage::SyncStep2(_))) => {}
            other => panic!("expected Sync(SyncStep2(..)), got {:?}", other),
        }
        assert!(rx_b.try_recv().is_err());
    }

    #[test]
    fn test_handle_data_awareness_updates_client_owner_and_broadcasts() {
        let mut room = seed_room(vec![]);
        let (conn_a, mut rx_a) = insert_conn(&mut room);
        let (_conn_b, mut rx_b) = insert_conn(&mut room);

        let (client_id, frame) = awareness_frame(r#"{"name":"a"}"#);
        handle_data(&mut room, conn_a, frame.clone());

        assert_eq!(room.client_owner.get(&client_id), Some(&conn_a));
        assert!(rx_a.try_recv().is_err());
        let received = rx_b.try_recv().expect("broadcast to other connection");
        assert_eq!(received, frame);
    }

    #[test]
    fn test_handle_data_no_broadcast_for_a_redundant_update() {
        let mut room = seed_room(vec![]);
        let (conn_a, mut rx_a) = insert_conn(&mut room);
        let (_conn_b, mut rx_b) = insert_conn(&mut room);

        let frame = doc_update_frame("a.typ", "hello");
        handle_data(&mut room, conn_a, frame.clone());
        rx_b.try_recv().expect("first broadcast for the real change");

        // Re-applying the exact same update integrates nothing new, so the
        // doc's update observer never fires and there is no second broadcast.
        handle_data(&mut room, conn_a, frame);
        assert!(rx_a.try_recv().is_err());
        assert!(rx_b.try_recv().is_err());
    }

    #[test]
    fn test_handle_data_broadcasts_a_deletion_to_others() {
        // A deletion is the regression that motivated capturing the applied
        // update instead of diffing the state vector: deleting doesn't advance
        // the SV, so an SV diff would drop it and peers would never see it.
        let file_id = ObjectId::new();
        let key = file_id.to_hex();
        let mut room = seed_room(vec![(
            file_id,
            "a.typ".to_string(),
            "hello".to_string(),
            blob(),
        )]);

        // A peer that has already synced the room's content (so it shares the
        // same item ids), which then deletes the leading character.
        let client = Doc::new();
        let synced = room
            .awareness
            .doc()
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default());
        client
            .transact_mut()
            .apply_update(Update::decode_v1(&synced).unwrap())
            .unwrap();
        let ctext = client.get_or_insert_text(key.as_str());
        let before = client.transact().state_vector();
        {
            let mut txn = client.transact_mut();
            ctext.remove_range(&mut txn, 0, 1);
        }
        let delete_update = client.transact().encode_state_as_update_v1(&before);
        let frame = YMessage::Sync(SyncMessage::Update(delete_update)).encode_v1();

        let (conn_a, mut rx_a) = insert_conn(&mut room);
        let (_conn_b, mut rx_b) = insert_conn(&mut room);
        handle_data(&mut room, conn_a, frame);

        // The deletion reached the other peer (not just the sender)...
        let received = rx_b.try_recv().expect("deletion broadcast to other connection");
        match YMessage::decode_v1(&received) {
            Ok(YMessage::Sync(SyncMessage::Update(_))) => {}
            other => panic!("expected Sync(Update(..)), got {:?}", other),
        }
        assert!(rx_a.try_recv().is_err());

        // ...and the shared document reflects it.
        let txn = room.awareness.doc().transact();
        assert_eq!(txn.get_text(key.as_str()).unwrap().get_string(&txn), "ello");
    }

    #[test]
    fn test_reconcile_renames_a_duplicate_sibling_and_broadcasts_to_all() {
        use crate::models::tree::{Node, NodeContent, ProjectTree};

        let mut room = seed_room(vec![]);
        // Two files share "notes.typ" at the root, as two clients each creating
        // it concurrently would produce once their updates merge.
        let dup = ProjectTree::from_nodes([
            Node {
                id: "1".to_string(),
                parent: None,
                name: "notes.typ".to_string(),
                content: NodeContent::File { blob: blob() },
            },
            Node {
                id: "2".to_string(),
                parent: None,
                name: "notes.typ".to_string(),
                content: NodeContent::File { blob: blob() },
            },
        ]);
        {
            let doc = room.awareness.doc();
            let nodes = nodes_map(doc);
            let mut txn = doc.transact_mut();
            write_tree(&mut txn, &nodes, &dup);
        }

        let (_conn_a, mut rx_a) = insert_conn(&mut room);
        let (_conn_b, mut rx_b) = insert_conn(&mut room);

        reconcile_tree(&mut room);

        // The tree is unique again: the lowest id keeps the name, the other is
        // suffixed, and the result validates.
        let nodes = nodes_map(room.awareness.doc());
        let txn = room.awareness.doc().transact();
        let tree = read_tree(&txn, &nodes).unwrap();
        tree.validate().unwrap();
        assert_eq!(tree.get("1").unwrap().name, "notes.typ");
        assert_eq!(tree.get("2").unwrap().name, "notes (2).typ");

        // The correction reached *every* connection (broadcast_all, no origin
        // excluded) as a Sync update.
        for rx in [&mut rx_a, &mut rx_b] {
            let received = rx.try_recv().expect("correction broadcast");
            assert!(matches!(
                YMessage::decode_v1(&received),
                Ok(YMessage::Sync(SyncMessage::Update(_)))
            ));
        }
    }

    #[test]
    fn test_reconcile_is_a_noop_for_a_unique_tree() {
        let mut room = seed_room(vec![(
            ObjectId::new(),
            "main.typ".to_string(),
            "hi".to_string(),
            blob(),
        )]);
        let (_conn_a, mut rx_a) = insert_conn(&mut room);

        reconcile_tree(&mut room);

        // No clash, so no correction is sent.
        assert!(rx_a.try_recv().is_err());
    }

    #[test]
    fn test_apply_blobs_updates_node_blob_and_broadcasts() {
        // A file seeded with the stale placeholder blob; a flush bumps it to the
        // freshly-uploaded sha/size and broadcasts to every connection.
        let file_id = ObjectId::new();
        let key = file_id.to_hex();
        let mut room = seed_room(vec![(
            file_id,
            "main.typ".to_string(),
            "hi".to_string(),
            blob(),
        )]);
        let (_conn_a, mut rx_a) = insert_conn(&mut room);

        let fresh = Blob {
            sha256: "b".repeat(64),
            size: 5,
        };
        apply_blobs(&mut room, vec![(key.clone(), fresh)]);

        let nodes = nodes_map(room.awareness.doc());
        let txn = room.awareness.doc().transact();
        let node_blob = read_tree(&txn, &nodes).unwrap().get(&key).unwrap().blob().cloned();
        assert_eq!(
            node_blob,
            Some(Blob {
                sha256: "b".repeat(64),
                size: 5
            })
        );

        let received = rx_a.try_recv().expect("flush broadcast");
        assert!(matches!(
            YMessage::decode_v1(&received),
            Ok(YMessage::Sync(SyncMessage::Update(_)))
        ));
    }

    #[test]
    fn test_reconcile_breaks_a_cycle_and_broadcasts_to_all() {
        use crate::models::tree::{Node, NodeContent, ProjectTree};

        let folder = |id: &str, parent: &str| Node {
            id: id.to_string(),
            parent: Some(parent.to_string()),
            name: id.to_string(),
            content: NodeContent::Folder,
        };

        let mut room = seed_room(vec![]);
        // a↔b cycle, as two peers each moving one under the other would merge to.
        let cyclic = ProjectTree::from_nodes([folder("a", "b"), folder("b", "a")]);
        {
            let doc = room.awareness.doc();
            let nodes = nodes_map(doc);
            let mut txn = doc.transact_mut();
            write_tree(&mut txn, &nodes, &cyclic);
        }

        let (_conn_a, mut rx_a) = insert_conn(&mut room);

        reconcile_tree(&mut room);

        // The cycle is broken (lowest id reparented to the root) and the tree
        // now validates.
        let nodes = nodes_map(room.awareness.doc());
        let txn = room.awareness.doc().transact();
        let tree = read_tree(&txn, &nodes).unwrap();
        tree.validate().unwrap();
        assert_eq!(tree.get("a").unwrap().parent, None);
        assert_eq!(tree.get("b").unwrap().parent.as_deref(), Some("a"));

        let received = rx_a.try_recv().expect("correction broadcast");
        assert!(matches!(
            YMessage::decode_v1(&received),
            Ok(YMessage::Sync(SyncMessage::Update(_)))
        ));
    }

    #[test]
    fn test_apply_blobs_is_a_noop_when_the_blob_is_unchanged() {
        let file_id = ObjectId::new();
        let key = file_id.to_hex();
        let mut room = seed_room(vec![(
            file_id,
            "main.typ".to_string(),
            "hi".to_string(),
            blob(),
        )]);
        let (_conn_a, mut rx_a) = insert_conn(&mut room);

        // The node already carries `blob()`, so re-applying it changes nothing.
        apply_blobs(&mut room, vec![(key, blob())]);
        assert!(rx_a.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_gc_sweeps_a_blob_orphaned_across_two_passes() {
        use crate::storage::{InMemoryObjectStore, sha256_hex};

        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let store = ProjectStore::new(Arc::new(InMemoryObjectStore::new()));
                let project_id = ObjectId::new();
                let pid = project_id.to_hex();
                let file_id = ObjectId::new();

                // A room with one file `main.typ` = "hi"; its node references the
                // "hi" blob. Store that blob plus an unreferenced orphan.
                let hi = Blob {
                    sha256: sha256_hex(b"hi"),
                    size: 2,
                };
                let room = seed_room(vec![(
                    file_id,
                    "main.typ".to_string(),
                    "hi".to_string(),
                    hi.clone(),
                )]);
                store.put_blob(&pid, b"hi").await.unwrap();
                let orphan = store.put_blob(&pid, b"garbage").await.unwrap();

                let (tx, mut rx) = mpsc::unbounded_channel();

                // Pass 1 (no prior candidates): the orphan is only *reported*,
                // not deleted — the two-pass grace.
                gc_room(project_id, &room, &store, &tx, HashSet::new());
                let prev = match rx.recv().await {
                    Some(Command::GcSwept { orphans, .. }) => orphans,
                    other => panic!("expected GcSwept, got {:?}", other.is_some()),
                };
                assert!(prev.contains(&orphan.sha256));
                assert!(store.get_blob(&pid, &orphan.sha256).await.unwrap().is_some());

                // Pass 2 (orphan was pending): now it is deleted, and the
                // referenced blob survives.
                gc_room(project_id, &room, &store, &tx, prev);
                let _ = rx.recv().await;
                assert_eq!(store.get_blob(&pid, &orphan.sha256).await.unwrap(), None);
                assert!(store.get_blob(&pid, &hi.sha256).await.unwrap().is_some());
            })
            .await;
    }

    #[test]
    fn test_snapshot_round_trip_rebuilds_the_same_document() {
        // Idle eviction persists a snapshot and drops the room; a rejoin
        // rebuilds it via `from_snapshot`. The rebuild must be the *same*
        // document (text + tree), so a reconnecting client re-syncs without the
        // CRDT re-inserting — and duplicating — content.
        let file_id = ObjectId::new();
        let key = file_id.to_hex();
        let room = seed_room(vec![(
            file_id,
            "main.typ".to_string(),
            "hello".to_string(),
            blob(),
        )]);

        let snapshot = encode_doc(room.awareness.doc());
        let restored = RoomState::from_snapshot(&snapshot);

        let doc = restored.awareness.doc();
        // `nodes_map` opens its own transaction, so resolve it *before* holding
        // the read txn below — grabbing both at once would deadlock the doc.
        let nodes = nodes_map(doc);
        let txn = doc.transact();
        assert_eq!(
            txn.get_text(key.as_str()).unwrap().get_string(&txn),
            "hello"
        );
        let tree = read_tree(&txn, &nodes).unwrap();
        assert_eq!(tree.get(&key).unwrap().name, "main.typ");
        // A freshly restored, unoccupied room is again a candidate for eviction.
        assert!(restored.empty_since.is_some());
    }

    #[test]
    fn test_strip_text_empties_only_blob_backed_overlays() {
        let backed = ObjectId::new();
        let unbacked = ObjectId::new();
        let mut room = seed_room(vec![
            (
                backed,
                "a.typ".to_string(),
                "hello".to_string(),
                Blob {
                    sha256: sha256_hex(b"hello"),
                    size: 5,
                },
            ),
            // `blob()`'s sha does not match "world", so this file is not backed.
            (unbacked, "b.typ".to_string(), "world".to_string(), blob()),
        ]);

        strip_text(&mut room);

        let doc = room.awareness.doc();
        let txn = doc.transact();
        // The blob-backed overlay was emptied; the unbacked one kept its bytes.
        assert_eq!(
            txn.get_text(backed.to_hex().as_str()).unwrap().get_string(&txn),
            ""
        );
        assert_eq!(
            txn.get_text(unbacked.to_hex().as_str())
                .unwrap()
                .get_string(&txn),
            "world"
        );
    }

    #[test]
    fn test_apply_remat_fills_empty_overlays_and_broadcasts() {
        // Start from a stripped room (empty overlay), then rematerialize.
        let file_id = ObjectId::new();
        let key = file_id.to_hex();
        let mut room = seed_room(vec![(
            file_id,
            "main.typ".to_string(),
            "hello".to_string(),
            Blob {
                sha256: sha256_hex(b"hello"),
                size: 5,
            },
        )]);
        strip_text(&mut room);
        let (_conn, mut rx) = insert_conn(&mut room);

        apply_remat(&mut room, vec![(key.clone(), "hello".to_string())]);

        let doc = room.awareness.doc();
        let txn = doc.transact();
        assert_eq!(txn.get_text(key.as_str()).unwrap().get_string(&txn), "hello");
        // The refill was broadcast to every connection.
        let received = rx.try_recv().expect("remat broadcast");
        assert!(matches!(
            YMessage::decode_v1(&received),
            Ok(YMessage::Sync(SyncMessage::Update(_)))
        ));
    }

    #[tokio::test]
    async fn test_strip_then_rematerialize_round_trips_through_a_blob() {
        use crate::storage::InMemoryObjectStore;

        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let store = ProjectStore::new(Arc::new(InMemoryObjectStore::new()));
                let project_id = ObjectId::new();
                let pid = project_id.to_hex();
                let file_id = ObjectId::new();
                let key = file_id.to_hex();

                // A sizable body so the storage win is unambiguous.
                let content = "lorem ipsum ".repeat(200);
                store.put_blob(&pid, content.as_bytes()).await.unwrap();
                let mut room = seed_room(vec![(
                    file_id,
                    "main.typ".to_string(),
                    content.clone(),
                    Blob {
                        sha256: sha256_hex(content.as_bytes()),
                        size: content.len() as u64,
                    },
                )]);

                // Evict: the stripped snapshot drops the text bytes but keeps the
                // (now-empty) overlay and the tree.
                let full = encode_doc(room.awareness.doc());
                strip_text(&mut room);
                let stripped = encode_doc(room.awareness.doc());
                assert!(stripped.len() < full.len());

                let mut rebuilt = RoomState::from_snapshot(&stripped);
                {
                    let doc = rebuilt.awareness.doc();
                    let txn = doc.transact();
                    assert_eq!(txn.get_text(key.as_str()).unwrap().get_string(&txn), "");
                }

                // Rejoin rematerializes the overlay from the blob.
                let (tx, mut rx) = mpsc::unbounded_channel();
                rematerialize(project_id, &rebuilt, &store, &tx);
                let texts = match rx.recv().await {
                    Some(Command::ApplyRemat { texts, .. }) => texts,
                    other => panic!("expected ApplyRemat, got {:?}", other.is_some()),
                };
                apply_remat(&mut rebuilt, texts);

                let doc = rebuilt.awareness.doc();
                let txn = doc.transact();
                assert_eq!(
                    txn.get_text(key.as_str()).unwrap().get_string(&txn),
                    content
                );
            })
            .await;
    }

    #[test]
    fn test_retract_connection_removes_owned_awareness_and_returns_retraction() {
        let mut room = seed_room(vec![]);
        let (conn_a, _rx_a) = insert_conn(&mut room);
        let (_conn_b, _rx_b) = insert_conn(&mut room);

        let (client_id, frame) = awareness_frame(r#"{"name":"a"}"#);
        handle_data(&mut room, conn_a, frame);
        assert_eq!(room.client_owner.get(&client_id), Some(&conn_a));

        let retraction = retract_connection(&mut room, conn_a).expect("retraction message");

        assert!(!room.client_owner.contains_key(&client_id));
        assert!(room.awareness.state::<serde_json::Value>(client_id).is_none());

        match YMessage::decode_v1(&retraction) {
            Ok(YMessage::Awareness(update)) => {
                let entry = update
                    .clients
                    .get(&client_id)
                    .expect("retracted client entry present");
                assert_eq!(entry.json.as_ref(), "null");
            }
            other => panic!("expected Awareness(..) retraction, got {:?}", other),
        }
    }

    #[test]
    fn test_retract_connection_none_when_connection_owns_nothing() {
        let mut room = seed_room(vec![]);
        let (conn_a, _rx_a) = insert_conn(&mut room);
        let (conn_b, _rx_b) = insert_conn(&mut room);

        let (client_id, frame) = awareness_frame(r#"{"name":"b"}"#);
        handle_data(&mut room, conn_b, frame);
        assert_eq!(room.client_owner.get(&client_id), Some(&conn_b));

        let result = retract_connection(&mut room, conn_a);

        assert!(result.is_none());
        assert_eq!(room.client_owner.get(&client_id), Some(&conn_b));
    }
}

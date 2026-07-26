use std::{
    collections::HashMap,
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
    task::LocalSet,
    time::{Instant, interval},
};
use tracing::{debug, info, warn};
use yrs::{
    ClientID, Doc, GetString, Map, Out, ReadTxn, Text, Transact, Update,
    sync::{Awareness, DefaultProtocol, Message as YMessage, Protocol, SyncMessage},
    updates::decoder::Decode as _,
    updates::encoder::{Encode, Encoder, EncoderV1},
};

use crate::config::WsConfig;
use crate::crdt::snapshot::encode_doc;
use crate::crdt::{nodes_map, read_tree, seed::nodes_from_files, write_tree};
use crate::models::project::FileContent;
use crate::models::response::ApiResponse;
use crate::models::tree::ProjectTree;
use crate::models::user::UserClaims;
use crate::repo::project::{MongoProjectRepo, ProjectRepo};
use crate::storage::{Blob, ProjectStore};

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

/// A `(file_id, path, text, blob)` tuple used to hydrate a *fresh* room's Y.Doc
/// (nodes + text) from stored files when there is no snapshot yet. The blob is
/// the file's content already uploaded to the object store, so the file node
/// references bytes that exist. Text roots are keyed by the file's **id**
/// (stable across renames), not its path.
type SeedFile = (ObjectId, String, String, Blob);

/// Handshake and start WebSocket handler with heartbeats.
pub async fn ws(
    id: web::Path<String>,
    req: HttpRequest,
    stream: web::Payload,
    data: actix_web::web::Data<crate::AppState>,
    project_server: web::Data<ProjectServer>,
    ws_config: web::Data<WsConfig>,
    store: web::Data<ProjectStore>,
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
    // Y.Doc snapshot; otherwise seed a fresh doc from the stored files, uploading
    // each text as a blob first so its file node references bytes that exist.
    let store: &ProjectStore = store.get_ref();
    let project_hex = project_id.to_hex();
    let snapshot = store.get_snapshot(&project_hex).await.ok().flatten();
    let mut seed: Vec<SeedFile> = Vec::new();
    if snapshot.is_none() {
        for file in project.files {
            if let FileContent::Text { text } = file.content {
                match store.put_blob(&project_hex, text.as_bytes()).await {
                    Ok(blob) => seed.push((file.id, file.path, text, blob)),
                    Err(e) => warn!("seed blob upload failed for {}: {e:?}", file.id.to_hex()),
                }
            }
        }
    }

    let (res, session, stream) = match actix_ws::handle(&req, stream) {
        Ok(tuple) => tuple,
        Err(e) => return Err(WebSocketError::HandshakeFailed(e)),
    };

    rt::spawn(handle_ws(
        project_server.as_ref().clone(),
        project_id,
        snapshot,
        seed,
        session,
        stream,
        ws_config.as_ref().clone(),
    ));

    Ok(res)
}

/// Per-connection loop. Bridges this WebSocket to the single-threaded room
/// manager: client frames are forwarded as [`Command::Data`], and messages the
/// manager routes back (initial sync, peers' updates, awareness) arrive on
/// `out_rx` and are written to the socket.
async fn handle_ws(
    project_server: ProjectServer,
    project_id: ObjectId,
    snapshot: Option<Vec<u8>>,
    seed: Vec<SeedFile>,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
    ws_config: WsConfig,
) {
    let heartbeat_interval = Duration::from_secs(ws_config.heartbeat_interval_secs);
    let client_timeout = Duration::from_secs(ws_config.client_timeout_secs);
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(heartbeat_interval);

    let conn_id = ObjectId::new();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    project_server.join(project_id, snapshot, seed, conn_id, out_tx);
    info!("WS handler: joined project {}", project_id.to_hex());

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
                let _ = session.ping(b"").await;
            }

            else => break None,
        }
    };

    project_server.leave(project_id, conn_id);
    info!("WS handler: left project {}", project_id.to_hex());
    let _ = session.close(close_reason).await;
}

/// Commands sent from connection handlers (any worker thread) to the
/// single-threaded room manager. Everything here is `Send`; the `yrs` document
/// itself never leaves the manager thread.
enum Command {
    Join {
        project_id: ObjectId,
        /// Prior Y.Doc snapshot bytes, if any — restores the room directly.
        snapshot: Option<Vec<u8>>,
        /// Fallback seed (from Mongo files) used only when there is no snapshot.
        seed: Vec<SeedFile>,
        conn_id: ObjectId,
        out: UnboundedSender<Vec<u8>>,
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
        // to cross threads.
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("build room-manager runtime");
            let local = LocalSet::new();
            local.block_on(&rt, room_manager(cmd_rx, project_repo, ws_config, store));
        });
        ProjectServer { cmd_tx }
    }

    fn join(
        &self,
        project_id: ObjectId,
        snapshot: Option<Vec<u8>>,
        seed: Vec<SeedFile>,
        conn_id: ObjectId,
        out: UnboundedSender<Vec<u8>>,
    ) {
        let _ = self.cmd_tx.send(Command::Join {
            project_id,
            snapshot,
            seed,
            conn_id,
            out,
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
}

/// One live collaboration room: the shared CRDT document plus its connections.
/// Lives entirely on the room-manager thread.
struct RoomState {
    awareness: Awareness,
    conns: HashMap<ObjectId, UnboundedSender<Vec<u8>>>,
    /// Which connection last reported each awareness client id, so a
    /// connection's cursor/presence can be retracted when it leaves instead
    /// of lingering as a ghost participant (see `handle_data`/`Leave`).
    client_owner: HashMap<ClientID, ObjectId>,
    /// Last text persisted per text-root key (file id hex), to skip unchanged
    /// files. The file id list itself is derived from the `nodes` map at
    /// persist time, not tracked here.
    last: HashMap<String, String>,
    /// Whether the Y.Doc changed since the last snapshot, so persist can skip
    /// re-snapshotting an unchanged room.
    dirty: bool,
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
            last: HashMap::new(),
            dirty: false, // the snapshot we loaded is already durable
        }
    }

    /// Seed a fresh room's Y.Doc from stored files: a text root per file (keyed
    /// by id) plus the derived `nodes` tree. Used only when a project has no
    /// snapshot yet. The server is authoritative on cold start; clients connect
    /// empty and sync against this, avoiding duplicated initial content.
    fn new(seed: Vec<SeedFile>) -> RoomState {
        let doc = Doc::new();
        // Create the text roots and nodes map up front — each `get_or_insert`
        // opens its own internal txn, so it must precede the write txn below.
        let mut roots = Vec::with_capacity(seed.len());
        let mut node_files = Vec::with_capacity(seed.len());
        for (id, path, text, blob) in seed {
            let id_hex = id.to_hex();
            roots.push((doc.get_or_insert_text(id_hex.as_str()), text));
            node_files.push((id_hex, path, blob));
        }
        let nodes = nodes_map(&doc);
        let tree = ProjectTree::from_nodes(nodes_from_files(node_files));
        {
            let mut txn = doc.transact_mut();
            for (root, text) in &roots {
                if !text.is_empty() {
                    root.insert(&mut txn, 0, text);
                }
            }
            write_tree(&mut txn, &nodes, &tree);
        }
        RoomState {
            awareness: Awareness::new(doc),
            conns: HashMap::new(),
            client_owner: HashMap::new(),
            last: HashMap::new(),
            dirty: true, // a fresh seed needs an initial snapshot
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

/// Single-threaded owner of every room. Serves commands and periodically
/// flushes text to MongoDB.
async fn room_manager(
    mut cmd_rx: UnboundedReceiver<Command>,
    repo: MongoProjectRepo,
    ws_config: WsConfig,
    store: ProjectStore,
) {
    let mut rooms: HashMap<ObjectId, RoomState> = HashMap::new();
    let mut persist_tick = interval(Duration::from_secs(ws_config.persist_interval_secs));

    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(Command::Join { project_id, snapshot, seed, conn_id, out }) => {
                        let room = rooms.entry(project_id).or_insert_with(|| match &snapshot {
                            Some(bytes) => RoomState::from_snapshot(bytes),
                            None => RoomState::new(seed),
                        });
                        // Send the initial sync step 1 + awareness state.
                        let mut encoder = EncoderV1::new();
                        if DefaultProtocol.start(&room.awareness, &mut encoder).is_ok() {
                            let _ = out.send(encoder.to_vec());
                        }
                        room.conns.insert(conn_id, out);
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
                                // Keep the room (and its CRDT document) in memory
                                // even with no connections. Re-deriving the doc
                                // from text on every (re)join produces independent
                                // insertions of the same characters, which the CRDT
                                // merges into DUPLICATED content. A reconnecting
                                // client must re-sync against the SAME document.
                                // Just persist now.
                                persist_room(project_id, room, &repo, &store);
                            }
                        }
                    }
                    None => break,
                }
            }
            _ = persist_tick.tick() => {
                for (project_id, room) in rooms.iter_mut() {
                    persist_room(*project_id, room, &repo, &store);
                }
            }
        }
    }
}

/// Apply one client frame to the room's document and fan the result out.
fn handle_data(room: &mut RoomState, conn_id: ObjectId, data: Vec<u8>) {
    let is_awareness = data.first() == Some(&MSG_AWARENESS);

    // Capture the exact update(s) applied to the shared document while the
    // protocol runs, so we can relay them verbatim. We must NOT diff the state
    // vector before/after to detect changes: a deletion only adds tombstones and
    // does *not* advance the state vector, so an SV diff silently drops deletes
    // (they would reach peers only when piggy-backed on a later insertion).
    // `observe_update_v1` fires for inserts and deletes alike, and only when the
    // transaction actually changed something — so a redundant update stays a
    // no-op. The `Arc<Mutex<_>>` is to satisfy the observer's `Send + Sync`
    // bound; this all runs on the single room-manager thread, so it never
    // contends.
    let applied: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = applied.clone();
    let subscription = room
        .awareness
        .doc()
        .observe_update_v1(move |_txn, event| {
            if let Ok(mut updates) = sink.lock() {
                updates.push(event.update.clone());
            }
        });
    if let Err(e) = &subscription {
        warn!("WS: failed to observe doc updates: {e:?}");
    }

    let replies = DefaultProtocol.handle(&mut room.awareness, &data);
    drop(subscription); // stop observing before the doc is touched again

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
    let updates = std::mem::take(&mut *applied.lock().unwrap());
    if !updates.is_empty() {
        room.dirty = true;
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
    let doc = room.awareness.doc();
    let nodes = nodes_map(doc);

    // One observer spans both passes, capturing whatever they change to relay.
    let applied: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = applied.clone();
    let subscription = doc.observe_update_v1(move |_txn, event| {
        if let Ok(mut updates) = sink.lock() {
            updates.push(event.update.clone());
        }
    });

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

    drop(subscription);

    let updates = std::mem::take(&mut *applied.lock().unwrap());
    if !updates.is_empty() {
        room.dirty = true;
        for update in updates {
            let msg = YMessage::Sync(SyncMessage::Update(update)).encode_v1();
            broadcast_all(room, &msg);
        }
    }
}

/// Persist the room's Y.Doc if it changed since the last snapshot. Dual-write:
/// the whole doc (nodes + text) goes to a MinIO snapshot (the CRDT authority),
/// the derived tree projection to Mongo (the listing cache), and each changed
/// file's text back to Mongo `files` (so REST loads keep working during the
/// migration). All the `!Send` doc work happens synchronously up front; only the
/// IO is spawned onto this thread's LocalSet.
fn persist_room(
    project_id: ObjectId,
    room: &mut RoomState,
    repo: &MongoProjectRepo,
    store: &ProjectStore,
) {
    if !room.dirty {
        return;
    }

    // Phase 1 (sync, holds the doc): encode the snapshot, derive the projection,
    // and read each file's current text. Outputs are owned/`Send`.
    let (snapshot_bytes, projection, file_texts) = {
        let doc = room.awareness.doc();
        let snapshot_bytes = encode_doc(doc);
        let nodes = nodes_map(doc);
        let txn = doc.transact();
        let tree = read_tree(&txn, &nodes).ok();
        let projection = tree.as_ref().and_then(|t| t.projection().ok());
        let file_texts: Vec<(String, String)> = tree
            .as_ref()
            .map(|t| {
                t.iter()
                    .filter(|n| n.is_file())
                    .filter_map(|n| {
                        txn.get_text(n.id.as_str())
                            .map(|txt| (n.id.clone(), txt.get_string(&txn)))
                    })
                    .collect()
            })
            .unwrap_or_default();
        (snapshot_bytes, projection, file_texts)
    };

    // Dedup text against what was last persisted (sync; mutates room.last).
    let mut changed = Vec::new();
    for (id_hex, text) in file_texts {
        if room.last.get(&id_hex).is_some_and(|prev| prev == &text) {
            continue;
        }
        room.last.insert(id_hex.clone(), text.clone());
        if let Ok(file_id) = ObjectId::parse_str(&id_hex) {
            changed.push((file_id, text));
        }
    }
    room.dirty = false;

    // Phase 2 (async, no doc borrow): write to the durable stores.
    let repo = repo.clone();
    let store = store.clone();
    tokio::task::spawn_local(async move {
        let pid = project_id.to_hex();
        if let Err(e) = store.put_snapshot(&pid, &snapshot_bytes).await {
            warn!("snapshot save failed in {pid}: {e:?}");
        }
        if let Some(projection) = projection {
            if let Err(e) = repo.update_tree(project_id, projection).await {
                warn!("projection update failed in {pid}: {e:?}");
            }
        }
        for (file_id, text) in changed {
            let size = text.len() as i64;
            if let Err(e) = repo
                .update_file_content(project_id, file_id, FileContent::Text { text }, size)
                .await
            {
                warn!("text persist failed in {pid}: {e:?}");
            }
        }
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

    #[test]
    fn test_room_state_new_seeds_text_and_nodes() {
        let id_a = ObjectId::new();
        let id_b = ObjectId::new();
        let room = RoomState::new(vec![
            (id_a, "main.typ".to_string(), "hello".to_string(), blob()),
            (id_b, "chapters/intro.typ".to_string(), String::new(), blob()),
        ]);

        let nodes = nodes_map(room.awareness.doc());
        let txn = room.awareness.doc().transact();

        // Text roots are keyed by the file id (hex), not the path.
        assert_eq!(
            txn.get_text(id_a.to_hex().as_str()).unwrap().get_string(&txn),
            "hello"
        );
        // Empty seed text still declares the root type, but inserts nothing.
        assert_eq!(
            txn.get_text(id_b.to_hex().as_str()).unwrap().get_string(&txn),
            ""
        );

        // The nodes map holds both files plus the derived `chapters` folder.
        let tree = read_tree(&txn, &nodes).unwrap();
        tree.validate().unwrap();
        assert_eq!(tree.path_of(&id_a.to_hex()).unwrap(), "main.typ");
        assert_eq!(
            tree.path_of(&id_b.to_hex()).unwrap(),
            "chapters/intro.typ"
        );
    }

    #[test]
    fn test_handle_data_broadcasts_doc_update_to_others_not_sender() {
        let mut room = RoomState::new(vec![]);
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
            RoomState::new(vec![(ObjectId::new(), "a.typ".to_string(), "hi".to_string(), blob())]);
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
        let mut room = RoomState::new(vec![]);
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
        let mut room = RoomState::new(vec![]);
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
        let mut room = RoomState::new(vec![(
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

        let mut room = RoomState::new(vec![]);
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
        let mut room = RoomState::new(vec![(
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
    fn test_reconcile_breaks_a_cycle_and_broadcasts_to_all() {
        use crate::models::tree::{Node, NodeContent, ProjectTree};

        let folder = |id: &str, parent: &str| Node {
            id: id.to_string(),
            parent: Some(parent.to_string()),
            name: id.to_string(),
            content: NodeContent::Folder,
        };

        let mut room = RoomState::new(vec![]);
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
    fn test_retract_connection_removes_owned_awareness_and_returns_retraction() {
        let mut room = RoomState::new(vec![]);
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
        let mut room = RoomState::new(vec![]);
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

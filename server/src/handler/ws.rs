use std::{collections::HashMap, thread, time::Duration};

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
    ClientID, Doc, GetString, ReadTxn, Text, Transact,
    sync::{Awareness, DefaultProtocol, Message as YMessage, Protocol, SyncMessage},
    updates::decoder::Decode as _,
    updates::encoder::{Encode, Encoder, EncoderV1},
};

use crate::config::WsConfig;
use crate::models::project::FileContent;
use crate::models::response::ApiResponse;
use crate::models::user::UserClaims;
use crate::repo::project::{MongoProjectRepo, ProjectRepo};

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

/// A `(file_id, path, text)` triple used to hydrate a room from stored files.
type FileSeed = (ObjectId, String, String);

/// Handshake and start WebSocket handler with heartbeats.
pub async fn ws(
    id: web::Path<String>,
    req: HttpRequest,
    stream: web::Payload,
    data: actix_web::web::Data<crate::AppState>,
    project_server: web::Data<ProjectServer>,
    ws_config: web::Data<WsConfig>,
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

    // Seed data to hydrate the room's CRDT document from the stored text files.
    // Only the *first* connection to a project uses it; later joiners sync
    // against the already-live document.
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
    let seed: Vec<FileSeed> = project
        .files
        .into_iter()
        .filter_map(|file| match file.content {
            FileContent::Text { text } => Some((file.id, file.path, text)),
            FileContent::Binary { .. } => None,
        })
        .collect();

    let (res, session, stream) = match actix_ws::handle(&req, stream) {
        Ok(tuple) => tuple,
        Err(e) => return Err(WebSocketError::HandshakeFailed(e)),
    };

    rt::spawn(handle_ws(
        project_server.as_ref().clone(),
        project_id,
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
    seed: Vec<FileSeed>,
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
    project_server.join(project_id, seed, conn_id, out_tx);
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
        seed: Vec<FileSeed>,
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
    pub fn new(project_repo: MongoProjectRepo, ws_config: WsConfig) -> Self {
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
            local.block_on(&rt, room_manager(cmd_rx, project_repo, ws_config));
        });
        ProjectServer { cmd_tx }
    }

    fn join(
        &self,
        project_id: ObjectId,
        seed: Vec<FileSeed>,
        conn_id: ObjectId,
        out: UnboundedSender<Vec<u8>>,
    ) {
        let _ = self.cmd_tx.send(Command::Join {
            project_id,
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
    /// path -> file id, for writing text snapshots back to the right file.
    files: HashMap<String, ObjectId>,
    /// Last text persisted per path, to skip unchanged files.
    last: HashMap<String, String>,
}

impl RoomState {
    fn new(seed: Vec<FileSeed>) -> RoomState {
        let doc = Doc::new();
        // Seed from stored text. The server is authoritative on cold start;
        // clients connect empty and receive this via sync, which avoids two
        // parties both inserting the initial text (CRDT would merge those into
        // duplicated content).
        let mut files = HashMap::new();
        for (id, path, text) in seed {
            let root = doc.get_or_insert_text(path.as_str());
            if !text.is_empty() {
                let mut txn = doc.transact_mut();
                root.insert(&mut txn, 0, &text);
            }
            files.insert(path, id);
        }
        RoomState {
            awareness: Awareness::new(doc),
            conns: HashMap::new(),
            client_owner: HashMap::new(),
            files,
            last: HashMap::new(),
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
) {
    let mut rooms: HashMap<ObjectId, RoomState> = HashMap::new();
    let mut persist_tick = interval(Duration::from_secs(ws_config.persist_interval_secs));

    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(Command::Join { project_id, seed, conn_id, out }) => {
                        let room = rooms.entry(project_id).or_insert_with(|| RoomState::new(seed));
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
                                // Just flush its text now.
                                persist_room(project_id, room, &repo);
                            }
                        }
                    }
                    None => break,
                }
            }
            _ = persist_tick.tick() => {
                for (project_id, room) in rooms.iter_mut() {
                    persist_room(*project_id, room, &repo);
                }
            }
        }
    }
}

/// Apply one client frame to the room's document and fan the result out.
fn handle_data(room: &mut RoomState, conn_id: ObjectId, data: Vec<u8>) {
    let is_awareness = data.first() == Some(&MSG_AWARENESS);

    // Run the protocol against the shared document, and diff the state before /
    // after to capture exactly what this frame changed.
    let before = room.awareness.doc().transact().state_vector();
    let replies = DefaultProtocol.handle(&mut room.awareness, &data);
    let doc_update = {
        let txn = room.awareness.doc().transact();
        (txn.state_vector() != before).then(|| txn.encode_state_as_update_v1(&before))
    };

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

    // Applied document changes and awareness frames go to everyone else.
    if let Some(update) = doc_update {
        let msg = YMessage::Sync(SyncMessage::Update(update)).encode_v1();
        broadcast(room, conn_id, &msg);
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

/// Flush each changed file's current CRDT text back to MongoDB. Whole-text
/// snapshot (not a delta), so the at-rest store stays plain text and REST loads,
/// preview, and PDF export never need to understand the CRDT.
fn persist_room(project_id: ObjectId, room: &mut RoomState, repo: &MongoProjectRepo) {
    let snapshot: Vec<(String, ObjectId, String)> = {
        let txn = room.awareness.doc().transact();
        room.files
            .iter()
            .filter_map(|(path, id)| {
                txn.get_text(path.as_str())
                    .map(|text| (path.clone(), *id, text.get_string(&txn)))
            })
            .collect()
    };

    for (path, id, text) in snapshot {
        if room.last.get(&path).is_some_and(|prev| prev == &text) {
            continue;
        }
        room.last.insert(path, text.clone());
        let repo = repo.clone();
        // Snapshot is already taken (no document borrow held across the await),
        // so the write can run as its own task on this thread's LocalSet.
        tokio::task::spawn_local(async move {
            let size = text.len() as i64;
            if let Err(e) = repo
                .update_file_content(project_id, id, FileContent::Text { text }, size)
                .await
            {
                warn!("WS persist failed in {}: {:?}", project_id.to_hex(), e);
            }
        });
    }
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

    #[test]
    fn test_room_state_new_seeds_text_and_files_map() {
        let id_a = ObjectId::new();
        let id_b = ObjectId::new();
        let room = RoomState::new(vec![
            (id_a, "a.typ".to_string(), "hello".to_string()),
            (id_b, "b.typ".to_string(), String::new()),
        ]);

        let txn = room.awareness.doc().transact();
        assert_eq!(txn.get_text("a.typ").unwrap().get_string(&txn), "hello");
        // Empty seed text still declares the root type, but must not insert
        // any characters into it.
        assert_eq!(txn.get_text("b.typ").unwrap().get_string(&txn), "");
        drop(txn);

        assert_eq!(room.files.get("a.typ"), Some(&id_a));
        assert_eq!(room.files.get("b.typ"), Some(&id_b));
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
        let mut room = RoomState::new(vec![(
            ObjectId::new(),
            "a.typ".to_string(),
            "hi".to_string(),
        )]);
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
    fn test_handle_data_no_broadcast_when_state_vector_unchanged() {
        let mut room = RoomState::new(vec![]);
        let (conn_a, mut rx_a) = insert_conn(&mut room);
        let (_conn_b, mut rx_b) = insert_conn(&mut room);

        let frame = doc_update_frame("a.typ", "hello");
        handle_data(&mut room, conn_a, frame.clone());
        rx_b.try_recv().expect("first broadcast for the real change");

        // Re-applying the exact same update is a no-op against the doc's
        // state vector, so it must not trigger a second broadcast.
        handle_data(&mut room, conn_a, frame);
        assert!(rx_a.try_recv().is_err());
        assert!(rx_b.try_recv().is_err());
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

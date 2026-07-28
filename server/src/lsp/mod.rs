//! A minimal LSP (JSON-RPC over stdio) client for driving a tinymist worker.
//!
//! This is only the **transport**: `Content-Length`-framed JSON-RPC, request/
//! response correlation by id, and a stream of server→client notifications
//! (the load-bearing one being `textDocument/publishDiagnostics`). It is generic
//! over any [`AsyncRead`]/[`AsyncWrite`] pair, so it drives a real child
//! process's stdio in production and an in-memory duplex in tests — no tinymist
//! binary needed to exercise the framing and correlation.
//!
//! The per-room worker that mirrors a project and fans out diagnostics lives in
//! [`room`]; see `docs/Architecture - Compilation and Project Model.md` §3.

pub mod room;

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use serde_json::{Value, json};
use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader,
};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc, oneshot};
use tracing::{debug, warn};

/// A server→client message that carries a `method` (a notification, or a
/// server-initiated request we don't answer yet). Text is owned so it can cross
/// the channel to whatever fans it out.
#[derive(Debug, Clone)]
pub struct Notification {
    pub method: String,
    pub params: Value,
}

/// Why an LSP request didn't produce a result.
#[derive(Debug)]
pub enum LspError {
    /// The peer answered with a JSON-RPC `error` object.
    Rpc(Value),
    /// The transport closed before the response arrived (worker died).
    Closed,
}

impl std::fmt::Display for LspError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LspError::Rpc(e) => write!(f, "lsp error response: {e}"),
            LspError::Closed => write!(f, "lsp transport closed"),
        }
    }
}

impl std::error::Error for LspError {}

type Pending = Arc<Mutex<HashMap<i64, oneshot::Sender<Result<Value, Value>>>>>;

/// A live LSP connection. Cheap to clone-by-`Arc` intent: one writer task owns
/// the output half, one reader task owns the input half, and requests correlate
/// through a shared pending map.
///
/// Cheaply cloneable — all state is shared (`Arc`/channel), so every clone talks
/// to the same worker. This lets both the room manager (mirroring edits) and the
/// per-connection query path hold a handle to one room's worker.
#[derive(Clone)]
pub struct LspClient {
    outgoing: mpsc::UnboundedSender<Vec<u8>>,
    pending: Pending,
    next_id: Arc<AtomicI64>,
}

impl LspClient {
    /// Wire a client to a peer's `stdin`/`stdout`. Spawns a writer task (drains
    /// outgoing frames to `writer`) and a reader task (parses frames from
    /// `reader`, routing responses to pending requests and notifications to the
    /// returned channel). Both tasks end when their half closes.
    pub fn new<R, W>(reader: R, writer: W) -> (LspClient, mpsc::UnboundedReceiver<Notification>)
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (notify_tx, notify_rx) = mpsc::unbounded_channel::<Notification>();
        let pending: Pending = Arc::new(Mutex::new(HashMap::new()));

        // Writer: frame and flush each outgoing message.
        let mut writer = writer;
        tokio::spawn(async move {
            while let Some(bytes) = outgoing_rx.recv().await {
                if writer.write_all(&bytes).await.is_err() || writer.flush().await.is_err() {
                    break;
                }
            }
        });

        // Reader: decode frames, correlate responses, forward notifications.
        let reader_pending = pending.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(reader);
            loop {
                match read_frame(&mut reader).await {
                    Ok(Some(body)) => {
                        dispatch(&body, &reader_pending, &notify_tx).await;
                    }
                    Ok(None) => break, // clean EOF
                    Err(e) => {
                        warn!("lsp read error: {e}");
                        break;
                    }
                }
            }
            // The peer is gone: drop every pending request so awaiters get
            // `Closed` instead of hanging forever.
            let mut map = reader_pending.lock().await;
            map.clear();
        });

        (
            LspClient {
                outgoing: outgoing_tx,
                pending,
                next_id: Arc::new(AtomicI64::new(1)),
            },
            notify_rx,
        )
    }

    /// Send a request and await its correlated response. `Err(Closed)` if the
    /// transport dropped before a reply.
    pub async fn request(&self, method: &str, params: Value) -> Result<Value, LspError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);

        let frame = encode_frame(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        }));
        if self.outgoing.send(frame).is_err() {
            self.pending.lock().await.remove(&id);
            return Err(LspError::Closed);
        }

        match rx.await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(err)) => Err(LspError::Rpc(err)),
            Err(_) => Err(LspError::Closed), // sender dropped = transport closed
        }
    }

    /// Fire a notification (no response expected). Best-effort: a closed
    /// transport is silently dropped, matching LSP's fire-and-forget semantics.
    pub fn notify(&self, method: &str, params: Value) {
        let frame = encode_frame(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }));
        let _ = self.outgoing.send(frame);
    }

    // ---- LSP verbs used to mirror a room's document set into the worker ----
    // These are the minimal set for P1 (diagnostics): the room is the sole
    // document owner, so only it issues these; browsers issue *queries*, not
    // edits (see the architecture doc §3, "Multi-client LSP").

    /// `initialize` + `initialized`. `root_uri` is the worker's workspace root
    /// (`file://…`), under which the project's files live. Minimal client
    /// capabilities are enough for push diagnostics.
    pub async fn initialize(&self, root_uri: &str) -> Result<Value, LspError> {
        let result = self
            .request(
                "initialize",
                json!({
                    "processId": std::process::id(),
                    "rootUri": root_uri,
                    "capabilities": {},
                }),
            )
            .await?;
        self.notify("initialized", json!({}));
        Ok(result)
    }

    /// Open a text document in the worker's in-memory overlay (no disk write).
    pub fn did_open(&self, uri: &str, text: &str) {
        self.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": "typst",
                    "version": 1,
                    "text": text,
                }
            }),
        );
    }

    /// Replace a document's overlay text (full-document sync — simplest and
    /// robust; incremental ranges are a later optimization).
    pub fn did_change(&self, uri: &str, version: i64, text: &str) {
        self.notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri, "version": version },
                "contentChanges": [ { "text": text } ],
            }),
        );
    }

    /// Close a document's overlay (e.g. the file was deleted or renamed away).
    pub fn did_close(&self, uri: &str) {
        self.notify(
            "textDocument/didClose",
            json!({ "textDocument": { "uri": uri } }),
        );
    }

    /// Select the compile entry (`tinymist.pinMain`), so diagnostics are
    /// computed against `uri` as the document root.
    pub async fn pin_main(&self, uri: &str) -> Result<Value, LspError> {
        self.request(
            "workspace/executeCommand",
            json!({ "command": "tinymist.pinMain", "arguments": [uri] }),
        )
        .await
    }
}

/// Build a `file://` URI for a project-relative path under a workspace root.
/// The worker's world resolves imports/images by path, so a file's URI is its
/// tree path joined to the room's staging root — stable across the id-keyed
/// overlay because the projection derives the path.
pub fn file_uri(workspace_root: &Path, path: &str) -> String {
    let rel = path.trim_start_matches('/');
    format!("file://{}/{}", workspace_root.display(), rel)
}

/// A tinymist LSP subprocess and the client driving it over stdio. Dropping the
/// worker leaves the child running detached; call [`Worker::shutdown`] to reap
/// it. One worker serves one room (all its collaborators) in production.
pub struct Worker {
    pub client: LspClient,
    child: Child,
}

impl Worker {
    /// Spawn `binary lsp` with piped stdio and wire an [`LspClient`] to it.
    /// `binary` is the version-pinned tinymist chosen by [`crate::config::LspConfig::binary_for`].
    /// stderr is discarded; tinymist logs there and we don't surface it in P1.
    pub fn spawn(
        binary: &Path,
    ) -> std::io::Result<(Worker, mpsc::UnboundedReceiver<Notification>)> {
        let mut child = Command::new(binary)
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()?;
        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let (client, notifications) = LspClient::new(stdout, stdin);
        Ok((Worker { client, child }, notifications))
    }

    /// Terminate the worker process and wait for it to exit.
    pub async fn shutdown(mut self) {
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
    }
}

/// Route one decoded message: a `result`/`error` with an `id` completes a
/// pending request; anything with a `method` is surfaced as a notification.
async fn dispatch(body: &[u8], pending: &Pending, notify_tx: &mpsc::UnboundedSender<Notification>) {
    let Ok(msg) = serde_json::from_slice::<Value>(body) else {
        warn!("lsp: dropping non-JSON frame");
        return;
    };

    // A response carries an id and either `result` or `error`, no `method`.
    let is_response =
        msg.get("method").is_none() && (msg.get("result").is_some() || msg.get("error").is_some());
    if is_response {
        if let Some(id) = msg.get("id").and_then(Value::as_i64) {
            if let Some(tx) = pending.lock().await.remove(&id) {
                let payload = match msg.get("error") {
                    Some(err) => Err(err.clone()),
                    None => Ok(msg.get("result").cloned().unwrap_or(Value::Null)),
                };
                let _ = tx.send(payload);
            } else {
                debug!("lsp: response for unknown id {id}");
            }
        }
        return;
    }

    // Otherwise it's a notification (or a server→client request we don't answer
    // in P1). Surface it by method; a server request's `id` is dropped for now.
    if let Some(method) = msg.get("method").and_then(Value::as_str) {
        let _ = notify_tx.send(Notification {
            method: method.to_string(),
            params: msg.get("params").cloned().unwrap_or(Value::Null),
        });
    }
}

/// Frame a JSON message as `Content-Length: N\r\n\r\n<body>` (LSP base
/// protocol). Only `Content-Length` is emitted; `Content-Type` is optional and
/// defaults correctly.
fn encode_frame(msg: &Value) -> Vec<u8> {
    let body = serde_json::to_vec(msg).expect("serialize json message");
    let mut frame = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    frame.extend_from_slice(&body);
    frame
}

/// Read one framed message body from `reader`, or `None` at a clean EOF. Parses
/// the header block for `Content-Length`, then reads exactly that many body
/// bytes. Header names are matched case-insensitively per the base protocol.
async fn read_frame<R: AsyncBufReadExt + Unpin>(
    reader: &mut R,
) -> std::io::Result<Option<Vec<u8>>> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            return Ok(None); // EOF before any header
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break; // blank line ends the header block
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().ok();
            }
        }
    }

    let len = content_length.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "lsp frame missing Content-Length",
        )
    })?;
    let mut body = vec![0u8; len];
    tokio::io::AsyncReadExt::read_exact(reader, &mut body).await?;
    Ok(Some(body))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use tokio::io::{AsyncWriteExt, duplex};

    #[test]
    fn encode_frame_prefixes_content_length() {
        let frame = encode_frame(&json!({"a": 1}));
        let text = String::from_utf8(frame).unwrap();
        assert!(text.starts_with("Content-Length: 7\r\n\r\n"));
        assert!(text.ends_with("{\"a\":1}"));
    }

    #[test]
    fn file_uri_joins_root_and_relative_path() {
        let root = Path::new("/srv/lsp/proj");
        assert_eq!(file_uri(root, "main.typ"), "file:///srv/lsp/proj/main.typ");
        // A leading slash on the path doesn't double up.
        assert_eq!(
            file_uri(root, "/chapters/intro.typ"),
            "file:///srv/lsp/proj/chapters/intro.typ"
        );
    }

    #[tokio::test]
    async fn read_frame_parses_a_framed_body() {
        let (mut a, b) = duplex(1024);
        a.write_all(b"Content-Length: 7\r\n\r\n{\"a\":1}").await.unwrap();
        drop(a);
        let mut reader = BufReader::new(b);
        let body = read_frame(&mut reader).await.unwrap().unwrap();
        assert_eq!(serde_json::from_slice::<Value>(&body).unwrap(), json!({"a": 1}));
        // Next read hits EOF.
        assert!(read_frame(&mut reader).await.unwrap().is_none());
    }

    // A fake peer that echoes each request id back as a `result`, so the client
    // can prove request/response correlation over a real duplex.
    #[tokio::test]
    async fn request_correlates_with_its_response() {
        let (client_read, mut peer_write) = duplex(4096);
        let (mut peer_read, client_write) = duplex(4096);
        let (client, _notifications) = LspClient::new(client_read, client_write);

        // Peer task: read one request frame, reply with a result echoing the id.
        tokio::spawn(async move {
            let mut reader = BufReader::new(&mut peer_read);
            let body = read_frame(&mut reader).await.unwrap().unwrap();
            let req: Value = serde_json::from_slice(&body).unwrap();
            let reply = encode_frame(&json!({
                "jsonrpc": "2.0",
                "id": req["id"],
                "result": { "echoed": req["params"]["x"] },
            }));
            peer_write.write_all(&reply).await.unwrap();
        });

        let result = client.request("ping", json!({ "x": 42 })).await.unwrap();
        assert_eq!(result, json!({ "echoed": 42 }));
    }

    #[tokio::test]
    async fn notifications_are_surfaced_by_method() {
        let (client_read, mut peer_write) = duplex(4096);
        let (_peer_read, client_write) = duplex(4096);
        let (_client, mut notifications) = LspClient::new(client_read, client_write);

        let note = encode_frame(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": { "uri": "file:///main.typ", "diagnostics": [] },
        }));
        peer_write.write_all(&note).await.unwrap();

        let got = notifications.recv().await.unwrap();
        assert_eq!(got.method, "textDocument/publishDiagnostics");
        assert_eq!(got.params["uri"], "file:///main.typ");
    }

    #[tokio::test]
    async fn request_returns_closed_when_the_transport_drops() {
        let (client_read, peer_write) = duplex(4096);
        let (_peer_read, client_write) = duplex(4096);
        let (client, _notifications) = LspClient::new(client_read, client_write);

        // Drop the peer's write half so the reader sees EOF and clears pending.
        drop(peer_write);
        let err = client.request("ping", json!({})).await.unwrap_err();
        assert!(matches!(err, LspError::Closed));
    }

    // End-to-end against a real tinymist binary. This runs for real in the
    // dedicated "LSP Integration" CI workflow, which builds tinymist and sets
    // `CADUCEUS_TINYMIST_BIN`; the coverage job excludes it via `--skip`. When
    // the env var is unset (a local `cargo test`, or a safety net) it skips
    // cleanly rather than panicking. Run it locally with the binary:
    //
    //   CADUCEUS_TINYMIST_BIN=/path/to/tinymist \
    //     cargo test -p server --lib lsp:: -- --ignored --nocapture
    //
    // Doubles as the P1-0 spike re-validation: drive initialize → pinMain →
    // publishDiagnostics purely over an in-memory `didOpen`, and assert a broken
    // doc surfaces a diagnostic with the disk untouched.
    #[tokio::test]
    #[ignore = "requires a tinymist binary; set CADUCEUS_TINYMIST_BIN"]
    async fn real_tinymist_reports_diagnostics_for_a_broken_doc() {
        let Some((worker, mut notes, root)) = spawn_real_worker().await else {
            return;
        };
        let main = file_uri(&root, "main.typ");

        // A Typst source that references an undefined variable → an error.
        worker.client.did_open(&main, "#let a = 1\n#nope\n");
        worker.client.pin_main(&main).await.ok();

        let hit = wait_for_diagnostics(&mut notes, &[main.clone()]).await;
        assert_eq!(hit.as_deref(), Some(main.as_str()), "expected a diagnostic");
        worker.shutdown().await;
    }

    // Cross-file: an error in an *imported* file must surface (the load-bearing
    // spike claim). We assert a diagnostic lands on either file — tinymist may
    // attribute it to the broken import or to the importing file's failed load.
    #[tokio::test]
    #[ignore = "requires a tinymist binary; set CADUCEUS_TINYMIST_BIN"]
    async fn real_tinymist_reports_cross_file_diagnostics() {
        let Some((worker, mut notes, root)) = spawn_real_worker().await else {
            return;
        };
        let main = file_uri(&root, "main.typ");
        let chapter = file_uri(&root, "chapter.typ");

        worker.client.did_open(&main, "#import \"chapter.typ\": *\n");
        worker.client.did_open(&chapter, "#nope\n"); // undefined in the import
        worker.client.pin_main(&main).await.ok();

        let hit = wait_for_diagnostics(&mut notes, &[main, chapter]).await;
        assert!(hit.is_some(), "expected a cross-file diagnostic");
        worker.shutdown().await;
    }

    /// Spawn a real worker (or `None` to skip when no binary), initialize it
    /// against a temp workspace root, and return the worker, its notification
    /// stream, and the root. Files are only ever in-memory overlays — nothing is
    /// written to the root on disk.
    async fn spawn_real_worker() -> Option<(Worker, mpsc::UnboundedReceiver<Notification>, PathBuf)>
    {
        let bin = match std::env::var("CADUCEUS_TINYMIST_BIN") {
            Ok(bin) => bin,
            Err(_) => {
                eprintln!("skipping: CADUCEUS_TINYMIST_BIN not set");
                return None;
            }
        };
        let root = std::env::temp_dir().join("caduceus-lsp-it");
        std::fs::create_dir_all(&root).unwrap();
        let (worker, notes) = Worker::spawn(Path::new(&bin)).unwrap();
        let root_uri = format!("file://{}", root.display());
        worker.client.initialize(&root_uri).await.expect("initialize");
        Some((worker, notes, root))
    }

    /// Wait (bounded) for a `publishDiagnostics` carrying a non-empty list for
    /// any of `uris`; returns which uri, or `None` on timeout.
    async fn wait_for_diagnostics(
        notes: &mut mpsc::UnboundedReceiver<Notification>,
        uris: &[String],
    ) -> Option<String> {
        tokio::time::timeout(std::time::Duration::from_secs(20), async {
            loop {
                let note = notes.recv().await?;
                if note.method != "textDocument/publishDiagnostics" {
                    continue;
                }
                let uri = note.params["uri"].as_str().unwrap_or_default().to_string();
                let nonempty = note.params["diagnostics"]
                    .as_array()
                    .is_some_and(|d| !d.is_empty());
                if nonempty && uris.iter().any(|u| *u == uri) {
                    return Some(uri);
                }
            }
        })
        .await
        .ok()
        .flatten()
    }
}

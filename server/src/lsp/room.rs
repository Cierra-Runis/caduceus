//! Per-room tinymist worker: the layer between the collaboration room and the
//! LSP [`Worker`]. It owns one project's worker process, mirrors the room's text
//! files into it, streams diagnostics to every connected client, and forwards
//! browser queries. One instance per live room (all its collaborators share it),
//! spawned lazily and torn down with the room — the room is the single document
//! owner (browsers issue queries, not edits); see the architecture doc §3.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use super::{LspClient, LspError, Notification, Worker, file_uri};

/// A diagnostics update for one file, fanned out to every room client. `path` is
/// the project-relative tree path (not a `file://` URI); `diagnostics` is the
/// LSP diagnostics array verbatim (empty = the file is now clean).
#[derive(Debug, Clone)]
pub struct FileDiagnostics {
    pub path: String,
    pub diagnostics: Value,
}

/// One room's worker. `start` spawns tinymist, mirrors the initial file set, and
/// begins pumping diagnostics; `did_change` mirrors a live edit; `subscribe` +
/// `latest` feed a newly-connected client; `request` forwards a query.
pub struct RoomWorker {
    worker: Worker,
    root: PathBuf,
    /// Per-file `didChange` version counter (LSP requires monotonic versions).
    /// A std mutex (not tokio's) so mirroring an edit is a synchronous call from
    /// the single-threaded room manager — no `.await` in its hot loop.
    versions: Mutex<HashMap<String, i64>>,
    /// The most recent diagnostics per file, so a client connecting mid-session
    /// sees existing problems without waiting for the next recompile.
    latest: Latest,
    diagnostics_tx: broadcast::Sender<FileDiagnostics>,
    pump: JoinHandle<()>,
}

type Latest = Arc<Mutex<HashMap<String, Value>>>;

impl RoomWorker {
    /// Spawn a worker for a project and mirror its files. `files` is
    /// `(tree_path, text)` for every text file; `entry` is the compile root's
    /// path (pinned via `tinymist.pinMain`). Nothing is written to `root` on
    /// disk — text lives only in the worker's in-memory overlay.
    pub async fn start(
        binary: &Path,
        root: PathBuf,
        files: &[(String, String)],
        entry: Option<&str>,
    ) -> Result<RoomWorker, StartError> {
        let (worker, notes) = Worker::spawn(binary).map_err(StartError::Spawn)?;

        let root_uri = format!("file://{}", root.display());
        worker
            .client
            .initialize(&root_uri)
            .await
            .map_err(StartError::Initialize)?;

        let mut versions = HashMap::new();
        for (path, text) in files {
            worker.client.did_open(&file_uri(&root, path), text);
            versions.insert(path.clone(), 1);
        }
        if let Some(entry) = entry {
            // Best-effort: a project with no valid entry still gets per-file
            // diagnostics; pinning just focuses the compile.
            let _ = worker.client.pin_main(&file_uri(&root, entry)).await;
        }

        let (diagnostics_tx, _) = broadcast::channel(256);
        let latest: Latest = Arc::new(Mutex::new(HashMap::new()));
        let pump = spawn_pump(notes, root.clone(), diagnostics_tx.clone(), latest.clone());

        Ok(RoomWorker {
            worker,
            root,
            versions: Mutex::new(versions),
            latest,
            diagnostics_tx,
            pump,
        })
    }

    /// Subscribe to the diagnostics stream. Pair with [`RoomWorker::latest`] to
    /// prime a new subscriber with the current state.
    pub fn subscribe(&self) -> broadcast::Receiver<FileDiagnostics> {
        self.diagnostics_tx.subscribe()
    }

    /// The current diagnostics per file, for priming a just-connected client.
    pub fn latest(&self) -> Vec<FileDiagnostics> {
        self.latest
            .lock()
            .unwrap()
            .iter()
            .map(|(path, diagnostics)| FileDiagnostics {
                path: path.clone(),
                diagnostics: diagnostics.clone(),
            })
            .collect()
    }

    /// Mirror a live edit: bump the file's version and push its new full text.
    /// A file not opened at `start` is opened first. Synchronous so the room
    /// manager can call it inline as CRDT text updates arrive.
    pub fn did_change(&self, path: &str, text: &str) {
        let uri = file_uri(&self.root, path);
        let mut versions = self.versions.lock().unwrap();
        match versions.get_mut(path) {
            Some(version) => {
                *version += 1;
                self.worker.client.did_change(&uri, *version, text);
            }
            None => {
                self.worker.client.did_open(&uri, text);
                versions.insert(path.to_string(), 1);
            }
        }
    }

    /// A cloneable handle to the worker's LSP client, for forwarding browser
    /// queries off the room-manager thread (the client is `Send + Clone`).
    pub fn client(&self) -> LspClient {
        self.worker.client.clone()
    }

    /// Forward a browser LSP query (completion, hover, …) to the worker.
    pub async fn request(&self, method: &str, params: Value) -> Result<Value, LspError> {
        self.worker.client.request(method, params).await
    }

    /// Kill the worker and stop the diagnostics pump.
    pub async fn shutdown(self) {
        self.pump.abort();
        self.worker.shutdown().await;
    }
}

/// Pump the worker's notifications: turn each `publishDiagnostics` into a
/// [`FileDiagnostics`] (mapping the `file://` uri back to a tree path), cache it
/// as the file's latest, and broadcast it. Ends when the worker's stream closes.
fn spawn_pump(
    mut notes: tokio::sync::mpsc::UnboundedReceiver<Notification>,
    root: PathBuf,
    tx: broadcast::Sender<FileDiagnostics>,
    latest: Latest,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(note) = notes.recv().await {
            if note.method != "textDocument/publishDiagnostics" {
                continue;
            }
            let Some(uri) = note.params["uri"].as_str() else {
                continue;
            };
            let Some(path) = uri_to_path(&root, uri) else {
                continue; // a uri outside the project (e.g. a package) — ignore
            };
            let diagnostics = note.params["diagnostics"].clone();
            latest.lock().unwrap().insert(path.clone(), diagnostics.clone());
            // `send` errors only when there are no subscribers; that's fine, the
            // latest cache still primes the next one.
            let _ = tx.send(FileDiagnostics { path, diagnostics });
        }
    })
}

/// Map a `file://<root>/<path>` uri back to its project-relative `<path>`, or
/// `None` if it isn't under the room's workspace root.
fn uri_to_path(root: &Path, uri: &str) -> Option<String> {
    let prefix = format!("file://{}/", root.display());
    uri.strip_prefix(&prefix).map(str::to_string)
}

/// Why a [`RoomWorker`] failed to start.
#[derive(Debug)]
pub enum StartError {
    Spawn(std::io::Error),
    Initialize(LspError),
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartError::Spawn(e) => write!(f, "spawn tinymist: {e}"),
            StartError::Initialize(e) => write!(f, "initialize tinymist: {e}"),
        }
    }
}

impl std::error::Error for StartError {}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn uri_to_path_strips_the_workspace_root() {
        let root = Path::new("/srv/lsp/p1");
        assert_eq!(
            uri_to_path(root, "file:///srv/lsp/p1/chapters/intro.typ").as_deref(),
            Some("chapters/intro.typ")
        );
        // A uri outside the root (a package, say) maps to nothing.
        assert_eq!(uri_to_path(root, "file:///usr/share/typst/pkg.typ"), None);
    }

    // Real end-to-end against tinymist: a broken file streams a diagnostic, and
    // a did_change that fixes it streams an empty (cleared) diagnostics list.
    // Skips when CADUCEUS_TINYMIST_BIN is unset (see the LSP Integration CI job).
    #[tokio::test]
    #[ignore = "requires a tinymist binary; set CADUCEUS_TINYMIST_BIN"]
    async fn room_worker_streams_then_clears_diagnostics() {
        use std::time::Duration;

        let Ok(bin) = std::env::var("CADUCEUS_TINYMIST_BIN") else {
            eprintln!("skipping: CADUCEUS_TINYMIST_BIN not set");
            return;
        };
        let root = std::env::temp_dir().join("caduceus-roomworker-it");
        std::fs::create_dir_all(&root).unwrap();

        let files = vec![("main.typ".to_string(), "#let a = 1\n#nope\n".to_string())];
        let rw = RoomWorker::start(Path::new(&bin), root, &files, Some("main.typ"))
            .await
            .expect("start worker");
        let mut sub = rw.subscribe();

        // A non-empty diagnostic for main.typ arrives.
        let broken = await_main_diag(&mut sub, Duration::from_secs(20)).await;
        assert!(broken.is_some_and(|d| !d.is_empty()), "expected an error");

        // Fix the file; the worker re-publishes an empty (cleared) list.
        rw.did_change("main.typ", "#let a = 1\n#a\n");
        let cleared = await_main_diag(&mut sub, Duration::from_secs(20)).await;
        assert_eq!(cleared.as_deref(), Some(&[][..]), "expected diagnostics cleared");

        rw.shutdown().await;
    }

    /// Wait (bounded) for the next diagnostics for `main.typ`, returning its
    /// diagnostics array (possibly empty), or `None` on timeout/close.
    async fn await_main_diag(
        sub: &mut broadcast::Receiver<FileDiagnostics>,
        within: std::time::Duration,
    ) -> Option<Vec<Value>> {
        tokio::time::timeout(within, async {
            loop {
                let fd = sub.recv().await.ok()?;
                if fd.path == "main.typ" {
                    return fd.diagnostics.as_array().cloned();
                }
            }
        })
        .await
        .ok()
        .flatten()
    }
}

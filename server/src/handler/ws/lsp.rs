//! Pure helpers for the LSP-over-WebSocket bridge (see the `handle_lsp_*`
//! functions in the parent module). These are stateless message transforms —
//! classifying a browser LSP frame, translating document URIs between the
//! browser's root-relative form and the worker's workspace root, and building a
//! diagnostics notification — split out to keep the room-manager module focused.

use serde_json::{Value, json};

/// What to do with one browser LSP message. The room is the sole document owner
/// and the worker is already `initialize`d, so the client's lifecycle and
/// document-sync traffic must not reach the worker.
pub(super) enum LspDispatch {
    /// Answer this request locally (id + method are handled by the bridge, e.g.
    /// `initialize`/`shutdown`) — never forwarded to the worker.
    Reply(Value),
    /// Forward this query to the worker; route the response back under `id`.
    Forward {
        id: Value,
        method: String,
        params: Value,
    },
    /// Drop it (a notification the room owns, or a request-less frame).
    Drop,
}

/// Classify one browser LSP message. `initialize` is answered with the bridge's
/// synthetic capabilities (the worker was already initialized by the room);
/// `shutdown` acks; the client's lifecycle/document-sync notifications are
/// dropped; every other request is forwarded.
pub(super) fn classify_lsp_message(msg: &Value) -> LspDispatch {
    let method = msg.get("method").and_then(|m| m.as_str());
    let id = msg.get("id").cloned();
    match (method, id) {
        (Some("initialize"), Some(id)) => LspDispatch::Reply(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "capabilities": {
                    "textDocumentSync": 1,
                    "completionProvider": { "triggerCharacters": ["#", ".", "@", "("] },
                    "hoverProvider": true,
                    "definitionProvider": true,
                    "documentSymbolProvider": true,
                },
                "serverInfo": { "name": "caduceus-tinymist-bridge" },
            },
        })),
        (Some("shutdown"), Some(id)) => {
            LspDispatch::Reply(json!({ "jsonrpc": "2.0", "id": id, "result": null }))
        }
        // The room owns document sync and the lifecycle; drop these.
        (
            Some(
                "initialized" | "exit" | "textDocument/didOpen" | "textDocument/didChange"
                | "textDocument/didClose" | "textDocument/didSave" | "$/cancelRequest",
            ),
            _,
        ) => LspDispatch::Drop,
        (Some(method), Some(id)) => LspDispatch::Forward {
            id,
            method: method.to_string(),
            params: msg.get("params").cloned().unwrap_or(Value::Null),
        },
        // A response from the client, or a request-less frame — nothing to do.
        _ => LspDispatch::Drop,
    }
}

/// A `publishDiagnostics` frame for a browser: the file's tree `path` becomes a
/// root-relative `file:///<path>` URI (the browser's workspace root is `/`).
pub(super) fn publish_diagnostics_frame(path: &str, diagnostics: &Value) -> Vec<u8> {
    let note = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": { "uri": format!("file:///{path}"), "diagnostics": diagnostics },
    });
    serde_json::to_vec(&note).unwrap_or_default()
}

/// Rewrite every `from` URI prefix to `to` throughout a JSON value. A blunt
/// string substitution over the serialized form — enough for P1, where the only
/// `file://` strings are document URIs and the two roots never collide.
pub(super) fn rewrite_uris(value: Value, from: &str, to: &str) -> Value {
    let text = value.to_string();
    if !text.contains(from) {
        return value;
    }
    match serde_json::from_str(&text.replace(from, to)) {
        Ok(rewritten) => rewritten,
        Err(_) => value,
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn classify_lsp_message_handles_lifecycle_and_forwards_queries() {
        // initialize is answered locally with capabilities, never forwarded.
        let init = json!({ "id": 1, "method": "initialize", "params": {} });
        match classify_lsp_message(&init) {
            LspDispatch::Reply(r) => {
                assert!(r["result"]["capabilities"]["hoverProvider"].as_bool().unwrap());
            }
            _ => panic!("initialize must be answered locally"),
        }
        // The client's document sync is dropped (the room owns it).
        assert!(matches!(
            classify_lsp_message(&json!({ "method": "textDocument/didChange" })),
            LspDispatch::Drop
        ));
        // A genuine query is forwarded.
        let hover = json!({ "id": 7, "method": "textDocument/hover", "params": { "x": 1 } });
        match classify_lsp_message(&hover) {
            LspDispatch::Forward { id, method, .. } => {
                assert_eq!(id, json!(7));
                assert_eq!(method, "textDocument/hover");
            }
            _ => panic!("a query must be forwarded"),
        }
    }

    #[test]
    fn rewrite_uris_swaps_root_prefixes_both_ways() {
        let browser = json!({ "textDocument": { "uri": "file:///main.typ" } });
        // Browser (root = /) → worker (root = /tmp/x).
        let worker = rewrite_uris(browser.clone(), "file:///", "file:///tmp/x/");
        assert_eq!(worker["textDocument"]["uri"], "file:///tmp/x/main.typ");
        // …and back is exactly the original.
        let back = rewrite_uris(worker, "file:///tmp/x/", "file:///");
        assert_eq!(back, browser);
    }

    #[test]
    fn publish_diagnostics_frame_uses_a_root_relative_uri() {
        let frame = publish_diagnostics_frame("chapters/intro.typ", &json!([{ "message": "oops" }]));
        let v: Value = serde_json::from_slice(&frame).unwrap();
        assert_eq!(v["method"], "textDocument/publishDiagnostics");
        assert_eq!(v["params"]["uri"], "file:///chapters/intro.typ");
        assert_eq!(v["params"]["diagnostics"][0]["message"], "oops");
    }
}

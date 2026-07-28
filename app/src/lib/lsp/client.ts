// A minimal LSP client over the server's `/ws/project/{id}/lsp` bridge.
//
// The bridge speaks vscode-ws-jsonrpc style: one bare JSON-RPC message per
// WebSocket text frame (no Content-Length framing). The *room* owns document
// sync server-side, so this client never sends didOpen/didChange — it only
// drives the lifecycle (initialize/initialized), issues queries (completion,
// hover, …), and receives `publishDiagnostics`. URIs are root-relative
// `file:///<path>`; the server translates to the worker's workspace root.

/// Called whenever the server publishes diagnostics for a file (`path` is the
/// project-relative tree path; an empty array clears the file).
export type DiagnosticsHandler = (
  path: string,
  diagnostics: LspDiagnostic[],
) => void;

/// One LSP diagnostic for a file (a subset of the spec — what the UI needs).
export interface LspDiagnostic {
  message: string;
  range: LspRange;
  /// 1 = error, 2 = warning, 3 = information, 4 = hint.
  severity?: number;
  source?: string;
}

export interface LspPosition {
  /// Zero-based.
  character: number;
  /// Zero-based.
  line: number;
}

export interface LspRange {
  end: LspPosition;
  start: LspPosition;
}

interface Pending {
  reject: (reason: unknown) => void;
  resolve: (value: unknown) => void;
}

export class LspClient {
  private nextId = 1;
  private onDiagnostics: DiagnosticsHandler;
  private pending = new Map<number, Pending>();
  // Resolves once `initialize` is acknowledged, so queries wait for readiness.
  private ready: Promise<void>;
  private ws: WebSocket;

  constructor(url: string, onDiagnostics: DiagnosticsHandler) {
    this.onDiagnostics = onDiagnostics;
    this.ws = new WebSocket(url);
    this.ready = new Promise<void>((resolve) => {
      this.ws.addEventListener('open', () => {
        // The bridge answers `initialize` synthetically (the room already
        // initialized the worker); on its reply we're ready to query.
        void this.request('initialize', {
          capabilities: {},
          processId: null,
          rootUri: 'file:///',
        }).then(() => {
          this.notify('initialized', {});
          resolve();
        });
      });
    });
    this.ws.addEventListener('message', (event) => this.onMessage(event));
  }

  dispose(): void {
    for (const { reject } of this.pending.values()) {
      reject(new Error('lsp client disposed'));
    }
    this.pending.clear();
    this.ws.close();
  }

  /// Fire a notification (no response expected).
  notify(method: string, params: unknown): void {
    this.send({ jsonrpc: '2.0', method, params });
  }

  /// Send a request and await its correlated result. Waits for `initialize`
  /// unless this *is* the initialize request.
  async request(method: string, params: unknown): Promise<unknown> {
    if (method !== 'initialize') await this.ready;
    const id = this.nextId++;
    return new Promise<unknown>((resolve, reject) => {
      this.pending.set(id, { reject, resolve });
      this.send({ id, jsonrpc: '2.0', method, params });
    });
  }

  private onMessage(event: MessageEvent): void {
    let msg: Record<string, unknown>;
    try {
      msg = JSON.parse(String(event.data));
    } catch {
      return;
    }
    // A response to one of our requests.
    if (typeof msg.id === 'number' && ('result' in msg || 'error' in msg)) {
      const entry = this.pending.get(msg.id);
      if (!entry) return;
      this.pending.delete(msg.id);
      if ('error' in msg) entry.reject(msg.error);
      else entry.resolve(msg.result);
      return;
    }
    // A server notification — diagnostics are the one we render.
    if (msg.method === 'textDocument/publishDiagnostics') {
      const params = msg.params as {
        diagnostics?: LspDiagnostic[];
        uri?: string;
      };
      if (params?.uri) {
        this.onDiagnostics(uriToPath(params.uri), params.diagnostics ?? []);
      }
    }
  }

  private send(message: unknown): void {
    if (this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }
}

// Strip the browser's root-relative `file:///` prefix back to a tree path.
function uriToPath(uri: string): string {
  return uri.replace(/^file:\/\/\//, '');
}

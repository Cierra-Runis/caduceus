import { useEffect, useMemo, useState } from 'react';

import { env } from '@/lib/env';
import { LspClient, LspDiagnostic } from '@/lib/lsp/client';

export interface LspState {
  /// The live client, for issuing queries (completion, hover) — null until the
  /// socket is created (browser-only) or after teardown.
  client: LspClient | null;
  /// Diagnostics per file path (empty entries pruned).
  diagnostics: Map<string, LspDiagnostic[]>;
  /// Aggregate counts across all files, for the status bar.
  errors: number;
  warnings: number;
}

// Connects to the project's server-side tinymist worker over
// `/ws/project/{id}/lsp` and exposes its diagnostics (plus the client for
// language queries). The room owns document sync server-side, so this only
// receives diagnostics and issues queries — it never mirrors text. Safe when
// LSP is unconfigured server-side: the socket idles and no diagnostics arrive.
export function useLsp(projectId: string): LspState {
  const [client, setClient] = useState<LspClient | null>(null);
  const [diagnostics, setDiagnostics] = useState<Map<string, LspDiagnostic[]>>(
    () => new Map(),
  );

  useEffect(() => {
    const url = `${env.NEXT_PUBLIC_WS_URL}/project/${projectId}/lsp`;
    const lsp = new LspClient(url, (path, list) => {
      setDiagnostics((prev) => {
        const next = new Map(prev);
        if (list.length === 0) next.delete(path);
        else next.set(path, list);
        return next;
      });
    });
    setClient(lsp);
    return () => {
      lsp.dispose();
      setClient(null);
      setDiagnostics(new Map());
    };
  }, [projectId]);

  return useMemo(() => {
    let errors = 0;
    let warnings = 0;
    for (const list of diagnostics.values()) {
      for (const d of list) {
        // LSP severity: 1 error, 2 warning, 3 info, 4 hint. Missing = error.
        if (d.severity === 2) warnings += 1;
        else if (d.severity === undefined || d.severity === 1) errors += 1;
      }
    }
    return { client, diagnostics, errors, warnings };
  }, [client, diagnostics]);
}

'use client';

import { RefObject, useEffect, useRef, useState } from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';

import {
    compileProject,
    registerFonts,
    TypstAssetFile,
    TypstSourceFile,
} from '@/lib/typst';
import { cn } from '@/lib/utils';

export interface PreviewPanelProps {
  /// Non-font binary assets (images, data, …) the document may load, as bytes.
  assets: TypstAssetFile[];
  entryPath: null | string;
  files: Record<string, string>;
  /// Identity of the current font set, so fonts are registered only on change.
  fontKey: string;
  /// Raw bytes of the project's font files, registered into the font book.
  fonts: Uint8Array[];
  previewPanelRef: RefObject<null | PanelImperativeHandle>;
}

// Debounced client-side compile loop: project files -> typst.ts WASM -> SVG.
// Each user compiles their own buffers locally, so the preview has zero
// round-trip latency and the server stays out of the hot path.
const DEBOUNCE_MS = 200;

export function PreviewPanel({
  assets,
  entryPath,
  files,
  fontKey,
  fonts,
  previewPanelRef,
}: PreviewPanelProps) {
  const [svg, setSvg] = useState('');
  const [diagnostics, setDiagnostics] = useState<string[]>([]);
  // A thrown, non-Typst failure (WASM/setup) distinct from compile diagnostics.
  const [fatal, setFatal] = useState<null | string>(null);
  // Monotonic counter so a slow compile can't overwrite a newer one's result.
  const seq = useRef(0);

  useEffect(() => {
    if (!entryPath) {
      setFatal('This project has no entry file to compile.');
      setSvg('');
      setDiagnostics([]);
      return;
    }
    // `.typ` files go in as editable source; every other text file is a data
    // asset the document may `#read`/`#image`, so it is shadowed as bytes
    // alongside the binary assets.
    const sources: TypstSourceFile[] = [];
    const textAssets: TypstAssetFile[] = [];
    for (const [path, text] of Object.entries(files)) {
      if (path.endsWith('.typ')) sources.push({ path, text });
      else textAssets.push({ bytes: new TextEncoder().encode(text), path });
    }
    const allAssets = [...textAssets, ...assets];
    const timer = setTimeout(async () => {
      const mySeq = ++seq.current;
      try {
        // Fonts must be in the book before the compile that uses them; this
        // no-ops once the set is registered.
        await registerFonts(fonts, fontKey);
        const out = await compileProject(entryPath, sources, allAssets);
        if (mySeq === seq.current) {
          setSvg(out.svg);
          setDiagnostics(out.diagnostics);
          setFatal(null);
        }
      } catch (e) {
        if (mySeq === seq.current) {
          setFatal(e instanceof Error ? e.message : String(e));
        }
      }
    }, DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [assets, entryPath, files, fontKey, fonts]);

  return (
    <Panel
      collapsible
      defaultSize={50}
      id='preview'
      minSize={20}
      panelRef={previewPanelRef}
    >
      <div className='flex h-full flex-col'>
        <div className='flex-1 overflow-auto p-4'>
          {fatal ? (
            <pre className='text-sm whitespace-pre-wrap text-red-500'>
              {fatal}
            </pre>
          ) : (
            <div dangerouslySetInnerHTML={{ __html: svg }} />
          )}
        </div>
        {diagnostics.length > 0 && (
          <DiagnosticsList diagnostics={diagnostics} />
        )}
      </div>
    </Panel>
  );
}

// Compile diagnostics (warnings + errors), each a `unix`-formatted line such as
// `main.typ:2:9-3:15: warning: unknown font family: nw`. Errors are colored red,
// everything else amber.
function DiagnosticsList({ diagnostics }: { diagnostics: string[] }) {
  return (
    <ul
      className={`
        max-h-40 shrink-0 overflow-auto border-t bg-muted/30 font-mono text-xs
      `}
    >
      {diagnostics.map((line, index) => (
        <li
          className={cn(
            'border-b px-3 py-1 whitespace-pre-wrap last:border-b-0',
            / error:/.test(line) ? 'text-red-500' : 'text-amber-600',
          )}
          key={`${index}-${line}`}
        >
          {line}
        </li>
      ))}
    </ul>
  );
}

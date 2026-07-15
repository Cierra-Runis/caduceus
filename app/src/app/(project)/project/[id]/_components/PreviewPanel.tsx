'use client';

import { RefObject, useEffect, useRef, useState } from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';

import { compileProject } from '@/lib/typst';

export interface PreviewPanelProps {
  entryPath: null | string;
  files: Record<string, string>;
  previewPanelRef: RefObject<null | PanelImperativeHandle>;
}

// Debounced client-side compile loop: project files -> typst.ts WASM -> SVG.
// Each user compiles their own buffers locally, so the preview has zero
// round-trip latency and the server stays out of the hot path.
const DEBOUNCE_MS = 200;

export function PreviewPanel({
  entryPath,
  files,
  previewPanelRef,
}: PreviewPanelProps) {
  const [svg, setSvg] = useState('');
  const [error, setError] = useState<null | string>(null);
  // Monotonic counter so a slow compile can't overwrite a newer one's result.
  const seq = useRef(0);

  useEffect(() => {
    if (!entryPath) {
      setError('This project has no entry file to compile.');
      return;
    }
    const sources = Object.entries(files).map(([path, text]) => ({
      path,
      text,
    }));
    const timer = setTimeout(async () => {
      const mySeq = ++seq.current;
      try {
        const out = await compileProject(entryPath, sources);
        if (mySeq === seq.current) {
          setSvg(out);
          setError(null);
        }
      } catch (e) {
        if (mySeq === seq.current) {
          setError(e instanceof Error ? e.message : String(e));
        }
      }
    }, DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [entryPath, files]);

  return (
    <Panel
      collapsible
      defaultSize={50}
      id='preview'
      minSize={20}
      panelRef={previewPanelRef}
    >
      <div className='h-full overflow-auto p-4'>
        {error ? (
          <pre className='text-sm whitespace-pre-wrap text-red-500'>{error}</pre>
        ) : (
          <div dangerouslySetInnerHTML={{ __html: svg }} />
        )}
      </div>
    </Panel>
  );
}

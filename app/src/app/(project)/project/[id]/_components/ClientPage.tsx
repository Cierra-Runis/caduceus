'use client';

import { GripVerticalIcon } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import {
  ImperativePanelHandle,
  PanelGroup,
  PanelResizeHandle,
} from 'react-resizable-panels';
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

import { env } from '@/lib/env';
import { ProjectDetail } from '@/lib/types/project';

import { EditorPanel } from './EditorPanel';
import { PreviewPanel } from './PreviewPanel';
import { Sidebar } from './Sidebar';
import { SidebarPanel } from './SidebarPanel';

export function ClientPage({ project }: { project: ProjectDetail }) {
  const sidebarPanelRef = useRef<ImperativePanelHandle>(null);
  const editorPanelRef = useRef<ImperativePanelHandle>(null);
  const previewPanelRef = useRef<ImperativePanelHandle>(null);

  // One Y.Doc per project; each text file is a Y.Text keyed by its path. The
  // doc is pure JS (SSR-safe); the WebSocket provider is browser-only, so it is
  // created in an effect. Persistence is server-side — the room flushes CRDT
  // text back to Mongo — so there is no client-side autosave here.
  const [ydoc] = useState(() => new Y.Doc());
  const [provider, setProvider] = useState<null | WebsocketProvider>(null);

  const textPaths = useMemo(
    () =>
      project.files
        .filter((file) => file.content.kind === 'text')
        .map((file) => file.path),
    [project],
  );
  const entry = useMemo(() => entryPath(project), [project]);
  const [focus, setFocus] = useState(() => entry ?? textPaths[0] ?? '');

  useEffect(() => {
    const ws = new WebsocketProvider(
      `${env.NEXT_PUBLIC_WS_URL}/project`,
      project.id,
      ydoc,
    );
    setProvider(ws);
    return () => ws.destroy();
  }, [project.id, ydoc]);

  // Mirror the CRDT text into React state for the preview compiler. Updates
  // arrive once the provider syncs the server-seeded content and on every edit.
  const [files, setFiles] = useState<Record<string, string>>({});
  useEffect(() => {
    const sync = () => {
      const next: Record<string, string> = {};
      for (const path of textPaths) next[path] = ydoc.getText(path).toString();
      setFiles(next);
    };
    sync();
    ydoc.on('update', sync);
    return () => ydoc.off('update', sync);
  }, [ydoc, textPaths]);

  return (
    <div className='flex h-screen'>
      <Sidebar sidebarPanelRef={sidebarPanelRef} />
      <PanelGroup direction='horizontal'>
        <SidebarPanel
          entry={entry}
          focus={focus}
          onSelect={setFocus}
          paths={textPaths}
          sidebarPanelRef={sidebarPanelRef}
        />
        <PanelResizeHandle className='flex w-4 items-center justify-center'>
          <GripVerticalIcon className='w-4' />
        </PanelResizeHandle>
        <EditorPanel
          editorPanelRef={editorPanelRef}
          path={focus}
          provider={provider}
          ydoc={ydoc}
        />
        <PanelResizeHandle className='flex w-4 items-center justify-center'>
          <GripVerticalIcon className='w-4' />
        </PanelResizeHandle>
        <PreviewPanel
          entryPath={entry}
          files={files}
          previewPanelRef={previewPanelRef}
        />
      </PanelGroup>
    </div>
  );
}

// The compile root, resolved from the project's `entry` (a file id) to its path.
function entryPath(project: ProjectDetail): null | string {
  const entry = project.files.find((file) => file.id === project.entry);
  return entry?.content.kind === 'text' ? entry.path : null;
}

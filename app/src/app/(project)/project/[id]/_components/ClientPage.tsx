'use client';

import { GripVerticalIcon } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import {
    Group,
    Separator,
    usePanelRef,
} from 'react-resizable-panels';
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

import { useUserMe } from '@/hooks/api/user/me';
import { env } from '@/lib/env';
import { ProjectDetail } from '@/lib/types/project';
import { presenceColor, PresenceUser, syncRemoteCursorStyles } from '@/lib/yjs/presence';

import { EditorPanel } from './EditorPanel';
import { PresenceBar } from './PresenceBar';
import { PreviewPanel } from './PreviewPanel';
import { Sidebar } from './Sidebar';
import { SidebarPanel } from './SidebarPanel';

export function ClientPage({ project }: { project: ProjectDetail }) {
  const sidebarPanelRef = usePanelRef();
  const editorPanelRef = usePanelRef();
  const previewPanelRef = usePanelRef();

  const { data: me } = useUserMe();
  const localUser = useMemo<null | PresenceUser>(() => {
    const user = me?.payload;
    if (!user) return null;
    return {
      avatarUri: user.avatar_uri,
      color: presenceColor(user.id),
      id: user.id,
      name: user.nickname || user.username,
    };
  }, [me]);

  // One Y.Doc per project; each text file is a Y.Text keyed by its **id** (so a
  // rename never detaches the buffer). The doc is pure JS (SSR-safe); the
  // WebSocket provider is browser-only, so it is created in an effect.
  // Persistence is server-side — the room flushes CRDT text back to Mongo — so
  // there is no client-side autosave here.
  const [ydoc] = useState(() => new Y.Doc());
  const [provider, setProvider] = useState<null | WebsocketProvider>(null);

  const textFiles = useMemo(
    () =>
      project.files
        .filter((file) => file.content.kind === 'text')
        .map((file) => ({ id: file.id, path: file.path })),
    [project],
  );
  // The compile root as a *path* (typst resolves imports/images by path).
  const entry = useMemo(() => entryPath(project), [project]);
  // The entry as a file *id*, to mark it in the id-keyed file list.
  const entryId = useMemo(() => {
    const file = project.files.find((f) => f.id === project.entry);
    return file?.content.kind === 'text' ? file.id : null;
  }, [project]);
  // `focus` is the focused file's id — the editor's Y.Text key.
  const [focus, setFocus] = useState(() => entryId ?? textFiles[0]?.id ?? '');

  useEffect(() => {
    const ws = new WebsocketProvider(
      `${env.NEXT_PUBLIC_WS_URL}/project`,
      project.id,
      ydoc,
    );
    setProvider(ws);
    return () => ws.destroy();
  }, [project.id, ydoc]);

  // Publish who we are on the shared awareness map, so peers can render our
  // avatar/cursor. Waits on the profile fetch, so a slow `user/me` doesn't
  // block the provider from connecting.
  useEffect(() => {
    if (!provider || !localUser) return;
    provider.awareness.setLocalStateField('user', localUser);
  }, [provider, localUser]);

  // Keep remote cursor/selection decorations (rendered by y-monaco from
  // awareness, see Editor.tsx) colored and labeled. Lives here rather than in
  // Editor.tsx because it only needs the provider, not the focused file.
  useEffect(() => {
    if (!provider) return;
    return syncRemoteCursorStyles(provider.awareness);
  }, [provider]);

  // Mirror the CRDT text into React state for the preview compiler. Updates
  // arrive once the provider syncs the server-seeded content and on every edit.
  const [files, setFiles] = useState<Record<string, string>>({});
  useEffect(() => {
    const sync = () => {
      const next: Record<string, string> = {};
      // Preview/compile is indexed by path, but the CRDT text is read by id.
      for (const { id, path } of textFiles) {
        next[path] = ydoc.getText(id).toString();
      }
      setFiles(next);
    };
    sync();
    ydoc.on('update', sync);
    return () => ydoc.off('update', sync);
  }, [ydoc, textFiles]);

  return (
    <div className='relative flex h-screen'>
      <div className='absolute top-2 right-2 z-10'>
        <PresenceBar me={localUser} provider={provider} />
      </div>
      <Sidebar sidebarPanelRef={sidebarPanelRef} />
      <Group orientation='horizontal'>
        <SidebarPanel
          entry={entryId}
          files={textFiles}
          focus={focus}
          onSelect={setFocus}
          sidebarPanelRef={sidebarPanelRef}
        />
        <Separator className='flex w-4 items-center justify-center'>
          <GripVerticalIcon className='w-4' />
        </Separator>
        <EditorPanel
          editorPanelRef={editorPanelRef}
          provider={provider}
          textId={focus}
          ydoc={ydoc}
        />
        <Separator className='flex w-4 items-center justify-center'>
          <GripVerticalIcon className='w-4' />
        </Separator>
        <PreviewPanel
          entryPath={entry}
          files={files}
          previewPanelRef={previewPanelRef}
        />
      </Group>
    </div>
  );
}

// The compile root, resolved from the project's `entry` (a file id) to its path.
function entryPath(project: ProjectDetail): null | string {
  const entry = project.files.find((file) => file.id === project.entry);
  return entry?.content.kind === 'text' ? entry.path : null;
}

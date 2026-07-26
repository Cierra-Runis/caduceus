'use client';

import { GripVerticalIcon } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import {
    Group,
    Separator,
    usePanelRef,
} from 'react-resizable-panels';
import { toast } from 'sonner';
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

import { useUserMe } from '@/hooks/api/user/me';
import { useProjectNodes } from '@/hooks/useProjectNodes';
import { env } from '@/lib/env';
import { ProjectDetail } from '@/lib/types/project';
import { presenceColor, PresenceUser, syncRemoteCursorStyles } from '@/lib/yjs/presence';
import { createFile, createFolder, deleteNode, fileEntries, renameNode } from '@/lib/yjs/tree';

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

  // The file tree comes from the shared CRDT `nodes` map (the server's
  // authority, seeded on cold start and synced over the provider), not the REST
  // payload — so structure edits propagate through Yjs like text does. Empty
  // until the first sync arrives. The sidebar renders the whole tree; everything
  // else here only needs the files, with their derived paths.
  const nodes = useProjectNodes(ydoc);
  const textFiles = useMemo(() => fileEntries(nodes), [nodes]);
  // The compile entry is a project-level property (a file *id*), still carried
  // by the REST payload. Resolve it to a *path* against the CRDT-derived tree
  // (typst resolves imports/images by path).
  const entryId = project.entry;
  const entry = useMemo(
    () => textFiles.find((file) => file.id === entryId)?.path ?? null,
    [textFiles, entryId],
  );
  // `focus` is the focused file's id — the editor's Y.Text key.
  const [focus, setFocus] = useState('');
  // Pick a file to focus once the tree has synced, and re-pick if the focused
  // file disappears (e.g. deleted by a peer). Prefer the entry, else the first.
  useEffect(() => {
    if (focus && textFiles.some((file) => file.id === focus)) return;
    const next = textFiles.find((file) => file.id === entryId) ?? textFiles[0];
    if (next) setFocus(next.id);
  }, [textFiles, entryId, focus]);

  // File-tree edits write straight into the CRDT `nodes` map (and text roots);
  // useProjectNodes re-renders the tree, and the room persists it. Rejected
  // names surface as a toast and keep the inline input open.
  const handleCreateFile = (name: string, parent: null | string): boolean => {
    try {
      setFocus(createFile(ydoc, name, parent));
      return true;
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Could not create file');
      return false;
    }
  };
  const handleCreateFolder = (name: string, parent: null | string): boolean => {
    try {
      createFolder(ydoc, name, parent);
      return true;
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Could not create folder');
      return false;
    }
  };
  const handleRename = (id: string, name: string): boolean => {
    try {
      renameNode(ydoc, id, name);
      return true;
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Could not rename');
      return false;
    }
  };
  const handleDelete = (id: string) => {
    try {
      deleteNode(ydoc, id);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Could not delete');
    }
  };

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
          focus={focus}
          nodes={nodes}
          onCreateFile={handleCreateFile}
          onCreateFolder={handleCreateFolder}
          onDelete={handleDelete}
          onRename={handleRename}
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

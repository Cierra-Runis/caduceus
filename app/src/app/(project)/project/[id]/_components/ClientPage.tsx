'use client';

import { GripVerticalIcon } from 'lucide-react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
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
import { uploadBlob } from '@/lib/api/blob';
import { flushProject, updateProjectSettings } from '@/lib/api/project';
import { env } from '@/lib/env';
import { AutoSavePolicy, ProjectDetail } from '@/lib/types/project';
import { presenceColor, PresenceUser, syncRemoteCursorStyles } from '@/lib/yjs/presence';
import {
    createBinaryFile,
    createFile,
    createFolder,
    deleteNode,
    fileEntries,
    isBinaryPath,
    moveNode,
    readFileBlob,
    renameNode,
} from '@/lib/yjs/tree';

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
  // Auto-save policy (VS Code's `files.autoSave`), project-level and shared.
  // Governs *when* the server materializes live text into a durable blob — not
  // durability itself (edits are always synced + snapshotted). Seeded from the
  // REST payload; changing it persists to the project.
  const [autoSave, setAutoSave] = useState<AutoSavePolicy>(
    project.settings.autoSave,
  );
  const autoSaveDelay = project.settings.autoSaveDelay;
  const handleAutoSaveChange = (next: AutoSavePolicy) => {
    const prev = autoSave;
    setAutoSave(next); // optimistic
    updateProjectSettings(project.id, {
      autoSave: next,
      autoSaveDelay,
    }).catch((error) => {
      setAutoSave(prev); // revert on failure
      toast.error(
        error instanceof Error ? error.message : 'Could not update auto-save',
      );
    });
  };

  // Ask the server to flush the room's live text into blobs now. Fire-and-
  // forget: a failure is harmless (the periodic snapshot still holds the text).
  const flush = useCallback(() => {
    void flushProject(project.id).catch(() => undefined);
  }, [project.id]);

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
  // Upload a binary file: send its bytes to the server, then create a file node
  // referencing the returned blob (no text overlay).
  const handleUpload = async (file: File) => {
    try {
      const { sha256, size } = await uploadBlob(
        project.id,
        await file.arrayBuffer(),
      );
      setFocus(createBinaryFile(ydoc, file.name, null, sha256, size));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Could not upload file');
    }
  };

  // When the focused file is binary, the editor shows a preview instead of
  // Monaco — opening it as text would create an overlay the server flushes over
  // the blob. `nodes` in scope keeps this current as the blob is (re)synced.
  const focusPath = textFiles.find((file) => file.id === focus)?.path ?? null;
  const focusBlob =
    focusPath && isBinaryPath(focusPath) ? readFileBlob(ydoc, focus) : undefined;
  const binaryFile =
    focusPath && focusBlob
      ? {
          path: focusPath,
          projectId: project.id,
          sha256: focusBlob.sha256,
          size: focusBlob.size,
        }
      : null;

  const handleMove = (id: string, parent: null | string) => {
    try {
      moveNode(ydoc, id, parent);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Could not move');
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
      // Skip binary files: they have no text overlay, and calling getText would
      // create one that the server then flushes empty over the blob.
      for (const { id, path } of textFiles) {
        if (isBinaryPath(path)) continue;
        next[path] = ydoc.getText(id).toString();
      }
      setFiles(next);
    };
    sync();
    ydoc.on('update', sync);
    return () => ydoc.off('update', sync);
  }, [ydoc, textFiles]);

  // ── Auto-save triggers (per `files.autoSave`) ────────────────────────────
  // Each policy detects its moment on the client and asks the server to flush.

  // Ctrl/Cmd+S always saves, whatever the policy (a manual save, VS Code-style).
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key === 's') {
        event.preventDefault();
        flush();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [flush]);

  // afterDelay: debounce a flush `autoSaveDelay` ms after the last edit.
  useEffect(() => {
    if (autoSave !== 'afterDelay') return;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const onUpdate = () => {
      clearTimeout(timer);
      timer = setTimeout(flush, autoSaveDelay);
    };
    ydoc.on('update', onUpdate);
    return () => {
      clearTimeout(timer);
      ydoc.off('update', onUpdate);
    };
  }, [autoSave, autoSaveDelay, ydoc, flush]);

  // onWindowChange: flush when the window/tab loses focus or is hidden.
  useEffect(() => {
    if (autoSave !== 'onWindowChange') return;
    const onHide = () => {
      if (document.visibilityState === 'hidden') flush();
    };
    window.addEventListener('blur', flush);
    document.addEventListener('visibilitychange', onHide);
    return () => {
      window.removeEventListener('blur', flush);
      document.removeEventListener('visibilitychange', onHide);
    };
  }, [autoSave, flush]);

  // onFocusChange: flush when the focused file changes (switching tabs). The
  // ref skips the initial focus assignment, which isn't a "change".
  const prevFocus = useRef(focus);
  useEffect(() => {
    if (
      autoSave === 'onFocusChange' &&
      prevFocus.current &&
      prevFocus.current !== focus
    ) {
      flush();
    }
    prevFocus.current = focus;
  }, [autoSave, focus, flush]);

  return (
    <div className='relative flex h-screen'>
      <div className='absolute top-2 right-2 z-10'>
        <PresenceBar me={localUser} provider={provider} />
      </div>
      <Sidebar sidebarPanelRef={sidebarPanelRef} />
      <Group orientation='horizontal'>
        <SidebarPanel
          autoSave={autoSave}
          entry={entryId}
          focus={focus}
          nodes={nodes}
          onAutoSaveChange={handleAutoSaveChange}
          onCreateFile={handleCreateFile}
          onCreateFolder={handleCreateFolder}
          onDelete={handleDelete}
          onMove={handleMove}
          onRename={handleRename}
          onSelect={setFocus}
          onUpload={handleUpload}
          sidebarPanelRef={sidebarPanelRef}
        />
        <Separator className='flex w-4 items-center justify-center'>
          <GripVerticalIcon className='w-4' />
        </Separator>
        <EditorPanel
          binary={binaryFile}
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

'use client';

import { GripVerticalIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
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
import { flushProject, updateProjectSettings } from '@/lib/api/project';
import { env } from '@/lib/env';
import { sha256Hex } from '@/lib/hash';
import { AutoSavePolicy, ProjectDetail } from '@/lib/types/project';
import { presenceColor, PresenceUser, syncRemoteCursorStyles } from '@/lib/yjs/presence';
import {
    createBinaryFile,
    createFile,
    createFolder,
    createTextFile,
    deleteNode,
    ensureFolderPath,
    fileEntries,
    isBinaryFile,
    moveNode,
    readFileBlob,
    renameNode,
} from '@/lib/yjs/tree';

import { EditorPanel } from './EditorPanel';
import { PresenceBar } from './PresenceBar';
import { PreviewPanel } from './PreviewPanel';
import { SettingsPanel } from './SettingsPanel';
import { Sidebar, SidebarView } from './Sidebar';
import { SidebarPanel } from './SidebarPanel';
import { StatusBar } from './StatusBar';
import { UploadDialog } from './UploadDialog';

export function ClientPage({ project }: { project: ProjectDetail }) {
  const t = useTranslations('Editor');
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
  // Which view the sidebar panel shows (files vs settings), driven by the
  // activity bar.
  const [activePanel, setActivePanel] = useState<SidebarView>('files');
  const autoSaveDelay = project.settings.autoSaveDelay;
  const handleAutoSaveChange = (next: AutoSavePolicy) => {
    const prev = autoSave;
    setAutoSave(next); // optimistic
    updateProjectSettings(project.id, {
      autoSave: next,
      autoSaveDelay,
    }).catch((error) => {
      setAutoSave(prev); // revert on failure
      toast.error(error instanceof Error ? error.message : t('errors.autoSave'));
    });
  };

  // Ask the server to flush the room's live text into blobs now. Fire-and-
  // forget: a failure is harmless (the periodic snapshot still holds the text).
  const flush = useCallback(() => {
    void flushProject(project.id).catch(() => undefined);
  }, [project.id]);

  // `focus` is the active tab's file id — the editor's Y.Text key. `openTabs`
  // is the ordered set of files open in the tab strip (VS Code-style).
  const [focus, setFocus] = useState('');
  const [openTabs, setOpenTabs] = useState<string[]>([]);

  // Open a file in a tab (adding it if new) and focus it. Used by the sidebar,
  // and after creating/uploading a file.
  const openFile = (id: string) => {
    setOpenTabs((prev) => (prev.includes(id) ? prev : [...prev, id]));
    setFocus(id);
  };
  // Close a tab; if it was active, focus the neighbor that slides into its slot
  // (or the one before it, or nothing when the last tab closes).
  const closeTab = (id: string) => {
    const index = openTabs.indexOf(id);
    const next = openTabs.filter((tab) => tab !== id);
    setOpenTabs(next);
    if (focus === id) setFocus(next[index] ?? next[index - 1] ?? '');
  };

  // Drop tabs whose file disappeared (deleted here or by a peer).
  useEffect(() => {
    setOpenTabs((prev) => {
      const next = prev.filter((id) => textFiles.some((file) => file.id === id));
      return next.length === prev.length ? prev : next;
    });
  }, [textFiles]);
  // Pick a file to focus once the tree has synced, and re-pick if the focused
  // file disappears. Prefer the entry, else the first; open it as a tab too.
  useEffect(() => {
    if (focus && textFiles.some((file) => file.id === focus)) return;
    const next = textFiles.find((file) => file.id === entryId) ?? textFiles[0];
    if (next) {
      setFocus(next.id);
      setOpenTabs((prev) => (prev.includes(next.id) ? prev : [...prev, next.id]));
    }
  }, [textFiles, entryId, focus]);

  // Per-file unsaved-changes state for the tab dots: a text file is "dirty" when
  // its live text no longer hashes to its recorded blob (i.e. the edits haven't
  // been flushed to a blob yet). Binary files never dirty (no text overlay).
  // Recomputed for open tabs on a short debounce after each doc update.
  const [dirty, setDirty] = useState<Record<string, boolean>>({});
  useEffect(() => {
    let cancelled = false;
    const recompute = async () => {
      const entries = await Promise.all(
        openTabs.map(async (id): Promise<[string, boolean]> => {
          const file = textFiles.find((f) => f.id === id);
          if (!file || isBinaryFile(ydoc, id)) return [id, false];
          const blob = readFileBlob(ydoc, id);
          const text = ydoc.getText(id).toString();
          const hash = await sha256Hex(text);
          return [id, blob ? hash !== blob.sha256 : text.length > 0];
        }),
      );
      if (!cancelled) setDirty(Object.fromEntries(entries));
    };
    void recompute();
    let timer: ReturnType<typeof setTimeout> | undefined;
    const onUpdate = () => {
      clearTimeout(timer);
      timer = setTimeout(() => void recompute(), 120);
    };
    ydoc.on('update', onUpdate);
    return () => {
      cancelled = true;
      clearTimeout(timer);
      ydoc.off('update', onUpdate);
    };
  }, [ydoc, openTabs, textFiles]);

  // Tab descriptors for the editor's tab strip (name from the tree, dirty dot).
  const tabs = useMemo(
    () =>
      openTabs.map((id) => ({
        dirty: dirty[id] ?? false,
        id,
        name: nodes.find((node) => node.id === id)?.name ?? id,
      })),
    [openTabs, nodes, dirty],
  );

  // File-tree edits write straight into the CRDT `nodes` map (and text roots);
  // useProjectNodes re-renders the tree, and the room persists it. Rejected
  // names surface as a toast and keep the inline input open.
  const handleCreateFile = (name: string, parent: null | string): boolean => {
    try {
      openFile(createFile(ydoc, name, parent));
      return true;
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('errors.createFile'));
      return false;
    }
  };
  const handleCreateFolder = (name: string, parent: null | string): boolean => {
    try {
      createFolder(ydoc, name, parent);
      return true;
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('errors.createFolder'));
      return false;
    }
  };
  const handleRename = (id: string, name: string): boolean => {
    try {
      renameNode(ydoc, id, name);
      return true;
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('errors.rename'));
      return false;
    }
  };
  const handleDelete = (id: string) => {
    try {
      deleteNode(ydoc, id);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('errors.delete'));
    }
  };
  // The upload dialog uploads blobs (with progress); this creates the file node
  // for each finished blob, recreating any folders in its relative path. A file
  // detected as text (`text` provided) becomes an editable text file; otherwise
  // a binary blob. Does NOT focus the new file — a bulk upload shouldn't hijack
  // the editor. Throws on a duplicate name so the dialog surfaces it per row.
  const [uploadOpen, setUploadOpen] = useState(false);
  const createUploadedNode = (
    relativePath: string,
    sha256: string,
    size: number,
    text?: string,
  ) => {
    const segments = relativePath.split('/').filter(Boolean);
    const name = segments.pop();
    if (!name) throw new Error('empty file name');
    const parent = ensureFolderPath(ydoc, segments);
    if (text === undefined) createBinaryFile(ydoc, name, parent, sha256, size);
    else createTextFile(ydoc, name, parent, text, sha256, size);
  };

  // When the focused file is binary, the editor shows a preview instead of
  // Monaco — opening it as text would create an overlay the server flushes over
  // the blob. `nodes` in scope keeps this current as the blob is (re)synced.
  const focusPath = textFiles.find((file) => file.id === focus)?.path ?? null;
  const focusBlob =
    focusPath && isBinaryFile(ydoc, focus) ? readFileBlob(ydoc, focus) : undefined;
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
      toast.error(error instanceof Error ? error.message : t('errors.move'));
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
        if (isBinaryFile(ydoc, id)) continue;
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

  // Ambient state for the status bar: how many open files carry unflushed
  // edits, and the entry file's own name (derived from its path).
  const dirtyCount = useMemo(
    () => Object.values(dirty).filter(Boolean).length,
    [dirty],
  );
  const entryName = entry ? (entry.split('/').pop() ?? entry) : null;

  return (
    <div className='flex h-screen flex-col'>
      <div className='relative flex min-h-0 flex-1'>
      <div className='absolute top-2 right-2 z-10'>
        <PresenceBar me={localUser} provider={provider} />
      </div>
      <Sidebar
        activePanel={activePanel}
        onSelectPanel={setActivePanel}
        sidebarPanelRef={sidebarPanelRef}
      />
      <Group orientation='horizontal'>
        {activePanel === 'files' ? (
          <SidebarPanel
            entry={entryId}
            focus={focus}
            nodes={nodes}
            onCreateFile={handleCreateFile}
            onCreateFolder={handleCreateFolder}
            onDelete={handleDelete}
            onMove={handleMove}
            onRename={handleRename}
            onSelect={openFile}
            onUpload={() => setUploadOpen(true)}
            sidebarPanelRef={sidebarPanelRef}
          />
        ) : (
          <SettingsPanel
            autoSave={autoSave}
            onAutoSaveChange={handleAutoSaveChange}
            sidebarPanelRef={sidebarPanelRef}
          />
        )}
        <Separator className='flex w-4 items-center justify-center'>
          <GripVerticalIcon className='w-4' />
        </Separator>
        <EditorPanel
          binary={binaryFile}
          editorPanelRef={editorPanelRef}
          onCloseTab={closeTab}
          onSelectTab={openFile}
          provider={provider}
          tabs={tabs}
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
      <StatusBar
        autoSave={autoSave}
        dirtyCount={dirtyCount}
        entryName={entryName}
        meId={localUser?.id ?? null}
        provider={provider}
      />
      <UploadDialog
        onOpenChange={setUploadOpen}
        onUploaded={createUploadedNode}
        open={uploadOpen}
        projectId={project.id}
      />
    </div>
  );
}

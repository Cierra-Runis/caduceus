'use client';

import { GripVerticalIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useMemo, useState } from 'react';
import {
    Group,
    Separator,
    usePanelRef,
} from 'react-resizable-panels';
import { toast } from 'sonner';
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

import { useUserMe } from '@/hooks/api/user/me';
import { deleteAsset, fetchBinaryAssets, uploadAsset } from '@/lib/api/asset';
import { env } from '@/lib/env';
import { ProjectDetail } from '@/lib/types/project';
import { TypstBinaryFile } from '@/lib/typst';
import { presenceColor, PresenceUser, syncRemoteCursorStyles } from '@/lib/yjs/presence';

import { EditorPanel } from './EditorPanel';
import { PresenceBar } from './PresenceBar';
import { PreviewPanel } from './PreviewPanel';
import { Sidebar } from './Sidebar';
import { SidebarFile, SidebarPanel } from './SidebarPanel';

export function ClientPage({ project }: { project: ProjectDetail }) {
  const t = useTranslations('Project');
  const router = useRouter();
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
  // The full tree for the sidebar: text files are editable, binary assets are
  // shown for reference (so users can `#image(...)` them) but not opened.
  const fileList = useMemo<SidebarFile[]>(
    () =>
      project.files.map((file) => ({
        id: file.id,
        kind: file.content.kind,
        path: file.path,
      })),
    [project],
  );
  const entry = useMemo(() => entryPath(project), [project]);
  const [focus, setFocus] = useState(() => entry ?? textPaths[0] ?? '');

  // Binary assets live outside the CRDT doc (they're referenced by storage key,
  // not synced text), so fetch their bytes and map them into the compiler VFS
  // for `#image(...)`. Refetched whenever the file tree changes — e.g. after an
  // upload triggers a server refresh.
  const [assets, setAssets] = useState<TypstBinaryFile[]>([]);
  useEffect(() => {
    let cancelled = false;
    fetchBinaryAssets(project)
      .then((fetched) => {
        if (!cancelled) setAssets(fetched);
      })
      .catch(() => {
        if (!cancelled) setAssets([]);
      });
    return () => {
      cancelled = true;
    };
  }, [project]);

  const [uploading, setUploading] = useState(false);
  const onUpload = useCallback(
    async (file: File) => {
      setUploading(true);
      try {
        await uploadAsset(project.id, file.name, file);
        // Re-run the server component so the new file appears in the tree and
        // its bytes are refetched for the preview.
        router.refresh();
        toast.success(t('asset.uploaded', { path: file.name }));
      } catch (error) {
        toast.error(t('asset.uploadFailed', { path: file.name }), {
          description: error instanceof Error ? error.message : String(error),
        });
      } finally {
        setUploading(false);
      }
    },
    [project.id, router, t],
  );

  const [deletingId, setDeletingId] = useState<null | string>(null);
  const onDelete = useCallback(
    async (file: SidebarFile) => {
      if (!window.confirm(t('asset.deleteConfirm', { path: file.path }))) return;
      setDeletingId(file.id);
      try {
        await deleteAsset(project.id, file.id);
        // Reload the server component so the tree drops the file and the
        // preview's binary assets are refetched.
        router.refresh();
        toast.success(t('asset.deleted', { path: file.path }));
      } catch (error) {
        toast.error(t('asset.deleteFailed', { path: file.path }), {
          description: error instanceof Error ? error.message : String(error),
        });
      } finally {
        setDeletingId(null);
      }
    },
    [project.id, router, t],
  );

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
      for (const path of textPaths) next[path] = ydoc.getText(path).toString();
      setFiles(next);
    };
    sync();
    ydoc.on('update', sync);
    return () => ydoc.off('update', sync);
  }, [ydoc, textPaths]);

  return (
    <div className='relative flex h-screen'>
      <div className='absolute top-2 right-2 z-10'>
        <PresenceBar me={localUser} provider={provider} />
      </div>
      <Sidebar sidebarPanelRef={sidebarPanelRef} />
      <Group orientation='horizontal'>
        <SidebarPanel
          deletingId={deletingId}
          entry={entry}
          files={fileList}
          focus={focus}
          onDelete={onDelete}
          onSelect={setFocus}
          onUpload={onUpload}
          sidebarPanelRef={sidebarPanelRef}
          uploading={uploading}
        />
        <Separator className='flex w-4 items-center justify-center'>
          <GripVerticalIcon className='w-4' />
        </Separator>
        <EditorPanel
          editorPanelRef={editorPanelRef}
          path={focus}
          provider={provider}
          ydoc={ydoc}
        />
        <Separator className='flex w-4 items-center justify-center'>
          <GripVerticalIcon className='w-4' />
        </Separator>
        <PreviewPanel
          assets={assets}
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

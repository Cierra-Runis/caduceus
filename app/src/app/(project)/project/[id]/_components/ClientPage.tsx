'use client';

import { GripVerticalIcon } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import {
    Group,
    Separator,
    usePanelRef,
} from 'react-resizable-panels';
import useSWR from 'swr';
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

import { useUserMe } from '@/hooks/api/user/me';
import { fetchProjectDetail, fileRawUrl } from '@/lib/api/project';
import { env } from '@/lib/env';
import { ProjectDetail } from '@/lib/types/project';
import { TypstAssetFile } from '@/lib/typst';
import { presenceColor, PresenceUser, syncRemoteCursorStyles } from '@/lib/yjs/presence';

import { EditorPanel } from './EditorPanel';
import { FileExplorerPanel } from './FileExplorerPanel';
import { PresenceBar } from './PresenceBar';
import { PreviewPanel } from './PreviewPanel';
import { Sidebar } from './Sidebar';

export function ClientPage({ project: initialProject }: { project: ProjectDetail }) {
  const sidebarPanelRef = usePanelRef();
  const editorPanelRef = usePanelRef();
  const previewPanelRef = usePanelRef();

  // The file tree is server-authoritative: seed from the server-rendered detail
  // and revalidate after any structural change so paths / ids / entry stay in
  // sync. Text *content* still flows through Yjs (below), untouched by this.
  const { data: project, mutate: refresh } = useSWR(
    `project/${initialProject.id}/detail`,
    () => fetchProjectDetail(initialProject.id).then((res) => res.payload),
    { fallbackData: initialProject },
  );

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
  const entry = useMemo(() => entryPath(project), [project]);
  const [focus, setFocus] = useState(() => entry ?? textPaths[0] ?? '');

  // The focused file's id + kind, so the center panel can render the editor
  // (text) or a read-only preview (binary).
  const focusFile = useMemo(() => {
    const file = project.files.find((f) => f.path === focus);
    return file
      ? {
          families: file.font?.families,
          fileId: file.id,
          kind: file.content.kind,
        }
      : null;
  }, [project.files, focus]);

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

  // Binary assets (images, fonts, …) the document may `#image`/`#read`. Text
  // files ride the Yjs mirror above; binaries are static bytes, so fetch each
  // once from the server and hand them to the compiler as shadow files —
  // without this, `#image("logo.png")` fails with "cannot read file".
  const binaryFiles = useMemo(
    () => project.files.filter((file) => file.content.kind === 'binary'),
    [project.files],
  );
  // A stable key so the fetch only re-runs when the set of binaries changes,
  // not on every text edit (which leaves `project.files` untouched anyway).
  const binaryKey = useMemo(
    () => binaryFiles.map((f) => `${f.id}:${f.path}`).join('|'),
    [binaryFiles],
  );
  // Fonts are registered into the font book by family, not shadowed by path, so
  // they're kept separate from the mapShadow assets. This key changes only when
  // the font set changes, so registration doesn't re-run on every edit.
  const fontKey = useMemo(
    () =>
      binaryFiles
        .filter((f) => f.font)
        .map((f) => f.id)
        .join('|'),
    [binaryFiles],
  );
  const [assets, setAssets] = useState<TypstAssetFile[]>([]);
  const [fonts, setFonts] = useState<Uint8Array[]>([]);
  useEffect(() => {
    let cancelled = false;
    Promise.all(
      binaryFiles.map(async (file) => {
        const res = await fetch(fileRawUrl(project.id, file.id), {
          credentials: 'include',
        });
        const bytes = new Uint8Array(await res.arrayBuffer());
        return { bytes, isFont: Boolean(file.font), path: file.path };
      }),
    )
      .then((loaded) => {
        if (cancelled) return;
        setAssets(
          loaded
            .filter((item) => !item.isFont)
            .map((item) => ({ bytes: item.bytes, path: item.path })),
        );
        setFonts(loaded.filter((item) => item.isFont).map((item) => item.bytes));
      })
      .catch(() => {
        // A failed asset fetch just means the preview can't load that file;
        // the compiler surfaces its own "cannot read file" error.
      });
    return () => {
      cancelled = true;
    };
    // Keyed on `binaryKey` (a digest of the binary files) so the fetch only
    // re-runs when the binary set changes; `binaryFiles` updates in lockstep.
  }, [project.id, binaryKey]);

  return (
    <div className='relative flex h-screen'>
      <div className='absolute top-2 right-2 z-10'>
        <PresenceBar me={localUser} provider={provider} />
      </div>
      <Sidebar sidebarPanelRef={sidebarPanelRef} />
      <Group orientation='horizontal'>
        <FileExplorerPanel
          focus={focus}
          onSelect={setFocus}
          project={project}
          refresh={refresh}
          sidebarPanelRef={sidebarPanelRef}
        />
        <Separator className='flex w-4 items-center justify-center'>
          <GripVerticalIcon className='w-4' />
        </Separator>
        <EditorPanel
          editorPanelRef={editorPanelRef}
          focusFile={focusFile}
          path={focus}
          projectId={project.id}
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
          fontKey={fontKey}
          fonts={fonts}
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

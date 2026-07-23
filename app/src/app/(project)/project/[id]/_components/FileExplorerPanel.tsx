'use client';

import {
    ChevronDownIcon,
    ChevronRightIcon,
    DownloadIcon,
    FileIcon,
    FilePlusIcon,
    FileTextIcon,
    FolderIcon,
    FolderOpenIcon,
    FolderPlusIcon,
    ImageIcon,
    PencilIcon,
    TargetIcon,
    Trash2Icon,
} from 'lucide-react';
import { useTranslations } from 'next-intl';
import { RefObject, useMemo, useState } from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';
import { toast } from 'sonner';

import { Button } from '@/components/ui/button';
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuTrigger,
} from '@/components/ui/context-menu';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
    apiErrorMessage,
    createFile,
    createFolder,
    deleteFile,
    deleteFolder,
    fileRawUrl,
    renameFile,
    setEntry,
} from '@/lib/api/project';
import {
    buildFileTree,
    joinPath,
    parentDir,
    TreeNode,
} from '@/lib/fileTree';
import { ProjectDetail } from '@/lib/types/project';
import { cn } from '@/lib/utils';

import { UploadDialog } from './UploadDialog';

export interface FileExplorerPanelProps {
  focus: string;
  onSelect: (path: string) => void;
  project: ProjectDetail;
  /// Revalidate the project detail after a structural change so the tree
  /// reflects the server's authoritative view.
  refresh: () => Promise<unknown>;
  sidebarPanelRef: RefObject<null | PanelImperativeHandle>;
}

/// What is currently being typed into an inline input: a brand-new file/folder
/// under `parent`, or a rename of an existing node.
type Draft =
  | { kind: 'create-file' | 'create-folder'; parent: string }
  | { kind: 'rename'; node: TreeNode };

interface TreeListProps {
  collapsed: Set<string>;
  depth: number;
  draft: Draft | null;
  entryFileId: null | string;
  focus: string;
  nodes: TreeNode[];
  onCancelDraft: () => void;
  onCommitDraft: (name: string) => void;
  onDelete: (node: TreeNode) => void;
  onDownload: (node: TreeNode) => void;
  onNewFile: (parent: string) => void;
  onNewFolder: (parent: string) => void;
  onRename: (node: TreeNode) => void;
  onSelect: (path: string) => void;
  onSetEntry: (node: TreeNode) => void;
  onToggle: (path: string) => void;
}

export function FileExplorerPanel({
  focus,
  onSelect,
  project,
  refresh,
  sidebarPanelRef,
}: FileExplorerPanelProps) {
  const t = useTranslations('FileExplorer');

  const tree = useMemo(
    () => buildFileTree(project.files, project.directories),
    [project.files, project.directories],
  );

  // Everything expanded by default, VS Code-like, but the user can collapse.
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [draft, setDraft] = useState<Draft | null>(null);
  const [busy, setBusy] = useState(false);

  const entryFileId = project.entry;

  const toggle = (path: string) =>
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });

  const expand = (path: string) =>
    setCollapsed((prev) => {
      if (!prev.has(path)) return prev;
      const next = new Set(prev);
      next.delete(path);
      return next;
    });

  async function run(op: () => Promise<unknown>, failTitle: string) {
    setBusy(true);
    try {
      await op();
      await refresh();
    } catch (error) {
      toast.error(failTitle, { description: await apiErrorMessage(error) });
    } finally {
      setBusy(false);
    }
  }

  async function commitDraft(name: string) {
    const trimmed = name.trim();
    const current = draft;
    setDraft(null);
    if (!current || !trimmed) return;

    if (current.kind === 'rename') {
      const target = joinPath(parentDir(current.node.path), trimmed);
      if (target === current.node.path) return;
      if (current.node.type === 'file' && current.node.fileId) {
        const fileId = current.node.fileId;
        await run(
          () => renameFile(project.id, fileId, target),
          t('errors.renameFailed'),
        );
      } else {
        // Folder rename is a follow-up (needs a multi-file path rewrite);
        // surface it clearly rather than silently doing nothing.
        toast.error(t('errors.renameFailed'), {
          description: t('errors.folderRenameUnsupported'),
        });
      }
      return;
    }

    const target = joinPath(current.parent, trimmed);
    if (current.kind === 'create-file') {
      await run(() => createFile(project.id, target), t('errors.createFailed'));
      onSelect(target);
    } else {
      await run(
        () => createFolder(project.id, target),
        t('errors.createFailed'),
      );
      expand(target);
    }
  }

  function startCreate(kind: 'create-file' | 'create-folder', parent: string) {
    if (parent) expand(parent);
    setDraft({ kind, parent });
  }

  return (
    <Panel
      collapsible
      defaultSize={0}
      id='sidebar'
      minSize={10}
      panelRef={sidebarPanelRef}
    >
      <div className='flex h-full flex-col'>
        <div
          className={`
            flex items-center justify-between gap-1 border-b px-3 py-2
          `}
        >
          <span
            className={`
              text-xs font-semibold tracking-wide uppercase opacity-70
            `}
          >
            {t('title')}
          </span>
          <div className='flex items-center'>
            <Button
              aria-label={t('actions.newFile')}
              disabled={busy}
              onClick={() => startCreate('create-file', '')}
              size='icon'
              title={t('actions.newFile')}
              variant='ghost'
            >
              <FilePlusIcon className='size-4' />
            </Button>
            <Button
              aria-label={t('actions.newFolder')}
              disabled={busy}
              onClick={() => startCreate('create-folder', '')}
              size='icon'
              title={t('actions.newFolder')}
              variant='ghost'
            >
              <FolderPlusIcon className='size-4' />
            </Button>
            <UploadDialog
              directories={project.directories}
              files={project.files}
              projectId={project.id}
              refresh={refresh}
            />
          </div>
        </div>

        <ScrollArea className='flex-1'>
          <ContextMenu>
            <ContextMenuTrigger asChild>
              <div className='min-h-full py-1'>
                <TreeList
                  collapsed={collapsed}
                  depth={0}
                  draft={draft}
                  entryFileId={entryFileId}
                  focus={focus}
                  nodes={tree}
                  onCancelDraft={() => setDraft(null)}
                  onCommitDraft={commitDraft}
                  onDelete={(node) =>
                    node.type === 'file' && node.fileId
                      ? run(
                          () => deleteFile(project.id, node.fileId as string),
                          t('errors.deleteFailed'),
                        )
                      : run(
                          () => deleteFolder(project.id, node.path),
                          t('errors.deleteFailed'),
                        )
                  }
                  onDownload={(node) =>
                    node.fileId &&
                    window.open(fileRawUrl(project.id, node.fileId), '_blank')
                  }
                  onNewFile={(parent) => startCreate('create-file', parent)}
                  onNewFolder={(parent) =>
                    startCreate('create-folder', parent)
                  }
                  onRename={(node) => setDraft({ kind: 'rename', node })}
                  onSelect={onSelect}
                  onSetEntry={(node) =>
                    node.fileId &&
                    run(
                      () => setEntry(project.id, node.fileId as string),
                      t('errors.setEntryFailed'),
                    )
                  }
                  onToggle={toggle}
                />
                {draft?.kind !== 'rename' && draft?.parent === '' && (
                  <DraftRow
                    depth={0}
                    icon={
                      draft.kind === 'create-folder' ? (
                        <FolderIcon className='size-4 opacity-70' />
                      ) : (
                        <FileIcon className='size-4 opacity-70' />
                      )
                    }
                    onCancel={() => setDraft(null)}
                    onCommit={commitDraft}
                    placeholder={
                      draft.kind === 'create-folder'
                        ? t('placeholders.folderName')
                        : t('placeholders.fileName')
                    }
                  />
                )}
              </div>
            </ContextMenuTrigger>
            <ContextMenuContent>
              <ContextMenuItem onClick={() => startCreate('create-file', '')}>
                <FilePlusIcon /> {t('actions.newFile')}
              </ContextMenuItem>
              <ContextMenuItem
                onClick={() => startCreate('create-folder', '')}
              >
                <FolderPlusIcon /> {t('actions.newFolder')}
              </ContextMenuItem>
            </ContextMenuContent>
          </ContextMenu>
        </ScrollArea>
      </div>
    </Panel>
  );
}

function DraftRow({
  defaultValue,
  depth,
  icon,
  onCancel,
  onCommit,
  placeholder,
}: {
  defaultValue?: string;
  depth: number;
  icon: React.ReactNode;
  onCancel: () => void;
  onCommit: (name: string) => void;
  placeholder: string;
}) {
  return (
    <div
      className='flex w-full items-center gap-1.5 py-1 pr-2'
      style={{ paddingLeft: 8 + depth * 12 }}
    >
      <span className='size-3.5 shrink-0' />
      {icon}
      <input
        autoFocus
        className={`
          w-full rounded-sm border bg-background px-1 text-sm outline-none
          focus:ring-1 focus:ring-ring
        `}
        defaultValue={defaultValue}
        onBlur={(e) => onCommit(e.currentTarget.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') onCommit(e.currentTarget.value);
          else if (e.key === 'Escape') onCancel();
        }}
        placeholder={placeholder}
      />
    </div>
  );
}

function TreeList(props: TreeListProps) {
  const { nodes } = props;
  return (
    <ul>
      {nodes.map((node) => (
        <TreeRow key={node.type + ':' + node.path} node={node} {...props} />
      ))}
    </ul>
  );
}

function TreeRow(props: { node: TreeNode } & TreeListProps) {
  const {
    collapsed,
    depth,
    draft,
    entryFileId,
    focus,
    node,
    onCancelDraft,
    onCommitDraft,
    onDelete,
    onDownload,
    onNewFile,
    onNewFolder,
    onRename,
    onSelect,
    onSetEntry,
    onToggle,
  } = props;
  const t = useTranslations('FileExplorer');

  const isRenaming = draft?.kind === 'rename' && draft.node.path === node.path;
  const isEntry = node.type === 'file' && node.fileId === entryFileId;
  const isOpen = node.type === 'folder' && !collapsed.has(node.path);

  const rowIcon =
    node.type === 'folder' ? (
      isOpen ? (
        <FolderOpenIcon className='size-4 shrink-0 opacity-70' />
      ) : (
        <FolderIcon className='size-4 shrink-0 opacity-70' />
      )
    ) : node.kind === 'binary' ? (
      <ImageIcon className='size-4 shrink-0 opacity-70' />
    ) : (
      <FileTextIcon className='size-4 shrink-0 opacity-70' />
    );

  const handleActivate = () => {
    if (node.type === 'folder') {
      onToggle(node.path);
    } else if (node.kind === 'binary') {
      onDownload(node);
    } else {
      onSelect(node.path);
    }
  };

  return (
    <li>
      <ContextMenu>
        <ContextMenuTrigger asChild>
          {isRenaming ? (
            <DraftRow
              defaultValue={node.name}
              depth={depth}
              icon={rowIcon}
              onCancel={onCancelDraft}
              onCommit={onCommitDraft}
              placeholder={node.name}
            />
          ) : (
            <button
              className={cn(
                `flex w-full items-center gap-1.5 py-1 pr-2 text-left text-sm`,
                node.path === focus ? 'bg-accent' : 'hover:bg-accent/50',
              )}
              onClick={handleActivate}
              style={{ paddingLeft: 8 + depth * 12 }}
              type='button'
            >
              {node.type === 'folder' ? (
                isOpen ? (
                  <ChevronDownIcon className='size-3.5 shrink-0 opacity-60' />
                ) : (
                  <ChevronRightIcon className='size-3.5 shrink-0 opacity-60' />
                )
              ) : (
                <span className='size-3.5 shrink-0' />
              )}
              {rowIcon}
              <span className='truncate'>{node.name}</span>
              {isEntry && (
                <span className='ml-auto text-xs opacity-50'>
                  {t('entry')}
                </span>
              )}
            </button>
          )}
        </ContextMenuTrigger>
        <ContextMenuContent>
          {node.type === 'folder' && (
            <>
              <ContextMenuItem onClick={() => onNewFile(node.path)}>
                <FilePlusIcon /> {t('actions.newFile')}
              </ContextMenuItem>
              <ContextMenuItem onClick={() => onNewFolder(node.path)}>
                <FolderPlusIcon /> {t('actions.newFolder')}
              </ContextMenuItem>
              <ContextMenuSeparator />
            </>
          )}
          {node.type === 'file' && node.kind === 'text' && (
            <ContextMenuItem onClick={() => onSetEntry(node)}>
              <TargetIcon /> {t('actions.setAsEntry')}
            </ContextMenuItem>
          )}
          {node.type === 'file' && node.kind === 'binary' && (
            <ContextMenuItem onClick={() => onDownload(node)}>
              <DownloadIcon /> {t('actions.download')}
            </ContextMenuItem>
          )}
          {node.type === 'file' && (
            <ContextMenuItem onClick={() => onRename(node)}>
              <PencilIcon /> {t('actions.rename')}
            </ContextMenuItem>
          )}
          <ContextMenuItem
            onClick={() => onDelete(node)}
            variant='destructive'
          >
            <Trash2Icon /> {t('actions.delete')}
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>

      {node.type === 'folder' && isOpen && (
        <>
          <TreeList {...props} depth={depth + 1} nodes={node.children} />
          {draft &&
            draft.kind !== 'rename' &&
            draft.parent === node.path && (
              <DraftRow
                depth={depth + 1}
                icon={
                  draft.kind === 'create-folder' ? (
                    <FolderIcon className='size-4 opacity-70' />
                  ) : (
                    <FileIcon className='size-4 opacity-70' />
                  )
                }
                onCancel={onCancelDraft}
                onCommit={onCommitDraft}
                placeholder={
                  draft.kind === 'create-folder'
                    ? t('placeholders.folderName')
                    : t('placeholders.fileName')
                }
              />
            )}
        </>
      )}
    </li>
  );
}

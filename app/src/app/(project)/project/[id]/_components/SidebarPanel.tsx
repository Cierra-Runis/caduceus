'use client';

import {
    ChevronRightIcon,
    FileIcon,
    FilePlusIcon,
    FolderIcon,
    FolderPlusIcon,
    PencilIcon,
    Trash2Icon,
    UploadIcon,
} from 'lucide-react';
import {
    DragEvent,
    KeyboardEvent,
    ReactNode,
    RefObject,
    useMemo,
    useRef,
    useState,
} from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';

import { cn } from '@/lib/utils';
import { TreeNode } from '@/lib/yjs/tree';

export interface SidebarPanelProps {
  /// Id of the compile entry file, marked in the list. Null if none.
  entry: null | string;
  /// Id of the file currently open in the editor.
  focus: string;
  /// Every node (files and folders); the panel builds the tree from `parent`.
  nodes: TreeNode[];
  /// Create a file with this name under `parent` (null = root); returns whether
  /// it succeeded (a rejected name keeps the input open).
  onCreateFile: (name: string, parent: null | string) => boolean;
  /// Create a folder with this name under `parent` (null = root).
  onCreateFolder: (name: string, parent: null | string) => boolean;
  /// Delete the node with this id (a folder takes its whole subtree).
  onDelete: (id: string) => void;
  /// Move the node with this id under `parent` (null = root).
  onMove: (id: string, parent: null | string) => void;
  /// Rename the node with this id; returns whether it succeeded.
  onRename: (id: string, name: string) => boolean;
  onSelect: (id: string) => void;
  /// Upload a picked file as a binary blob at the root.
  onUpload: (file: File) => void;
  sidebarPanelRef: RefObject<null | PanelImperativeHandle>;
}

/// What the inline name input is currently for: creating a `kind` under
/// `parent`, or renaming the node `id`.
type Editing =
  | { id: string; mode: 'rename' }
  | { kind: 'file' | 'folder'; mode: 'create'; parent: null | string };

export function SidebarPanel({
  entry,
  focus,
  nodes,
  onCreateFile,
  onCreateFolder,
  onDelete,
  onMove,
  onRename,
  onSelect,
  onUpload,
  sidebarPanelRef,
}: SidebarPanelProps) {
  // A folder is expanded unless it is in `collapsed`, so a fresh tree shows
  // everything. `editing` drives the single inline input (create or rename).
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [editing, setEditing] = useState<Editing | null>(null);
  const fileInput = useRef<HTMLInputElement>(null);
  // Drag-to-move: `dragging` holds the grabbed node id; `dropTarget` is the
  // folder id currently hovered (or '' for the root) so it can be highlighted.
  // The empty string is safe as the root sentinel — node ids are 24-hex.
  const dragging = useRef<null | string>(null);
  const [dropTarget, setDropTarget] = useState<null | string>(null);

  const startDrag = (event: DragEvent, id: string) => {
    dragging.current = id;
    event.dataTransfer.setData('text/plain', id);
    event.dataTransfer.effectAllowed = 'move';
  };
  const allowDrop = (event: DragEvent, target: string) => {
    event.preventDefault();
    event.stopPropagation(); // a folder hover shouldn't also count as root
    event.dataTransfer.dropEffect = 'move';
    setDropTarget(target);
  };
  const drop = (event: DragEvent, parent: null | string) => {
    event.preventDefault();
    event.stopPropagation();
    const id = dragging.current ?? event.dataTransfer.getData('text/plain');
    dragging.current = null;
    setDropTarget(null);
    if (id) onMove(id, parent);
  };

  // Group nodes by parent, folders before files then by name, so the tree
  // renders in a stable order.
  const childrenOf = useMemo(() => {
    const map = new Map<null | string, TreeNode[]>();
    for (const node of nodes) {
      const siblings = map.get(node.parent) ?? [];
      siblings.push(node);
      map.set(node.parent, siblings);
    }
    for (const siblings of map.values()) {
      siblings.sort((a, b) =>
        a.kind === b.kind
          ? a.name.localeCompare(b.name)
          : a.kind === 'folder'
            ? -1
            : 1,
      );
    }
    return map;
  }, [nodes]);

  const expand = (id: string) =>
    setCollapsed((prev) => {
      const next = new Set(prev);
      next.delete(id);
      return next;
    });
  const toggle = (id: string) =>
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });

  const startCreate = (parent: null | string, kind: 'file' | 'folder') => {
    if (parent !== null) expand(parent);
    setEditing({ kind, mode: 'create', parent });
  };

  const requestDelete = (node: TreeNode) => {
    const hasChildren = (childrenOf.get(node.id)?.length ?? 0) > 0;
    if (
      hasChildren &&
      !window.confirm(`Delete “${node.name}” and everything inside it?`)
    ) {
      return;
    }
    onDelete(node.id);
  };

  // Render the children of `parent` at indent `depth`, recursing into folders.
  const renderChildren = (parent: null | string, depth: number): ReactNode => {
    const rows: ReactNode[] = [];

    if (editing?.mode === 'create' && editing.parent === parent) {
      rows.push(
        <li className='py-1' key='__new__' style={{ paddingLeft: pad(depth) }}>
          <NameInput
            onCancel={() => setEditing(null)}
            onSubmit={(name) => {
              const create =
                editing.kind === 'folder' ? onCreateFolder : onCreateFile;
              if (create(name, parent)) setEditing(null);
            }}
            placeholder={editing.kind === 'folder' ? 'folder name' : 'file name'}
          />
        </li>,
      );
    }

    for (const node of childrenOf.get(parent) ?? []) {
      const renaming = editing?.mode === 'rename' && editing.id === node.id;
      if (renaming) {
        rows.push(
          <li className='py-1' key={node.id} style={{ paddingLeft: pad(depth) }}>
            <NameInput
              initial={node.name}
              onCancel={() => setEditing(null)}
              onSubmit={(name) => {
                if (name === node.name || onRename(node.id, name)) {
                  setEditing(null);
                }
              }}
            />
          </li>,
        );
        continue;
      }

      const isFolder = node.kind === 'folder';
      const open = isFolder && !collapsed.has(node.id);
      rows.push(
        <li
          key={node.id}
          {...(isFolder && {
            onDragOver: (event: DragEvent) => allowDrop(event, node.id),
            onDrop: (event: DragEvent) => drop(event, node.id),
          })}
        >
          <div
            className={cn(
              'group flex items-center',
              isFolder && dropTarget === node.id && 'bg-accent/60',
            )}
            draggable
            onDragEnd={() => {
              dragging.current = null;
              setDropTarget(null);
            }}
            onDragStart={(event) => startDrag(event, node.id)}
          >
            <button
              aria-current={node.id === focus ? 'true' : undefined}
              className={cn(
                'flex min-w-0 flex-1 items-center gap-1 py-1 pr-2 text-left text-sm',
                node.id === focus ? 'bg-accent' : 'hover:bg-accent/50',
              )}
              onClick={() => (isFolder ? toggle(node.id) : onSelect(node.id))}
              style={{ paddingLeft: pad(depth) }}
            >
              {isFolder ? (
                <ChevronRightIcon
                  className={cn(
                    'size-3.5 shrink-0 opacity-60 transition-transform',
                    open && 'rotate-90',
                  )}
                />
              ) : (
                <span className='w-3.5 shrink-0' />
              )}
              {isFolder ? (
                <FolderIcon className='size-4 shrink-0 opacity-60' />
              ) : (
                <FileIcon className='size-4 shrink-0 opacity-60' />
              )}
              <span className='truncate'>{node.name}</span>
              {node.id === entry && (
                <span className='ml-auto text-xs opacity-50'>entry</span>
              )}
            </button>
            <span
              className={`
                flex shrink-0 items-center pr-2 opacity-0
                group-focus-within:opacity-100 group-hover:opacity-100
              `}
            >
              {isFolder && (
                <>
                  <IconButton
                    icon={<FilePlusIcon className='size-3.5' />}
                    label={`New file in ${node.name}`}
                    onClick={() => startCreate(node.id, 'file')}
                  />
                  <IconButton
                    icon={<FolderPlusIcon className='size-3.5' />}
                    label={`New folder in ${node.name}`}
                    onClick={() => startCreate(node.id, 'folder')}
                  />
                </>
              )}
              <IconButton
                icon={<PencilIcon className='size-3.5' />}
                label={`Rename ${node.name}`}
                onClick={() => setEditing({ id: node.id, mode: 'rename' })}
              />
              <IconButton
                icon={<Trash2Icon className='size-3.5' />}
                label={`Delete ${node.name}`}
                onClick={() => requestDelete(node)}
              />
            </span>
          </div>
          {open && <ul>{renderChildren(node.id, depth + 1)}</ul>}
        </li>,
      );
    }

    return rows;
  };

  return (
    <Panel
      collapsible
      defaultSize={0}
      id='sidebar'
      minSize={10}
      panelRef={sidebarPanelRef}
    >
      <div className='flex items-center justify-between px-3 py-2'>
        <span className='text-xs font-medium opacity-60'>Files</span>
        <span className='flex items-center'>
          <IconButton
            icon={<FilePlusIcon className='size-4' />}
            label='New file'
            onClick={() => startCreate(null, 'file')}
          />
          <IconButton
            icon={<FolderPlusIcon className='size-4' />}
            label='New folder'
            onClick={() => startCreate(null, 'folder')}
          />
          <IconButton
            icon={<UploadIcon className='size-4' />}
            label='Upload file'
            onClick={() => fileInput.current?.click()}
          />
          <input
            className='hidden'
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (file) onUpload(file);
              event.target.value = ''; // allow re-picking the same file
            }}
            ref={fileInput}
            type='file'
          />
        </span>
      </div>

      <ul
        className={cn(
          'flex min-h-24 flex-col pb-2',
          dropTarget === '' && 'bg-accent/30',
        )}
        onDragOver={(event) => allowDrop(event, '')}
        onDrop={(event) => drop(event, null)}
      >
        {renderChildren(null, 0)}
      </ul>
    </Panel>
  );
}

/// A small hover-revealed icon button used for the row actions.
function IconButton({
  icon,
  label,
  onClick,
}: {
  icon: ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      aria-label={label}
      className='rounded-sm p-1 hover:bg-accent'
      onClick={onClick}
      title={label}
    >
      {icon}
    </button>
  );
}

/// A small inline text input for creating/renaming: submits its trimmed value on
/// Enter, cancels on Escape or blur.
function NameInput({
  initial = '',
  onCancel,
  onSubmit,
  placeholder,
}: {
  initial?: string;
  onCancel: () => void;
  onSubmit: (value: string) => void;
  placeholder?: string;
}) {
  const [value, setValue] = useState(initial);
  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') onSubmit(value.trim());
    else if (event.key === 'Escape') onCancel();
  };
  return (
    <input
      autoFocus
      className={`
        w-full rounded-sm border bg-background px-2 py-1 text-sm outline-none
        focus:border-ring
      `}
      onBlur={onCancel}
      onChange={(event) => setValue(event.target.value)}
      onKeyDown={onKeyDown}
      placeholder={placeholder}
      value={value}
    />
  );
}

/// Indent (in px) for a row at tree `depth`.
function pad(depth: number): number {
  return depth * 12 + 8;
}

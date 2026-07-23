import { ProjectFile } from '@/lib/types/project';

/// A node in the rendered file tree. Folders are *derived*: any directory
/// implied by a file path, unioned with the project's explicit (possibly empty)
/// directories. Files carry their stable id so actions address them by id, not
/// by a path that a rename could change under us.
export interface TreeNode {
  children: TreeNode[];
  fileId?: string;
  /// Present when the file is a font, carrying the families it provides.
  font?: { families: string[] };
  kind?: 'binary' | 'text';
  name: string;
  path: string;
  type: 'file' | 'folder';
}

/// Every folder path in the tree (for "expand all" / initial expansion).
export function allFolderPaths(nodes: TreeNode[]): string[] {
  const out: string[] = [];
  const walk = (list: TreeNode[]) => {
    for (const node of list) {
      if (node.type === 'folder') {
        out.push(node.path);
        walk(node.children);
      }
    }
  };
  walk(nodes);
  return out;
}

/// Build a nested, sorted tree from the flat file list plus explicit
/// directories. Folders sort before files, each alphabetically
/// (locale-aware, case-insensitive), matching VS Code's explorer.
export function buildFileTree(
  files: ProjectFile[],
  directories: string[],
): TreeNode[] {
  const root: TreeNode = {
    children: [],
    name: '',
    path: '',
    type: 'folder',
  };

  const folderAt = (path: string): TreeNode => {
    if (path === '') return root;
    const segments = path.split('/');
    let node = root;
    let acc = '';
    for (const segment of segments) {
      acc = acc ? `${acc}/${segment}` : segment;
      let child = node.children.find(
        (c) => c.type === 'folder' && c.name === segment,
      );
      if (!child) {
        child = { children: [], name: segment, path: acc, type: 'folder' };
        node.children.push(child);
      }
      node = child;
    }
    return node;
  };

  // Explicit directories first, so an empty folder still materializes.
  for (const dir of directories) folderAt(dir);

  for (const file of files) {
    const segments = file.path.split('/');
    const name = segments.pop() as string;
    const parent = folderAt(segments.join('/'));
    parent.children.push({
      children: [],
      fileId: file.id,
      font: file.font,
      kind: file.content.kind,
      name,
      path: file.path,
      type: 'file',
    });
  }

  sortTree(root);
  return root.children;
}

/// Join a parent directory and a leaf name into a project-root-relative path.
export function joinPath(parent: string, name: string): string {
  return parent ? `${parent}/${name}` : name;
}

/// The parent directory of a path (`''` for a top-level entry).
export function parentDir(path: string): string {
  const idx = path.lastIndexOf('/');
  return idx === -1 ? '' : path.slice(0, idx);
}

function sortTree(node: TreeNode): void {
  node.children.sort((a, b) => {
    if (a.type !== b.type) return a.type === 'folder' ? -1 : 1;
    return a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
  });
  for (const child of node.children) {
    if (child.type === 'folder') sortTree(child);
  }
}

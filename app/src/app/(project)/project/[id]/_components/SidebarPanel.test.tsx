import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { type ReactNode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { TreeNode } from '@/lib/yjs/tree';

import { SidebarPanel } from './SidebarPanel';

// `Panel` needs a `PanelGroup` context (and ResizeObserver); this test only
// cares about the tree logic, so stub the container to a plain div.
vi.mock('react-resizable-panels', () => ({
  Panel: ({ children }: { children: ReactNode }) => <div>{children}</div>,
}));

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

// A folder `chapters` holding `intro.typ`, plus a root-level `main.typ`.
const nodes: TreeNode[] = [
  { id: 'dir', kind: 'folder', name: 'chapters', parent: null },
  { id: 'intro', kind: 'file', name: 'intro.typ', parent: 'dir' },
  { id: 'main', kind: 'file', name: 'main.typ', parent: null },
];

/// Non-null helper so the tests avoid `!` (forbidden by lint).
function must<T>(value: null | T | undefined): T {
  if (value == null) throw new Error('expected an element');
  return value;
}

function renderPanel(
  overrides: Partial<Parameters<typeof SidebarPanel>[0]> = {},
) {
  return render(
    <SidebarPanel
      entry={null}
      focus=''
      nodes={nodes}
      onCreateFile={vi.fn(() => true)}
      onCreateFolder={vi.fn(() => true)}
      onDelete={vi.fn()}
      onMove={vi.fn()}
      onRename={vi.fn(() => true)}
      onSelect={vi.fn()}
      onUpload={vi.fn()}
      sidebarPanelRef={{ current: null }}
      {...overrides}
    />,
  );
}

describe('SidebarPanel', () => {
  it('renders folders with their nested files', () => {
    renderPanel();
    expect(screen.getByText('chapters')).toBeTruthy();
    expect(screen.getByText('intro.typ')).toBeTruthy();
    expect(screen.getByText('main.typ')).toBeTruthy();
  });

  it('collapses a folder to hide its children', async () => {
    renderPanel();
    await userEvent.click(screen.getByRole('button', { name: 'chapters' }));
    expect(screen.queryByText('intro.typ')).toBeNull();
  });

  it('selects a file but toggles a folder', async () => {
    const onSelect = vi.fn();
    renderPanel({ onSelect });
    await userEvent.click(screen.getByRole('button', { name: 'main.typ' }));
    expect(onSelect).toHaveBeenCalledWith('main');
    await userEvent.click(screen.getByRole('button', { name: 'chapters' }));
    expect(onSelect).toHaveBeenCalledTimes(1); // the folder click did not select
  });

  it('labels the entry file', () => {
    renderPanel({ entry: 'main' });
    expect(
      screen.getByRole('button', { name: /^main\.typ/ }).textContent,
    ).toContain('entry');
  });

  it('creates a file at the root', async () => {
    const onCreateFile = vi.fn(() => true);
    renderPanel({ onCreateFile });
    await userEvent.click(screen.getByRole('button', { name: 'New file' }));
    await userEvent.type(
      screen.getByPlaceholderText('file name'),
      'notes.typ{Enter}',
    );
    expect(onCreateFile).toHaveBeenCalledWith('notes.typ', null);
  });

  it('creates a file inside a folder', async () => {
    const onCreateFile = vi.fn(() => true);
    renderPanel({ onCreateFile });
    await userEvent.click(
      screen.getByRole('button', { name: 'New file in chapters' }),
    );
    await userEvent.type(
      screen.getByPlaceholderText('file name'),
      'part2.typ{Enter}',
    );
    expect(onCreateFile).toHaveBeenCalledWith('part2.typ', 'dir');
  });

  it('creates a folder at the root', async () => {
    const onCreateFolder = vi.fn(() => true);
    renderPanel({ onCreateFolder });
    await userEvent.click(screen.getByRole('button', { name: 'New folder' }));
    await userEvent.type(
      screen.getByPlaceholderText('folder name'),
      'assets{Enter}',
    );
    expect(onCreateFolder).toHaveBeenCalledWith('assets', null);
  });

  it('renames a node, seeding the input with its current name', async () => {
    const onRename = vi.fn(() => true);
    renderPanel({ onRename });
    await userEvent.click(
      screen.getByRole('button', { name: 'Rename intro.typ' }),
    );
    const input = screen.getByDisplayValue('intro.typ');
    await userEvent.clear(input);
    await userEvent.type(input, 'preface.typ{Enter}');
    expect(onRename).toHaveBeenCalledWith('intro', 'preface.typ');
  });

  it('deletes a file without a confirm', async () => {
    const onDelete = vi.fn();
    renderPanel({ onDelete });
    await userEvent.click(
      screen.getByRole('button', { name: 'Delete main.typ' }),
    );
    expect(onDelete).toHaveBeenCalledWith('main');
  });

  it('confirms before deleting a non-empty folder', async () => {
    const onDelete = vi.fn();
    const confirm = vi.spyOn(window, 'confirm').mockReturnValue(true);
    renderPanel({ onDelete });
    await userEvent.click(
      screen.getByRole('button', { name: 'Delete chapters' }),
    );
    expect(confirm).toHaveBeenCalled();
    expect(onDelete).toHaveBeenCalledWith('dir');
  });

  it('does not delete a folder when the confirm is dismissed', async () => {
    const onDelete = vi.fn();
    vi.spyOn(window, 'confirm').mockReturnValue(false);
    renderPanel({ onDelete });
    await userEvent.click(
      screen.getByRole('button', { name: 'Delete chapters' }),
    );
    expect(onDelete).not.toHaveBeenCalled();
  });

  it('shows the auto-save selector and reports a change', async () => {
    const onAutoSaveChange = vi.fn();
    renderPanel({ autoSave: 'onFocusChange', onAutoSaveChange });
    const select = screen.getByRole('combobox');
    expect((select as HTMLSelectElement).value).toBe('onFocusChange');
    await userEvent.selectOptions(select, 'afterDelay');
    expect(onAutoSaveChange).toHaveBeenCalledWith('afterDelay');
  });

  it('omits the auto-save selector when no handler is given', () => {
    renderPanel();
    expect(screen.queryByRole('combobox')).toBeNull();
  });

  it('moves a node when dropped onto a folder', () => {
    const onMove = vi.fn();
    renderPanel({ onMove });
    const dataTransfer = { getData: () => '', setData: vi.fn() };
    // Drag the root-level `main.typ` onto the `chapters` folder.
    const row = must(screen.getByRole('button', { name: 'main.typ' }).parentElement);
    fireEvent.dragStart(row, { dataTransfer });
    const folder = must(
      screen.getByRole('button', { name: 'chapters' }).closest('li'),
    );
    fireEvent.drop(folder, { dataTransfer });
    expect(onMove).toHaveBeenCalledWith('main', 'dir');
  });

  it('moves a node to the root when dropped on the root area', () => {
    const onMove = vi.fn();
    renderPanel({ onMove });
    const dataTransfer = { getData: () => '', setData: vi.fn() };
    // Drag the nested `intro.typ` out to the root list.
    const row = must(screen.getByRole('button', { name: 'intro.typ' }).parentElement);
    fireEvent.dragStart(row, { dataTransfer });
    const root = must(screen.getByRole('button', { name: 'main.typ' }).closest('ul'));
    fireEvent.drop(root, { dataTransfer });
    expect(onMove).toHaveBeenCalledWith('intro', null);
  });
});

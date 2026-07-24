import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { type ReactNode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { SidebarPanel } from './SidebarPanel';

// `Panel` needs a `PanelGroup` context (and ResizeObserver); this test only
// cares about the file-list logic, so stub the container to a plain div.
vi.mock('react-resizable-panels', () => ({
  Panel: ({ children }: { children: ReactNode }) => <div>{children}</div>,
}));

afterEach(cleanup);

const files = [
  { id: 'id-main', path: 'main.typ' },
  { id: 'id-intro', path: 'chapters/intro.typ' },
];

// Default no-op handlers; individual tests override what they assert on.
function renderPanel(
  overrides: Partial<Parameters<typeof SidebarPanel>[0]> = {},
) {
  return render(
    <SidebarPanel
      entry={null}
      files={files}
      focus=''
      onCreateFile={vi.fn(() => true)}
      onDelete={vi.fn()}
      onRename={vi.fn(() => true)}
      onSelect={vi.fn()}
      sidebarPanelRef={{ current: null }}
      {...overrides}
    />,
  );
}

describe('SidebarPanel', () => {
  it('lists each file by its path', () => {
    renderPanel();
    expect(screen.getByText('main.typ')).toBeTruthy();
    expect(screen.getByText('chapters/intro.typ')).toBeTruthy();
  });

  it('selects by id, not path', async () => {
    const onSelect = vi.fn();
    renderPanel({ onSelect });
    await userEvent.click(screen.getByText('chapters/intro.typ'));
    expect(onSelect).toHaveBeenCalledWith('id-intro');
  });

  it('marks the focused file (by id) with aria-current', () => {
    renderPanel({ focus: 'id-intro' });
    // Exact names pick the row buttons, not the "Rename/Delete <path>" ones.
    expect(
      screen
        .getByRole('button', { name: 'chapters/intro.typ' })
        .getAttribute('aria-current'),
    ).toBe('true');
    expect(
      screen
        .getByRole('button', { name: 'main.typ' })
        .getAttribute('aria-current'),
    ).toBeNull();
  });

  it('labels only the entry file', () => {
    renderPanel({ entry: 'id-main' });
    // The entry row's name is "main.typ entry"; the other stays exact.
    expect(
      screen.getByRole('button', { name: /^main\.typ/ }).textContent,
    ).toContain('entry');
    expect(
      screen.getByRole('button', { name: 'chapters/intro.typ' }).textContent,
    ).not.toContain('entry');
  });

  it('creates a file from the new-file input on Enter', async () => {
    const onCreateFile = vi.fn(() => true);
    renderPanel({ onCreateFile });
    await userEvent.click(screen.getByRole('button', { name: 'New file' }));
    await userEvent.type(
      screen.getByPlaceholderText('file name'),
      'notes.typ{Enter}',
    );
    expect(onCreateFile).toHaveBeenCalledWith('notes.typ');
  });

  it('renames a file, seeding the input with its current name', async () => {
    const onRename = vi.fn(() => true);
    renderPanel({ onRename });
    await userEvent.click(
      screen.getByRole('button', { name: 'Rename chapters/intro.typ' }),
    );
    const input = screen.getByDisplayValue('intro.typ');
    await userEvent.clear(input);
    await userEvent.type(input, 'preface.typ{Enter}');
    expect(onRename).toHaveBeenCalledWith('id-intro', 'preface.typ');
  });

  it('deletes a file by id', async () => {
    const onDelete = vi.fn();
    renderPanel({ onDelete });
    await userEvent.click(
      screen.getByRole('button', { name: 'Delete main.typ' }),
    );
    expect(onDelete).toHaveBeenCalledWith('id-main');
  });
});

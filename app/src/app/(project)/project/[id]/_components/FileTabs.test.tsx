import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { FileTab, FileTabs } from './FileTabs';

afterEach(cleanup);

const tabs: FileTab[] = [
  { dirty: false, id: 'a', name: 'main.typ' },
  { dirty: true, id: 'b', name: 'intro.typ' },
];

function renderTabs(overrides: Partial<Parameters<typeof FileTabs>[0]> = {}) {
  return render(
    <FileTabs
      activeId='a'
      onClose={vi.fn()}
      onSelect={vi.fn()}
      tabs={tabs}
      {...overrides}
    />,
  );
}

describe('FileTabs', () => {
  it('renders a tab per open file', () => {
    renderTabs();
    expect(screen.getByRole('button', { name: 'main.typ' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'intro.typ' })).toBeTruthy();
  });

  it('renders nothing when there are no tabs', () => {
    const { container } = renderTabs({ tabs: [] });
    expect(container.firstChild).toBeNull();
  });

  it('selects a tab on click', async () => {
    const onSelect = vi.fn();
    renderTabs({ onSelect });
    await userEvent.click(screen.getByRole('button', { name: 'intro.typ' }));
    expect(onSelect).toHaveBeenCalledWith('b');
  });

  it('closes a tab from its close button', async () => {
    const onClose = vi.fn();
    renderTabs({ onClose });
    await userEvent.click(screen.getByRole('button', { name: 'Close main.typ' }));
    expect(onClose).toHaveBeenCalledWith('a');
  });

  it('closes a tab on middle-click', () => {
    const onClose = vi.fn();
    renderTabs({ onClose });
    // The tab row is the parent of its select button.
    const row = screen.getByRole('button', { name: 'intro.typ' }).parentElement;
    if (!row) throw new Error('expected a tab row');
    fireEvent(row, new MouseEvent('auxclick', { bubbles: true, button: 1 }));
    expect(onClose).toHaveBeenCalledWith('b');
  });

  it('marks a dirty tab so its unsaved state is visible', () => {
    renderTabs();
    // The clean tab shows only an always-hidden-until-hover ×; the dirty tab
    // additionally renders the unsaved dot (a bg-current bullet).
    const dirtyClose = screen.getByRole('button', { name: 'Close intro.typ' });
    expect(dirtyClose.querySelector('.rounded-full')).not.toBeNull();
    const cleanClose = screen.getByRole('button', { name: 'Close main.typ' });
    expect(cleanClose.querySelector('.rounded-full')).toBeNull();
  });
});

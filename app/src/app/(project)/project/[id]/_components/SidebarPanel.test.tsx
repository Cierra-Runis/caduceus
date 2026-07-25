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

describe('SidebarPanel', () => {
  it('lists each file by its path', () => {
    render(
      <SidebarPanel
        entry={null}
        files={files}
        focus=''
        onSelect={vi.fn()}
        sidebarPanelRef={{ current: null }}
      />,
    );
    expect(screen.getByText('main.typ')).toBeTruthy();
    expect(screen.getByText('chapters/intro.typ')).toBeTruthy();
  });

  it('selects by id, not path', async () => {
    const onSelect = vi.fn();
    render(
      <SidebarPanel
        entry={null}
        files={files}
        focus=''
        onSelect={onSelect}
        sidebarPanelRef={{ current: null }}
      />,
    );
    await userEvent.click(screen.getByText('chapters/intro.typ'));
    expect(onSelect).toHaveBeenCalledWith('id-intro');
  });

  it('marks the focused file (by id) with aria-current', () => {
    render(
      <SidebarPanel
        entry={null}
        files={files}
        focus='id-intro'
        onSelect={vi.fn()}
        sidebarPanelRef={{ current: null }}
      />,
    );
    expect(
      screen.getByRole('button', { name: /intro\.typ/ }).getAttribute('aria-current'),
    ).toBe('true');
    expect(
      screen.getByRole('button', { name: /main\.typ/ }).getAttribute('aria-current'),
    ).toBeNull();
  });

  it('labels only the entry file', () => {
    render(
      <SidebarPanel
        entry='id-main'
        files={files}
        focus=''
        onSelect={vi.fn()}
        sidebarPanelRef={{ current: null }}
      />,
    );
    expect(
      screen.getByRole('button', { name: /main\.typ/ }).textContent,
    ).toContain('entry');
    expect(
      screen.getByRole('button', { name: /intro\.typ/ }).textContent,
    ).not.toContain('entry');
  });
});

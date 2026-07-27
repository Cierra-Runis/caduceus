import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { type ReactNode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { withIntl } from '@/test/intl';

import { SettingsPanel } from './SettingsPanel';

// `Panel` needs a `PanelGroup` context; stub it to a plain div for this test.
vi.mock('react-resizable-panels', () => ({
  Panel: ({ children }: { children: ReactNode }) => <div>{children}</div>,
}));

afterEach(cleanup);

function renderPanel(
  overrides: Partial<Parameters<typeof SettingsPanel>[0]> = {},
) {
  return render(
    withIntl(
      <SettingsPanel
        autoSave='onFocusChange'
        onAutoSaveChange={vi.fn()}
        sidebarPanelRef={{ current: null }}
        {...overrides}
      />,
    ),
  );
}

describe('SettingsPanel', () => {
  it('shows the current auto-save policy', () => {
    renderPanel();
    expect((screen.getByRole('combobox') as HTMLSelectElement).value).toBe(
      'onFocusChange',
    );
  });

  it('reports a policy change', async () => {
    const onAutoSaveChange = vi.fn();
    renderPanel({ onAutoSaveChange });
    await userEvent.selectOptions(screen.getByRole('combobox'), 'afterDelay');
    expect(onAutoSaveChange).toHaveBeenCalledWith('afterDelay');
  });
});

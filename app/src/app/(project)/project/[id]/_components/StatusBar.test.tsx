import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { withIntl } from '@/test/intl';

import { StatusBar } from './StatusBar';

afterEach(cleanup);

function renderBar(overrides: Partial<Parameters<typeof StatusBar>[0]> = {}) {
  return render(
    withIntl(
      <StatusBar
        autoSave='onFocusChange'
        dirtyCount={0}
        entryName='main.typ'
        meId='u1'
        // A null provider is the pre-connection state: offline + no peers.
        provider={null}
        {...overrides}
      />,
    ),
  );
}

describe('StatusBar', () => {
  it('shows offline, solo, saved, and the entry file when disconnected and clean', () => {
    renderBar();
    // getByText throws when absent, so a successful lookup is the assertion.
    expect(screen.getByText('Offline')).toBeTruthy();
    expect(screen.getByText('Just you')).toBeTruthy();
    expect(screen.getByText('Saved')).toBeTruthy();
    expect(screen.getByText('main.typ')).toBeTruthy();
    expect(screen.getByText(/Typst/)).toBeTruthy();
  });

  it('pluralizes the unsaved-file count instead of "Saved"', () => {
    renderBar({ dirtyCount: 2 });
    expect(screen.getByText('2 unsaved files')).toBeTruthy();
    expect(screen.queryByText('Saved')).toBeNull();
  });
});

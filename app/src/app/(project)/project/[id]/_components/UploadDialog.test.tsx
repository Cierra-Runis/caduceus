import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { withIntl } from '@/test/intl';

import { UploadDialog } from './UploadDialog';

// Stub the XHR upload so the test drives progress + result deterministically.
const uploadBlobWithProgress = vi.fn(
  async (
    _projectId: string,
    _file: File,
    onProgress: (fraction: number) => void,
  ) => {
    onProgress(1);
    return { sha256: 'a'.repeat(64), size: 3 };
  },
);
vi.mock('@/lib/api/blob', () => ({
  uploadBlobWithProgress: (
    projectId: string,
    file: File,
    onProgress: (fraction: number) => void,
  ) => uploadBlobWithProgress(projectId, file, onProgress),
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

function pickFile(file: File) {
  // The dialog renders into a portal (document.body), not the test container.
  const input = document.querySelector('input[type="file"]');
  if (!input) throw new Error('no file input');
  fireEvent.change(input, { target: { files: [file] } });
}

function renderDialog(overrides: Partial<Parameters<typeof UploadDialog>[0]> = {}) {
  return render(
    withIntl(
      <UploadDialog
        onOpenChange={vi.fn()}
        onUploaded={vi.fn()}
        open
        projectId="p1"
        {...overrides}
      />,
    ),
  );
}

describe('UploadDialog', () => {
  it('stages a picked file', () => {
    renderDialog();
    pickFile(new File(['hi'], 'logo.png'));
    expect(screen.getByText('logo.png')).toBeTruthy();
  });

  it('uploads staged files and reports each blob', async () => {
    const onUploaded = vi.fn();
    renderDialog({ onUploaded });
    pickFile(new File(['hi'], 'logo.png'));
    await userEvent.click(screen.getByRole('button', { name: 'Upload' }));
    await waitFor(() =>
      expect(onUploaded).toHaveBeenCalledWith('logo.png', 'a'.repeat(64), 3),
    );
    expect(uploadBlobWithProgress).toHaveBeenCalledTimes(1);
  });

  it('surfaces a node-creation failure on the row', async () => {
    const onUploaded = vi.fn(() => {
      throw new Error('"logo.png" already exists here');
    });
    renderDialog({ onUploaded });
    pickFile(new File(['hi'], 'logo.png'));
    await userEvent.click(screen.getByRole('button', { name: 'Upload' }));
    await waitFor(() =>
      expect(screen.getByText('"logo.png" already exists here')).toBeTruthy(),
    );
  });
});

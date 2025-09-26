'use client';

import { RefObject } from 'react';
import { ImperativePanelHandle, Panel } from 'react-resizable-panels';

export interface PreviewPanelProps {
  previewPanelRef: RefObject<ImperativePanelHandle | null>;
}

export function PreviewPanel({ previewPanelRef }: PreviewPanelProps) {
  return (
    <Panel
      collapsible
      defaultSize={50}
      id='preview'
      minSize={20}
      order={2}
      ref={previewPanelRef}
    ></Panel>
  );
}

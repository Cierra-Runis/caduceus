'use client';

import { RefObject } from 'react';
import { ImperativePanelHandle, Panel } from 'react-resizable-panels';

import { Editor } from './Editor';

export interface EditorPanelProps {
  editorPanelRef: RefObject<ImperativePanelHandle | null>;
}

export function EditorPanel({ editorPanelRef }: EditorPanelProps) {
  return (
    <Panel
      collapsible
      defaultSize={50}
      id='editor'
      minSize={20}
      order={1}
      ref={editorPanelRef}
      style={{ overflow: 'auto' }}
    >
      <Editor />
    </Panel>
  );
}

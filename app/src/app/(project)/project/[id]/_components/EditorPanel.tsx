'use client';

import { RefObject } from 'react';
import { ImperativePanelHandle, Panel } from 'react-resizable-panels';
import { SendJsonMessage } from 'react-use-websocket/dist/lib/types';

import { Editor } from './Editor';

export interface EditorPanelProps {
  editorPanelRef: RefObject<ImperativePanelHandle | null>;
  sendMessage: SendJsonMessage;
}

export function EditorPanel({ editorPanelRef, sendMessage }: EditorPanelProps) {
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
      <Editor sendMessage={sendMessage} />
    </Panel>
  );
}

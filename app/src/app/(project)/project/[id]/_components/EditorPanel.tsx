'use client';

import { RefObject } from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

import { Editor } from './Editor';

export interface EditorPanelProps {
  editorPanelRef: RefObject<null  | PanelImperativeHandle>;
  provider: null | WebsocketProvider;
  textId: string;
  ydoc: Y.Doc;
}

export function EditorPanel({
  editorPanelRef,
  provider,
  textId,
  ydoc,
}: EditorPanelProps) {
  return (
    <Panel
      collapsible
      defaultSize={50}
      id='editor'
      minSize={20}
      panelRef={editorPanelRef}
      style={{ overflow: 'auto' }}
    >
      <Editor provider={provider} textId={textId} ydoc={ydoc} />
    </Panel>
  );
}

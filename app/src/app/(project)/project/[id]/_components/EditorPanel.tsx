'use client';

import { RefObject } from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

import { Editor } from './Editor';

export interface EditorPanelProps {
  editorPanelRef: RefObject<null  | PanelImperativeHandle>;
  path: string;
  provider: null | WebsocketProvider;
  ydoc: Y.Doc;
}

export function EditorPanel({
  editorPanelRef,
  path,
  provider,
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
      <Editor path={path} provider={provider} ydoc={ydoc} />
    </Panel>
  );
}

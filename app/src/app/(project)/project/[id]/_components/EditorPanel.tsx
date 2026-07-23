'use client';

import { RefObject } from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

import { Editor } from './Editor';
import { FilePreview } from './FilePreview';

export interface EditorPanelProps {
  editorPanelRef: RefObject<null | PanelImperativeHandle>;
  /// The focused file, or null when nothing is open. A binary file renders a
  /// read-only preview; a text file renders the collaborative editor.
  focusFile: {
    fileId: string;
    kind: 'binary' | 'text';
  } | null;
  path: string;
  projectId: string;
  provider: null | WebsocketProvider;
  ydoc: Y.Doc;
}

export function EditorPanel({
  editorPanelRef,
  focusFile,
  path,
  projectId,
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
      {focusFile?.kind === 'binary' ? (
        <FilePreview
          fileId={focusFile.fileId}
          path={path}
          projectId={projectId}
        />
      ) : (
        <Editor path={path} provider={provider} ydoc={ydoc} />
      )}
    </Panel>
  );
}

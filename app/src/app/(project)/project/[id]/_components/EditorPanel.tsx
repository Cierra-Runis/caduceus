'use client';

import { RefObject } from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

import { BinaryFileView } from './BinaryFileView';
import { Editor } from './Editor';
import { FileTab, FileTabs } from './FileTabs';

export interface EditorPanelProps {
  /// When set, the focused file is binary: render its preview instead of the
  /// code editor, so Monaco never opens it (which would create a text overlay
  /// the server would flush empty over the blob).
  binary: BinaryFile | null;
  editorPanelRef: RefObject<null | PanelImperativeHandle>;
  /// Close the tab with this id.
  onCloseTab: (id: string) => void;
  /// Focus the tab with this id.
  onSelectTab: (id: string) => void;
  provider: null | WebsocketProvider;
  /// Open editor tabs, in order; `textId` is the active one.
  tabs: FileTab[];
  textId: string;
  ydoc: Y.Doc;
}

interface BinaryFile {
  path: string;
  projectId: string;
  sha256: string;
  size: number;
}

export function EditorPanel({
  binary,
  editorPanelRef,
  onCloseTab,
  onSelectTab,
  provider,
  tabs,
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
    >
      <div className='flex h-full flex-col'>
        <FileTabs
          activeId={textId}
          onClose={onCloseTab}
          onSelect={onSelectTab}
          tabs={tabs}
        />
        <div className='min-h-0 flex-1 overflow-auto'>
          {tabs.length === 0 ? (
            <div className='grid h-full place-items-center text-sm opacity-50'>
              No file open
            </div>
          ) : binary ? (
            <BinaryFileView
              path={binary.path}
              projectId={binary.projectId}
              sha256={binary.sha256}
              size={binary.size}
            />
          ) : (
            <Editor provider={provider} textId={textId} ydoc={ydoc} />
          )}
        </div>
      </div>
    </Panel>
  );
}

'use client';

import { Spinner } from '@heroui/spinner';
import MonacoEditor from '@monaco-editor/react';
import { editor, IDisposable } from 'monaco-editor/esm/vs/editor/editor.api';
import { useTheme } from 'next-themes';
import { useEffect, useRef } from 'react';
import { SendJsonMessage } from 'react-use-websocket/dist/lib/types';

import {
  EditorCursorSelectionChangedMessage,
  EditorEditMessage,
  MessageType,
} from '@/lib/message';

export function Editor({ sendMessage }: { sendMessage: SendJsonMessage }) {
  const editorRef = useRef<editor.IStandaloneCodeEditor>(null);
  const disposableRef = useRef<IDisposable>(null);
  const { resolvedTheme } = useTheme();

  const handleMount = (editor: editor.IStandaloneCodeEditor) => {
    editorRef.current = editor;
    disposableRef.current = editor.onDidChangeCursorSelection((e) => {
      sendMessage<EditorCursorSelectionChangedMessage>({
        data: e,
        type: MessageType.CURSOR_SELECTION_CHANGED,
      });
    });
  };

  useEffect(() => {
    return () => {
      disposableRef.current?.dispose();
      disposableRef.current = null;
    };
  }, []);

  return (
    <MonacoEditor
      loading={<Spinner />}
      onChange={(_, event) =>
        sendMessage<EditorEditMessage>({
          data: event,
          type: MessageType.EDIT,
        })
      }
      onMount={handleMount}
      options={{
        cursorBlinking: 'smooth',
        fontFamily: 'var(--font-mono)',
        fontLigatures: true,
        smoothScrolling: true,
      }}
      theme={resolvedTheme === 'dark' ? 'vs-dark' : 'light'}
    />
  );
}

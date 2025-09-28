'use client';

import { Spinner } from '@heroui/spinner';
import MonacoEditor from '@monaco-editor/react';
import { useTheme } from 'next-themes';

export function Editor() {
  const { resolvedTheme } = useTheme();

  return (
    <MonacoEditor
      loading={<Spinner />}
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

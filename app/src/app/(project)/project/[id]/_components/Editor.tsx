'use client';

import { Spinner } from '@heroui/spinner';
import MonacoEditor from '@monaco-editor/react';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';

export function Editor() {
  const [mounted, setMounted] = useState(false);
  const { resolvedTheme } = useTheme();

  useEffect(() => setMounted(true), []);

  if (!mounted)
    return (
      <div className='flex h-full items-center justify-center'>
        <Spinner />
      </div>
    );

  return (
    <MonacoEditor
      loading={<Spinner />}
      theme={resolvedTheme === 'dark' ? 'vs-dark' : 'light'}
    />
  );
}

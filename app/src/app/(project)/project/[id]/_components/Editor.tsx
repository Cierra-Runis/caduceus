'use client';

import { Spinner } from '@heroui/spinner';
import CodeMirror from '@uiw/react-codemirror';
import { useTheme } from 'next-themes';
import { useCallback, useEffect, useState } from 'react';

export function Editor() {
  const [value, setValue] = useState("console.log('hello world!');");
  const [mounted, setMounted] = useState(false);
  const { resolvedTheme } = useTheme();

  useEffect(() => setMounted(true), []);

  const onChange = useCallback((val: string) => {
    console.log('val:', val);
    setValue(val);
  }, []);

  if (!mounted)
    return (
      <div className='flex h-full items-center justify-center'>
        <Spinner />
      </div>
    );

  return (
    <CodeMirror
      onChange={onChange}
      placeholder='Please enter some code...'
      theme={resolvedTheme === 'dark' ? 'dark' : 'light'}
      value={value}
    />
  );
}

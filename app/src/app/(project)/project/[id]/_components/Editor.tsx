'use client';

import { Spinner } from '@heroui/spinner';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';

export function Editor() {
  const [value, setValue] = useState("console.log('hello world!');");
  const [mounted, setMounted] = useState(false);
  const { resolvedTheme } = useTheme();

  useEffect(() => setMounted(true), []);

  if (!mounted)
    return (
      <div className='flex h-full items-center justify-center'>
        <Spinner />
      </div>
    );

  return <></>;
}

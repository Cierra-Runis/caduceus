'use client';

import { Button } from '@heroui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@heroui/popover';
import { CircularProgress } from '@heroui/progress';
import { IconDevices, IconMoon, IconSun } from '@tabler/icons-react';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';

const __themes = {
  dark: {
    icon: <IconMoon className='w-5' />,
    title: '暗色',
  },
  light: {
    icon: <IconSun className='w-5' />,
    title: '亮色',
  },
  system: {
    // Default theme is light
    icon: <IconDevices className='w-5' />,
    title: '系统',
  },
} as const;

type ThemeVariant = keyof typeof __themes;

export default function ThemeButton() {
  const [mounted, setMounted] = useState(false);
  const { resolvedTheme, setTheme, theme } = useTheme();

  useEffect(() => setMounted(true), []);

  if (!mounted) {
    return <CircularProgress size='sm' />;
  }

  return (
    <Popover aria-label='主题'>
      <PopoverTrigger>
        <Button
          isIconOnly
          size='sm'
          startContent={
            __themes[(resolvedTheme || 'system') as ThemeVariant].icon
          }
          variant='light'
        />
      </PopoverTrigger>
      <PopoverContent className='flex p-2 gap-1 flex-row'>
        {Object.entries(__themes).map(([key, { icon, title }]) => (
          <Button
            aria-label={title}
            isIconOnly
            key={key}
            onPress={() => setTheme(key)}
            size='sm'
            startContent={icon}
            variant={theme === key ? 'faded' : 'light'}
          />
        ))}
      </PopoverContent>
    </Popover>
  );
}

export function ThemeButtons() {
  const [mounted, setMounted] = useState(false);
  const { setTheme, theme } = useTheme();
  useEffect(() => setMounted(true), []);

  return (
    <>
      {Object.entries(__themes).map(([key, { icon, title }]) => (
        <Button
          aria-label={title}
          isIconOnly
          key={key}
          onPress={() => setTheme(key)}
          size='sm'
          startContent={icon}
          variant={!mounted ? 'light' : theme === key ? 'faded' : 'light'}
        />
      ))}
    </>
  );
}

'use client';

import { MonitorSmartphoneIcon, MoonIcon, SunIcon } from 'lucide-react';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';

import { Button } from '@/components/ui/button';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { Spinner } from '@/components/ui/spinner';

const __themes = {
  dark: {
    icon: <MoonIcon />,
    title: 'Dark',
  },
  light: {
    icon: <SunIcon />,
    title: 'Light',
  },
  system: {
    // Default theme is light
    icon: <MonitorSmartphoneIcon />,
    title: 'System',
  },
} as const;

type ThemeVariant = keyof typeof __themes;

export function ThemeButton() {
  const [mounted, setMounted] = useState(false);
  const { resolvedTheme, setTheme, theme } = useTheme();

  useEffect(() => setMounted(true), []);

  if (!mounted) {
    return <Spinner />;
  }

  return (
    <Popover aria-label='Theme Selector'>
      <PopoverTrigger>
        <Button size='sm'>
          {__themes[(resolvedTheme || 'system') as ThemeVariant].icon}
        </Button>
      </PopoverTrigger>
      <PopoverContent className='flex flex-row gap-1 p-2'>
        {Object.entries(__themes).map(([key, { icon, title }]) => (
          <Button
            aria-label={title}
            key={key}
            onClick={() => setTheme(key)}
            size='icon'
            variant={theme === key ? 'outline' : 'ghost'}
          >
            {icon}
          </Button>
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
          key={key}
          onClick={() => setTheme(key)}
          size='icon'
          variant={!mounted ? 'outline' : theme === key ? 'outline' : 'ghost'}
        >
          {icon}
        </Button>
      ))}
    </>
  );
}

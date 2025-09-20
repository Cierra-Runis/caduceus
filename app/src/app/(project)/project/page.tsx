'use client';

import { Divider } from '@heroui/divider';
import { Navbar, NavbarContent } from '@heroui/navbar';
import { Spinner } from '@heroui/spinner';
import CodeMirror from '@uiw/react-codemirror';
import { useTheme } from 'next-themes';
import { useCallback, useEffect, useState } from 'react';

import { logout } from '@/actions/auth';
import { ThemeButtons } from '@/components/buttons/ThemeButton';
import { Button } from '@heroui/button';
import { ScrollShadow } from '@heroui/scroll-shadow';
import {
  IconArchive,
  IconLogout,
  IconMap,
  IconPencil,
  IconSearch,
  IconSettings,
} from '@tabler/icons-react';

export default function Page() {
  const [value, setValue] = useState("console.log('hello world!');");
  const [mounted, setMounted] = useState(false);
  const { resolvedTheme } = useTheme();

  useEffect(() => setMounted(true), []);

  const onChange = useCallback((val: string) => {
    console.log('val:', val);
    setValue(val);
  }, []);

  if (!mounted) return <Spinner className='h-full' />;

  return (
    <div className='flex h-screen'>
      <ScrollShadow
        className='bg-default-50 relative flex h-full min-w-18 flex-col items-center overflow-auto pt-11 transition-all'
        hideScrollBar
      >
        <ScrollShadow className='flex w-full flex-1 flex-col' hideScrollBar>
          <Button
            className='aspect-square h-auto w-full flex-shrink-0'
            isIconOnly
            radius='none'
            variant='light'
          >
            <IconArchive />
          </Button>
          <Button
            className='aspect-square h-auto w-full flex-shrink-0'
            isIconOnly
            radius='none'
            variant='light'
          >
            <IconSearch />
          </Button>
          <Button
            className='aspect-square h-auto w-full flex-shrink-0'
            isIconOnly
            radius='none'
            variant='light'
          >
            <IconMap />
          </Button>
          <Button
            className='aspect-square h-auto w-full flex-shrink-0'
            isIconOnly
            radius='none'
            variant='light'
          >
            <IconPencil />
          </Button>
        </ScrollShadow>
        <div className='flex w-full flex-shrink-0 flex-col items-center'>
          <div className='flex aspect-square h-auto w-full flex-shrink-0 items-center justify-center'>
            <Button
              isIconOnly
              startContent={<IconSettings />}
              variant='light'
            />
          </div>
          <div className='flex aspect-square h-auto w-full flex-shrink-0 items-center justify-center'>
            <Button
              isIconOnly
              onPress={logout}
              startContent={<IconLogout />}
              variant='light'
            />
          </div>
        </div>
      </ScrollShadow>
      <section className='flex h-full flex-1 flex-col'>
        <Navbar
          className='bg-default-50 h-11'
          classNames={{ wrapper: 'pl-0 pr-1.5' }}
          maxWidth='full'
        >
          <NavbarContent className='gap-1' justify='end'>
            <ThemeButtons />
          </NavbarContent>
        </Navbar>
        <div className='flex h-full overflow-auto'>
          <div className='flex-1 overflow-auto scroll-auto'>
            <CodeMirror
              onChange={onChange}
              theme={resolvedTheme === 'dark' ? 'dark' : 'light'}
              value={value}
            />
          </div>
          <Divider orientation='vertical' />
          <div className='flex-1 overflow-auto scroll-auto'></div>
        </div>
      </section>
    </div>
  );
}

import { Image } from '@heroui/image';
import NextLink from 'next/link';

import { ServerBadge } from '@/components/badges/ServerBadge';
import { ThemeButtons } from '@/components/buttons/ThemeButton';
import { NavBar } from '@/components/roots/NavBar';

export default function Layout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <div className='flex h-screen flex-col'>
      <NavBar />
      {children}
      <footer className='mx-auto flex w-full max-w-7xl flex-col px-6 pb-4 md:flex-row-reverse md:justify-between'>
        <div className='flex items-center justify-center gap-2'>
          <ThemeButtons />
        </div>
        <div className='flex flex-col'>
          <div className='flex items-center justify-center gap-4 md:justify-start'>
            <div className='flex items-center gap-1'>
              <Image
                alt='Caduceus Icon'
                className='w-8 dark:invert'
                src='favicon.svg'
              />
              <span className='text-small font-medium'>Caduceus</span>
            </div>
            <div className='border-medium border-default text-small text-default-500 box-border inline-flex h-7 max-w-fit min-w-min items-center justify-between gap-2 rounded-full border-none bg-transparent whitespace-nowrap'>
              <ServerBadge />
              {/* TODO: Use `useServerStatus` */}
              <span className='flex-1 font-normal text-inherit'>
                Backend Status
              </span>
            </div>
          </div>
          <NextLink
            className='text-tiny text-default-400 text-center md:text-start'
            href='https://github.com/Cierra-Runis/caduceus/blob/main/LICENSE'
          >
            MIT Licensed | © 2025 Cierra Runis
          </NextLink>
        </div>
      </footer>
    </div>
  );
}

import type { Metadata } from 'next';

import { Image } from '@heroui/image';
import { Saira } from 'next/font/google';
import NextLink from 'next/link';

import '@/styles/globals.css';
import { ServerBadge } from '@/components/badges/ServerBadge';
import { ThemeButtons } from '@/components/buttons/ThemeButton';
import { NavBar } from '@/components/roots/NavBar';
import { Providers } from '@/components/roots/Providers';

const sans = Saira({
  subsets: ['latin'],
  variable: '--font-sans',
});

export const metadata: Metadata = {
  description: 'An open-source alternative to Typst App.',
  title: { default: 'Caduceus', template: '%s | Caduceus' },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html className={sans.variable} lang='en' suppressHydrationWarning>
      <body className='min-h-screen bg-background font-sans text-foreground antialiased'>
        <Providers>
          <div className='relative h-screen'>
            <NavBar />
            {children}
            <footer className='flex w-full flex-col mx-auto max-w-7xl px-6 pb-4 md:flex-row-reverse md:justify-between'>
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
                  <div className='max-w-fit min-w-min inline-flex items-center justify-between box-border whitespace-nowrap border-medium border-default bg-transparent h-7 text-small rounded-full text-default-500 border-none gap-2'>
                    <ServerBadge />
                    {/* TODO: Use `useServerStatus` */}
                    <span className='flex-1 text-inherit font-normal'>
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
        </Providers>
      </body>
    </html>
  );
}

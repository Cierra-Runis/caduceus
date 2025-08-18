import { ServerBadge } from '@/components/badges/ServerBadge';
import { NavBar } from '@/components/roots/NavBar';
import { Providers } from '@/components/roots/Providers';
import '@/styles/globals.css';
import { Image } from '@heroui/image';
import type { Metadata } from 'next';
import { Saira } from 'next/font/google';

const sans = Saira({
  variable: '--font-sans',
  subsets: ['latin'],
});

export const metadata: Metadata = {
  title: { default: 'Caduceus', template: '%s | Caduceus' },
  description: 'An open-source alternative to Typst App.',
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang='en' suppressHydrationWarning className={sans.variable}>
      <body className='min-h-screen bg-background font-sans text-foreground antialiased'>
        <Providers>
          <div className='relative h-screen'>
            <NavBar />
            {children}
            <footer className='flex w-full flex-col'>
              <div className='mx-auto w-full max-w-7xl px-6 pb-6 md:flex md:items-center md:justify-between lg:px-8'>
                <div className='flex flex-col items-center justify-center gap-2 md:order-2 md:items-end'>
                  <div
                    className='relative flex flex-col gap-2'
                    aria-label='Select a theme'
                    role='radiogroup'
                  >
                    <div
                      className='flex flex-col flex-wrap data-[orientation=horizontal]:flex-row gap-0 items-center'
                      role='presentation'
                      data-orientation='horizontal'
                    >
                      114514
                    </div>
                  </div>
                </div>
                <div className='mt-4 md:order-1 md:mt-0'>
                  <div className='flex items-center justify-center gap-3 md:justify-start'>
                    <div className='flex items-center gap-1'>
                      <Image src="favicon.svg" alt='Caduceus Icon' className='w-6 dark:invert'></Image>
                      <span className='text-small font-medium'>Caduceus</span>
                    </div>
                    <div
                      className='shrink-0 bg-divider border-none w-divider h-4'
                      role='separator'
                      data-orientation='vertical'
                      aria-orientation='vertical'
                    ></div>
                    <div className='relative max-w-fit min-w-min inline-flex items-center justify-between box-border whitespace-nowrap border-medium border-default bg-transparent h-7 text-small rounded-full text-default-500 border-none px-0'>
                      <ServerBadge />
                      {/* TODO: Use `useServerStatus` */}
                      <span className='flex-1 text-inherit font-normal px-2'>
                        All systems operational
                      </span>
                    </div>
                  </div>
                  <p className='text-tiny text-default-400 text-center md:text-start'>
                    © 2024 Acme Inc. All rights reserved.
                  </p>
                </div>
              </div>
            </footer>
          </div>
        </Providers>
      </body>
    </html>
  );
}

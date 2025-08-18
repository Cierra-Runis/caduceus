import { ServerBadge } from '@/components/badges/ServerBadge';
import { ThemeButtons } from '@/components/buttons/ThemeButton';
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
            <footer className='flex w-full flex-col mx-auto max-w-7xl px-6 pb-6 md:flex-row-reverse md:justify-between'>
              <div className='flex items-center justify-center gap-2'>
                <ThemeButtons />
              </div>
              <div>
                <div className='flex items-center justify-center gap-3 md:justify-start'>
                  <div className='flex items-center gap-1'>
                    <Image src="favicon.svg" alt='Caduceus Icon' className='w-6 dark:invert'></Image>
                    <span className='text-small font-medium'>Caduceus</span>
                  </div>
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
            </footer>
          </div>
        </Providers>
      </body>
    </html>
  );
}

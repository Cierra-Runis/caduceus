import { Saira } from 'next/font/google';

import { Providers } from '@/components/roots/Providers';
import Sidebar from '@/components/Sidebar';
import '@/styles/globals.css';

const sans = Saira({
  subsets: ['latin'],
  variable: '--font-sans',
});

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html className={sans.variable} lang='en' suppressHydrationWarning>
      <body className='min-h-screen bg-background font-sans text-foreground antialiased'>
        <Providers>
          <div className='relative h-screen flex w-full'>
            <Sidebar />
            <section className='w-full flex flex-col p-4 h-full'>
              <header className='rounded-medium border-small border-divider flex items-center gap-3 p-4 flex-shrink-0'>
                <h2 className='text-medium text-default-700 font-medium'>
                  Overview
                </h2>
              </header>
              <main className='flex-1 mt-4 overflow-auto'>
                <div className='rounded-medium border-small border-divider flex h-full w-full flex-col gap-4 p-4'>
                  {children}
                </div>
              </main>
            </section>
          </div>
        </Providers>
      </body>
    </html>
  );
}

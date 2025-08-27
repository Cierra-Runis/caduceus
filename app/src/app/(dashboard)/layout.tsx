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
      <body className='bg-background text-foreground min-h-screen font-sans antialiased'>
        <Providers>
          <div className='relative flex h-screen w-full'>
            <Sidebar />
            <section className='flex h-full w-full flex-col p-4'>
              <header className='rounded-medium border-small border-divider flex flex-shrink-0 items-center gap-3 p-4'>
                <h2 className='text-medium text-default-700 font-medium'>
                  Overview
                </h2>
              </header>
              <main className='mt-4 flex-1 overflow-auto'>
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

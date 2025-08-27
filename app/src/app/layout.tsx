import { Metadata } from 'next';

import '@/styles/globals.css';
import { Saira } from 'next/font/google';

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
      <body className='bg-background text-foreground min-h-screen font-sans antialiased'>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}

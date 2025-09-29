import { Metadata } from 'next';

import '@/styles/globals.css';
import { NextIntlClientProvider, useLocale } from 'next-intl';
import { Cascadia_Code, Saira } from 'next/font/google';
import Script from 'next/script';

import { Providers } from '@/components/roots/Providers';

const sans = Saira({
  subsets: ['latin'],
  variable: '--font-sans',
});

const mono = Cascadia_Code({
  subsets: ['latin'],
  variable: '--font-mono',
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
  const locale = useLocale();
  return (
    <html
      className={`
        ${sans.variable}
        ${mono.variable}
      `}
      lang={locale}
      suppressHydrationWarning
    >
      <head>
        <Script
          async
          crossOrigin='anonymous'
          src='//unpkg.com/react-scan/dist/auto.global.js'
        />
      </head>
      <body
        className={`
          min-h-screen bg-background font-sans text-foreground antialiased
        `}
      >
        <NextIntlClientProvider>
          <Providers>{children}</Providers>
        </NextIntlClientProvider>
      </body>
    </html>
  );
}

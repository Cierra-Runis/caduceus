import { Metadata } from 'next';

import '@/styles/globals.css';
import { NextIntlClientProvider, useLocale } from 'next-intl';
import { Kode_Mono, Saira } from 'next/font/google';
import Script from 'next/script';

import { Providers } from '@/components/roots/Providers';
import { cn } from '@/lib/utils';

const geist = Saira({ subsets: ['latin'], variable: '--font-sans' });

const mono = Kode_Mono({
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
      className={cn(mono.variable, geist.variable)}
      lang={locale}
      suppressHydrationWarning
    >
      <head>
        {process.env.NODE_ENV === 'development' && (
          <Script
            async
            crossOrigin='anonymous'
            src='//unpkg.com/react-scan/dist/auto.global.js'
          />
        )}
      </head>
      <body className={`min-h-screen antialiased`}>
        <NextIntlClientProvider>
          <Providers>{children}</Providers>
        </NextIntlClientProvider>
      </body>
    </html>
  );
}

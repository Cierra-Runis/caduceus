'use client';

import { HeroUIProvider } from '@heroui/system';
import { ToastProvider } from '@heroui/toast';
import { ThemeProvider as NextThemesProvider } from 'next-themes';
import { useRouter } from 'next/navigation';
import React from 'react';

export interface ProvidersProps {
  children: React.ReactNode;
}

export function Providers({ children }: ProvidersProps) {
  const router = useRouter();

  return (
    // FIXME: https://github.com/heroui-inc/heroui/issues/5643
    <HeroUIProvider
      navigate={(path, options) => router.push(path as never, options)}
    >
      <ToastProvider />
      <NextThemesProvider attribute='class' enableSystem>
        {children}
      </NextThemesProvider>
    </HeroUIProvider>
  );
}

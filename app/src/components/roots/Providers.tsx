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
    <HeroUIProvider navigate={(path) => router.push(path as never)}>
      <ToastProvider />
      <NextThemesProvider attribute='class' enableSystem>
        {children}
      </NextThemesProvider>
    </HeroUIProvider>
  );
}

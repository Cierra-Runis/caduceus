import { ThemeProvider as NextThemesProvider } from 'next-themes';
import React from 'react';

import { Toaster } from '@/components/ui/sonner';
import { TooltipProvider } from '@/components/ui/tooltip';

export interface ProvidersProps {
  children: React.ReactNode;
}

export function Providers({ children }: ProvidersProps) {
  return (
    <NextThemesProvider attribute='class' enableSystem>
      <Toaster />
      <TooltipProvider>{children}</TooltipProvider>
    </NextThemesProvider>
  );
}

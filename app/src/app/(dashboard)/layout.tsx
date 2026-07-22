import { CSSProperties } from 'react';

import { ServerBadge } from '@/components/badges/ServerBadge';
import { ThemeButtons } from '@/components/buttons/ThemeButton';
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar';
import '@/styles/globals.css';

import { NavBar } from './_components/NavBar';
import { SideBar } from './_components/SideBar';

export default function Layout({ children, header }: LayoutProps<'/'>) {
  return (
    <SidebarProvider
      style={
        {
          '--sidebar-width': '4.5rem',
        } as CSSProperties
      }
    >
      <SideBar />
      <SidebarInset className='flex h-screen w-18 flex-col'>
        <NavBar header={header} />
        {children}
        <footer
          className={`
            mx-auto flex w-full flex-col px-6 pb-4
            md:flex-row-reverse md:justify-between
          `}
        >
          <div className='flex items-center justify-center gap-2'>
            <ThemeButtons />
          </div>
          <div
            className={`
              flex items-center justify-center gap-4
              md:justify-start
            `}
          >
            <ServerBadge />
          </div>
        </footer>
      </SidebarInset>
    </SidebarProvider>
  );
}

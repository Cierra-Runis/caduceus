import { ServerBadge } from '@/components/badges/ServerBadge';
import { ThemeButtons } from '@/components/buttons/ThemeButton';
import { NavBar } from '@/components/roots/NavBar';
import '@/styles/globals.css';

export default function Layout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <div className='flex h-screen flex-col'>
      <NavBar />
      {children}
      <footer
        className={`
          mx-auto flex w-full max-w-7xl flex-col px-6 pb-4
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
    </div>
  );
}

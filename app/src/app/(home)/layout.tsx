import { useTranslations } from 'next-intl';
import NextLink from 'next/link';

import { ServerBadge } from '@/components/badges/ServerBadge';
import { ThemeButtons } from '@/components/buttons/ThemeButton';
import { Icon } from '@/components/Icon';
import { NavBar } from '@/components/roots/NavBar';
import '@/styles/globals.css';

export default function Layout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const t = useTranslations();
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
        <div className='flex flex-col'>
          <div
            className={`
              flex items-center justify-center gap-4
              md:justify-start
            `}
          >
            <div className='flex items-center gap-1'>
              <Icon className='w-8' />
              <span className='text-small font-medium'>
                {t('Layout.caduceus')}
              </span>
            </div>
            <div
              className={`
                box-border inline-flex h-7 max-w-fit min-w-min items-center
                justify-between gap-2 rounded-full border-medium border-none
                border-default bg-transparent text-small whitespace-nowrap
                text-foreground-500
              `}
            >
              <ServerBadge />
              {/* TODO: Use `useServerStatus` */}
              <span className='flex-1 font-normal text-inherit'>
                {t('Layout.backendStatus')}
              </span>
            </div>
          </div>
          <NextLink
            className={`
              text-center text-tiny text-foreground-500
              md:text-start
            `}
            href='https://github.com/Cierra-Runis/caduceus/blob/main/LICENSE'
          >
            {t('Layout.licenseText')}
          </NextLink>
        </div>
      </footer>
    </div>
  );
}

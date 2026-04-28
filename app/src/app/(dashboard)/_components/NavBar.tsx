import { useTranslations } from 'next-intl';
import NextLink from 'next/link';

import { logout } from '@/actions/auth';
import {
  Menubar,
  MenubarContent,
  MenubarGroup,
  MenubarItem,
  MenubarMenu,
  MenubarTrigger,
} from '@/components/ui/menubar';

export const NavBar = () => {
  const t = useTranslations();
  return (
    <header
      className={`sticky top-0 z-40 h-auto flex-none border-b bg-background`}
    >
      <div className='mx-auto flex h-11 items-center gap-4 px-0.5'>
        <Menubar className='w-full gap-4 border-none'>
          <MenubarMenu>
            <MenubarTrigger className='font-bold'>
              {t('Layout.caduceus')}
            </MenubarTrigger>
            <MenubarContent>
              <MenubarGroup>
                <MenubarItem asChild>
                  <NextLink href='#'>{t('Layout.about')}</NextLink>
                  {/* <NextLink href='/about'>{t('Layout.about')}</NextLink> */}
                </MenubarItem>
                <MenubarItem asChild>
                  <NextLink href='/dashboard/settings'>
                    {t('Layout.accountSettings')}
                  </NextLink>
                </MenubarItem>
                <MenubarItem onSelect={logout}>
                  {t('Layout.logout')}
                </MenubarItem>
                <MenubarItem asChild>
                  <NextLink href='/home'>{t('Layout.goToLanding')}</NextLink>
                </MenubarItem>
              </MenubarGroup>
            </MenubarContent>
          </MenubarMenu>
          <MenubarMenu>
            <MenubarTrigger>{t('Layout.project')}</MenubarTrigger>
            <MenubarContent>
              <MenubarGroup>
                <MenubarItem>{t('Layout.newProject')}</MenubarItem>
                <MenubarItem>{t('Layout.incomingInvites')}</MenubarItem>
              </MenubarGroup>
            </MenubarContent>
          </MenubarMenu>
        </Menubar>
      </div>
    </header>
  );
};

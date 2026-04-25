import { useTranslations } from 'next-intl';
import NextLink from 'next/link';

import { Avatar, AvatarImage } from '@/components/ui/avatar';
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
} from '@/components/ui/navigation-menu';

import { Button } from '../ui/button';

export const NavBar = () => {
  const t = useTranslations();
  return (
    <header
      className={`sticky top-0 z-40 h-auto flex-none border-b bg-background`}
    >
      <div className='mx-auto flex h-16 max-w-7xl items-center gap-4 px-6'>
        <div className='flex items-center gap-4'>
          <NextLink className='mr-auto flex items-center gap-2' href='/'>
            <Avatar>
              <AvatarImage src='/icon.svg' />
            </Avatar>
            <span className='font-bold'>{t('Layout.caduceus')}</span>
          </NextLink>
          <NavigationMenu>
            <NavigationMenuList className='gap-4'>
              <NavigationMenuItem>
                <NavigationMenuLink asChild>
                  <NextLink href='https://github.com/Cierra-Runis/caduceus/wiki'>
                    {t('Layout.wiki')}
                  </NextLink>
                </NavigationMenuLink>
              </NavigationMenuItem>
              <NavigationMenuItem>
                <NavigationMenuLink asChild></NavigationMenuLink>
              </NavigationMenuItem>
              <NavigationMenuItem>
                <NavigationMenuLink asChild></NavigationMenuLink>
              </NavigationMenuItem>
            </NavigationMenuList>
          </NavigationMenu>
        </div>
        <div className='ml-auto flex items-center gap-4'>
          <Button asChild variant='ghost'>
            <NextLink href='/login'>{t('Login.login')}</NextLink>
          </Button>
          <Button asChild>
            <NextLink href='/register'>{t('Register.register')}</NextLink>
          </Button>
        </div>
      </div>
    </header>
  );
};

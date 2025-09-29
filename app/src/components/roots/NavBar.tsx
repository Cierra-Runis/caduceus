import { Button } from '@heroui/button';
import { Image } from '@heroui/image';
import { Navbar, NavbarBrand, NavbarContent, NavbarItem } from '@heroui/navbar';
import { useTranslations } from 'next-intl';
import NextLink from 'next/link';

export const NavBar = () => {
  const t = useTranslations();
  return (
    <Navbar isBordered maxWidth='xl' shouldHideOnScroll>
      <NavbarBrand className='max-w-fit gap-3'>
        <NextLink className='flex items-center gap-2' href='/'>
          <Image
            alt={t('Layout.caduceus')}
            className='w-12 dark:invert'
            src='favicon.svg'
          ></Image>
          <span className='hidden text-lg font-semibold sm:inline'>
            {t('Layout.caduceus')}
          </span>
        </NextLink>
      </NavbarBrand>
      <NavbarContent className='basis-full gap-1' justify='end'>
        <NavbarItem>
          <Button
            as={NextLink}
            className='text-sm font-medium'
            href='https://github.com/Cierra-Runis/caduceus/wiki'
            size='sm'
            variant='light'
          >
            {t('Layout.wiki')}
          </Button>
        </NavbarItem>
        <NavbarItem>
          <Button
            as={NextLink}
            className='text-sm font-medium'
            href='/login'
            size='sm'
            variant='light'
          >
            {t('Login.login')}
          </Button>
        </NavbarItem>
        <NavbarItem>
          <Button
            as={NextLink}
            className='text-sm font-medium'
            color='primary'
            href='/register'
            size='sm'
            variant='shadow'
          >
            {t('Register.register')}
          </Button>
        </NavbarItem>
      </NavbarContent>
    </Navbar>
  );
};

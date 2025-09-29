import { Button } from '@heroui/button';
import { Image } from '@heroui/image';
import { Navbar, NavbarBrand, NavbarContent, NavbarItem } from '@heroui/navbar';
import NextLink from 'next/link';

export const NavBar = () => {
  return (
    <Navbar isBordered maxWidth='xl' shouldHideOnScroll>
      <NavbarBrand className='max-w-fit gap-3'>
        <NextLink className='flex items-center gap-2' href='/'>
          <Image
            alt='Caduceus Icon'
            className='w-12 dark:invert'
            src='favicon.svg'
          ></Image>
          <span className='hidden text-lg font-semibold sm:inline'>
            Caduceus
          </span>
        </NextLink>
      </NavbarBrand>
      <NavbarContent className='basis-full' justify='end'>
        <NavbarItem>
          <NextLink
            className='text-sm font-medium'
            href='https://github.com/Cierra-Runis/caduceus/wiki'
          >
            Docs
          </NextLink>
        </NavbarItem>
        <NavbarItem>
          <NextLink className='text-sm font-medium' href='/login'>
            Login
          </NextLink>
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
            Register
          </Button>
        </NavbarItem>
      </NavbarContent>
    </Navbar>
  );
};

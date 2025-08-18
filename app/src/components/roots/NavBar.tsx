'use client';

import { Button } from '@heroui/button';
import { Image } from '@heroui/image';
import { Navbar, NavbarBrand, NavbarContent, NavbarItem } from '@heroui/navbar';
import NextLink from 'next/link';
import { useState } from 'react';


export const NavBar = () => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  return (
    <Navbar
      isBordered
      isMenuOpen={isMenuOpen}
      maxWidth='xl'
      onMenuOpenChange={setIsMenuOpen}
      shouldHideOnScroll
    >
      <NavbarBrand className='max-w-fit gap-3'>
        <NextLink href='/' className='flex items-center gap-2'>
          <Image src="favicon.svg" alt='Caduceus Icon' className='w-12 dark:invert'></Image>
          <span className='hidden sm:inline font-semibold text-lg'>Caduceus</span>
        </NextLink>
      </NavbarBrand>
      <NavbarContent className='basis-full' justify='end'>
        <NavbarItem>
          <NextLink href='/docs' className='text-sm font-medium'>
            Docs
          </NextLink>
        </NavbarItem>
        <NavbarItem>
          <NextLink href='/login' className='text-sm font-medium'>
            Login
          </NextLink>
        </NavbarItem>
        <NavbarItem>
          <Button href='/register' className='text-sm font-medium' variant='faded' as={NextLink}>
            Register
          </Button>
        </NavbarItem>
      </NavbarContent>
    </Navbar>
  );
};

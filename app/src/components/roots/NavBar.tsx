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
          <NextLink className='text-sm font-medium' href='/docs'>
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
            href='/register'
            variant='faded'
          >
            Register
          </Button>
        </NavbarItem>
      </NavbarContent>
    </Navbar>
  );
};

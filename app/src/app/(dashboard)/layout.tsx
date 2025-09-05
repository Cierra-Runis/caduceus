'use client';
import { Breadcrumbs } from '@heroui/breadcrumbs';
import { Button, ButtonGroup } from '@heroui/button';
import { Divider } from '@heroui/divider';
import { Listbox, ListboxItem } from '@heroui/listbox';
import { Navbar, NavbarBrand, NavbarContent, NavbarItem } from '@heroui/navbar';
import { Tooltip } from '@heroui/tooltip';
import Link from 'next/link';

import { logout } from '@/actions/auth';
import { Sidebar } from '@/components/Sidebar';

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className='flex h-screen'>
      <Sidebar />
      <Divider orientation='vertical' />
      <section className='flex flex-1 flex-col'>
        <Navbar isBordered maxWidth='full'>
          <NavbarBrand>
            <ButtonGroup size='sm' variant='light'>
              <Tooltip
                content={
                  <Listbox>
                    <ListboxItem key='about'>About Caduceus</ListboxItem>
                    <ListboxItem href='/dashboard/settings' key='settings'>
                      About Caduceus
                    </ListboxItem>
                    <ListboxItem key='logout' onPress={logout}>
                      Logout
                    </ListboxItem>
                  </Listbox>
                }
                placement='bottom-start'
              >
                <Button>Caduceus</Button>
              </Tooltip>
              <Tooltip
                content='Caduceus - AI Medical Assistant'
                placement='bottom-start'
              >
                <Button>Caduceus</Button>
              </Tooltip>
              <Tooltip
                content='Caduceus - AI Medical Assistant'
                placement='bottom-start'
              >
                <Button>Caduceus</Button>
              </Tooltip>
              <Tooltip
                content='Caduceus - AI Medical Assistant'
                placement='bottom-start'
              >
                <Button>Caduceus</Button>
              </Tooltip>
              <Tooltip
                content='Caduceus - AI Medical Assistant'
                placement='bottom-start'
              >
                <Button>Caduceus</Button>
              </Tooltip>
              <Tooltip
                content='Caduceus - AI Medical Assistant'
                placement='bottom-start'
              >
                <Button>Caduceus</Button>
              </Tooltip>
            </ButtonGroup>
          </NavbarBrand>
          <NavbarContent className='hidden gap-4 sm:flex' justify='center'>
            <Breadcrumbs></Breadcrumbs>
          </NavbarContent>
          <NavbarContent justify='end'>
            <NavbarItem className='hidden lg:flex'>
              <Link href='#'>Login</Link>
            </NavbarItem>
            <NavbarItem>
              <Button as={Link} color='primary' href='#' variant='flat'>
                Sign Up
              </Button>
            </NavbarItem>
          </NavbarContent>
        </Navbar>
        <div className='flex-1 overflow-auto'>{children}</div>
      </section>
    </div>
  );
}

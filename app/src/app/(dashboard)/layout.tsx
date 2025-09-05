'use client';
import { Button, ButtonGroup } from '@heroui/button';
import { Listbox, ListboxItem } from '@heroui/listbox';
import { Navbar, NavbarBrand, NavbarContent } from '@heroui/navbar';
import { Tooltip } from '@heroui/tooltip';

import { logout } from '@/actions/auth';
import { ThemeButtons } from '@/components/buttons/ThemeButton';
import { Sidebar } from '@/components/Sidebar';
import '@/styles/globals.css';

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className='flex h-screen'>
      <Sidebar />
      <section className='flex flex-1 flex-col'>
        <Navbar
          className='bg-default-50 h-11'
          classNames={{ wrapper: 'pl-0 pr-1.5' }}
          maxWidth='full'
        >
          <NavbarBrand className='gap-2'>
            {/* TODO: Use Menubar */}
            <ButtonGroup variant='light'>
              <Tooltip
                content={
                  <Listbox>
                    <ListboxItem href='/about' key='about'>
                      About Caduceus
                    </ListboxItem>
                    <ListboxItem href='/dashboard/settings' key='settings'>
                      Settings
                    </ListboxItem>
                    <ListboxItem key='logout' onPress={logout}>
                      Logout
                    </ListboxItem>
                    <ListboxItem href='/home' key='home'>
                      Go to landing page
                    </ListboxItem>
                  </Listbox>
                }
                placement='bottom-start'
              >
                <Button className='font-bold'>Caduceus</Button>
              </Tooltip>
              <Tooltip
                content={
                  <Listbox>
                    <ListboxItem key='new-project'>New Project</ListboxItem>
                    <ListboxItem key='incoming-invites'>
                      Incoming Invites
                    </ListboxItem>
                  </Listbox>
                }
                placement='bottom-start'
              >
                <Button>Project</Button>
              </Tooltip>
              <Tooltip
                content={
                  <Listbox>
                    <ListboxItem key='new-team'>New Team</ListboxItem>
                    <ListboxItem key='manage-teams'>Manage Teams</ListboxItem>
                    <ListboxItem key='incoming-invites'>
                      Incoming Invites
                    </ListboxItem>
                  </Listbox>
                }
                placement='bottom-start'
              >
                <Button>Team</Button>
              </Tooltip>
              <Tooltip
                content={
                  <Listbox>
                    <ListboxItem
                      href='https://typst.app/docs/tutorial'
                      key='tutorial'
                    >
                      Tutorial
                    </ListboxItem>
                    <ListboxItem
                      href='https://typst.app/docs/reference/'
                      key='reference'
                    >
                      Reference
                    </ListboxItem>
                    <ListboxItem key='feedback'>Feedback</ListboxItem>
                    <ListboxItem href='/contact' key='contact'>
                      Contact
                    </ListboxItem>
                  </Listbox>
                }
                placement='bottom-start'
              >
                <Button>Help</Button>
              </Tooltip>
            </ButtonGroup>
            <NavbarContent className='gap-1' justify='end'>
              <ThemeButtons />
            </NavbarContent>
          </NavbarBrand>
        </Navbar>
        <div className='flex-1 overflow-auto'>{children}</div>
      </section>
    </div>
  );
}

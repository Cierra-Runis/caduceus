'use client';
import { Button, ButtonGroup } from '@heroui/button';
import { Chip } from '@heroui/chip';
import { Listbox, ListboxItem } from '@heroui/listbox';
import { Navbar, NavbarBrand, NavbarContent } from '@heroui/navbar';
import { Tooltip } from '@heroui/tooltip';
import { IconCloud } from '@tabler/icons-react';

import { logout } from '@/actions/auth';
import { ThemeButtons } from '@/components/buttons/ThemeButton';
import { Sidebar } from '@/components/Sidebar';
import { useUserMe } from '@/hooks/useUserMe';
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
              <CaduceusButton />
              <ProjectButton />
              <TeamButton />
              <HelpButton />
            </ButtonGroup>
          </NavbarBrand>
          <NavbarContent justify='center'>
            <Header />
          </NavbarContent>
          <NavbarContent className='gap-1' justify='end'>
            <ThemeButtons />
          </NavbarContent>
        </Navbar>
        <div className='flex-1 overflow-auto'>{children}</div>
      </section>
    </div>
  );
}

function CaduceusButton() {
  return (
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
  );
}

// TODO: Dynamic header info
function Header() {
  const { data } = useUserMe();
  return (
    <Chip startContent={<IconCloud className='w-[1.25em]' />} variant='light'>
      {data?.payload?.username ?? 'Caduceus'}
    </Chip>
  );
}

function HelpButton() {
  return (
    <Tooltip
      content={
        <Listbox>
          <ListboxItem href='https://typst.app/docs/tutorial' key='tutorial'>
            Tutorial
          </ListboxItem>
          <ListboxItem href='https://typst.app/docs/reference/' key='reference'>
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
  );
}

function ProjectButton() {
  return (
    <Tooltip
      content={
        <Listbox>
          <ListboxItem key='new-project'>New Project</ListboxItem>
          <ListboxItem key='incoming-invites'>Incoming Invites</ListboxItem>
        </Listbox>
      }
      placement='bottom-start'
    >
      <Button>Project</Button>
    </Tooltip>
  );
}

function TeamButton() {
  return (
    <Tooltip
      content={
        <Listbox>
          <ListboxItem key='new-team'>New Team</ListboxItem>
          <ListboxItem key='manage-teams'>Manage Teams</ListboxItem>
          <ListboxItem key='incoming-invites'>Incoming Invites</ListboxItem>
        </Listbox>
      }
      placement='bottom-start'
    >
      <Button>Team</Button>
    </Tooltip>
  );
}

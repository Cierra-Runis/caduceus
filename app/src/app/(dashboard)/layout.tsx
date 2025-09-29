'use client';

import { Button, ButtonGroup } from '@heroui/button';
import { Chip } from '@heroui/chip';
import { Listbox, ListboxItem } from '@heroui/listbox';
import { Navbar, NavbarBrand, NavbarContent } from '@heroui/navbar';
import { Tooltip } from '@heroui/tooltip';
import { IconCloud } from '@tabler/icons-react';
import { useTranslations } from 'next-intl';
import NextLink from 'next/link';

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
          className='h-11 bg-content1'
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
  const t = useTranslations('Layout');
  return (
    <Tooltip
      content={
        <Listbox>
          <ListboxItem as={NextLink} href='/about' key='about'>
            {t('about')}
          </ListboxItem>
          <ListboxItem as={NextLink} href='/dashboard/settings' key='settings'>
            {t('accountSettings')}
          </ListboxItem>
          <ListboxItem key='logout' onPress={logout}>
            {t('logout')}
          </ListboxItem>
          <ListboxItem as={NextLink} href='/home' key='home'>
            {t('goToLanding')}
          </ListboxItem>
        </Listbox>
      }
      placement='bottom-start'
    >
      <Button className='font-bold'>{t('caduceus')}</Button>
    </Tooltip>
  );
}

// TODO: Dynamic header info
function Header() {
  const { data } = useUserMe();
  const t = useTranslations('Layout');
  return (
    <Chip startContent={<IconCloud className='w-[1.25em]' />} variant='light'>
      {data?.payload?.username ?? t('caduceus')}
    </Chip>
  );
}

function HelpButton() {
  const t = useTranslations('Layout');
  return (
    <Tooltip
      content={
        <Listbox>
          <ListboxItem
            as={NextLink}
            href='https://typst.app/docs/tutorial'
            key='tutorial'
          >
            {t('tutorial')}
          </ListboxItem>
          <ListboxItem
            as={NextLink}
            href='https://typst.app/docs/reference/'
            key='reference'
          >
            {t('reference')}
          </ListboxItem>
          <ListboxItem key='feedback'>{t('feedback')}</ListboxItem>
          <ListboxItem as={NextLink} href='/contact' key='contact'>
            {t('contact')}
          </ListboxItem>
        </Listbox>
      }
      placement='bottom-start'
    >
      <Button>{t('help')}</Button>
    </Tooltip>
  );
}

function ProjectButton() {
  const t = useTranslations('Layout');
  return (
    <Tooltip
      content={
        <Listbox>
          <ListboxItem key='new-project'>{t('newProject')}</ListboxItem>
          <ListboxItem key='incoming-invites'>
            {t('incomingInvites')}
          </ListboxItem>
        </Listbox>
      }
      placement='bottom-start'
    >
      <Button>{t('project')}</Button>
    </Tooltip>
  );
}

function TeamButton() {
  const t = useTranslations('Layout');
  return (
    <Tooltip
      content={
        <Listbox>
          <ListboxItem key='new-team'>{t('newTeam')}</ListboxItem>
          <ListboxItem key='manage-teams'>{t('manageTeams')}</ListboxItem>
          <ListboxItem key='incoming-invites'>
            {t('incomingInvites')}
          </ListboxItem>
        </Listbox>
      }
      placement='bottom-start'
    >
      <Button>{t('team')}</Button>
    </Tooltip>
  );
}

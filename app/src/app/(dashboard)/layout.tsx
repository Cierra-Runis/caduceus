import { ButtonGroup } from '@heroui/button';
import { Navbar, NavbarBrand, NavbarContent } from '@heroui/navbar';

import { ThemeButtons } from '@/components/buttons/ThemeButton';
import { Sidebar } from '@/components/Sidebar';
import '@/styles/globals.css';

import { CaduceusButton } from './_components/CaduceusButton';
import { Header } from './_components/Header';
import { HelpButton } from './_components/HelpButton';
import { ProjectButton } from './_components/ProjectButton';
import { TeamButton } from './_components/TeamButton';

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

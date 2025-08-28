'use client';

import { Listbox, ListboxItem, ListboxSection } from '@heroui/listbox';
import { ScrollShadow } from '@heroui/scroll-shadow';
import {
  IconHome,
  IconLogout,
  IconPackage,
  IconSettings,
  IconSubtask,
  IconUser,
} from '@tabler/icons-react';
import { useTranslations } from 'next-intl';
import { usePathname } from 'next/navigation';

export default function Sidebar() {
  const pathname = usePathname();
  const t = useTranslations('Sidebar');

  return (
    <aside className='border-r-small border-divider transition-width  relative flex h-full w-72 flex-col justify-between p-6'>
      <header className='flex flex-col items-center justify-center gap-4'>
        <h2 className='text-2xl font-bold'>Caduceus</h2>
        {/* <User avatarProps={{
          src: "https://images.unsplash.com/photo-1508214751196-bcfd4ca60f91?ixlib=rb-4.0.3&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D&auto=format&fit=crop&w=1470&q=80",
        }}
          description=("Proje{tct Manager")
          name}="Jane Doe"
        /> */}
      </header>

      <ScrollShadow className='flex flex-1 items-center'>
        <Listbox
          classNames={{ base: 'w-full' }}
          selectedKeys={[pathname]}
          selectionMode='single'
        >
          <ListboxSection title='Overview'>
            <ListboxItem
              classNames={{ description: 'text-md' }}
              description={t('home')}
              href='/dashboard'
              key='/dashboard'
              startContent={<IconHome />}
            />
            <ListboxItem
              classNames={{ description: 'text-md' }}
              description={t('projects')}
              href='/dashboard/projects'
              key='/dashboard/projects'
              startContent={<IconPackage />}
            />
            <ListboxItem
              classNames={{ description: 'text-md' }}
              description={t('tasks')}
              href='/dashboard/tasks'
              key='/dashboard/tasks'
              startContent={<IconSubtask />}
            />
            <ListboxItem
              classNames={{ description: 'text-md' }}
              description={t('teams')}
              href='/dashboard/teams'
              key='/dashboard/teams'
              startContent={<IconUser />}
            />
          </ListboxSection>
          <ListboxSection title='Organization'>
            <ListboxItem
              classNames={{ description: 'text-md' }}
              description={t('members')}
              href='/dashboard/members'
              key='/dashboard/members'
              startContent={<IconUser />}
            />
            <ListboxItem
              classNames={{ description: 'text-md' }}
              description={t('settings')}
              href='/dashboard/settings'
              key='/dashboard/settings'
              startContent={<IconSettings />}
            />
          </ListboxSection>
        </Listbox>
      </ScrollShadow>

      <Listbox className='mt-auto'>
        <ListboxSection>
          <ListboxItem
            classNames={{ description: 'text-md' }}
            description={t('logout')}
            href='/logout'
            key='/logout'
            startContent={<IconLogout />}
          />
        </ListboxSection>
      </Listbox>
    </aside>
  );
}

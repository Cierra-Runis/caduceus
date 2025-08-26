"use client";

import { Providers } from '@/components/roots/Providers';
import "@/styles/globals.css";
import { Button } from '@heroui/button';
import {
  Listbox,
  ListboxItem,
  ListboxSection
} from "@heroui/listbox";
import { ScrollShadow } from "@heroui/scroll-shadow";
import { IconHome, IconLabel, IconListTree, IconLogout, IconPackage, IconSettings, IconSubtask, IconUser } from "@tabler/icons-react";
import { Saira } from 'next/font/google';
import { useState } from 'react';

const sans = Saira({
  variable: '--font-sans',
  subsets: ['latin'],
});

export default function Layout({
  children,
}: {
  children: React.ReactNode;
}) {
  const [isCollapsed, setIsCollapsed] = useState(false);

  return (
    <html lang="en" suppressHydrationWarning className={sans.variable}>
      <body className="min-h-screen bg-background font-sans text-foreground antialiased">
        <Providers>
          <div className="relative h-screen flex w-full">
            <aside className="border-r-small border-divider transition-width  relative flex justify-between h-full w-72 flex-col p-6">
              <header className="flex flex-col items-center justify-center gap-4">
                <h2 className='text-2xl font-bold'>Caduceus</h2>
                {/* <User avatarProps={{
                  src: "https://images.unsplash.com/photo-1508214751196-bcfd4ca60f91?ixlib=rb-4.0.3&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D&auto=format&fit=crop&w=1470&q=80",
                }}
                  description="Project Manager"
                  name="Jane Doe"
                /> */}
              </header>

              <ScrollShadow className="flex-1 flex items-center">
                <Listbox classNames={{ base: "w-full" }}>
                  <ListboxSection title="Overview">
                    <ListboxItem
                      key="home"
                      description="Home"
                      classNames={{ description: "text-md" }}
                      startContent={<IconHome />}
                    />
                    <ListboxItem
                      key="projects"
                      description="Projects"
                      classNames={{ description: "text-md" }}
                      startContent={<IconPackage />}
                    />
                    <ListboxItem
                      key="tasks"
                      description="Tasks"
                      classNames={{ description: "text-md" }}
                      startContent={<IconSubtask />}
                    />
                    <ListboxItem
                      key="teams"
                      description="Teams"
                      classNames={{ description: "text-md" }}
                      startContent={<IconUser />}
                    />
                    <ListboxItem
                      key="tracker"
                      description="Tracker"
                      classNames={{ description: "text-md" }}
                      startContent={<IconListTree />}
                    />
                  </ListboxSection>
                  <ListboxSection title="Organization">
                    <ListboxItem
                      key="members"
                      description="Members"
                      classNames={{ description: "text-md" }}
                      startContent={<IconUser />}
                    />
                    <ListboxItem
                      key="settings"
                      description="Settings"
                      classNames={{ description: "text-md" }}
                      startContent={<IconSettings />}
                    />
                  </ListboxSection>
                </Listbox>
              </ScrollShadow>

              <Listbox className='mt-auto'>
                <ListboxSection>
                  <ListboxItem
                    key="collapse"
                    description={isCollapsed ? "Expand" : "Collapse"}
                    classNames={{ description: "text-md" }}
                    startContent={<IconLabel />}
                  />
                  <ListboxItem
                    key="logout"
                    description="Log Out"
                    classNames={{ description: "text-md" }}
                    startContent={<IconLogout />}
                  />
                </ListboxSection>
              </Listbox>
            </aside>

            <section className="w-full flex flex-col p-4 h-full">
              <header className="rounded-medium border-small border-divider flex items-center gap-3 p-4 flex-shrink-0">
                <Button variant="ghost" size="sm" isIconOnly startContent={<IconLabel />} onPress={() => { }} />
                <h2 className="text-medium text-default-700 font-medium">Overview</h2>
              </header>
              <main className="flex-1 mt-4 overflow-auto">
                <div className="rounded-medium border-small border-divider flex h-full w-full flex-col gap-4 p-4">
                  {children}
                </div>
              </main>
            </section>
          </div>
        </Providers>
      </body>
    </html >
  )
}
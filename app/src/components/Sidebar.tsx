"use client";

import {
  Listbox,
  ListboxItem,
  ListboxSection
} from "@heroui/listbox";
import { ScrollShadow } from "@heroui/scroll-shadow";
import { IconHome, IconListTree, IconLogout, IconPackage, IconSettings, IconSubtask, IconUser } from "@tabler/icons-react";

export default function Sidebar() {
  return <aside className="border-r-small border-divider transition-width  relative flex justify-between h-full w-72 flex-col p-6">
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
            startContent={<IconHome />} />
          <ListboxItem
            key="projects"
            description="Projects"
            classNames={{ description: "text-md" }}
            startContent={<IconPackage />} />
          <ListboxItem
            key="tasks"
            description="Tasks"
            classNames={{ description: "text-md" }}
            startContent={<IconSubtask />} />
          <ListboxItem
            key="teams"
            description="Teams"
            classNames={{ description: "text-md" }}
            startContent={<IconUser />} />
          <ListboxItem
            key="tracker"
            description="Tracker"
            classNames={{ description: "text-md" }}
            startContent={<IconListTree />} />
        </ListboxSection>
        <ListboxSection title="Organization">
          <ListboxItem
            key="members"
            description="Members"
            classNames={{ description: "text-md" }}
            startContent={<IconUser />} />
          <ListboxItem
            key="settings"
            description="Settings"
            classNames={{ description: "text-md" }}
            startContent={<IconSettings />} />
        </ListboxSection>
      </Listbox>
    </ScrollShadow>

    <Listbox className='mt-auto'>
      <ListboxSection>
        <ListboxItem
          key="logout"
          description="Log Out"
          classNames={{ description: "text-md" }}
          startContent={<IconLogout />} />
      </ListboxSection>
    </Listbox>
  </aside>;
}

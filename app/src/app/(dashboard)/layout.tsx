"use client";

import {
  Listbox,
  ListboxItem,
  ListboxSection
} from "@heroui/listbox";
import { ScrollShadow } from "@heroui/scroll-shadow";
import { IconHome, IconListTree, IconPackage, IconSettings, IconSubtask, IconUser } from "@tabler/icons-react";

export default function Layout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="relative flex h-[100% - var(--navbar-height)] w-full items-start overflow-x-auto overflow-y-auto transition-colors duration-200 justify-center dark">
      <div className="flex h-full min-h-192 w-full">
        <div className="border-r-small! border-divider transition-width relative flex h-full w-72 flex-col p-6">
          <div className="flex items-center gap-3 px-3">
            <div className="bg-foreground flex h-8 w-8 items-center justify-center rounded-full">
              <svg fill="none" height="32" viewBox="0 0 32 32" width="32" className="text-background">
                <path d="M17.6482 10.1305L15.8785 7.02583L7.02979 22.5499H10.5278L17.6482 10.1305ZM19.8798 14.0457L18.11 17.1983L19.394 19.4511H16.8453L15.1056 22.5499H24.7272L19.8798 14.0457Z" fill="currentColor">
                </path>
              </svg>
            </div>
            <span className="text-small font-bold uppercase opacity-100">Acme</span>
          </div>
          {/* <span aria-hidden="true" className="w-px h-px block" style="margin-left: 0.25rem; margin-top: 2rem;">
          </span> */}
          <div className="flex items-center gap-3 px-3">
            <span className="flex relative justify-center items-center box-border overflow-hidden align-middle z-0 outline-solid outline-transparent data-[focus-visible=true]:z-10 data-[focus-visible=true]:outline-2 data-[focus-visible=true]:outline-focus data-[focus-visible=true]:outline-offset-2 w-8 h-8 text-tiny bg-default text-default-foreground rounded-full ring-2 ring-offset-2 ring-offset-background dark:ring-offset-background-dark ring-default flex-none">
              <img className="flex object-cover w-full h-full transition-opacity !duration-500 opacity-0 data-[loaded=true]:opacity-100" alt="avatar" src="https://i.pravatar.cc/150?u=a04258114e29026708c" data-loaded="true" />
            </span>
            <div className="flex max-w-full flex-col">
              <p className="text-small text-default-600 truncate font-medium">John Doe</p>
              <p className="text-tiny text-default-400 truncate">Product Designer</p>
            </div>
          </div>
          <ScrollShadow >
            <div data-slot="base" className="w-full relative flex flex-col gap-1 p-1 overflow-clip list-none">
              <nav data-slot="list" className="w-full flex flex-col gap-0.5 outline-solid outline-transparent items-center" role="listbox">
                <Listbox>
                  <ListboxSection title="Overview">
                    <ListboxItem
                      key="home"
                      description="Home"
                      startContent={<IconHome />}
                    />
                    <ListboxItem
                      key="projects"
                      description="Projects"
                      startContent={<IconPackage />}
                    />
                    <ListboxItem
                      key="tasks"
                      description="Tasks"
                      startContent={<IconSubtask />}
                    />
                    <ListboxItem
                      key="teams"
                      description="Teams"
                      startContent={<IconUser />}
                    />
                    <ListboxItem
                      key="tracker"
                      description="Tracker"
                      startContent={<IconListTree />}
                    />
                  </ListboxSection>
                  <ListboxSection title="Organization">
                    <ListboxItem
                      key="members"
                      description="Members"
                      startContent={<IconUser />}
                    />
                    <ListboxItem
                      key="settings"
                      description="Settings"
                      startContent={<IconSettings />}
                    />
                  </ListboxSection>
                </Listbox>
                <Listbox>
                  <ListboxSection>
                    <ListboxItem></ListboxItem>
                    <ListboxItem></ListboxItem>
                    <ListboxItem></ListboxItem>
                  </ListboxSection>
                </Listbox>
              </nav>
            </div>
          </ScrollShadow>

          {/* <span aria-hidden="true" className="w-px h-px block" style="margin-left: 0.25rem; margin-top: 0.5rem;">
          </span> */}
          <div className="mt-auto flex flex-col">
            <button type="button" tabindex="0" data-react-aria-pressable="true" className="z-0 group relative inline-flex items-center box-border appearance-none select-none whitespace-nowrap font-normal subpixel-antialiased overflow-hidden tap-highlight-transparent transform-gpu data-[pressed=true]:scale-[0.97] cursor-pointer outline-solid outline-transparent data-[focus-visible=true]:z-10 data-[focus-visible=true]:outline-2 data-[focus-visible=true]:outline-focus data-[focus-visible=true]:outline-offset-2 px-4 min-w-20 h-10 text-small gap-2 rounded-medium w-full [&amp;&gt;svg]:max-w-[theme(spacing.8)] transition-transform-colors-opacity motion-reduce:transition-none bg-transparent data-[hover=true]:bg-default/40 text-default-500 data-[hover=true]:text-foreground justify-start truncate">
              <svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" role="img" className="text-default-500 flex-none iconify iconify--solar" focusable="false" width="24" height="24" viewBox="0 0 24 24">
                <g fill="none">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5" opacity=".5">
                  </circle>
                  <path stroke="currentColor" stroke-linecap="round" stroke-width="1.5" d="M12 17v-6">
                  </path>
                  <circle cx="1" cy="1" r="1" fill="currentColor" transform="matrix(1 0 0 -1 11 9)">
                  </circle>
                </g>
              </svg>Help &amp; Information</button>
            <button type="button" tabindex="0" data-react-aria-pressable="true" className="z-0 group relative inline-flex items-center box-border appearance-none select-none whitespace-nowrap font-normal subpixel-antialiased overflow-hidden tap-highlight-transparent transform-gpu data-[pressed=true]:scale-[0.97] cursor-pointer outline-solid outline-transparent data-[focus-visible=true]:z-10 data-[focus-visible=true]:outline-2 data-[focus-visible=true]:outline-focus data-[focus-visible=true]:outline-offset-2 px-4 min-w-20 h-10 text-small gap-2 rounded-medium [&amp;&gt;svg]:max-w-[theme(spacing.8)] transition-transform-colors-opacity motion-reduce:transition-none bg-transparent data-[hover=true]:bg-default/40 text-default-500 data-[hover=true]:text-foreground justify-start">
              <svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" role="img" className="text-default-500 flex-none rotate-180 iconify iconify--solar" focusable="false" width="24" height="24" viewBox="0 0 24 24">
                <g fill="none" stroke="currentColor" stroke-width="1.5">
                  <circle cx="12" cy="12" r="10" opacity=".5">
                  </circle>
                  <path stroke-linecap="round" d="M15 12H9">
                  </path>
                </g>
              </svg>Log Out</button>
          </div>
        </div>
        <div className="w-full flex-1 flex-col p-4">
          <header className="rounded-medium border-small border-divider flex items-center gap-3 p-4">
            <button type="button" tabindex="0" data-react-aria-pressable="true" className="z-0 group relative inline-flex items-center justify-center box-border appearance-none select-none whitespace-nowrap font-normal subpixel-antialiased overflow-hidden tap-highlight-transparent transform-gpu data-[pressed=true]:scale-[0.97] cursor-pointer outline-solid outline-transparent data-[focus-visible=true]:z-10 data-[focus-visible=true]:outline-2 data-[focus-visible=true]:outline-focus data-[focus-visible=true]:outline-offset-2 text-tiny gap-2 rounded-small px-0 !gap-0 transition-transform-colors-opacity motion-reduce:transition-none bg-transparent text-default-foreground data-[hover=true]:bg-default/40 min-w-8 w-8 h-8">
              <svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" role="img" className="text-default-500 iconify iconify--solar" width="24" height="24" viewBox="0 0 24 24">
                <path fill="currentColor" d="M9.944 2.25c-1.838 0-3.294 0-4.433.153c-1.172.158-2.121.49-2.87 1.238c-.748.749-1.08 1.698-1.238 2.87c-.153 1.14-.153 2.595-.153 4.433v2.112c0 1.838 0 3.294.153 4.433c.158 1.172.49 2.121 1.238 2.87c.749.748 1.698 1.08 2.87 1.238c1.14.153 2.595.153 4.433.153h5.022l.072-.001c1.384-.004 2.523-.027 3.451-.152c1.172-.158 2.121-.49 2.87-1.238c.748-.749 1.08-1.698 1.238-2.87c.153-1.14.153-2.595.153-4.433v-2.112c0-1.838 0-3.294-.153-4.433c-.158-1.172-.49-2.121-1.238-2.87c-.749-.748-1.698-1.08-2.87-1.238c-.928-.125-2.067-.148-3.45-.152h-.073l-.91-.001zm4.306 1.5H10c-1.907 0-3.261.002-4.29.14c-1.005.135-1.585.389-2.008.812S3.025 5.705 2.89 6.71c-.138 1.029-.14 2.383-.14 4.29v2c0 1.907.002 3.262.14 4.29c.135 1.005.389 1.585.812 2.008s1.003.677 2.009.812c1.028.138 2.382.14 4.289.14h4.25zm1.5 16.494c1.034-.01 1.858-.042 2.54-.134c1.005-.135 1.585-.389 2.008-.812s.677-1.003.812-2.009c.138-1.027.14-2.382.14-4.289v-2c0-1.907-.002-3.261-.14-4.29c-.135-1.005-.389-1.585-.812-2.008s-1.003-.677-2.009-.812c-.68-.092-1.505-.123-2.539-.134z" >
                </path>
              </svg>
            </button>
            <h2 className="text-medium text-default-700 font-medium">Overview</h2>

          </header>

          <main className="mt-4 h-full w-full overflow-visible">
            <div className="rounded-medium border-small border-divider flex h-[90%] w-full flex-col gap-4">
              {children}
            </div>
          </main>
        </div>
      </div>
    </div>
  )
}
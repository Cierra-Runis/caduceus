'use client';

import { useTranslations } from 'next-intl';
import { useRef } from 'react';
import { ImperativePanelHandle } from 'react-resizable-panels';
import useWebSocket from 'react-use-websocket';
import { toast } from 'sonner';

import { env } from '@/lib/env';
import { Project } from '@/lib/types/project';

import { Sidebar } from './Sidebar';

export function ClientPage({ project }: { project: Project }) {
  const t = useTranslations('Project');
  const sidebarPanelRef = useRef<ImperativePanelHandle>(null);
  // const editorPanelRef = useRef<ImperativePanelHandle>(null);
  // const previewPanelRef = useRef<ImperativePanelHandle>(null);

  // const { sendJsonMessage } =
  useWebSocket(`${env.NEXT_PUBLIC_WS_URL}/project/${project.id}`, {
    onClose: () =>
      toast.warning(t('ws.closed'), {
        duration: 2000,
      }),
    onError: () =>
      toast.error(t('ws.error'), {
        duration: 2000,
      }),
    onMessage: (message) =>
      toast(t('ws.event', { type: message.type }), {
        description: `${message.data}`,
        duration: 2000,
      }),
    onOpen: () =>
      toast.success(t('ws.open'), {
        duration: 2000,
      }),
    onReconnectStop: () =>
      toast.warning(t('ws.reconnectStopped'), {
        duration: 2000,
      }),
    reconnectAttempts: 10,
    reconnectInterval: 3000,
    retryOnError: true,
    shouldReconnect: () => true,
  });

  return (
    <div className='flex h-screen'>
      <Sidebar sidebarPanelRef={sidebarPanelRef} />
      {/* <section className='flex h-full flex-1 flex-col'>
        <Navbar
          className='h-11 bg-content1'
          classNames={{ wrapper: 'pl-0 pr-1.5' }}
          maxWidth='f1 h-1
        >
          <NavbarContent justify='center'>
            {t.rich('title', { name: project.name })}
          </NavbarContent>
          <NavbarContent className='gap-1' justify='end'>
            <ThemeButtons />
          </NavbarContent> */}
      {/* </Navbar>
        <PanelGroup direction='horizontal'>
          <SidebarPanel sidebarPanelRef={sidebarPanelRef} />
          <PanelResizeHandle className='flex w-4 items-center justify-center'>
            <IconGripVertical className='w-4' />
          </PanelResizeHandle>
          <EditorPanel
            editorPanelRef={editorPanelRef}
            sendMessage={sendJsonMessage}
          />
          <PanelResizeHandle className='flex w-4 items-center justify-center'>
            <IconGripVertical className='w-4' />
          </PanelResizeHandle>
          <PreviewPanel previewPanelRef={previewPanelRef} />
        </PanelGroup>
      </section> */}
    </div>
  );
}

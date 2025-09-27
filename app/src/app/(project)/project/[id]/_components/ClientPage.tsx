'use client';

import { Button } from '@heroui/button';
import { Navbar, NavbarContent } from '@heroui/navbar';
import { addToast } from '@heroui/toast';
import { IconGripVertical } from '@tabler/icons-react';
import { useRef } from 'react';
import {
  ImperativePanelHandle,
  PanelGroup,
  PanelResizeHandle,
} from 'react-resizable-panels';
import useWebSocket from 'react-use-websocket';

import { ThemeButtons } from '@/components/buttons/ThemeButton';
import { ProjectPayload } from '@/lib/api/project';

import { EditorPanel } from './EditorPanel';
import { PreviewPanel } from './PreviewPanel';
import { Sidebar } from './Sidebar';
import { SidebarPanel } from './SidebarPanel';

export function ClientPage({ project }: { project: ProjectPayload }) {
  const sidebarPanelRef = useRef<ImperativePanelHandle>(null);
  const editorPanelRef = useRef<ImperativePanelHandle>(null);
  const previewPanelRef = useRef<ImperativePanelHandle>(null);

  const { sendMessage } = useWebSocket('ws://localhost:8080/ws', {
    onClose: (e) =>
      addToast({
        color: 'warning',
        description: `${e.code}: ${e.reason}`,
        shouldShowTimeoutProgress: true,
        timeout: 2000,
        title: 'WebSocket connection closed',
      }),
    onError: (event) =>
      addToast({
        color: 'danger',
        description: `${event}`,
        shouldShowTimeoutProgress: true,
        timeout: 2000,
        title: `WebSocket error`,
      }),
    onMessage: (message) =>
      addToast({
        color: 'primary',
        description: `${message.data}`,
        shouldShowTimeoutProgress: true,
        timeout: 2000,
        title: `WebSocket ${message.type} event`,
      }),
    onOpen: () =>
      addToast({
        color: 'success',
        shouldShowTimeoutProgress: true,
        timeout: 2000,
        title: 'WebSocket connection established',
      }),
    onReconnectStop: () =>
      addToast({
        color: 'danger',
        shouldShowTimeoutProgress: true,
        timeout: 2000,
        title: 'WebSocket reconnection stopped',
      }),
    reconnectAttempts: 10,
    reconnectInterval: 3000,
    retryOnError: true,
    shouldReconnect: () => true,
  });

  return (
    <div className='flex h-screen'>
      <Sidebar sidebarPanelRef={sidebarPanelRef} />
      <section className='flex h-full flex-1 flex-col'>
        <Navbar
          className='bg-default-50 h-11'
          classNames={{ wrapper: 'pl-0 pr-1.5' }}
          maxWidth='full'
        >
          <NavbarContent justify='center'>
            Project: {project.name}
          </NavbarContent>
          <NavbarContent className='gap-1' justify='end'>
            <ThemeButtons />
            <Button onPress={() => sendMessage('🦶')} size='sm' />
          </NavbarContent>
        </Navbar>
        <PanelGroup direction='horizontal'>
          <SidebarPanel sidebarPanelRef={sidebarPanelRef} />
          <PanelResizeHandle className='flex w-4 items-center justify-center'>
            <IconGripVertical className='w-4' />
          </PanelResizeHandle>
          <EditorPanel editorPanelRef={editorPanelRef} />
          <PanelResizeHandle className='flex w-4 items-center justify-center'>
            <IconGripVertical className='w-4' />
          </PanelResizeHandle>
          <PreviewPanel previewPanelRef={previewPanelRef} />
        </PanelGroup>
      </section>
    </div>
  );
}

'use client';

import { RefObject } from 'react';
import { ImperativePanelHandle, Panel } from 'react-resizable-panels';

export interface SidebarPanelProps {
  sidebarPanelRef: RefObject<ImperativePanelHandle | null>;
}

export function SidebarPanel({ sidebarPanelRef }: SidebarPanelProps) {
  return (
    <Panel
      collapsible
      defaultSize={0}
      id='sidebar'
      minSize={10}
      order={0}
      ref={sidebarPanelRef}
    >
      Open by Sidebar
    </Panel>
  );
}

'use client';

import {
  CheckIcon,
  CircleCheckIcon,
  FileIcon,
  PencilLineIcon,
  RefreshCwIcon,
  TriangleAlertIcon,
  UsersIcon,
  WifiIcon,
  WifiOffIcon,
  XCircleIcon,
} from 'lucide-react';
import { useTranslations } from 'next-intl';
import { WebsocketProvider } from 'y-websocket';

import { useAwareness } from '@/hooks/useAwareness';
import {
  ConnectionState,
  useConnectionStatus,
} from '@/hooks/useConnectionStatus';
import { AutoSavePolicy } from '@/lib/types/project';
import { cn } from '@/lib/utils';

// A VS Code-style bottom status bar for the editor: ambient, glanceable state
// that doesn't belong in a panel. Sourced from the y-websocket provider,
// awareness, the dirty map, the project settings, and the tinymist diagnostics
// store (the problems segment).
export function StatusBar({
  autoSave,
  dirtyCount,
  entryName,
  errors,
  meId,
  provider,
  warnings,
}: {
  autoSave: AutoSavePolicy;
  dirtyCount: number;
  entryName: null | string;
  /// Total tinymist error diagnostics across the project.
  errors: number;
  meId: null | string;
  provider: null | WebsocketProvider;
  /// Total tinymist warning diagnostics across the project.
  warnings: number;
}) {
  const t = useTranslations('Editor.status');
  const tAutoSave = useTranslations('Editor.autoSave');
  const { state, synced } = useConnectionStatus(provider);
  const others = useAwareness(provider, meId);
  const connection = connectionView(state, synced);
  const saved = dirtyCount === 0;
  const clean = errors === 0 && warnings === 0;

  return (
    <footer className='flex h-6 shrink-0 items-center gap-4 border-t bg-muted/40 px-3 text-xs text-muted-foreground select-none'>
      <span className={cn('flex items-center gap-1', connection.className)}>
        <connection.Icon
          className={cn('size-3.5', connection.spin && 'animate-spin')}
        />
        {t(connection.key)}
      </span>

      <span className='flex items-center gap-1'>
        <UsersIcon className='size-3.5' />
        {t('collaborators', { count: others.length })}
      </span>

      <span className='flex items-center gap-1'>
        {saved ? (
          <CheckIcon className='size-3.5' />
        ) : (
          <PencilLineIcon className='size-3.5 text-amber-500' />
        )}
        {saved ? t('saved') : t('unsaved', { count: dirtyCount })}
        <span className='opacity-70'>· {tAutoSave(autoSave)}</span>
      </span>

      {clean ? (
        <span className='flex items-center gap-1 text-emerald-600 dark:text-emerald-400'>
          <CircleCheckIcon className='size-3.5' />
          {t('noProblems')}
        </span>
      ) : (
        <span className='flex items-center gap-3'>
          <span className='flex items-center gap-1 text-destructive'>
            <XCircleIcon className='size-3.5' />
            {errors}
          </span>
          <span className='flex items-center gap-1 text-amber-500'>
            <TriangleAlertIcon className='size-3.5' />
            {warnings}
          </span>
        </span>
      )}

      <span className='ml-auto flex items-center gap-1'>
        <FileIcon className='size-3.5' />
        {entryName ?? '—'}
        <span className='opacity-70'>· {t('typst')}</span>
      </span>
    </footer>
  );
}

// Map the raw (transport, synced) pair to a single user-facing connection
// state: offline (socket down) → reconnecting (socket flapping) → syncing
// (connected but the first document sync hasn't landed) → live.
function connectionView(state: ConnectionState, synced: boolean) {
  if (state === 'disconnected') {
    return {
      className: 'text-destructive',
      Icon: WifiOffIcon,
      key: 'offline',
      spin: false,
    } as const;
  }
  if (state === 'connecting') {
    return { className: '', Icon: RefreshCwIcon, key: 'reconnecting', spin: true } as const;
  }
  if (!synced) {
    return { className: '', Icon: RefreshCwIcon, key: 'syncing', spin: true } as const;
  }
  return {
    className: 'text-emerald-600 dark:text-emerald-400',
    Icon: WifiIcon,
    key: 'live',
    spin: false,
  } as const;
}

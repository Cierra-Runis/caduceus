'use client';

import { useTranslations } from 'next-intl';
import { WebsocketProvider } from 'y-websocket';

import {
    Avatar,
    AvatarFallback,
    AvatarGroup,
    AvatarGroupCount,
    AvatarImage,
} from '@/components/ui/avatar';
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from '@/components/ui/tooltip';
import { useAwareness } from '@/hooks/useAwareness';
import { PresenceUser } from '@/lib/yjs/presence';

const MAX_VISIBLE = 4;

export interface PresenceBarProps {
  me: null | PresenceUser;
  provider: null | WebsocketProvider;
}

// Who else is in this project's room right now, and (via remote cursor
// styling in Editor's ClientPage, see lib/yjs/presence.ts) where their
// cursors are.
export function PresenceBar({ me, provider }: PresenceBarProps) {
  const t = useTranslations('Project');
  const remote = useAwareness(provider, me?.id ?? null);
  const users = me ? [me, ...remote] : remote;
  if (users.length === 0) return null;

  const visible = users.slice(0, MAX_VISIBLE);
  const overflow = users.length - visible.length;

  return (
    <div
      className={`
        flex items-center gap-2 rounded-full border bg-card/80 px-2 py-1
        shadow-sm backdrop-blur-sm
      `}
    >
      <AvatarGroup>
        {visible.map((user) => (
          <Tooltip key={user.id}>
            <TooltipTrigger asChild>
              <Avatar size='sm' style={{ boxShadow: `0 0 0 2px ${user.color}` }}>
                <AvatarImage src={user.avatarUri ?? '/icon.svg'} />
                <AvatarFallback>
                  {user.name.slice(0, 1).toUpperCase()}
                </AvatarFallback>
              </Avatar>
            </TooltipTrigger>
            <TooltipContent>{user.name}</TooltipContent>
          </Tooltip>
        ))}
        {overflow > 0 && <AvatarGroupCount>+{overflow}</AvatarGroupCount>}
      </AvatarGroup>
      <span className='pr-1 text-xs text-muted-foreground'>
        {t('presence.online', { count: users.length })}
      </span>
    </div>
  );
}

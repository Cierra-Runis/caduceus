import { useEffect, useState } from 'react';
import { WebsocketProvider } from 'y-websocket';

import { PresenceUser } from '@/lib/yjs/presence';

// Live list of other participants in the current provider's room, sourced
// from Yjs awareness. Deduped by `user.id` so a reconnecting tab doesn't
// double-count; `meId` excludes every tab of the *local* account (not just
// the current clientID), since the same person can have several tabs open at
// once and each is its own awareness client. Entries without a `user` field
// (not yet published, see ClientPage.tsx) are skipped.
export function useAwareness(
  provider: null | WebsocketProvider,
  meId: null | string,
): PresenceUser[] {
  const [users, setUsers] = useState<PresenceUser[]>([]);

  useEffect(() => {
    if (!provider) {
      setUsers([]);
      return;
    }

    const { awareness } = provider;
    const sync = () => {
      const localId = awareness.clientID;
      const byId = new Map<string, PresenceUser>();
      awareness.getStates().forEach((state, clientId) => {
        if (clientId === localId) return;
        const user = state.user as PresenceUser | undefined;
        if (!user || user.id === meId) return;
        byId.set(user.id, user);
      });
      setUsers([...byId.values()]);
    };

    sync();
    awareness.on('change', sync);
    return () => awareness.off('change', sync);
  }, [provider, meId]);

  return users;
}

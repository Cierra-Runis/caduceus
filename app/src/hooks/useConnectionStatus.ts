import { useEffect, useState } from 'react';
import { WebsocketProvider } from 'y-websocket';

export type ConnectionState = 'connected' | 'connecting' | 'disconnected';

export interface ConnectionStatus {
  // The websocket transport state, as reported by the provider's `status`
  // event ('connecting' between drops and reconnects, 'connected' once the
  // socket is open, 'disconnected' when it closes).
  state: ConnectionState;
  // Whether the initial full-document sync has completed (the provider's
  // `sync` event). A connected-but-unsynced room is still catching up.
  synced: boolean;
}

// Live connection + initial-sync state of a y-websocket provider, for ambient
// status display (see StatusBar). These signals are emitted by the provider
// already but nothing else renders them. A missing provider reads as
// disconnected + unsynced.
export function useConnectionStatus(
  provider: null | WebsocketProvider,
): ConnectionStatus {
  const [status, setStatus] = useState<ConnectionStatus>({
    state: 'connecting',
    synced: false,
  });

  useEffect(() => {
    if (!provider) {
      setStatus({ state: 'disconnected', synced: false });
      return;
    }

    // Seed from the provider's current state so a late mount doesn't sit on a
    // stale default until the next event, then track changes.
    setStatus({
      state: provider.wsconnected ? 'connected' : 'connecting',
      synced: provider.synced,
    });

    const onStatus = ({ status: state }: { status: ConnectionState }) =>
      setStatus((prev) => ({ ...prev, state }));
    const onSync = (synced: boolean) =>
      setStatus((prev) => ({ ...prev, synced }));

    provider.on('status', onStatus);
    provider.on('sync', onSync);
    return () => {
      provider.off('status', onStatus);
      provider.off('sync', onSync);
    };
  }, [provider]);

  return status;
}

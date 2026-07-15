import { Awareness } from 'y-protocols/awareness';

// A remote participant's identity, published as the `user` field of their
// awareness state. Attribution only; document access is enforced server-side.
export interface PresenceUser {
  avatarUri: null | string;
  color: string;
  id: string;
  name: string;
}

const PRESENCE_COLOR_COUNT = 5;

// Deterministically maps a user id to one of the app's existing categorical
// chart colors (`--chart-1`..`--chart-5`, defined in globals.css), so remote
// cursors/avatars stay on-palette and theme-aware without inventing new
// colors. Same user always gets the same color across sessions/tabs.
export function presenceColor(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) | 0;
  }
  const index = (Math.abs(hash) % PRESENCE_COLOR_COUNT) + 1;
  return `var(--chart-${index})`;
}

const STYLE_ELEMENT_ID = 'y-remote-cursor-styles';

// Keeps a single injected <style> tag in sync with the awareness map, so
// y-monaco's automatically-rendered `yRemoteSelection-<clientID>` /
// `yRemoteSelectionHead-<clientID>` decorations get a color and name label
// per remote client. y-monaco only emits the class names; it has no styling
// or name-tag support of its own, so the per-client CSS (including the
// literal name, baked into the generated rule text) is generated here. This
// mirrors the approach used by the official Yjs Monaco demo. Base layout
// rules (position, caret sizing) live in globals.css; only color/content are
// dynamic. Returns a cleanup function that removes the tag and unsubscribes.
export function syncRemoteCursorStyles(awareness: Awareness): () => void {
  const style = document.createElement('style');
  style.id = STYLE_ELEMENT_ID;
  document.head.appendChild(style);

  const render = () => {
    const localId = awareness.clientID;
    let css = '';
    awareness.getStates().forEach((state, clientId) => {
      if (clientId === localId) return;
      const user = state.user as PresenceUser | undefined;
      if (!user) return;
      const name = user.name.replace(/["\\]/g, '');
      css += `
        .yRemoteSelection-${clientId} { background-color: color-mix(in oklch, ${user.color} 35%, transparent); }
        .yRemoteSelectionHead-${clientId} { border-left-color: ${user.color}; border-top-color: ${user.color}; }
        .yRemoteSelectionHead-${clientId}::after { border-color: ${user.color}; }
        .yRemoteSelectionHead-${clientId}::before { content: '${name}'; background-color: ${user.color}; }
      `;
    });
    style.textContent = css;
  };

  render();
  awareness.on('change', render);
  return () => {
    awareness.off('change', render);
    style.remove();
  };
}

'use client';

import { useTranslations } from 'next-intl';
import { RefObject } from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';

import { AutoSavePolicy } from '@/lib/types/project';

/// The auto-save policies, in the order shown in the selector.
const AUTO_SAVE_POLICIES: AutoSavePolicy[] = [
  'off',
  'afterDelay',
  'onFocusChange',
  'onWindowChange',
];

export interface SettingsPanelProps {
  /// Current auto-save policy.
  autoSave: AutoSavePolicy;
  /// Change the auto-save policy (persisted project-wide).
  onAutoSaveChange: (policy: AutoSavePolicy) => void;
  sidebarPanelRef: RefObject<null | PanelImperativeHandle>;
}

/// The project's settings panel, a sibling of the file tree in the sidebar slot
/// (toggled from the activity bar). Currently holds the shared auto-save policy;
/// more project-level settings land here over time.
export function SettingsPanel({
  autoSave,
  onAutoSaveChange,
  sidebarPanelRef,
}: SettingsPanelProps) {
  const t = useTranslations('Editor');
  return (
    <Panel
      collapsible
      defaultSize={0}
      id='sidebar'
      minSize={10}
      panelRef={sidebarPanelRef}
    >
      <div className='flex items-center px-3 py-2'>
        <span className='text-xs font-medium opacity-60'>{t('settings')}</span>
      </div>
      <div className='flex flex-col gap-1 px-3 py-1 text-sm'>
        <label className='flex flex-col gap-1'>
          <span className='text-xs opacity-70'>{t('autoSave.label')}</span>
          <select
            className='rounded-sm border bg-background p-1'
            onChange={(event) =>
              onAutoSaveChange(event.target.value as AutoSavePolicy)
            }
            value={autoSave}
          >
            {AUTO_SAVE_POLICIES.map((policy) => (
              <option key={policy} value={policy}>
                {t(`autoSave.${policy}`)}
              </option>
            ))}
          </select>
        </label>
      </div>
    </Panel>
  );
}

'use client';

import { SettingItem } from '@/hooks/useKioskNavigation';

interface KioskSettingsMenuProps {
  selectedSetting: SettingItem;
  screenTimeoutEnabled: boolean;
  isLoading: boolean;
}

const settingConfig: Record<SettingItem, { icon: string; label: string }> = {
  screen_timeout: {
    icon: '☾',
    label: 'SCREEN TIMEOUT',
  },
  back: {
    icon: '←',
    label: 'BACK',
  },
};

const settings: SettingItem[] = ['screen_timeout', 'back'];

export function KioskSettingsMenu({
  selectedSetting,
  screenTimeoutEnabled,
  isLoading,
}: KioskSettingsMenuProps) {
  return (
    <div className="kiosk-settings">
      {/* Header */}
      <div className="kiosk-settings-header">SETTINGS</div>

      {/* Divider */}
      <div className="kiosk-divider" />

      {/* Settings options */}
      <div className="kiosk-settings-list">
        {settings.map((setting) => {
          const config = settingConfig[setting];
          const isSelected = setting === selectedSetting;
          const isDisabled = isLoading && setting !== selectedSetting;

          return (
            <div
              key={setting}
              className={`kiosk-setting-item ${
                isSelected ? 'kiosk-setting-selected' : ''
              } ${isDisabled ? 'kiosk-setting-disabled' : ''}`}
            >
              <span className="kiosk-setting-indicator">
                {isSelected ? '▶' : ' '}
              </span>
              <span className="kiosk-setting-icon">{config.icon}</span>
              <span className="kiosk-setting-label">{config.label}</span>
              {setting === 'screen_timeout' && (
                <span
                  className={`kiosk-setting-toggle ${
                    screenTimeoutEnabled ? 'kiosk-toggle-on' : 'kiosk-toggle-off'
                  }`}
                >
                  {screenTimeoutEnabled ? 'ON' : 'OFF'}
                </span>
              )}
            </div>
          );
        })}
      </div>

      {/* Hint */}
      <div className="kiosk-hint">
        <span className="kiosk-hint-text">↕ select • press</span>
      </div>
    </div>
  );
}

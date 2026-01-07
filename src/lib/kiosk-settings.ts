/**
 * Kiosk Settings API Helper
 *
 * Communicates with the local encoder.py HTTP server running on the Pi
 * to get/set kiosk settings like screen timeout.
 */

const LOCAL_SETTINGS_URL = 'http://localhost:8765/settings';

export interface KioskSettings {
  screenTimeoutEnabled: boolean;
}

/**
 * Fetch current settings from the local encoder.py server.
 * Returns defaults if the server is unreachable.
 */
export async function getLocalSettings(): Promise<KioskSettings> {
  try {
    const res = await fetch(LOCAL_SETTINGS_URL, {
      method: 'GET',
      headers: { 'Content-Type': 'application/json' },
    });
    if (!res.ok) throw new Error('Failed to fetch');
    return res.json();
  } catch {
    // Default if encoder.py not reachable (e.g., running in browser on desktop)
    return { screenTimeoutEnabled: true };
  }
}

/**
 * Update settings on the local encoder.py server.
 * Returns true if successful, false otherwise.
 */
export async function updateLocalSettings(
  settings: Partial<KioskSettings>
): Promise<boolean> {
  try {
    const res = await fetch(LOCAL_SETTINGS_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(settings),
    });
    return res.ok;
  } catch {
    // Server unreachable
    return false;
  }
}

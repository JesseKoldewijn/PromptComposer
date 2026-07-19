import { invoke } from '@tauri-apps/api/core';
import { ask } from '@tauri-apps/plugin-dialog';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

async function shouldSkipUpdateCheck(): Promise<boolean> {
  if (import.meta.env.DEV) {
    return true;
  }
  try {
    if (await invoke<boolean>('is_e2e_session')) {
      return true;
    }
  } catch {
    // Command unavailable (tests / non-Tauri) — continue.
  }
  return false;
}

/**
 * Best-effort startup update check. Never throws; never blocks app boot.
 * Skips in `tauri dev` and under the GUI e2e harness.
 */
export async function checkForAppUpdate(): Promise<void> {
  if (await shouldSkipUpdateCheck()) {
    return;
  }

  try {
    const update = await check();
    if (!update) {
      return;
    }

    const version = update.version;
    const ok = await ask(
      `Update to v${version} is available. Install and restart now?`,
      {
        title: 'Prompt Composer update',
        kind: 'info',
        okLabel: 'Install & restart',
        cancelLabel: 'Later',
      },
    );
    if (!ok) {
      return;
    }

    await update.downloadAndInstall();
    await relaunch();
  } catch {
    // Offline, missing latest.json, unsigned local build, etc.
  }
}

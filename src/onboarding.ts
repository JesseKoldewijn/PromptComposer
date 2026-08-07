import { invoke } from '@tauri-apps/api/core';
import { load } from '@tauri-apps/plugin-store';

/** Same file as archive metadata (`src-tauri` `STORE_FILENAME`). */
const STORE_FILENAME = 'settings.json';
const GETTING_STARTED_SEEN_KEY = 'gettingStartedSeen';

async function openSettingsStore() {
  return load(STORE_FILENAME, { autoSave: true });
}

export async function hasSeenGettingStarted(): Promise<boolean> {
  try {
    const store = await openSettingsStore();
    return (await store.get<boolean>(GETTING_STARTED_SEEN_KEY)) === true;
  } catch {
    return false;
  }
}

export async function markGettingStartedSeen(): Promise<void> {
  try {
    const store = await openSettingsStore();
    await store.set(GETTING_STARTED_SEEN_KEY, true);
    await store.save();
  } catch {
    // Offline / store unavailable — treat dismiss as best-effort.
  }
}

/** First open for real users; skipped under the GUI e2e harness. */
export async function shouldAutoShowGettingStarted(): Promise<boolean> {
  try {
    if (await invoke<boolean>('is_e2e_session')) {
      return false;
    }
  } catch {
    // Non-Tauri / tests — still respect the seen flag below.
  }
  return !(await hasSeenGettingStarted());
}

// Thin wrapper around @tauri-apps/plugin-updater with a friendly result type.

import { check, type Update } from "@tauri-apps/plugin-updater";

export type UpdateCheckResult =
  | { kind: "up_to_date"; current: string }
  | {
      kind: "available";
      current: string;
      next: string;
      notes: string | undefined;
      handle: Update;
    }
  | { kind: "error"; message: string };

export async function checkForUpdate(currentVersion: string): Promise<UpdateCheckResult> {
  try {
    const update = await check();
    if (!update || !update.available) {
      return { kind: "up_to_date", current: currentVersion };
    }
    return {
      kind: "available",
      current: currentVersion,
      next: update.version,
      notes: update.body ?? undefined,
      handle: update,
    };
  } catch (e) {
    return { kind: "error", message: String(e) };
  }
}

export async function installAndRestart(update: Update): Promise<void> {
  await update.downloadAndInstall();
  // The plugin will trigger a restart on most platforms once install
  // completes; on Linux AppImage it relies on tauri-plugin-process.
  // Phase 7-bis can wire `relaunch()` if needed.
}

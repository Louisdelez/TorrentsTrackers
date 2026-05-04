// Native desktop notifications via tauri-plugin-notification.

import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

let permissionState: "unknown" | "granted" | "denied" = "unknown";

async function ensurePermission(): Promise<boolean> {
  if (permissionState === "granted") return true;
  if (permissionState === "denied") return false;
  try {
    if (await isPermissionGranted()) {
      permissionState = "granted";
      return true;
    }
    const result = await requestPermission();
    permissionState = result === "granted" ? "granted" : "denied";
    return permissionState === "granted";
  } catch (e) {
    console.warn("notification permission check failed:", e);
    permissionState = "denied";
    return false;
  }
}

/// Fire a native notification iff the OS allows it AND the app window is
/// currently unfocused (so we don't spam when the user is reading the chat
/// already).
export async function notifyChatMessage(opts: {
  title: string;
  body: string;
}): Promise<void> {
  if (typeof document !== "undefined" && document.hasFocus()) return;
  if (!(await ensurePermission())) return;
  try {
    sendNotification({ title: opts.title, body: opts.body });
  } catch (e) {
    console.warn("sendNotification failed:", e);
  }
}

export async function notifyGeneric(title: string, body: string): Promise<void> {
  if (!(await ensurePermission())) return;
  try {
    sendNotification({ title, body });
  } catch (e) {
    console.warn("sendNotification failed:", e);
  }
}

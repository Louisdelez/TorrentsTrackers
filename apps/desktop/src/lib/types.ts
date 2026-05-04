// Mirror of the Rust IPC DTOs (see apps/desktop/src-tauri/src/ipc/dto.rs).
// Keep in sync; we don't run codegen yet (planned for Phase 3-bis with
// tauri-specta).

export type Category =
  | "Films"
  | "Series"
  | "Games"
  | "Music"
  | "Books"
  | "Software"
  | "Other";

export type Quality = "P480" | "P720" | "P1080" | "P2160" | { Other: string };

export type Language =
  | "FR"
  | "VOSTFR"
  | "EN"
  | "Multi"
  | { Other: string };

export type SourceKind =
  | "LocalFolder"
  | "HttpUrl"
  | "GitRepo"
  | "GoogleDrive"
  | "Dropbox"
  | "OneDrive"
  | "Server"
  | "Nostr"
  | "Ipfs";

export interface SourceDto {
  id: string;
  kind: SourceKind;
  endpoint: string;
  display_name: string;
  description: string | null;
  last_sync: string | null; // ISO 8601
  last_status: string;
  trust_level: "Unverified" | "Trusted" | "Modos";
}

export interface PoolDto {
  id: string;
  name: string;
  description: string | null;
  member_ids: string[];
  created_at: string;
}

export interface IdentityDto {
  npub: string;
  pubkey_hex: string;
  display_name: string | null;
  created_at: string;
}

export interface StatsDto {
  data_dir: string;
  db_path: string;
  sources: number;
  pools: number;
  entries: number;
}

export interface SearchHitDto {
  id: string; // ContentId hex
  title: string;
  magnet: string | null;
  category: Category;
  tags: string[];
  quality: Quality | null;
  languages: Language[];
  size_bytes: number | null;
  seeders: number | null;
  leechers: number | null;
  added_at: string;
  contributor_pubkey_hex: string;
  provenance: string[]; // source ids
  description: string | null;
}

export interface SearchQueryDto {
  text: string | null;
  scope: "all" | { source: string } | { pool: string };
  categories: Category[] | null;
  qualities: Quality[] | null;
  languages: Language[] | null;
  size_min: number | null;
  size_max: number | null;
  seeders_min: number | null;
  limit: number | null;
}

export type DownloadStateDto =
  | "initializing"
  | "live"
  | "paused"
  | "finished"
  | "error";

export interface DownloadInfo {
  id: number;
  title: string;
  progress_bytes: number;
  total_bytes: number;
  down_bps: number;
  up_bps: number;
  state: DownloadStateDto;
  finished: boolean;
}

export interface ChatMessage {
  id: string;
  channel: string;
  author_pubkey: string;
  content: string;
  reply_to: string | null;
  sent_at: string;
  signature: string;
}

export interface ChatServerDto {
  server_id: string;
  server_name: string;
  url: string;
}

export type ChatEventDto =
  | {
      kind: "authenticated";
      server_id: string;
      server_name: string;
      url: string;
    }
  | { kind: "message"; server_id: string; message: ChatMessage }
  | {
      kind: "history";
      server_id: string;
      channel: string;
      messages: ChatMessage[];
    }
  | {
      kind: "error";
      server_id: string;
      code: string;
      message: string;
    }
  | { kind: "disconnected"; server_id: string; reason: string };

export const CATEGORIES: { id: Category; label: string }[] = [
  { id: "Films", label: "Films" },
  { id: "Series", label: "Séries" },
  { id: "Games", label: "Jeux" },
  { id: "Music", label: "Musique" },
  { id: "Books", label: "Livres" },
  { id: "Software", label: "Logiciels" },
  { id: "Other", label: "Autre" },
];

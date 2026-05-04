// Thin wrappers around Tauri's `invoke` for type-safe IPC.

import { invoke } from "@tauri-apps/api/core";
import type {
  IdentityDto,
  PoolDto,
  SearchHitDto,
  SearchQueryDto,
  SourceDto,
  StatsDto,
} from "./types";

export const ipc = {
  async listSources(): Promise<SourceDto[]> {
    return invoke("list_sources");
  },
  async addSource(
    kind: string,
    endpoint: string,
    name: string | null,
  ): Promise<SourceDto> {
    return invoke("add_source", { kind, endpoint, name });
  },
  async syncSource(id: string): Promise<number> {
    return invoke("sync_source", { id });
  },
  async syncAllSources(): Promise<number> {
    return invoke("sync_all_sources");
  },
  async removeSource(id: string): Promise<void> {
    return invoke("remove_source", { id });
  },
  async listPools(): Promise<PoolDto[]> {
    return invoke("list_pools");
  },
  async createPool(name: string, sourceIds: string[]): Promise<PoolDto> {
    return invoke("create_pool", { name, sourceIds });
  },
  async removePool(id: string): Promise<void> {
    return invoke("remove_pool", { id });
  },
  async search(query: SearchQueryDto): Promise<SearchHitDto[]> {
    return invoke("search", { query });
  },
  async openMagnet(magnet: string): Promise<void> {
    return invoke("open_magnet", { magnet });
  },
  async identityShow(): Promise<IdentityDto | null> {
    return invoke("identity_show");
  },
  async identityInit(name: string | null): Promise<IdentityDto> {
    return invoke("identity_init", { name });
  },
  async identityExport(path: string, passphrase: string): Promise<number> {
    return invoke("identity_export", { path, passphrase });
  },
  async identityImport(
    path: string,
    passphrase: string,
    force: boolean,
  ): Promise<IdentityDto> {
    return invoke("identity_import", { path, passphrase, force });
  },
  async identityForget(): Promise<void> {
    return invoke("identity_forget");
  },
  async publish(args: {
    magnet: string;
    targetSourceId: string;
    title: string;
    category: string;
    tags: string[];
    quality: unknown | null;
    languages: unknown[];
    sizeBytes: number | null;
  }): Promise<SearchHitDto> {
    return invoke("publish", args);
  },
  async stats(): Promise<StatsDto> {
    return invoke("stats");
  },
};

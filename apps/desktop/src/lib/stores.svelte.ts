// Reactive global state. Svelte 5 `$state` rune wrapped in plain objects so
// imports stay ergonomic.

import type {
  Category,
  IdentityDto,
  PoolDto,
  SearchHitDto,
  SourceDto,
  StatsDto,
} from "./types";

interface AppState {
  identity: IdentityDto | null;
  sources: SourceDto[];
  pools: PoolDto[];
  stats: StatsDto | null;

  // current selection (drives the browse view)
  selectedCategory: Category | null;
  selectedSourceId: string | null;
  selectedPoolId: string | null;

  searchText: string;
  results: SearchHitDto[];
  searching: boolean;
  selectedEntryId: string | null;

  view: "browse" | "settings";
  syncing: boolean;
  toast: string | null;
}

export const app = $state<AppState>({
  identity: null,
  sources: [],
  pools: [],
  stats: null,
  selectedCategory: null,
  selectedSourceId: null,
  selectedPoolId: null,
  searchText: "",
  results: [],
  searching: false,
  selectedEntryId: null,
  view: "browse",
  syncing: false,
  toast: null,
});

export function showToast(msg: string, ms = 3000): void {
  app.toast = msg;
  setTimeout(() => {
    if (app.toast === msg) app.toast = null;
  }, ms);
}

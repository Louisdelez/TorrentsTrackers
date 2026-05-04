import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  preprocess: vitePreprocess(),
  // Don't force runes globally — let Svelte detect per-file. Our own
  // components use runes (`$state`, `$props`, ...), but third-party libs
  // (e.g. lucide-svelte 0.474) still rely on legacy `$$props`, which is
  // illegal under `runes: true`.
};

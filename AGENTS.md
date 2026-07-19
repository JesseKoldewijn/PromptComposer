# Agent notes

## Frontend framework

This project uses **Octane** (https://octanejs.dev/), not React.

Before writing or editing UI code, read the remote Octane agent guide:

- https://octanejs.dev/llms.txt

Also useful:

- Quick start: https://octanejs.dev/docs/quick-start
- Build tools (Vite): https://octanejs.dev/docs/build-tools
- Differences from React: https://octanejs.dev/docs/differences-from-react

Author UI in `.tsrx` where practical. Use `createRoot` from `octane` and the `@octanejs/vite-plugin` Vite plugin. Prefer native `onInput` for text fields (not React-style synthetic `onChange`).

## Backend

Heavy search/lookup/composition runs in the **Tauri Rust** backend (`src-tauri`). The Octane UI only invokes commands and displays results.

No workbook is bundled. Users upload an `.xlsx`, which is persisted under the app data directory (`prompt_archive.xlsx`) with metadata in Tauri store (`settings.json`).

Automated tests use `fixtures/minimal_prompt_archive.xlsx` (regenerate with `npm run fixture:gen`). E2E sets `PROMPT_COMPOSER_E2E=1` to enable `import_archive_from_path`.

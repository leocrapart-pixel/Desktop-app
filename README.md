# deskapp

Angular 22 + Tauri 2 desktop app for Windows. The UI is an Angular SPA rendered in
WebView2; everything native (filesystem, windows, processes) goes through Rust via
`invoke()`. The Rust side ships as a single self-contained `.exe` — no Node, no
browser engine bundled.

## Prerequisites (Windows)

1. **Rust** — install via [rustup](https://rust-lang.org/tools/install), default `x86_64-pc-windows-msvc` toolchain.
2. **Microsoft C++ Build Tools** — the installer's "Desktop development with C++" workload. Required to link the binary.
3. **WebView2** — already present on Windows 10 (1803+) and Windows 11. Skip unless targeting older.
4. **VBSCRIPT** — needed for the `.msi` half of the build. On by default; if you hit `failed to run light.exe`, enable it under Settings → Apps → Optional features → More Windows features.

## Develop

```sh
npm install
npm run tauri dev
```

Angular's dev server runs on port 1420 (matches `devUrl` in `tauri.conf.json`), and the
window hot-reloads on save. Editing Rust triggers a Rust rebuild and app restart.

`npm run start` alone gives you the UI in a browser — the `invoke()` calls will fail there,
since there is no Rust process behind them.

> **If `ng serve` picks the wrong port:** Angular's dev server lets an ambient `PORT`
> environment variable *override* the configured port — a CLI `--port` flag does not win
> against it (`@angular/build/src/builders/dev-server/options.js`). On a machine with `PORT`
> exported (Docker, some CI, WSL profiles) it will try that port instead and `npm run tauri dev`
> will open a blank window. Unset `PORT` for the dev server. CI here does not set it.

## Build the Windows installers

```sh
npm run tauri build
```

Artifacts:

| File | Path |
|---|---|
| MSI installer | `src-tauri/target/release/bundle/msi/*.msi` |
| NSIS setup `.exe` | `src-tauri/target/release/bundle/nsis/*-setup.exe` |
| Bare executable | `src-tauri/target/release/deskapp.exe` |

Both installers are unsigned. Windows SmartScreen will warn on first run until you
code-sign — see [Tauri's Windows signing guide](https://tauri.app/distribute/sign/windows/).

## CI

`.github/workflows/windows.yml` builds on `windows-latest` (push to `main`, or manual
dispatch) and uploads both installers as the `windows-installers` artifact. This is the
intended path when you have no Windows machine locally.

The `.msi` **cannot** be built on Linux or macOS — WiX only runs on Windows. Cross-compiling
the NSIS `.exe` from Linux is possible but unsupported and untested; use CI instead.

When you want tagged GitHub Releases instead of raw artifacts, swap the last two steps for
`tauri-apps/tauri-action@v1`.

## Notes

- **CSP** is set in `tauri.conf.json` and locked to `'self'`, with `connect-src` allowing
  `ipc:` so `invoke()` works. `style-src` includes `'unsafe-inline'` because Angular injects
  component styles at runtime as `<style>` elements. Tauri appends its own nonces/hashes for
  bundled code, so no further loosening should be needed. If you add a remote font or API,
  widen the specific directive (`font-src`, `connect-src`) rather than `default-src`.
- **Adding a native command**: write a `#[tauri::command]` fn in `src-tauri/src/lib.rs` and add
  it to `generate_handler![...]`, then `invoke('name', { args })` from Angular. Command args
  are camelCase on the JS side and snake_case in Rust by default.
- **Plugins** (fs, dialog, shell, updater) need a Rust dependency *and* a permission entry in
  `src-tauri/capabilities/default.json`. The capability file is the security boundary — the
  frontend cannot reach a plugin API that isn't listed there.
- **IDE**: VS Code + Tauri, rust-analyzer, Angular Language Service.

## Verified state

Built and verified on Linux:

- `ng build` → 196.7 kB raw / 54.05 kB transfer initial bundle (Angular 22, zoneless, OnPush by default, no zone.js), 2.4 s.
- Dev server serves the app on `:1420`, matching `devUrl`. Confirmed with `PORT` unset.
- `src-tauri/tauri.conf.json` validates against the live `schema.tauri.app/config/2` schema, and `tauri info` reports the CSP and `frontendDist` correctly (Tauri CLI 2.12.1, `@tauri-apps/api` 2.12.1).

**Not verified here:** Rust compilation and installer bundling. There is no Rust toolchain in
that environment, and neither the `.msi` nor a Windows-targeted `.exe` can be produced off
Windows. First `npm run tauri build` on Windows is the real test.

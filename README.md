# Rickroll

A Windows desktop app that rickrolls whoever opens it. Launch it and it opens
[the video](https://www.youtube.com/watch?v=dQw4w9WgXcQ) in the default browser; the window
itself is a small "you have been rickrolled" screen with a button to go again.

Angular 22 frontend rendered in WebView2, Tauri 2 shell in Rust. The installer is a single
self-contained `.exe` — no Node, no bundled browser engine.

## How it works

- `src-tauri/src/lib.rs` opens the URL from the `setup` hook, so it fires as the app starts and
  before the window paints.
- The window button calls the same Rust command again via `invoke('rickroll')`.
- The open happens in **Rust**, not in JS. The opener plugin's `openUrl` JS command is gated by
  a scope ACL that would have to be configured to allow `youtube.com`; the Rust `Opener` methods
  are not scope-checked, so no capability widening is needed. Verified in the plugin source
  (`tauri-plugin-opener` 2.7.0, `lib.rs`: *"the `Opener` methods themselves do not perform any
  scope check"*).
- Nothing is downloaded or bundled: the video streams from YouTube.

**Autoplay caveat.** The URL carries `autoplay=1`, but browsers block *audible* autoplay without
a user gesture, and a programmatic `open` is not a gesture. Depending on the browser's autoplay
policy and your history with youtube.com, the tab may land paused with the video cued. The tab is
in front of the user either way, which is most of the joke. If you want it to actually play
unprompted, add `&mute=1` to `RICKROLL` — muted autoplay is always allowed, but then it is a
silent rickroll.

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

Angular's dev server runs on port 1420 (matches `devUrl` in `tauri.conf.json`), and the window
hot-reloads on save. Editing Rust triggers a Rust rebuild and app restart.

`npm run start` alone gives you the UI in a browser, where `invoke()` has no Rust process behind
it — clicking the button will fail gracefully into the on-screen error message rather than
crashing.

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

The binary keeps the Cargo package name (`deskapp`); `productName: "Rickroll"` names the
installers, install path, and Start-menu entry.

Both installers are unsigned. Windows SmartScreen will warn on first run until you code-sign —
see [Tauri's Windows signing guide](https://tauri.app/distribute/sign/windows/).

## CI

`.github/workflows/windows.yml` builds on `windows-latest` (push to `main`, or manual dispatch)
and uploads both installers as the `windows-installers` artifact. This is the intended path when
you have no Windows machine locally.

The `.msi` **cannot** be built on Linux or macOS — WiX only runs on Windows. Cross-compiling the
NSIS `.exe` from Linux is possible but unsupported and untested; use CI instead.

When you want tagged GitHub Releases instead of raw artifacts, swap the last two steps for
`tauri-apps/tauri-action@v1`.

## Notes

- **CSP** is set in `tauri.conf.json` and locked to `'self'`, with `connect-src` allowing `ipc:`
  so `invoke()` works. `style-src` includes `'unsafe-inline'` because Angular injects component
  styles at runtime as `<style>` elements. Tauri appends its own nonces/hashes for bundled code.
  If you add a remote font or API, widen the specific directive (`font-src`, `connect-src`)
  rather than `default-src`.
- **No router.** The app is one screen, so `@angular/router` was removed along with `app.routes.ts`
  and `provideRouter`. That cut the initial bundle roughly in half — see below. Add it back only
  when there is a second screen.
- **Adding a native command**: write a `#[tauri::command]` fn in `src-tauri/src/lib.rs`, add it to
  `generate_handler![...]`, then `invoke('name', { args })` from Angular. Command args are
  camelCase on the JS side and snake_case in Rust by default.
- **Plugins** (fs, dialog, updater) need a Rust dependency *and* a permission entry in
  `src-tauri/capabilities/default.json`. The capability file is the security boundary — the
  frontend cannot reach a plugin API that isn't listed there. App-defined commands like
  `rickroll` are not part of that ACL.
- **IDE**: VS Code + Tauri, rust-analyzer, Angular Language Service.

## Verified state

Built and verified on Linux:

- `ng build` → **105.05 kB raw / 32.18 kB transfer** initial bundle, 2.4 s. Angular 22, zoneless,
  OnPush by default, no zone.js. (196.7 kB / 54.05 kB before dropping the router and the template
  demo — the router alone was ~45% of the bundle.)
- Dev server serves the app on `:1420`, matching `devUrl`. Confirmed with `PORT` unset.
- `src-tauri/tauri.conf.json` validates against the live `schema.tauri.app/config/2` schema, and
  `tauri info` reports the CSP and `frontendDist` correctly (Tauri CLI 2.12.1, `@tauri-apps/api`
  2.12.1).
- The opener plugin's Rust API was read at the resolved version (2.7.0) to confirm it bypasses the
  scope ACL.

**Not verified here:** Rust compilation, the `rickroll` command actually firing, and installer
bundling. There is no Rust toolchain in that environment, and neither the `.msi` nor a
Windows-targeted `.exe` can be produced off Windows. First `npm run tauri build` on Windows is
the real test.

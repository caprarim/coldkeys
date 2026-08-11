# ColdKeys

See every keyboard shortcut that launches an app, and add your own.

## What it does

- Lists your app launch keybinds in one window
- Add, edit or delete a shortcut, with a recorder that captures the key combo you press
- Run any app from the list without using its keybind
- Imports the shortcuts you already have on Linux

## How shortcuts work per platform

**Linux (GNOME)**
ColdKeys writes into GNOME custom shortcuts. Your keybinds keep working when ColdKeys is closed, because GNOME owns them. This also works on Wayland.

**Windows**
There is no GNOME, so ColdKeys registers the hotkeys itself. It lives in the system tray and needs to be running for the keybinds to fire.

## Downloads

Builds are produced by GitHub Actions on every push to `main`, and attached to a release when a `v*` tag is pushed.

- Windows: NSIS installer, `.exe`
- Linux: `.deb` and AppImage

## Running from source

```
npm install
npm run tauri dev
```

## Building

```
npm install
npm run tauri build
```

## Config

Your shortcut list is stored as `binds.json` in the ColdKeys app config directory.

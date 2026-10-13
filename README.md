# Cove

Cove is a "Dynamic Island" style app for desktop. It puts a small pill at the top-center of your screen that expands into a panel when clicked. Similar to the Dynamic Island on those iPhones.

It's built with [Tauri](https://tauri.app/) (Rust backend) and [Svelte](https://svelte.dev/) (frontend), using TypeScript and Vite.

## What it does

- Shows a frameless, transparent, always-on-top window docked to the top-center of the screen.
- The window stays out of the taskbar and has no title bar, so it floats cleanly over the desktop.
- Clicking the island animates it open into a larger panel. Clicking again collapses it back.
- The Rust side resizes and re-centers the window on every toggle so the animation stays anchored to the top.

## How it works

The frontend (`src/App.svelte`) renders the island and handles the expand/collapse animation. When the size changes it calls the `resize_island` Tauri command.

The backend (`src-tauri/src/lib.rs`):

- On startup, creates the window and centers it at the top of the primary monitor (`center_top`).
- Exposes the `resize_island(width, height)` command, which resizes the window and re-centers it horizontally.

Window options (transparent, always on top, no decorations, skip taskbar) are configured in `src-tauri/tauri.conf.json`.

## Project structure

```
src/                  Svelte frontend
  App.svelte          The island UI and animation
  main.ts             App entry point
src-tauri/            Rust / Tauri backend
  src/lib.rs          Window setup and resize command
  src/main.rs         Desktop entry point
  tauri.conf.json     Window and bundle configuration
```

## Requirements

- [Bun](https://bun.sh/)
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri's Windows prerequisites (WebView2, MSVC build tools)

## Development

Install dependencies:

```sh
bun i
```

Run the app in dev:

```sh
bun tauri dev
```

Build a release:

```sh
bun tauri build
```

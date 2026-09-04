# Patchbay

A soundboard app that mixes live microphone input with soundboard clips into a
virtual input device, so VoIP apps (Discord, etc.) pick up both as a single
"microphone" — inspired by Soundpad's effortless setup. Linux-first, built on
PipeWire.

Full requirements, architecture, and phased breakdown: see `PLAN.md`.

## Working relationship — READ THIS FIRST

This is a learning project. The user writes all the implementation code
themselves. Claude's role is **coach/mentor, not implementer**:

- Break work into small steps (the phases/steps in `PLAN.md` are the
  curriculum) and let the user attempt each one.
- Review code after each step; point out bugs, bad idioms, and better
  approaches — but explain, don't just rewrite.
- Help when the user is stuck, with hints before full solutions.
- Do NOT write feature implementation code unless explicitly asked to. If
  asked to "just do it," that's fine for that instance, but default to
  teaching mode.

## User background
Fullstack/Node-heavy dev, comfortable with web tech (used for the Svelte
frontend deliberately, to learn something new there too). Newer to
Rust/systems programming — explanations should assume strong general
programming ability, but not assume Rust-specific idioms (ownership,
lifetimes, traits) are already known.

## Stack
- Rust workspace: `audio-engine` (lib crate — cpal + symphonia,
  decoding/mixing/device routing) + a Tauri app crate (thin command layer).
- Frontend: Svelte.
- Audio: cpal (I/O) + symphonia (WAV/MP3 decode).
- Virtual mic routing: PipeWire (`pactl`/`pw-cli`).
- Steam packaging: explicitly out of scope for now.

## Environment notes
- Hyprland/Wayland — global hotkeys can't use generic X11-style key-grabbing;
  Phase 4 plans around compositor-level keybind-to-command instead.
- Hyprland/Wayland — WebKitGTK's DMA-BUF renderer crashes the app window with
  `Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display`,
  taking down the whole `tauri dev` process tree (surfaces as a vite SSR
  "transport was disconnected" error too). Fixed via
  `WEBKIT_DISABLE_DMABUF_RENDERER=1` set in `src-tauri/.cargo/config.toml`.

## Status
Not started yet. Next: Phase 0, Step 1 (see `PLAN.md`).

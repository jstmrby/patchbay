# Patchbay — Plan

A soundboard app that plays audio clips and mixes them with your live microphone
into a virtual input device, so VoIP apps (Discord, etc.) pick up both your voice
and soundboard clips as a single "microphone" — inspired by Soundpad's effortless
setup, built Linux-first on PipeWire.

This is a learning project. The user writes all the code; Claude acts as
coach/reviewer, breaking work into small steps and reviewing after each one.

## Requirements

**Functional**
- Load/manage a set of audio clips (WAV + MP3), each mapped to a UI trigger.
- Play a clip on demand with low enough latency to feel responsive.
- Play multiple clips overlapping (mixing), not just one-at-a-time.
- Capture live microphone input and mix it together with soundboard clips into
  one combined output stream — "extending" the mic, not replacing it.
- Route that combined output to a virtual recording device, and set it as the
  OS default input device, so VoIP apps pick it up automatically with no manual
  device selection.
- Manage the sound list from the UI (add/remove, not just a static config file).
- One clear "Enable Soundboard Mic" toggle that handles setup/teardown of the
  virtual device and default-device switching behind the scenes.
- (Later phase) Trigger playback via global hotkeys, not just mouse clicks.

**Non-functional**
- Rust core audio logic isolated in its own library crate — decoupled from
  whatever GUI shell sits on top.
- Cross-platform *intent* (Linux primary/first, Windows/macOS as later backend
  work). The "effortless setup" UX is a Linux/PipeWire-native solution for now —
  Windows/macOS equivalents (no APO/driver work) are a separate, later research
  problem.
- Steam packaging explicitly out of scope for now.

## Architecture

- `audio-engine` (Rust lib crate): device enumeration, decoding (symphonia),
  capture + playback + mixing (cpal), virtual-device targeting and
  default-device switching. No knowledge of Tauri or UI.
- `src-tauri` (Tauri app crate): thin command layer calling into `audio-engine`,
  plus OS integration glue (spawning `pactl`, hotkey IPC later).
- Svelte frontend: UI only, talks to Rust exclusively via Tauri
  commands/events.

## Phase 0 — Setup
1. Install Rust, Node, Tauri CLI; confirm PipeWire is your running audio server
   (`pactl info`).
2. Scaffold a Cargo workspace with two members: `audio-engine` (lib) and the
   Tauri app.
3. Wire a trivial Tauri command that calls one dummy function in
   `audio-engine`, prove the round trip end-to-end before writing anything
   real.

## Phase 1 — Core audio engine (no GUI yet, test via `cargo test`/small bin)
1. cpal: enumerate output devices, print the default device's supported
   config.
2. Play a generated sine wave through a cpal output stream — proves the
   callback/streaming model without decoding complexity yet.
3. Use symphonia to decode a WAV file to raw PCM, print
   format/duration/sample count.
4. Feed those decoded samples into the cpal stream — play a real WAV file
   end-to-end.
5. Extend decoding to MP3 via symphonia — same code path, different codec.
6. Handle sample-rate/channel mismatches between file and output device
   (resample or fail loudly and clearly).
7. Support multiple concurrent "voices" (mixing) — track active playing
   sounds, sum per-sample each callback, drop finished voices.
8. Open a cpal *input* stream on the real microphone, and feed its live
   samples into the same mixer as one more continuous "voice" alongside
   sound-clip voices — this is the "extend the mic" step, not just "play
   sounds."

## Phase 2 — Tauri + Svelte shell
1. Define Tauri commands (`list_sounds`, `play_sound`, `stop_sound`) wrapping
   Phase 1's engine.
2. Svelte UI: render sound entries as buttons from a config list, clicking
   invokes the command.
3. Add a file-picker (Tauri dialog plugin) so sounds can be added from the UI
   instead of hand-editing config.
4. Push "now playing" state from Rust to Svelte via Tauri **events** (not
   command responses) — different communication pattern, worth learning
   deliberately.

## Phase 3 — Virtual microphone routing with effortless setup (Linux first)
1. Manually create a PipeWire virtual sink via `pactl`/`pw-cli` from the
   terminal and confirm in Discord (or similar) that it shows up as a
   selectable mic — de-risk the OS-level concept before coding it.
2. From Rust/cpal, enumerate devices and target that virtual sink as the
   output instead of your default speakers.
3. Automate creating/tearing down the virtual device from the app itself on
   startup/shutdown.
4. Route the Phase 1.8 combined mic+soundboard mix into this virtual sink,
   instead of (or in addition to) real speakers.
5. Programmatically set the virtual sink as the **default recording device**
   (`pactl set-default-source`), and revert it on disable/exit.
6. Build the single "Enable Soundboard Mic" toggle in the UI that runs the
   full sequence (create device → start mixing → set as default) on, and
   cleanly reverses it off.
7. Stretch: fan out playback to your real speakers too (monitoring), so you
   can hear what you're sending without opening a separate app.

## Phase 4 — Global hotkeys
1. Because you're on Hyprland/Wayland, the app likely can't grab global keys
   itself — plan around Hyprland's own keybind-to-command config invoking your
   app (e.g. via a small local socket/CLI trigger) rather than fighting
   Wayland's security model.
2. Implement that local trigger mechanism in the Rust backend.
3. Stretch: look at Tauri's `global-hotkey` plugin as a contrast for
   platforms where OS-level global hotkeys are actually permitted (X11,
   Windows, macOS).

## Phase 5 — Stretch: other platforms
- cpal mostly abstracts Windows/macOS playback for you; the "effortless
  setup" UX there is a separate research problem — likely means asking the
  user to install VB-Cable/BlackHole once, then automating the
  default-device-switch around it. True zero-install parity (Soundpad's
  actual trick) would mean APO/driver-level work, out of scope unless tackled
  as its own project later.

## Status
Phase 0 complete (env confirmed, Cargo workspace + Tauri/Svelte scaffold in
place, dev round trip working). Phase 1, Step 1 done (cpal device enumeration
+ default output config printed). Phase 1, Step 2 done (generated sine wave
playing through a cpal output stream, phase-continuous across callbacks).
Phase 1, Step 3 done (symphonia decodes a WAV file; duration, sample rate,
real decoded sample format, and sample count all printed from actual PCM).
Next up: Phase 1, Step 4.

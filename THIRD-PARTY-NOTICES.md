# EmuFront Rust — third-party notices & open-source credit

Any code or data taken from someone else's open-source project is
credited here, in the app's About screen, and in the source file
header where it is used. Rule: no copied code without (a) a license
that allows it, (b) the project's copyright/license notice kept with
the copy, and (c) an entry below.

## Libraries EmuFront Rust is built with (from Cargo.toml)

- egui / eframe — Emil Ernerfeldt and contributors — MIT OR Apache-2.0
- serde / serde_json — Erik Tryzelaar, David Tolnay — MIT OR Apache-2.0
- image — The image-rs contributors — MIT OR Apache-2.0
- ureq — Martin Algesten and contributors — MIT OR Apache-2.0
- sha1 (RustCrypto) — RustCrypto contributors — MIT OR Apache-2.0
- game-music-emu (crate) — MIT. The library it wraps and bundles:
  libgme / Game_Music_Emu — Shay Green (blargg) and contributors —
  LGPL-2.1-or-later. Source: https://github.com/libgme/game-music-emu
  (Statically linked into the exe; this notice and the About screen
  carry the LGPL attribution.)
- rustysynth — sinshu — MIT (pure-Rust SoundFont MIDI synth;
  a port of the author's MeltySynth)
- rodio — the rodio contributors — MIT OR Apache-2.0 (playback
  via cpal — Apache-2.0)
- flate2 — MIT OR Apache-2.0 (default miniz_oxide backend, pure
  Rust; used to inflate .vgz)
- discord-rich-presence — MIT OR Apache-2.0 (pure-Rust Discord
  Rich Presence client; local IPC to the Discord app only)

## Music content (not code)

- GeneralUser GS SoundFont — S. Christian Collins — the complete
  work carries unrestricted usage rights per its LICENSE.txt.
  NOT bundled in this repo: provisioned at runtime into
  `data_dir()/Music/_soundfonts/` with LICENSE.txt kept beside it.
- The EmuFront Lounge Loop (`assets/lounge_loop.mid`) — an original
  composition for EmuFront by Huntergokuz & Maya. No third-party
  tune, no stock music, nothing to clear.

## Data & formats

- Box art — the libretro-thumbnails project (libretro contributors).
- Cheat codes — the libretro-database project.
- Game descriptions — Wikipedia contributors, CC BY-SA 4.0.
- IPS patch format — community format. UPS and BPS patch formats —
  by Near (byuu).
- MSU-1 patch index — Zeldix (zeldix.net) and its thread authors.
- MD+ / MSU-1 patch repos — the patch authors on GitHub (infinest
  and others). MD+ pack mirror — the MiNERVA Archive and the
  Complete Sega MD+ Collection contributors.
- ROM hack listings & files — RomHack Plaza and the hack authors.
- HD texture packs — the pack authors: GhostlyDark &
  intermissionfb (evilgames.eu), qashto, vStar925, and the Mesen HD
  pack creators.

## Design references (no code copied)

- ES-DE (EmulationStation Desktop Edition) — MIT — design
  inspiration for front-end behavior and system definitions; no
  ES-DE code is used.
- Playnite — MIT — design reference for emulator profile and
  library UX; no Playnite code is used.

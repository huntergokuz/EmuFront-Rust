# Changelog

EmuFront Rust's public version line starts at **0.07 Alpha** and climbs
(0.08, 0.09, …) toward 1.0. Earlier development happened as internal
builds; their finished work ships in 0.07 Alpha below.

## 0.07 Alpha — 2026-10-26

First public source release.

### Library & browsing
- Systems wall (A–Z, per-system game counts) and a box-art cover wall.
- Letter-range browsing (0–9 / A–G / H–M / N–T / U–Z) for large systems.
- Wall search/filter, single-step Back navigation, explicit rescans.
- Library snapshots: a 200,000-game library cold-loads in ~0.5 s.

### Launching
- RetroArch by default; PCSX2 for PlayStation 2, Dolphin for
  GameCube/Wii; Genesis family via Genesis Plus GX.
- Per-game "Launch with:" picks, remembered and locked; compatibility
  ratings that follow the effective pick (with an honest "no data").
- Emulators tab: standalone one-click launches of PCSX2, Dolphin, and
  DuckStation, with a remembered file-picker fallback.

### Game features
- Box art scraping in batches of 25 into `Artwork\Boxart\<System>\`,
  with a capped on-disk cache and background, scroll-aware loading.
- Per-game soundtrack player (SPC / NSF / VGM / MIDI) with track
  lists, no autoplay, and the EmuFront Lounge Loop fallback.
- ROM hack finder (RomHack Plaza, ROMhacking.net, GitHub) with
  built-in IPS/UPS/BPS patchers; originals are never modified.
  Downloads-folder watcher for patch files.
- Enhanced Games window: separate SNES MSU-1 and Genesis MD+
  catalogs, hunts across archive.org, Zeldix, GitHub, and the
  MiNERVA Archive; pick a pack and it downloads, patches, and stages
  itself.
- Texture packs per game (download / install / run).
- Cheats browser (libretro-database), Game Info (Wikipedia), and a
  BIOS Manager that reports present/missing files plainly.

### Emulator management
- Update checks for 21 emulator projects against official releases.
- RetroArch channel picker (Nightly / Stable) with in-place
  self-update at startup, preserving cores/system/saves.

### Shelves & presence (new)
- PC Ports shelf (Quiver Launcher ports, AmigaVision).
- Playtime & last-played tracking in a dedicated sidecar file.
- Continue Playing strip and New This Week shelf (baseline-stamped so
  an existing library never masquerades as new).
- Optional Discord Rich Presence — local IPC only, ships off until
  you paste your own application ID.

### Notes
- **No ROMs, no BIOS files, and no emulators are included.** You
  supply your own games, BIOS files, and emulator installs.
- Windows 10 64-bit or later; designed to stay responsive on modest
  hardware (low-spec mode; ~2008 dual-core / 4 GB RAM design floor).

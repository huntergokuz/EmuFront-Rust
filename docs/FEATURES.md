# EmuFront Rust — full feature list

Everything the current public build (**0.07 Alpha**) does, in detail.
Features marked *in development* are built and in testing, and land in
the next public build.

## Library & browsing

- **One library, every system.** EmuFront reads a single shared library
  config (the same `emufront_config.json` format as EmuFront Box) that
  can hold hundreds of thousands of games across every system you own.
- **Systems wall.** Home shows every system as a tile with its game
  count, in strict A–Z order — no system is ever pinned out of place.
- **Cover wall.** Inside a system, games are a wall of box art with
  names, virtualized and scroll-tuned so huge shelves stay fluid.
- **Letter-range browsing.** Large systems open into letter buckets
  (0–9 / A–G / H–M / N–T / U–Z) so you jump instead of scrolling.
- **Search everywhere.** Filter the wall as you type; search matches on
  normalized names (punctuation- and case-insensitive).
- **Back navigation.** One Back step returns from a game wall to where
  you were, the way you left it.
- **Rescan with care.** Library rescans are explicit, never automatic,
  and never touch your original files.

## Launching

- **Sensible defaults.** RetroArch is the default emulator for every
  system except PlayStation 2 (PCSX2) and GameCube/Wii (Dolphin), which
  launch their standalone emulators. The Genesis family runs through
  the Genesis Plus GX core.
- **"Launch with:" freedom.** Every emulator profile you have is
  offered for every game. A pick you make by hand is remembered and
  locked for that game — EmuFront never silently overrides it.
- **Compatibility that follows the pick.** The game panel rates
  compatibility for the emulator that will *actually* run the game and
  updates live when you change "Launch with:". Where no data exists it
  says "no data" — it never borrows another emulator's score.
- **Session tracking.** Launches from the GUI are watched: when the
  game exits, the session folds into that game's playtime, last-played
  stamp, and session count. Very short handoffs don't mint fake
  sessions.
- **Standalone emulator launching.** The Emulators tab launches PCSX2,
  Dolphin, and DuckStation on their own; a missing executable is picked
  once, through a file picker, and remembered.

## The game panel

Every selected game gets a panel with:

- Box art (large), title, system, and the file it will launch.
- The effective emulator and core, with the "Launch with:" picker.
- The live compatibility rating.
- **▶ LAUNCH.**
- The **soundtrack player** (below).
- One-click tools: **Hacks & patches…**, **Find MSU-1 version…** (or
  MD+), **Scrape box art**, **Cheats…**, **Texture packs…**,
  **Game info…**.

## Box art

- Art is fetched from the libretro-thumbnails project in **batches of
  25** and cached on disk under `Artwork\Boxart\<System>\`, with the
  flat shared cache still honored for older art.
- Scraping is background work: a paced worker, tuned to scroll
  direction, so browsing never waits on the network.
- The on-disk cache is capped (see Cache & Performance), and misses are
  remembered so the same absent cover isn't re-requested forever.

## Music — the per-game soundtrack player

- Plays **SPC** (SNES), **NSF** (NES), **VGM** (Genesis / Master
  System), and **MIDI** soundtrack sets from the game's own folder.
- The track list lives under the LAUNCH button; nothing ever
  autoplays. Switching games or launching a game stops playback.
- A game with no soundtrack set plays the **EmuFront Lounge Loop**
  (an original composition) with the note: *"No soundtrack found for
  this one — enjoy the ride."*
- Chiptune playback is libgme via the game-music-emu crate; MIDI is
  rustysynth with the GeneralUser GS SoundFont.

## ROM hacks & patches

- **Find from the game's page.** Search RomHack Plaza and
  ROMhacking.net without leaving the app; results pass the same
  unpack/classify path as manual picks.
- **Built-in patchers** for IPS, UPS, and BPS. Download → patch →
  stage happens as one flow; **your original ROM is never modified** —
  the patched result is always a new file in the staging area, and the
  game appears on the ⭐ Enhanced & Hacked shelf.
- **Downloads watcher.** Start a patch download in your browser and
  EmuFront notices the finished file in your Downloads folder and
  offers to build it.
- GitHub is also searched as a hunt source for patches and packs.

## Enhanced Games (MSU-1 / MD+)

- A dedicated **Enhanced Games** window with two strictly separate
  catalogs: Super Nintendo **MSU-1** and Sega Genesis **MD+ / MSU-MD**.
- Catalog entries show what's available, what's already installed
  (and where it landed in your wall), and whether a patch also exists
  in the other section.
- Hunt sources: archive.org, Zeldix's MSU-1 index, GitHub, and the
  MiNERVA Archive (the community rescue of Myrient; note that Minerva
  distributes by BitTorrent, so a Minerva-only pick explains that
  instead of pretending to download it).
- Same rule as hacks: you pick *which one*; EmuFront downloads,
  unpacks, patches, and stages it. Originals are never touched.
- Catalogs are cached on disk, so reopening the window is instant.

## Texture packs

- For systems/emulators that support them, each game's page offers
  texture packs: find, download, install, and run — including
  per-game options where the emulator supports them (e.g. Mesen HD
  packs, Dolphin and PCSX2 texture replacements).

## Cheats, Game Info, BIOS

- **Cheats…** browses cheat codes from libretro-database for the
  selected game and toggles them per game.
- **Game info…** shows a description (Wikipedia, CC BY-SA 4.0) and
  details for the game.
- **BIOS Manager** lists the BIOS files your emulators require and
  tells you plainly which you have and which you're missing. BIOS
  files are always your own — EmuFront never downloads them.

## Emulator management

- **Manage Emulators** checks **21 emulator projects** against their
  official releases (Dolphin, PCSX2, DuckStation, Mesen, SNES9x,
  Project64, mGBA, PPSSPP, melonDS, Azahar, RPCS3, Cemu, MAME, FBNeo,
  Flycast, xemu, Stella, BizHawk, VBA-M, and RetroArch's buildbot) and
  shows installed vs. latest at a glance.
- **RetroArch self-update.** Choose the Nightly or Stable channel once
  (you can be asked at every startup instead); EmuFront checks
  buildbot at launch and updates your RetroArch in place when a newer
  build exists, preserving your cores, system files, and saves.
- Emulator installs stay **standalone and separate** — EmuFront points
  at them via profiles; it never bundles, embeds, or modifies them.

## PC Ports

- A **PC Ports** shelf for native decompilation/recomp ports installed
  through **Quiver Launcher** (Ship of Harkinian, Banjo: Recompiled,
  and hundreds more from Quiver's catalog) — they launch like any
  other game.
- Standalone collections such as **AmigaVision** are first-class
  citizens of the same shelf.

## Playtime, shelves & presence (0.07 Alpha)

- **Playtime tracking** — total time, last played, and session counts
  per game, stored in EmuFront's own sidecar file
  (`emufront_rust_data.json` beside the executable) — feature data
  never pollutes the shared library config.
- **Continue Playing** — a Home strip of what you played most
  recently, with playtime.
- **New This Week** — games first seen in the last 7 days. Your
  existing library is baseline-stamped the first time the feature
  runs, so a 200,000-game library doesn't pretend it's all new.
- **Discord "Now Playing"** — optional Rich Presence while a tracked
  game runs, over Discord's local IPC only. It ships **off**: paste
  your own Discord application ID into Cache & Performance →
  Discord presence to enable it. No tokens are stored, nothing is
  sent anywhere else.

## Performance & data

- **Snapshot startup.** A derived-data snapshot makes cold starts
  fast — the 200k-game reference library loads in about half a
  second — with fingerprint checks so a stale snapshot is never
  trusted.
- **Cache & Performance window** — cover-cache size and cap, in-memory
  art limits, low-spec mode for older PCs (the design floor is a
  ~2008 dual-core with 4 GB RAM and OpenGL 2.0), and the Discord
  presence setting.
- **Everything is local.** Settings live beside the executable; the
  data folder defaults to an `EmuFront` folder beside the executable
  and can be pointed anywhere with the `EMUFRONT_DIR` environment
  variable. Nothing about your library is ever uploaded.

## Installer & first run

- A per-user installer (Inno Setup) with a first-run wizard: point it
  at your library folders and it builds your library.
- For RetroArch, PCSX2, Dolphin, and DuckStation the installer offers,
  per emulator: **use your existing install**, or **download it from
  its official source at your request** and install it standalone.
  Nothing is fetched silently, emulators are never bundled into
  EmuFront's folder, and BIOS files are never fetched — an import page
  helps you bring your own.

## In development

- **Favorites (♥) & star ratings**, **custom shelves**,
  **"Surprise me" random picker**, **backlog roulette** ("Tonight's
  challenge" — a dealt game from your unplayed backlog, with a beaten
  scoreboard), and **smart shelves** (Never played · Not played in a
  year · No box art yet). Favorites use ♥ so the ⭐ shelf keeps its
  meaning: installed Enhanced & Hacked content.
- An identification-first library scanner (games filed under their
  true system from identification, folder as fallback) with a
  built-in renamer that proposes canonical titles and always asks
  before renaming.
- macOS and Linux builds; an Android sibling app after that.

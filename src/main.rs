// EmuFront Rust — the Rust edition of EmuFront Box.
// Developed by Huntergokuz & Maya.
//
// Credits & thanks (open source / community work this builds on):
//   egui/eframe (MIT, Emil Ernerfeldt & contributors) — UI framework
//   IPS patch format (community); UPS & BPS formats by Near (byuu)
//   libretro-thumbnails — box artwork; libretro-database — cheat data
//   RomHack Plaza & the ROM-hacking community — hack listings/files
//   GitHub patch authors (infinest and others) — MD+/MSU-1 patch
//     repos and their release files, hunted since v1.03
//   MiNERVA Archive (the community Myrient rescue) — mirror of
//     the Complete Sega MD+ Collection, hunted since v1.03
//   Wikipedia contributors (CC BY-SA 4.0) — game descriptions
//   RetroArch / libretro team — the emulation backbone
//   libgme / Game_Music_Emu (LGPL-2.1+) via game-music-emu (MIT) —
//     chiptune playback (SPC/NSF/VGM); rustysynth (MIT) — MIDI;
//     GeneralUser GS SoundFont by S. Christian Collins; rodio
//     (MIT/Apache-2.0) — audio output. The lounge loop is an
//     original composition for EmuFront.
//   discord-rich-presence (MIT OR Apache-2.0) — Discord
//     "Now Playing" presence over local IPC (v1.07).
// EmuFront Rust (egui) — the Rust edition of EmuFront Box.
// System-icon home, cover wall for any system with lazy libretro
// art (reusing the Python app's artwork cache), game panel with
// engine pick, hacks/cheats/texture-pack tooling, and Launch.
// v1.01 adds the "PC Ports" system: native decompilation ports
// installed by Quiver Launcher, launched directly (no emulator).
// v1.02 adds MSU-MD / MD+ support for Genesis games (the
// Genesis form of MSU-1): a "Find MD+ version..." hunt across
// archive.org, RomHack Plaza and (for SNES) Zeldix, staging of
// patched ROM + cue + WAV/PCM packs with the ROM and cue
// names kept matched, and Genesis Plus GX launch routing.
// v1.03 adds GitHub as a hunt source for the Find flow
// (repository search -> the repo's latest release assets, or
// the repo archive when no release qualifies, through the
// same classify/patch/stage pipeline as every other source).
// v1.03 also adds Minerva (minerva-archive.org, the community
// Myrient rescue): its Internet Archive subtree mirrors the
// Complete Sega MD+ Collection, and the hunt walks that
// listing. Minerva itself only hands files out as torrents —
// see session-20261024/MINERVA-ADDENDUM.md for the evidence.
// v1.03 also adds the "🎵 Enhanced Games" catalog window
// (Python EmuFront Box parity — its v1.05–1.08 browser):
// Super Nintendo Enhanced (MSU-1) and Sega Genesis
// Enhanced (MD+) as two strictly separate catalogs, never
// one mixed list — Zeldix + archive.org + GitHub for
// SNES, archive.org + Minerva + GitHub for Genesis —
// matched against the library and built with the same
// one-pick automatic chain as the Find window.
// v1.03 also adds the "🕹 Emulators" tab: one-click
// standalone starts for the emulators the user asked to
// reach directly — PCSX2 (PlayStation 2), Dolphin
// (GameCube / Wii), DuckStation (PlayStation). Each
// entry's executable resolves from the standalone
// profiles in the shared config (never a hardcoded
// path); when no profile resolves — or the file is
// gone — the entry offers "Select executable…" and
// the user's pick is remembered in emufront_rust.json
// (Rust-side settings, never the shared config) and
// wins over profile resolution from then on.
// v1.05 is the release-performance batch (design:
// RELEASE-PERF-DESIGN-20261024.md Slice 2 + the
// OLD-SYSTEMS-OPTIMIZATION-20261024.md fold-ins): a
// fingerprinted derived-data snapshot beside the Rust
// settings makes cold starts skip the full config
// parse (the shared config stays the only source of
// truth — the snapshot is rebuildable and silently
// discarded when it doesn't match); the config itself
// is now read with a typed streaming parse instead of
// a serde_json DOM walk, so loading no longer holds
// the file, the DOM and the structs in RAM at once; a
// private 220x300 cover-rendition cache (176x240 in
// low-spec mode) under the same cache tree spares slow
// disks and CPUs the full-size decode on repeat views;
// shared-config writes moved to one background queue
// with a changed-under-us guard, so a "Launch with:"
// pick never freezes the UI; and a first-run machine
// probe + "Low-spec machine" mode (⚙ Cache &
// Performance) tunes budgets, workers, prefetch and UI
// scale for old hardware — detected defaults only ever
// apply to a brand-new install, never to an existing
// settings file.
// v1.06 is the slow-connections batch (design:
// RELEASE-PERF-DESIGN-20261024.md Slice 3): the
// Enhanced Games catalogs persist to the private cache
// tree with a 24-hour TTL (per system, still separate)
// — reopening is instant from the saved copy, a stale
// copy renders immediately and refreshes behind itself,
// and a Refresh button re-asks the sources on demand;
// per-game Find results stay session-only, because a
// hunt's value is its freshness. The RomHack Plaza
// link-finder learned more page shapes (a second hop to
// a download page, form-based downloads, single-quoted
// and unquoted hrefs, anchor text) so fewer entries end
// at the browser fallback; every link it finds still
// passes the same unpack/classify path as before. Wall
// prefetch is tuned to the scroll direction and paced
// per frame instead of flooding the art pool.
// v1.07 is the "the app knows what you played" batch
// (FEATURE-PROGRAM-PLAN-20261024.md): the Rust sidecar
// store lands (emufront_rust_data.json beside the exe —
// feature data NEVER goes in the shared Python config),
// Continue Playing + playtime tracking, Discord Now
// Playing (#3), and the New This Week shelf (#11). GUI
// launches now retain their child handle: update()
// watches it exit and folds the session into the
// sidecar's play map (one tracked session at a time;
// headless probe launches are never recorded). Home
// gains two strips above the systems grid — ▶ Continue
// Playing (most recent first, with playtime) and
// 🆕 New This Week (games first seen in the last 7
// days; the whole day-one library is baseline-stamped
// and can never pose as new). While a tracked game
// runs, Discord Rich Presence shows it — local IPC
// only, and only when an application ID is configured
// in settings with the toggle on; with no ID, presence
// is off and costs nothing.
// Reads the EXISTING emufront_config.json, shared with the Python
// build — the Python version stays; this is the Rust edition
// alongside it.
#![windows_subsystem = "windows"]

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::io::Read as IoRead;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::os::windows::process::CommandExt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, Sense, Vec2};
use sha1::{Digest, Sha1};

const APP_VERSION: &str = "0.07 Alpha";
const BG: Color32 = Color32::from_rgb(27, 27, 38);
const PANEL: Color32 = Color32::from_rgb(35, 35, 51);
const ACCENT: Color32 = Color32::from_rgb(124, 92, 255);
const GOLD: Color32 = Color32::from_rgb(255, 209, 77);
const TEXT_DIM: Color32 = Color32::from_rgb(154, 154, 176);
// Python Box wall label / placeholder text color (#cfcfe0).
const LABEL_COL: Color32 = Color32::from_rgb(207, 207, 224);

/// The EmuFront data folder (the shared config, the
/// artwork cache, staged Enhanced & Hacked content).
/// Resolved per user, never hardcoded: the
/// EMUFRONT_DIR environment variable wins when set;
/// otherwise the portable default — an "EmuFront"
/// folder beside the exe, the same beside-the-exe
/// layout the app's own settings already use.
fn data_dir() -> PathBuf {
    if let Ok(d) = std::env::var("EMUFRONT_DIR") {
        if !d.trim().is_empty() {
            return PathBuf::from(d);
        }
    }
    exe_dir().join("EmuFront")
}

/// The resolved data dir as a string, computed once —
/// the prefix staged content is recognized by, and
/// the root the catalog builder checks installs
/// against.
fn data_dir_str() -> &'static str {
    static S: std::sync::OnceLock<String> =
        std::sync::OnceLock::new();
    S.get_or_init(|| data_dir().to_string_lossy().into_owned())
}

/// Name of the art cache folder inside the data dir. This IS
/// the box-art ("Boxart") cache: both EmuFront apps read covers
/// from here as `<key>_Box Art.png`, so scraped art stays in the
/// one shared cache folder instead of splitting into a second,
/// Rust-only cache. Kept as a single named constant so the
/// folder can move in exactly one place if that ever changes.
const ART_CACHE_SUBDIR: &str = "Artwork";

fn art_dir() -> PathBuf {
    data_dir().join(ART_CACHE_SUBDIR)
}

/// Where a game's box art lives in the cache folder.
fn boxart_file(key: &str) -> PathBuf {
    art_dir().join(format!("{key}_Box Art.png"))
}

/// Per-system box-art subfolder inside the shared cache
/// (the user's rule, Oct 19 2026: scraped box art is organized
/// BY SYSTEM). New scrapes are written under
/// `Artwork\Boxart\<System>\`; reads check there first and
/// then the shared flat cache, so every cover cached before
/// this layout -- and everything the Python app cached --
/// stays visible exactly as before.
fn boxart_sys_dir(system: &str) -> PathBuf {
    let clean: String = system
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect();
    art_dir().join("Boxart").join(clean.trim())
}

fn boxart_sys_file(game: &Game) -> PathBuf {
    boxart_sys_dir(&game.system)
        .join(format!("{}_Box Art.png", art_key(&game.path)))
}

fn boxart_sys_file_key(system: &str, key: &str) -> PathBuf {
    boxart_sys_dir(system).join(format!("{key}_Box Art.png"))
}

/// Persist a freshly scraped cover in the per-system layout.
fn save_boxart(game: &Game, bytes: &[u8]) -> bool {
    let dest = boxart_sys_file(game);
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&dest, bytes).is_ok()
}

fn proto_cache() -> PathBuf {
    let d = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("thumbcache")))
        .unwrap_or_else(|| PathBuf::from("thumbcache"));
    let _ = std::fs::create_dir_all(&d);
    d
}

fn art_key(path: &str) -> String {
    let mut h = Sha1::new();
    h.update(path.as_bytes());
    let out = h.finalize();
    out.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// Word-wrap by character count, mirroring the Python Box
/// placeholder name wrap (lines of at most `max` chars).
fn wrap_chars(name: &str, max: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for w in name.split_whitespace() {
        if cur.chars().count() + w.chars().count() > max && !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
            cur = w.to_string();
        } else {
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(w);
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// How long a recorded "no art found" lasts before the display
/// path may try the network again (30 days). A manual scrape
/// always retries immediately.
const ART_MISS_TTL_SECS: i64 = 30 * 24 * 3600;

/// Box-art scraping runs in batches of this many games at a
/// time (the user's rule, Oct 19 2026) -- never a whole system or
/// the whole library in one uninterruptible pass.
const SCRAPE_BATCH: usize = 25;

/// Letter-range browsing (the user, Oct 19 2026): opening a system
/// with more than BUCKET_MIN games lands on a chooser of five
/// letter-range buckets -- his ROM-folder ranges -- instead of
/// the full wall. Systems at or under the threshold open
/// directly, exactly as before the feature.
const BUCKET_MIN: usize = 300;

/// Bucket labels in bucket-index order (0..=4).
const BUCKET_LABELS: [&str; 5] = ["0-9", "A\u{2013}G", "H\u{2013}M", "N\u{2013}T", "U\u{2013}Z"];

/// Which letter-range bucket a game belongs to, by the first
/// alphanumeric character of its display name (catalog prefixes
/// stripped first, matching how the wall presents names):
/// digit or non-A-Z lead (symbols, accented initials) -> 0
/// ("0-9"); A-G -> 1; H-M -> 2; N-T -> 3; U-Z -> 4.
fn bucket_of(name: &str) -> u8 {
    let stripped = strip_catalog_prefix_full(name);
    let c = match stripped.chars().next() {
        Some(c) => c,
        None => return 0,
    };
    if c.is_ascii_digit() {
        return 0;
    }
    match c.to_ascii_uppercase() {
        'A'..='G' => 1,
        'H'..='M' => 2,
        'N'..='T' => 3,
        'U'..='Z' => 4,
        _ => 0,
    }
}

/// Per-bucket game counts for one system, in bucket order.
/// One pass over that system's games; used by the bucket
/// chooser and by the headless --buckets proof.
fn bucket_counts(games: &[Game], system: &str) -> [usize; 5] {
    let mut counts = [0usize; 5];
    for g in games {
        if g.system == system {
            counts[bucket_of(&g.name) as usize] += 1;
        }
    }
    counts
}

/// Back navigation (the user, Oct 23 2026): one Back step from a
/// wall. Pure decision, kept separate from the UI so
/// --selftest can pin the whole mapping: inside a letter
/// bucket of a large real system, Back lands on the bucket
/// chooser (the same step the "All letters" button takes);
/// from the chooser itself, a small system, or either
/// special shelf, Back leaves the wall for the home grid.
/// The two v1.07 shelf walls. Their titles are their
/// identity everywhere the app branches on a wall title
/// (back navigation, the bucket chooser, shelf
/// membership in refresh_wall).
const CONTINUE_TITLE: &str = "▶ Continue Playing";
const NEW_TITLE: &str = "🆕 New This Week";

/// A wall title that is a shelf, not a real system: the
/// ⭐ Enhanced shelf, PC Ports, and the two v1.07
/// sidecar shelves. Shelves have no letter buckets, no
/// per-system art scrape, and Back leaves them for Home.
/// Pure — --selftest pins the table.
fn is_shelf_title(title: &str) -> bool {
    title.starts_with('⭐')
        || title == "PC Ports"
        || title == CONTINUE_TITLE
        || title == NEW_TITLE
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BackTarget {
    Chooser,
    Home,
}

fn back_target(title: &str, bucket: Option<u8>, system_total: usize) -> BackTarget {
    let shelf = is_shelf_title(title);
    if !shelf && bucket.is_some() && system_total > BUCKET_MIN {
        BackTarget::Chooser
    } else {
        BackTarget::Home
    }
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn miss_is_fresh(ts: i64, now: i64) -> bool {
    now - ts < ART_MISS_TTL_SECS
}

/// Persistent "no art found" index (art key -> unix time of the
/// miss), stored as JSON in the Artwork folder so a miss is not
/// retried over the network on every launch.
fn art_misses_path() -> PathBuf {
    art_dir().join("art_misses.json")
}

fn load_art_misses() -> HashMap<String, i64> {
    let mut out = HashMap::new();
    if let Ok(text) = std::fs::read_to_string(art_misses_path()) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(obj) = v.as_object() {
                for (k, val) in obj {
                    if let Some(ts) = val.as_i64() {
                        out.insert(k.clone(), ts);
                    }
                }
            }
        }
    }
    out
}

fn save_art_misses(m: &HashMap<String, i64>) {
    let mut obj = serde_json::Map::new();
    for (k, ts) in m {
        obj.insert(k.clone(), serde_json::Value::from(*ts));
    }
    let text = serde_json::Value::Object(obj).to_string();
    let tmp = art_misses_path().with_extension("json.tmp");
    if std::fs::write(&tmp, text).is_ok() {
        let _ = std::fs::rename(&tmp, art_misses_path());
    }
}

/// One line per NETWORK art lookup (disk hits are not logged) —
/// a small diagnostic in the Artwork folder that makes re-scrape
/// behaviour measurable.
fn log_art_fetch(game: &Game, hit: bool) {
    use std::io::Write as _;
    let line = format!(
        "{} {} {} {} | {}\n",
        now_ts(),
        art_key(&game.path),
        if hit { "hit" } else { "miss" },
        game.system,
        game.name
    );
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(art_dir().join("art_fetch.log"))
    {
        let _ = f.write_all(line.as_bytes());
    }
}

/// The first entry of an .m3u playlist (disc 1), resolved
/// relative to the playlist — the same resolution launches use.
fn m3u_first_entry(m3u: &Path) -> Option<String> {
    let text = std::fs::read_to_string(m3u).ok()?;
    let first = text
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && !l.starts_with('#'))?;
    let dir = m3u.parent().unwrap_or(Path::new(""));
    Some(dir.join(first).to_string_lossy().to_string())
}

/// Other file paths whose cache keys may hold this game's art.
/// For .m3u playlist entries the cover is often filed under the
/// disc image (.cue, or the .bin the cue points at) instead of
/// the playlist itself.
fn art_alt_paths(game: &Game) -> Vec<String> {
    let mut out = Vec::new();
    if game.path.to_lowercase().ends_with(".m3u") {
        if let Some(disc) = m3u_first_entry(Path::new(&game.path)) {
            out.push(disc.clone());
            if disc.to_lowercase().ends_with(".cue") {
                if let Some(bin) = cue_first_file(Path::new(&disc)) {
                    out.push(bin.to_string_lossy().to_string());
                }
            }
        }
    }
    out
}

fn libretro_repo(system: &str) -> Option<&'static str> {
    Some(match system {
        "Nintendo Entertainment System" => "Nintendo - Nintendo Entertainment System",
        "Super Nintendo" => "Nintendo - Super Nintendo Entertainment System",
        "Nintendo 64" => "Nintendo - Nintendo 64",
        "Nintendo Game Boy" => "Nintendo - Game Boy",
        "Nintendo Game Boy Color" => "Nintendo - Game Boy Color",
        "Nintendo Game Boy Advance" => "Nintendo - Game Boy Advance",
        "Nintendo DS" => "Nintendo - Nintendo DS",
        "Nintendo 3DS" => "Nintendo - Nintendo 3DS",
        "Nintendo GameCube" => "Nintendo - GameCube",
        "Nintendo Wii" => "Nintendo - Nintendo Wii",
        "Nintendo Wii U" => "Nintendo - Nintendo Wii U",
        "Sega Genesis" => "Sega - Mega Drive - Genesis",
        "Sega Master System" => "Sega - Master System",
        "Sega Game Gear" => "Sega - Game Gear",
        "Sega CD" => "Sega - Mega-CD - Sega CD",
        "Sega Saturn" => "Sega - Saturn",
        "Sony PlayStation" => "Sony - PlayStation",
        "Sony PlayStation 2" => "Sony - PlayStation 2",
        "Sony PlayStation 3" => "Sony - PlayStation 3",
        "Sony PlayStation Portable" => "Sony - PlayStation Portable",
        "Arcade" => "MAME",
        "Commodore 64" => "Commodore - 64",
        "Commodore Amiga" => "Commodore - Amiga",
        "Atari 2600" => "Atari - 2600",
        "Atari 7800" => "Atari - 7800",
        "Atari Lynx" => "Atari - Lynx",
        "TurboGrafx-16" => "NEC - PC Engine - TurboGrafx 16",
        "ColecoVision" => "Coleco - ColecoVision",
        "Intellivision" => "Mattel - Intellivision",
        "Vectrex" => "GCE - Vectrex",
        "Neo Geo Pocket Color" => "SNK - Neo Geo Pocket Color",
        "Neo Geo Pocket" => "SNK - Neo Geo Pocket",
        "WonderSwan" => "Bandai - WonderSwan",
        "WonderSwan Color" => "Bandai - WonderSwan Color",
        "Microsoft Xbox" => "Microsoft - Xbox",
        "3DO" => "The 3DO Company - 3DO",
        "Neo Geo" => "SNK - Neo Geo",
        "Neo Geo CD" => "SNK - Neo Geo CD",
        _ => return None,
    })
}

fn short_name(system: &str) -> &'static str {
    match system {
        "Super Nintendo" => "SNES",
        "Nintendo Entertainment System" => "NES",
        "Nintendo Game Boy" => "GB",
        "Nintendo Game Boy Color" => "GBC",
        "Nintendo Game Boy Advance" => "GBA",
        "Nintendo 64" => "N64",
        "Nintendo GameCube" => "GAMECUBE",
        "Nintendo Wii" => "WII",
        "Nintendo Wii U" => "WII U",
        "Nintendo DS" => "DS",
        "Nintendo 3DS" => "3DS",
        "Sega Genesis" => "GENESIS",
        "Sega Master System" => "MASTER",
        "Sega Game Gear" => "GEAR",
        "Sega Saturn" => "SATURN",
        "Sega Dreamcast" => "DREAMCAST",
        "Sega CD" => "SEGA CD",
        "Sony PlayStation" => "PS1",
        "Sony PlayStation 2" => "PS2",
        "Sony PlayStation 3" => "PS3",
        "Sony PlayStation Portable" => "PSP",
        "Microsoft Xbox" => "XBOX",
        "Arcade" => "ARCADE",
        "Commodore 64" => "C64",
        "Commodore Amiga" => "AMIGA",
        "Atari 2600" => "2600",
        "Atari 7800" => "7800",
        "Atari Lynx" => "LYNX",
        "TurboGrafx-16" => "TG-16",
        "ColecoVision" => "COLECO",
        "Intellivision" => "INTV",
        "Vectrex" => "VECTREX",
        "Neo Geo" => "NEO GEO",
        "Neo Geo CD" => "NEO CD",
        "Neo Geo Pocket" => "NGP",
        "Neo Geo Pocket Color" => "NGPC",
        "WonderSwan" => "WS",
        "WonderSwan Color" => "WSC",
        "3DO" => "3DO",
        "PC Ports" => "PC",
        _ => "GAMES",
    }
}

/// Strip catalog-number prefixes ("004 ", "01. ", "07-01 ") for
/// sorting, like the Python build. Keeps "007", "1942", "3D".
fn sort_key(name: &str) -> String {
    let n = name.trim();
    let bytes = n.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() && i < 3 {
        i += 1;
    }
    if i > 0 && i < n.len() {
        let rest = &n[i..];
        if let Some(r) = rest.strip_prefix(' ').or(rest.strip_prefix(". ")) {
            if !r.is_empty() && r.chars().next().unwrap().is_alphanumeric() {
                return r.to_lowercase();
            }
        }
        // "07-01 " style
        if rest.starts_with('-') && rest.len() > 4 {
            let inner = &rest[1..];
            let mut j = 0;
            let ib = inner.as_bytes();
            while j < ib.len() && ib[j].is_ascii_digit() && j < 3 {
                j += 1;
            }
            if j > 0 && inner[j..].starts_with(' ') {
                return inner[j + 1..].to_lowercase();
            }
        }
    }
    n.to_lowercase()
}

#[derive(Clone)]
struct Game {
    name: String,
    path: String,
    system: String,
    emulator: String,
    /// True hand-lock: set only by a deliberate "Launch with:"
    /// pick. A bare `emulator` value without this flag is a
    /// stale auto-pick from an old scan (Python parity).
    emulator_locked: bool,
    patch: String,
    cheats_text: String,
    cheats_enabled: bool,
}

#[derive(Clone)]
struct Profile {
    name: String,
    path: String,
    core: String,
    system: String,
    args: String,
}

// ---------------- typed config parsing (v1.05) ----------------------
// The config is big (~59 MB at the user's library size) and the
// old loader built three copies of it at once: the file text,
// a full serde_json DOM, and the extracted structs. That peak
// RAM is the single biggest old-machine pain (OLD-SYSTEMS
// study §2.1). The parse below is typed and streaming:
// serde_json walks the file once, allocating only the finished
// Game/Profile structs. It keeps the old hand-walk's exact
// lenient semantics, pinned by fixtures in --selftest (missing
// fields default, wrong-typed fields read as "" / false,
// unknown fields ignored, a non-array games/emulators section
// reads as empty, unparseable input reads as an empty
// library) — so --check/--sweep output is provably identical.
// The old DOM walk survives only as legacy_parse_config,
// used by those fixtures.

/// A string field read leniently: a JSON string is itself;
/// anything else (number, bool, null, object, array) reads
/// as "" — the old walk's `as_str().unwrap_or("")` rule.
struct LString(String);

impl<'de> serde::Deserialize<'de> for LString {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = String;
            fn expecting(
                &self,
                f: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                write!(f, "a string (anything else reads as empty)")
            }
            fn visit_str<E>(self, s: &str) -> Result<String, E> {
                Ok(s.to_string())
            }
            fn visit_string<E>(self, s: String) -> Result<String, E> {
                Ok(s)
            }
            fn visit_borrowed_str<E>(
                self,
                s: &'de str,
            ) -> Result<String, E> {
                Ok(s.to_string())
            }
            fn visit_bool<E>(self, _: bool) -> Result<String, E> {
                Ok(String::new())
            }
            fn visit_i64<E>(self, _: i64) -> Result<String, E> {
                Ok(String::new())
            }
            fn visit_u64<E>(self, _: u64) -> Result<String, E> {
                Ok(String::new())
            }
            fn visit_f64<E>(self, _: f64) -> Result<String, E> {
                Ok(String::new())
            }
            fn visit_unit<E>(self) -> Result<String, E> {
                Ok(String::new())
            }
            fn visit_none<E>(self) -> Result<String, E> {
                Ok(String::new())
            }
            fn visit_map<A>(
                self,
                mut m: A,
            ) -> Result<String, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                while let Some((_, _)) = m.next_entry::<
                    serde::de::IgnoredAny,
                    serde::de::IgnoredAny,
                >()?
                {}
                Ok(String::new())
            }
            fn visit_seq<A>(
                self,
                mut s: A,
            ) -> Result<String, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                while s
                    .next_element::<serde::de::IgnoredAny>()?
                    .is_some()
                {}
                Ok(String::new())
            }
        }
        d.deserialize_any(V).map(LString)
    }
}

/// A bool field read leniently: a JSON bool is itself;
/// anything else reads as false — the old walk's
/// `as_bool().unwrap_or(false)` rule.
struct LBool(bool);

impl<'de> serde::Deserialize<'de> for LBool {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = bool;
            fn expecting(
                &self,
                f: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                write!(f, "a bool (anything else reads as false)")
            }
            fn visit_bool<E>(self, b: bool) -> Result<bool, E> {
                Ok(b)
            }
            fn visit_i64<E>(self, _: i64) -> Result<bool, E> {
                Ok(false)
            }
            fn visit_u64<E>(self, _: u64) -> Result<bool, E> {
                Ok(false)
            }
            fn visit_f64<E>(self, _: f64) -> Result<bool, E> {
                Ok(false)
            }
            fn visit_str<E>(self, _: &str) -> Result<bool, E> {
                Ok(false)
            }
            fn visit_string<E>(self, _: String) -> Result<bool, E> {
                Ok(false)
            }
            fn visit_borrowed_str<E>(
                self,
                _: &'de str,
            ) -> Result<bool, E> {
                Ok(false)
            }
            fn visit_unit<E>(self) -> Result<bool, E> {
                Ok(false)
            }
            fn visit_none<E>(self) -> Result<bool, E> {
                Ok(false)
            }
            fn visit_map<A>(
                self,
                mut m: A,
            ) -> Result<bool, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                while let Some((_, _)) = m.next_entry::<
                    serde::de::IgnoredAny,
                    serde::de::IgnoredAny,
                >()?
                {}
                Ok(false)
            }
            fn visit_seq<A>(
                self,
                mut s: A,
            ) -> Result<bool, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                while s
                    .next_element::<serde::de::IgnoredAny>()?
                    .is_some()
                {}
                Ok(false)
            }
        }
        d.deserialize_any(V).map(LBool)
    }
}

/// One config game entry, read through the lenient field
/// readers. An entry that isn't an object at all reads as
/// a fully-default entry — the old walk pushed a Game of
/// defaults for those too (`g.get(...)` on a non-object is
/// None for every field).
struct CfgGameT {
    name: String,
    path: String,
    system: String,
    emulator: String,
    emulator_locked: bool,
    patch: String,
    cheats_text: String,
    cheats_enabled: bool,
}

impl CfgGameT {
    fn blank() -> Self {
        CfgGameT {
            name: String::new(),
            path: String::new(),
            system: String::new(),
            emulator: String::new(),
            emulator_locked: false,
            patch: String::new(),
            cheats_text: String::new(),
            cheats_enabled: false,
        }
    }
    fn into_game(self) -> Game {
        Game {
            name: self.name,
            path: self.path,
            system: self.system,
            emulator: self.emulator,
            emulator_locked: self.emulator_locked,
            patch: self.patch,
            cheats_text: self.cheats_text,
            cheats_enabled: self.cheats_enabled,
        }
    }
}

impl<'de> serde::Deserialize<'de> for CfgGameT {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = CfgGameT;
            fn expecting(
                &self,
                f: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                write!(f, "a game object (anything else reads as a default entry)")
            }
            fn visit_map<A>(
                self,
                mut m: A,
            ) -> Result<CfgGameT, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut g = CfgGameT::blank();
                while let Some(key) = m.next_key::<String>()? {
                    match key.as_str() {
                        "name" => g.name = m.next_value::<LString>()?.0,
                        "path" => g.path = m.next_value::<LString>()?.0,
                        "system" => {
                            g.system = m.next_value::<LString>()?.0
                        }
                        "emulator" => {
                            g.emulator = m.next_value::<LString>()?.0
                        }
                        "emulator_locked" => {
                            g.emulator_locked =
                                m.next_value::<LBool>()?.0
                        }
                        "patch" => g.patch = m.next_value::<LString>()?.0,
                        "cheats_text" => {
                            g.cheats_text = m.next_value::<LString>()?.0
                        }
                        "cheats_enabled" => {
                            g.cheats_enabled =
                                m.next_value::<LBool>()?.0
                        }
                        _ => {
                            m.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(g)
            }
            fn visit_str<E>(self, _: &str) -> Result<CfgGameT, E> {
                Ok(CfgGameT::blank())
            }
            fn visit_string<E>(
                self,
                _: String,
            ) -> Result<CfgGameT, E> {
                Ok(CfgGameT::blank())
            }
            fn visit_borrowed_str<E>(
                self,
                _: &'de str,
            ) -> Result<CfgGameT, E> {
                Ok(CfgGameT::blank())
            }
            fn visit_bool<E>(self, _: bool) -> Result<CfgGameT, E> {
                Ok(CfgGameT::blank())
            }
            fn visit_i64<E>(self, _: i64) -> Result<CfgGameT, E> {
                Ok(CfgGameT::blank())
            }
            fn visit_u64<E>(self, _: u64) -> Result<CfgGameT, E> {
                Ok(CfgGameT::blank())
            }
            fn visit_f64<E>(self, _: f64) -> Result<CfgGameT, E> {
                Ok(CfgGameT::blank())
            }
            fn visit_unit<E>(self) -> Result<CfgGameT, E> {
                Ok(CfgGameT::blank())
            }
            fn visit_none<E>(self) -> Result<CfgGameT, E> {
                Ok(CfgGameT::blank())
            }
            fn visit_seq<A>(
                self,
                mut s: A,
            ) -> Result<CfgGameT, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                while s
                    .next_element::<serde::de::IgnoredAny>()?
                    .is_some()
                {}
                Ok(CfgGameT::blank())
            }
        }
        d.deserialize_any(V)
    }
}

/// One config emulator profile entry, read the same way.
struct CfgProfileT {
    name: String,
    path: String,
    core: String,
    system: String,
    args: String,
}

impl CfgProfileT {
    fn blank() -> Self {
        CfgProfileT {
            name: String::new(),
            path: String::new(),
            core: String::new(),
            system: String::new(),
            args: String::new(),
        }
    }
    fn into_profile(self) -> Profile {
        Profile {
            name: self.name,
            path: self.path,
            core: self.core,
            system: self.system,
            args: self.args,
        }
    }
}

impl<'de> serde::Deserialize<'de> for CfgProfileT {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = CfgProfileT;
            fn expecting(
                &self,
                f: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                write!(f, "a profile object (anything else reads as a default entry)")
            }
            fn visit_map<A>(
                self,
                mut m: A,
            ) -> Result<CfgProfileT, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut p = CfgProfileT::blank();
                while let Some(key) = m.next_key::<String>()? {
                    match key.as_str() {
                        "name" => p.name = m.next_value::<LString>()?.0,
                        "path" => p.path = m.next_value::<LString>()?.0,
                        "core" => p.core = m.next_value::<LString>()?.0,
                        "system" => {
                            p.system = m.next_value::<LString>()?.0
                        }
                        "args" => p.args = m.next_value::<LString>()?.0,
                        _ => {
                            m.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(p)
            }
            fn visit_str<E>(self, _: &str) -> Result<CfgProfileT, E> {
                Ok(CfgProfileT::blank())
            }
            fn visit_string<E>(
                self,
                _: String,
            ) -> Result<CfgProfileT, E> {
                Ok(CfgProfileT::blank())
            }
            fn visit_borrowed_str<E>(
                self,
                _: &'de str,
            ) -> Result<CfgProfileT, E> {
                Ok(CfgProfileT::blank())
            }
            fn visit_bool<E>(
                self,
                _: bool,
            ) -> Result<CfgProfileT, E> {
                Ok(CfgProfileT::blank())
            }
            fn visit_i64<E>(self, _: i64) -> Result<CfgProfileT, E> {
                Ok(CfgProfileT::blank())
            }
            fn visit_u64<E>(self, _: u64) -> Result<CfgProfileT, E> {
                Ok(CfgProfileT::blank())
            }
            fn visit_f64<E>(self, _: f64) -> Result<CfgProfileT, E> {
                Ok(CfgProfileT::blank())
            }
            fn visit_unit<E>(self) -> Result<CfgProfileT, E> {
                Ok(CfgProfileT::blank())
            }
            fn visit_none<E>(self) -> Result<CfgProfileT, E> {
                Ok(CfgProfileT::blank())
            }
            fn visit_seq<A>(
                self,
                mut s: A,
            ) -> Result<CfgProfileT, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                while s
                    .next_element::<serde::de::IgnoredAny>()?
                    .is_some()
                {}
                Ok(CfgProfileT::blank())
            }
        }
        d.deserialize_any(V)
    }
}

/// The config's "games" section: an array of entries, or
/// (when it holds anything else) an empty list — the old
/// walk's `and_then(|g| g.as_array())` rule.
struct GamesT(Vec<Game>);

impl<'de> serde::Deserialize<'de> for GamesT {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = GamesT;
            fn expecting(
                &self,
                f: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                write!(f, "an array of game objects")
            }
            fn visit_seq<A>(
                self,
                mut s: A,
            ) -> Result<GamesT, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut out = Vec::new();
                while let Some(g) = s.next_element::<CfgGameT>()? {
                    out.push(g.into_game());
                }
                Ok(GamesT(out))
            }
            fn visit_map<A>(
                self,
                mut m: A,
            ) -> Result<GamesT, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                while let Some((_, _)) = m.next_entry::<
                    serde::de::IgnoredAny,
                    serde::de::IgnoredAny,
                >()?
                {}
                Ok(GamesT(Vec::new()))
            }
            fn visit_str<E>(self, _: &str) -> Result<GamesT, E> {
                Ok(GamesT(Vec::new()))
            }
            fn visit_string<E>(self, _: String) -> Result<GamesT, E> {
                Ok(GamesT(Vec::new()))
            }
            fn visit_borrowed_str<E>(
                self,
                _: &'de str,
            ) -> Result<GamesT, E> {
                Ok(GamesT(Vec::new()))
            }
            fn visit_bool<E>(self, _: bool) -> Result<GamesT, E> {
                Ok(GamesT(Vec::new()))
            }
            fn visit_i64<E>(self, _: i64) -> Result<GamesT, E> {
                Ok(GamesT(Vec::new()))
            }
            fn visit_u64<E>(self, _: u64) -> Result<GamesT, E> {
                Ok(GamesT(Vec::new()))
            }
            fn visit_f64<E>(self, _: f64) -> Result<GamesT, E> {
                Ok(GamesT(Vec::new()))
            }
            fn visit_unit<E>(self) -> Result<GamesT, E> {
                Ok(GamesT(Vec::new()))
            }
            fn visit_none<E>(self) -> Result<GamesT, E> {
                Ok(GamesT(Vec::new()))
            }
        }
        d.deserialize_any(V)
    }
}

/// The config's "emulators" section, same rule.
struct ProfilesT(Vec<Profile>);

impl<'de> serde::Deserialize<'de> for ProfilesT {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = ProfilesT;
            fn expecting(
                &self,
                f: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                write!(f, "an array of profile objects")
            }
            fn visit_seq<A>(
                self,
                mut s: A,
            ) -> Result<ProfilesT, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut out = Vec::new();
                while let Some(p) = s.next_element::<CfgProfileT>()? {
                    out.push(p.into_profile());
                }
                Ok(ProfilesT(out))
            }
            fn visit_map<A>(
                self,
                mut m: A,
            ) -> Result<ProfilesT, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                while let Some((_, _)) = m.next_entry::<
                    serde::de::IgnoredAny,
                    serde::de::IgnoredAny,
                >()?
                {}
                Ok(ProfilesT(Vec::new()))
            }
            fn visit_str<E>(self, _: &str) -> Result<ProfilesT, E> {
                Ok(ProfilesT(Vec::new()))
            }
            fn visit_string<E>(
                self,
                _: String,
            ) -> Result<ProfilesT, E> {
                Ok(ProfilesT(Vec::new()))
            }
            fn visit_borrowed_str<E>(
                self,
                _: &'de str,
            ) -> Result<ProfilesT, E> {
                Ok(ProfilesT(Vec::new()))
            }
            fn visit_bool<E>(self, _: bool) -> Result<ProfilesT, E> {
                Ok(ProfilesT(Vec::new()))
            }
            fn visit_i64<E>(self, _: i64) -> Result<ProfilesT, E> {
                Ok(ProfilesT(Vec::new()))
            }
            fn visit_u64<E>(self, _: u64) -> Result<ProfilesT, E> {
                Ok(ProfilesT(Vec::new()))
            }
            fn visit_f64<E>(self, _: f64) -> Result<ProfilesT, E> {
                Ok(ProfilesT(Vec::new()))
            }
            fn visit_unit<E>(self) -> Result<ProfilesT, E> {
                Ok(ProfilesT(Vec::new()))
            }
            fn visit_none<E>(self) -> Result<ProfilesT, E> {
                Ok(ProfilesT(Vec::new()))
            }
        }
        d.deserialize_any(V)
    }
}

/// The whole config file, in the typed streaming shape.
/// Anything but an object at the top level reads as an
/// empty library (the old walk's `v.get("games")` on a
/// non-object is None) — via the non-map visitors.
#[derive(Default)]
struct CfgFileT {
    games: Vec<Game>,
    profiles: Vec<Profile>,
}

impl<'de> serde::Deserialize<'de> for CfgFileT {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = CfgFileT;
            fn expecting(
                &self,
                f: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                write!(f, "an EmuFront config object")
            }
            fn visit_map<A>(
                self,
                mut m: A,
            ) -> Result<CfgFileT, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut out = CfgFileT::default();
                while let Some(key) = m.next_key::<String>()? {
                    match key.as_str() {
                        "games" => {
                            out.games = m.next_value::<GamesT>()?.0
                        }
                        "emulators" => {
                            out.profiles = m.next_value::<ProfilesT>()?.0
                        }
                        _ => {
                            m.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(out)
            }
            fn visit_seq<A>(
                self,
                mut s: A,
            ) -> Result<CfgFileT, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                while s
                    .next_element::<serde::de::IgnoredAny>()?
                    .is_some()
                {}
                Ok(CfgFileT::default())
            }
            fn visit_str<E>(self, _: &str) -> Result<CfgFileT, E> {
                Ok(CfgFileT::default())
            }
            fn visit_string<E>(
                self,
                _: String,
            ) -> Result<CfgFileT, E> {
                Ok(CfgFileT::default())
            }
            fn visit_borrowed_str<E>(
                self,
                _: &'de str,
            ) -> Result<CfgFileT, E> {
                Ok(CfgFileT::default())
            }
            fn visit_bool<E>(self, _: bool) -> Result<CfgFileT, E> {
                Ok(CfgFileT::default())
            }
            fn visit_i64<E>(self, _: i64) -> Result<CfgFileT, E> {
                Ok(CfgFileT::default())
            }
            fn visit_u64<E>(self, _: u64) -> Result<CfgFileT, E> {
                Ok(CfgFileT::default())
            }
            fn visit_f64<E>(self, _: f64) -> Result<CfgFileT, E> {
                Ok(CfgFileT::default())
            }
            fn visit_unit<E>(self) -> Result<CfgFileT, E> {
                Ok(CfgFileT::default())
            }
            fn visit_none<E>(self) -> Result<CfgFileT, E> {
                Ok(CfgFileT::default())
            }
        }
        d.deserialize_any(V)
    }
}

/// The typed streaming parse. Any hard syntax error reads
/// as an empty library — the old loader's
/// `unwrap_or(Value::Null)` behavior, kept deliberately:
/// a corrupt config must never stop the app from opening.
fn parse_config_bytes(bytes: &[u8]) -> (Vec<Game>, Vec<Profile>) {
    match serde_json::from_slice::<CfgFileT>(bytes) {
        Ok(f) => (f.games, f.profiles),
        Err(_) => (Vec::new(), Vec::new()),
    }
}

/// The pre-v1.05 loader's DOM walk, kept ONLY so
/// --selftest can run the same fixtures through both
/// parsers and pin identical results. Production code
/// never calls this.
fn legacy_parse_config(
    v: &serde_json::Value,
) -> (Vec<Game>, Vec<Profile>) {
    let field = |g: &serde_json::Value, k: &str| -> String {
        g.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string()
    };
    let flag = |g: &serde_json::Value, k: &str| -> bool {
        g.get(k).and_then(|x| x.as_bool()).unwrap_or(false)
    };
    let mut games = Vec::new();
    if let Some(arr) = v.get("games").and_then(|g| g.as_array()) {
        for g in arr {
            games.push(Game {
                name: field(g, "name"),
                path: field(g, "path"),
                system: field(g, "system"),
                emulator: field(g, "emulator"),
                emulator_locked: flag(g, "emulator_locked"),
                patch: field(g, "patch"),
                cheats_text: field(g, "cheats_text"),
                cheats_enabled: flag(g, "cheats_enabled"),
            });
        }
    }
    let mut profiles = Vec::new();
    if let Some(arr) = v.get("emulators").and_then(|g| g.as_array()) {
        for g in arr {
            profiles.push(Profile {
                name: field(g, "name"),
                path: field(g, "path"),
                core: field(g, "core"),
                system: field(g, "system"),
                args: field(g, "args"),
            });
        }
    }
    (games, profiles)
}

fn legacy_parse_str(text: &str) -> (Vec<Game>, Vec<Profile>) {
    let v: serde_json::Value =
        serde_json::from_str(text).unwrap_or(serde_json::Value::Null);
    legacy_parse_config(&v)
}

/// A fingerprint of the config file exactly as one load
/// saw it: size + mtime for the cheap tier, SHA-1 for the
/// confirmation tier (snapshot design §2.1). The size is
/// the parsed bytes' length — the snapshot may only claim
/// to represent bytes that were actually seen.
struct CfgFingerprint {
    size: u64,
    mtime_secs: u64,
    mtime_nanos: u32,
    sha1: String,
}

fn sha1_hex(bytes: &[u8]) -> String {
    let mut h = Sha1::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Stat-only view of the config file: (size, mtime secs,
/// mtime nanos). None when the file is absent/unreadable.
fn cfg_stat() -> Option<(u64, u64, u32)> {
    let m = std::fs::metadata(cfg_path()).ok()?;
    let t = m.modified().ok()?;
    let d = t.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some((m.len(), d.as_secs(), d.subsec_nanos()))
}

/// Read the config file's bytes once, with the
/// fingerprint computed over the exact bytes the parse
/// will see. A missing file reads as the old default
/// text (an empty library).
fn read_config_bytes() -> (Vec<u8>, CfgFingerprint) {
    let bytes = std::fs::read(cfg_path())
        .unwrap_or_else(|_| b"{\"games\":[],\"emulators\":[]}".to_vec());
    let (_, mtime_secs, mtime_nanos) =
        cfg_stat().unwrap_or((0, 0, 0));
    let fp = CfgFingerprint {
        size: bytes.len() as u64,
        mtime_secs,
        mtime_nanos,
        sha1: sha1_hex(&bytes),
    };
    (bytes, fp)
}

struct LoadedLib {
    games: Vec<Game>,
    profiles: Vec<Profile>,
    load_ms: f64,
    fingerprint: CfgFingerprint,
}

fn load_library_full() -> LoadedLib {
    let t = Instant::now();
    let (bytes, fp) = read_config_bytes();
    let (games, profiles) = parse_config_bytes(&bytes);
    LoadedLib {
        games,
        profiles,
        load_ms: t.elapsed().as_secs_f64() * 1000.0,
        fingerprint: fp,
    }
}

/// The library load the headless gates use (--check,
/// --buckets, --sweep, the CLI probes): always a full
/// parse of the real config — never the snapshot, so
/// the proofs prove the file itself.
fn load_library() -> (Vec<Game>, Vec<Profile>, f64) {
    let l = load_library_full();
    (l.games, l.profiles, l.load_ms)
}

// ---------------- derived data + the startup snapshot ---------------
// Everything the app derives from the config at load time,
// computed once: the search index (Slice 1), each game's art
// key and ⭐-shelf flag, the systems list in presentation
// order, each system's game indexes in wall order, and each
// system's letter-bucket membership. This bundle is exactly
// what the snapshot file stores (design §2.1), so a snapshot
// hit needs no derivation pass at all.

#[derive(Default)]
struct Derived {
    lower_names: Vec<String>,
    sort_keys: Vec<String>,
    art_keys: Vec<String>,
    star: Vec<bool>,
    systems: Vec<(String, usize)>,
    system_games: Vec<(String, Vec<usize>)>,
    system_buckets: Vec<(String, [Vec<usize>; 5])>,
}

impl Derived {
    fn system_list(&self, system: &str) -> Option<&Vec<usize>> {
        self.system_games
            .iter()
            .find(|(s, _)| s == system)
            .map(|(_, v)| v)
    }
    fn bucket_counts(&self, system: &str) -> Option<[usize; 5]> {
        self.system_buckets
            .iter()
            .find(|(s, _)| s == system)
            .map(|(_, b)| {
                [b[0].len(), b[1].len(), b[2].len(), b[3].len(), b[4].len()]
            })
    }
}

fn build_derived(games: &[Game]) -> Derived {
    let (lower_names, sort_keys) = build_search_index(games);
    let art_keys: Vec<String> =
        games.iter().map(|g| art_key(&g.path)).collect();
    let star: Vec<bool> = games
        .iter()
        .map(|g| g.path.starts_with(data_dir_str()))
        .collect();
    let systems = sorted_systems(games);
    let mut by_sys: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, g) in games.iter().enumerate() {
        by_sys.entry(g.system.clone()).or_default().push(i);
    }
    let mut system_games: Vec<(String, Vec<usize>)> = Vec::new();
    let mut system_buckets: Vec<(String, [Vec<usize>; 5])> = Vec::new();
    for (sys, _) in &systems {
        let mut idx = by_sys.get(sys).cloned().unwrap_or_default();
        // The exact wall order enter_wall sorts by (Slice 1):
        // precomputed sort key, then the name itself.
        idx.sort_by(|a, b| {
            sort_keys[*a]
                .cmp(&sort_keys[*b])
                .then(games[*a].name.cmp(&games[*b].name))
        });
        let mut buckets: [Vec<usize>; 5] = Default::default();
        for &i in &idx {
            buckets[bucket_of(&games[i].name) as usize].push(i);
        }
        system_buckets.push((sys.clone(), buckets));
        system_games.push((sys.clone(), idx));
    }
    Derived {
        lower_names,
        sort_keys,
        art_keys,
        star,
        systems,
        system_games,
        system_buckets,
    }
}

/// The private cache tree: beside the Rust settings,
/// never inside the shared data dir (design §2 preamble).
/// Everything in it is derived data the app can rebuild;
/// Clear Cache empties the whole tree and nothing breaks.
fn cache_dir() -> PathBuf {
    exe_dir().join("cache")
}

fn snapshot_dir() -> PathBuf {
    cache_dir().join("snapshot")
}

fn thumbs_dir() -> PathBuf {
    cache_dir().join("thumbs")
}

fn catalogs_dir() -> PathBuf {
    cache_dir().join("catalogs")
}

fn dir_size(p: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(rd) = std::fs::read_dir(p) {
        for e in rd.flatten() {
            let path = e.path();
            if path.is_dir() {
                total += dir_size(&path);
            } else if let Ok(m) = e.metadata() {
                total += m.len();
            }
        }
    }
    total
}

fn clear_cache_tree() {
    let _ = std::fs::remove_dir_all(cache_dir());
    let _ = std::fs::create_dir_all(snapshot_dir());
    let _ = std::fs::create_dir_all(thumbs_dir());
    let _ = std::fs::create_dir_all(catalogs_dir());
}

const SNAP_MAGIC: &str = "emufront-rust-snapshot";
const SNAP_SCHEMA: u32 = 1;

fn snapshot_path() -> PathBuf {
    snapshot_dir().join(format!("library-v{SNAP_SCHEMA}.json"))
}

/// The snapshot file (design §2.1): the library plus its
/// derived bundle, games as array rows in fixed field
/// order (cheaper to parse than object-per-game), under a
/// header carrying the config fingerprint it was built
/// from. It is a CACHE: the shared config stays the only
/// source of truth, and any doubt about this file means
/// the full parse runs instead.
#[derive(serde::Serialize, serde::Deserialize)]
struct SnapFile {
    magic: String,
    schema: u32,
    app: String,
    created: u64,
    cfg_size: u64,
    cfg_mtime_secs: u64,
    cfg_mtime_nanos: u32,
    cfg_sha1: String,
    /// (name, path, system, emulator, emulator_locked,
    /// patch, cheats_text, cheats_enabled)
    games: Vec<(String, String, String, String, bool, String, String, bool)>,
    /// (name, path, core, system, args)
    profiles: Vec<(String, String, String, String, String)>,
    lower: Vec<String>,
    sortk: Vec<String>,
    artk: Vec<String>,
    star: Vec<bool>,
    systems: Vec<(String, usize)>,
    sysgames: Vec<(String, Vec<usize>)>,
    buckets: Vec<(String, [Vec<usize>; 5])>,
}

fn build_snapshot(
    games: &[Game],
    profiles: &[Profile],
    derived: &Derived,
    fp: &CfgFingerprint,
) -> SnapFile {
    SnapFile {
        magic: SNAP_MAGIC.to_string(),
        schema: SNAP_SCHEMA,
        app: APP_VERSION.to_string(),
        created: now_epoch(),
        cfg_size: fp.size,
        cfg_mtime_secs: fp.mtime_secs,
        cfg_mtime_nanos: fp.mtime_nanos,
        cfg_sha1: fp.sha1.clone(),
        games: games
            .iter()
            .map(|g| {
                (
                    g.name.clone(),
                    g.path.clone(),
                    g.system.clone(),
                    g.emulator.clone(),
                    g.emulator_locked,
                    g.patch.clone(),
                    g.cheats_text.clone(),
                    g.cheats_enabled,
                )
            })
            .collect(),
        profiles: profiles
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    p.path.clone(),
                    p.core.clone(),
                    p.system.clone(),
                    p.args.clone(),
                )
            })
            .collect(),
        lower: derived.lower_names.clone(),
        sortk: derived.sort_keys.clone(),
        artk: derived.art_keys.clone(),
        star: derived.star.clone(),
        systems: derived.systems.clone(),
        sysgames: derived.system_games.clone(),
        buckets: derived.system_buckets.clone(),
    }
}

/// The sanity gate every snapshot must pass before a
/// single row of it is trusted: magic + schema, every
/// parallel vector the same length as the games list,
/// and the per-system lists accounting for every game
/// exactly (index totals and bounds). Pure —
/// --selftest pins the accept/reject shapes.
fn snap_valid(s: &SnapFile) -> bool {
    if s.magic != SNAP_MAGIC || s.schema != SNAP_SCHEMA {
        return false;
    }
    let n = s.games.len();
    if s.lower.len() != n
        || s.sortk.len() != n
        || s.artk.len() != n
        || s.star.len() != n
    {
        return false;
    }
    if s.systems.iter().map(|(_, c)| *c).sum::<usize>() != n {
        return false;
    }
    let mut total = 0usize;
    for (_, idx) in &s.sysgames {
        total += idx.len();
        if idx.iter().any(|&i| i >= n) {
            return false;
        }
    }
    if total != n {
        return false;
    }
    let mut btotal = 0usize;
    for (_, b) in &s.buckets {
        for v in b {
            btotal += v.len();
            if v.iter().any(|&i| i >= n) {
                return false;
            }
        }
    }
    btotal == n
}

fn snap_into_parts(
    s: SnapFile,
) -> (Vec<Game>, Vec<Profile>, Derived) {
    let games: Vec<Game> = s
        .games
        .into_iter()
        .map(|(name, path, system, emulator, locked, patch, ct, ce)| {
            Game {
                name,
                path,
                system,
                emulator,
                emulator_locked: locked,
                patch,
                cheats_text: ct,
                cheats_enabled: ce,
            }
        })
        .collect();
    let profiles: Vec<Profile> = s
        .profiles
        .into_iter()
        .map(|(name, path, core, system, args)| Profile {
            name,
            path,
            core,
            system,
            args,
        })
        .collect();
    let derived = Derived {
        lower_names: s.lower,
        sort_keys: s.sortk,
        art_keys: s.artk,
        star: s.star,
        systems: s.systems,
        system_games: s.sysgames,
        system_buckets: s.buckets,
    };
    (games, profiles, derived)
}

/// Which snapshot tier a startup lands in (design §2.1):
/// stat fast path first, SHA-1 confirmation when the
/// stat moved, full rebuild otherwise. Pure — --selftest
/// pins the table.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SnapTier {
    Use,
    UseRefreshStat,
    Rebuild,
}

fn snap_tier(stat_match: bool, sha_match: bool) -> SnapTier {
    if stat_match {
        SnapTier::Use
    } else if sha_match {
        SnapTier::UseRefreshStat
    } else {
        SnapTier::Rebuild
    }
}

/// Try the snapshot. Any failure at any stage — missing
/// file, bad JSON, wrong magic, fingerprint mismatch,
/// failed sanity — returns None and the caller does the
/// full parse. No error is ever shown for a cache miss;
/// that silence is a design requirement.
fn snap_try_load() -> Option<(Vec<Game>, Vec<Profile>, Derived)> {
    let raw = std::fs::read(snapshot_path()).ok()?;
    let snap: SnapFile = serde_json::from_slice(&raw).ok()?;
    if !snap_valid(&snap) {
        return None;
    }
    let stat_match = cfg_stat()
        .map(|(sz, ms, mn)| {
            sz == snap.cfg_size
                && ms == snap.cfg_mtime_secs
                && mn == snap.cfg_mtime_nanos
        })
        .unwrap_or(false);
    if snap_tier(stat_match, false) == SnapTier::Use {
        return Some(snap_into_parts(snap));
    }
    // Hash tier: the stat moved (a copy, a touch, a
    // cloud-sync rewrite) — is the content really the
    // same bytes the snapshot was built from?
    let (_bytes, fp) = read_config_bytes();
    if snap_tier(false, fp.sha1 == snap.cfg_sha1)
        != SnapTier::UseRefreshStat
    {
        return None;
    }
    let parts = snap_into_parts(snap);
    // Refresh the stored stat so the next start takes
    // the fast path again. Background write; the data
    // just loaded doesn't wait for it.
    let refreshed = build_snapshot(&parts.0, &parts.1, &parts.2, &fp);
    std::thread::spawn(move || persist_snapshot(&refreshed));
    Some(parts)
}

/// Write the snapshot atomically (tmp + rename, the
/// pattern save_cfg uses) and clear away snapshots from
/// other schema versions while we're here.
fn persist_snapshot(snap: &SnapFile) {
    let dir = snapshot_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(text) = serde_json::to_string(snap) {
        let tmp = dir.join(format!(".library-v{}.tmp", snap.schema));
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, snapshot_path());
        }
    }
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("library-v")
                && name.ends_with(".json")
                && e.path() != snapshot_path()
            {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
}

struct StartupLib {
    games: Vec<Game>,
    profiles: Vec<Profile>,
    derived: Derived,
    load_ms: f64,
}

/// The GUI's startup load: snapshot if it verifies, the
/// full typed parse otherwise (with the snapshot rebuilt
/// in the background for next time). Runs on Proto's
/// loader thread — the window is already up showing the
/// loading state while this happens.
fn load_startup_library() -> StartupLib {
    let t = Instant::now();
    if let Some((games, profiles, derived)) = snap_try_load() {
        return StartupLib {
            games,
            profiles,
            derived,
            load_ms: t.elapsed().as_secs_f64() * 1000.0,
        };
    }
    let l = load_library_full();
    let derived = build_derived(&l.games);
    let snap =
        build_snapshot(&l.games, &l.profiles, &derived, &l.fingerprint);
    std::thread::spawn(move || persist_snapshot(&snap));
    StartupLib {
        games: l.games,
        profiles: l.profiles,
        derived,
        load_ms: t.elapsed().as_secs_f64() * 1000.0,
    }
}

fn game_has_msu(game: &Game) -> bool {
    let p = Path::new(&game.path);
    if !p.starts_with(data_dir()) {
        return false;
    }
    if let Some(dir) = p.parent() {
        if let Ok(rd) = std::fs::read_dir(dir) {
            let names: Vec<String> = rd
                .flatten()
                .map(|e| e.file_name().to_string_lossy().to_lowercase())
                .collect();
            // .pcm staging counts too (Python parity): MD+
            // packs don't always carry a .msu file.
            if names.iter().any(|n| n.ends_with(".msu") || n.ends_with(".pcm")) {
                return true;
            }
            // MD+/MSU-MD signature (extension beyond the
            // Python build, which never stages cue sets): a
            // Genesis-family game whose own folder holds a
            // .cue with the SAME stem as the ROM -- the
            // on-disk form every staged MD+/MSU-MD set takes
            // (Genesis Plus GX requires the cue to carry the
            // loaded ROM's basename).
            if GENESIS_FAMILY.contains(&game.system.as_str()) {
                let rom = p
                    .file_name()
                    .map(|f| f.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                return msu_cue_matches(&rom, &names);
            }
        }
    }
    false
}

/// Pure predicate behind the MD+/MSU-MD cue trigger: true when
/// `names` (a lowercased folder listing) holds a .cue whose
/// stem equals the ROM file's stem.
fn msu_cue_matches(rom_file_name: &str, names: &[String]) -> bool {
    let stem = Path::new(rom_file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if stem.is_empty() {
        return false;
    }
    names.iter().any(|n| {
        n.ends_with(".cue")
            && Path::new(n)
                .file_stem()
                .map(|s| s.to_string_lossy().to_lowercase())
                == Some(stem.clone())
    })
}

/// What this game's CD-quality-audio hack format is called:
/// Genesis gets MD+ (also known as MSU-MD); everything else
/// calls it MSU-1. Mirrors Python's `msu_term`.
fn msu_term(game: &Game) -> &'static str {
    if game.system == "Sega Genesis" {
        "MD+"
    } else {
        "MSU-1"
    }
}

/// Does a search-result title look like a CD-audio hack pack?
/// Mirrors Python's `_msu_title_ok`: anything mentioning MSU
/// counts; Genesis also accepts the MD+ spellings.
fn msu_title_ok(title: &str, genesis: bool) -> bool {
    let t = title.to_lowercase();
    if t.contains("msu") {
        return true;
    }
    genesis && (t.contains("md+") || t.contains("md plus") || t.contains("msu-md"))
}

/// A pack-bundled ROM only counts as the CD-audio version when
/// its name says so. Mirrors Python's `_msu_nameish`.
fn msu_nameish(path: &Path) -> bool {
    let n = path
        .file_name()
        .map(|f| f.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    n.contains("msu") || n.contains("md+")
}

/// The name a staged .pcm track gets next to a built ROM:
/// Python's rule -- a trailing track number becomes
/// "<rom stem>-<n>.pcm"; a numberless file keeps its own name.
fn pcm_staged_name(file_name: &str, rom_stem: &str) -> String {
    let lower = file_name.to_lowercase();
    if let Some(base) = lower.strip_suffix(".pcm") {
        let digits: String = base
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        if !digits.is_empty() {
            return format!("{rom_stem}-{digits}.pcm");
        }
    }
    file_name.to_string()
}

/// Systems that make up the Genesis family. the user's rule
/// (Oct 19 addendum): in RetroArch the Genesis Plus GX wide
/// core is the pick for the whole family -- Genesis ROMs, Sega
/// CD and MD+ ride it, and Master System / Game Gear ride the
/// same core.
const GENESIS_FAMILY: &[&str] = &[
    "Sega Genesis",
    "Sega CD",
    "Sega Master System",
    "Sega Game Gear",
];

/// Tie-breaks for the RetroArch default in pick_profile: when
/// a system has several RetroArch profiles, prefer the one
/// whose core path contains the fragment.
const RA_CORE_PREF: &[(&str, &str)] = &[
    // Beetle PSX HW over mednafen_psx / PCSX-ReARMed / SwanStation
    ("Sony PlayStation", "mednafen_psx_hw"),
    // Mupen64Plus-Next (GLideN64) over ParaLLEl N64
    ("Nintendo 64", "mupen64plus"),
    ("Nintendo Game Boy", "mgba"),
    ("Nintendo Game Boy Color", "mgba"),
    ("Nintendo Game Boy Advance", "mgba"),
];

/// Per-system best-pick fragments (subset of the Python table):
/// the fallback for systems with no RetroArch profile. Matching
/// is case-sensitive against profile name OR core, first hit in
/// config order -- exactly as the pre-rewrite code behaved.
fn best_pick_frags(system: &str) -> &'static [&'static str] {
    match system {
        "Nintendo Entertainment System" => &["Mesen"],
        "Super Nintendo" => &["SNES9x", "Snes9x"],
        "Sony PlayStation" => &["Beetle", "Mednafen", "DuckStation"],
        "Sega Genesis" | "Sega CD" | "Sega Master System" | "Sega Game Gear" => {
            &["Kega Fusion", "BlastEm"]
        }
        "Nintendo 64" => &["Project64", "simple64"],
        "Game Boy" | "Nintendo Game Boy" | "Nintendo Game Boy Color"
        | "Nintendo Game Boy Advance" => &["mGBA"],
        "Sony PlayStation Portable" => &["PPSSPP"],
        "Arcade" => &["MAME"],
        "Sega Saturn" => &["Mednafen", "SSF"],
        "Nintendo DS" => &["melonDS", "DeSmuME"],
        "Nintendo 3DS" => &["Azahar", "Citra"],
        "Nintendo Wii" => &["Dolphin"],
        "Commodore 64" => &["VICE"],
        "Commodore Amiga" => &["WinUAE", "FS-UAE"],
        "Atari 2600" => &["Stella"],
        "Neo Geo" | "Neo Geo CD" => &["FBNeo", "FinalBurn"],
        "3DO" => &["4DO", "Opera"],
        _ => &[],
    }
}

fn pick_profile<'a>(game: &Game, profiles: &'a [Profile]) -> Option<&'a Profile> {
    // 1. Hand-lock first (Python parity: emu_front.py
    //    default_emulator_for honors a pick only when
    //    emulator_locked is set). A bare `emulator` value is a
    //    stale auto-pick baked in by old scans and is ignored.
    if game.emulator_locked && !game.emulator.is_empty() {
        if let Some(p) = profiles.iter().find(|p| p.name == game.emulator) {
            return Some(p);
        }
    }
    // 2. MSU / MD+ enhanced-audio routing. Genesis-family games
    //    with .msu/.pcm tracks are MD+ titles: they go to the
    //    Genesis Plus GX core, never to the SNES bsnes profile
    //    the old system-blind check sent them to (bug F3).
    if game_has_msu(game) {
        // Python parity (default_emulator_for): match on
        // profile name + core text, not the core alone -- a
        // standalone bsnes/Genesis Plus GX profile with no
        // libretro core string must still win.
        if GENESIS_FAMILY.contains(&game.system.as_str()) {
            if let Some(p) = profiles.iter().find(|p| {
                p.core.to_lowercase().contains("genesis_plus")
                    || p.name.to_lowercase().contains("genesis plus")
            }) {
                return Some(p);
            }
            // Python's Genesis fallback after GX is BlastEm
            // (never Kega Fusion: no MD+ support).
            if let Some(p) = profiles
                .iter()
                .find(|p| p.name.to_lowercase().contains("blastem"))
            {
                return Some(p);
            }
        } else {
            if let Some(p) = profiles.iter().find(|p| {
                format!("{} {}", p.name.to_lowercase(), p.core.to_lowercase())
                    .contains("bsnes")
            }) {
                return Some(p);
            }
            if let Some(p) = profiles
                .iter()
                .find(|p| p.name.to_lowercase().contains("ares"))
            {
                return Some(p);
            }
        }
    }
    // 3. the user's exceptions (one explicit path; bug F4):
    //    PS2 -> PCSX2, GameCube/Wii -> Dolphin. Prefer the
    //    profile filed under the game's own system (Wii games
    //    get the Wii Dolphin profile, not GameCube's).
    if game.system == "Sony PlayStation 2" {
        if let Some(p) = profiles
            .iter()
            .find(|p| p.system == game.system && p.name.contains("PCSX2"))
            .or_else(|| profiles.iter().find(|p| p.name.contains("PCSX2")))
        {
            return Some(p);
        }
    }
    if game.system == "Nintendo GameCube" || game.system == "Nintendo Wii" {
        if let Some(p) = profiles
            .iter()
            .find(|p| p.system == game.system && p.name.contains("Dolphin"))
            .or_else(|| profiles.iter().find(|p| p.name.contains("Dolphin")))
        {
            return Some(p);
        }
    }
    // 4. RetroArch default. Genesis family first: the Genesis
    //    Plus GX core profile, wherever it is filed (Sega CD
    //    has no RA profile of its own; SMS/GG ride the same
    //    wide core). Other systems: their own RA profile(s),
    //    tie-broken by RA_CORE_PREF, else config order.
    if GENESIS_FAMILY.contains(&game.system.as_str()) {
        if let Some(p) = profiles.iter().find(|p| {
            !p.core.is_empty() && p.core.to_lowercase().contains("genesis_plus")
        }) {
            return Some(p);
        }
    }
    let ra: Vec<&Profile> = profiles
        .iter()
        .filter(|p| p.system == game.system && !p.core.is_empty())
        .collect();
    if !ra.is_empty() {
        if let Some((_, frag)) = RA_CORE_PREF
            .iter()
            .find(|(sys, _)| *sys == game.system.as_str())
        {
            if let Some(p) = ra
                .iter()
                .copied()
                .find(|p| p.core.to_lowercase().contains(frag))
            {
                return Some(p);
            }
        }
        return Some(ra[0]);
    }
    // 5. Fallback (unchanged): best-pick fragments, then the
    //    first profile filed under the game's system. Systems
    //    with no RetroArch profile at all (Wii U/Cemu, PS3/
    //    RPCS3, Xbox/xemu, ...) keep their standalone default.
    for f in best_pick_frags(&game.system) {
        if let Some(p) = profiles
            .iter()
            .find(|p| p.name.contains(f) || p.core.contains(f))
        {
            return Some(p);
        }
    }
    profiles.iter().find(|p| p.system == game.system)
}

fn engine_label(game: &Game, profiles: &[Profile]) -> String {
    match pick_profile(game, profiles) {
        Some(p) => engine_label_p(game, p),
        None => "No emulator profile".to_string(),
    }
}

fn engine_label_p(_game: &Game, p: &Profile) -> String {
    if !p.core.is_empty() {
        let core = Path::new(&p.core)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        format!("RetroArch core: {core}")
    } else {
        format!("Emulator: {}", p.name)
    }
}

/// PC Ports entries are native Windows ports (Ship of Harkinian,
/// Starship, …) installed by Quiver Launcher. They are not ROMs and
/// have no emulator profile: the entry's path IS the program, run
/// directly from its own folder.
fn is_port(game: &Game) -> bool {
    game.system == "PC Ports"
}

fn launch_direct(game: &Game) -> Result<String, String> {
    launch_direct_started(game).map(|(m, _)| m)
}

/// launch_direct, but the child handle comes back: the
/// GUI's launches (v1.07) keep it so update() can watch
/// the process exit and total up the session's play
/// time. The plain wrappers stay for the headless
/// probes (--launch-one and friends exit at once by
/// design and must never write play records).
fn launch_direct_started(
    game: &Game,
) -> Result<(String, std::process::Child), String> {
    let exe = Path::new(&game.path);
    let dir = exe
        .parent()
        .ok_or_else(|| "port entry has no folder".to_string())?;
    Command::new(exe)
        .current_dir(dir)
        .spawn()
        .map(|c| (format!("Launched {}", game.name), c))
        .map_err(|e| format!("launch failed: {e}"))
}

fn launch(game: &Game, profiles: &[Profile]) -> Result<String, String> {
    launch_started(game, profiles).map(|(m, _)| m)
}

/// launch, returning the child handle (see
/// launch_direct_started for why both forms exist).
fn launch_started(
    game: &Game,
    profiles: &[Profile],
) -> Result<(String, std::process::Child), String> {
    if is_port(game) {
        return launch_direct_started(game);
    }
    let p = pick_profile(game, profiles)
        .ok_or_else(|| "no emulator profile for this system".to_string())?;
    launch_with_started(game, p)
}

/// Split a profile args string into arguments the way a
/// Windows command line is read: whitespace separates, and a
/// double-quoted span stays together (the quotes themselves are
/// not part of the argument).
fn split_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut has = false;
    for c in s.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                has = true;
            }
            c if c.is_whitespace() && !quoted => {
                if has {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
            }
            c => {
                cur.push(c);
                has = true;
            }
        }
    }
    if has {
        out.push(cur);
    }
    out
}

fn launch_with(game: &Game, p: &Profile) -> Result<String, String> {
    launch_with_started(game, p).map(|(m, _)| m)
}

/// launch_with, returning the child handle (see
/// launch_direct_started for why both forms exist).
fn launch_with_started(
    game: &Game,
    p: &Profile,
) -> Result<(String, std::process::Child), String> {
    // .m3u (multi-disc) resolves to disc 1 for standalones —
    // resolved BEFORE the command is built, like Python's
    // launch sites do, so a {rom} template gets the disc file.
    let mut rom = game.path.clone();
    if rom.to_lowercase().ends_with(".m3u") && p.core.is_empty() {
        if let Ok(text) = std::fs::read_to_string(&rom) {
            if let Some(first) = text.lines().map(|l| l.trim()).find(|l| !l.is_empty()) {
                let dir = Path::new(&rom).parent().unwrap_or(Path::new(""));
                rom = dir.join(first).to_string_lossy().to_string();
            }
        }
    }
    let mut cmd = Command::new(&p.path);
    // Python parity: the emulator runs from its own folder.
    if let Some(dir) = Path::new(&p.path).parent() {
        if !dir.as_os_str().is_empty() {
            cmd.current_dir(dir);
        }
    }
    if !p.core.is_empty() {
        cmd.arg("-L").arg(&p.core);
    }
    // Profile args are a TEMPLATE: Python's build_launch_command
    // substitutes {rom} with the game file (Dolphin
    // '-e "{rom}"', Cemu '-g "{rom}"', xemu
    // '-dvd_path "{rom}"' in the seeded config). Passing the
    // template through literally hands the emulator a file
    // named {rom} — Dolphin's 'The specified file "{rom}" does
    // not exist'. When the template names the ROM, the ROM is
    // NOT appended again (Python's rule).
    let mut rom_named = false;
    if !p.args.trim().is_empty() {
        rom_named = p.args.contains("{rom}");
        let expanded = p.args.replace("{rom}", &rom);
        for a in split_args(&expanded) {
            cmd.arg(a);
        }
    }
    // per-game cheats on RetroArch launches
    if game.cheats_enabled
        && p.path.to_lowercase().contains("retroarch")
    {
        if let Some(cht) = write_cht(game) {
            let dir = data_dir().join("Cheats");
            let cfgp = dir.join("emufront-append.cfg");
            let text = format!(
                "cheat_file_path = \"{}\"\napply_cheats_after_load = \"true\"\n",
                cht.to_string_lossy().replace('\\', "/")
            );
            if std::fs::write(&cfgp, text).is_ok() {
                cmd.arg("--appendconfig")
                    .arg(cfgp.to_string_lossy().as_ref());
            }
        }
    }
    if !rom_named {
        cmd.arg(&rom);
    }
    cmd.spawn()
        .map(|c| (format!("Launched {} via {}", game.name, p.name), c))
        .map_err(|e| format!("launch failed: {e}"))
}

// ---------------- Emulators tab (standalone starts) ----------------
// the user (Oct 24): "add a tab that links to the ps2 emulator
// and the dolphin emulator … if I click on it it will run
// executable" — DuckStation added the same hour, plus his
// fallback rule: "if it can't find the executable it allow
// me to select the executable files that correspond to
// that program". Each entry starts the emulator PROGRAM
// itself (no game loaded, no core, no args). Paths resolve
// from the standalone profiles in the shared config —
// never hardcoded, so moving an install only takes a
// profile edit (or a fresh pick in the tab).
//
// v1.04 (feature #15): a fourth row, AmigaVision, is the
// one deliberate exception to "no args" — it starts
// Amiberry WITH the registered "AmigaVision (AGS)"
// entry's .uae config as the argument, because booting
// Amiberry bare was never the ask (the user: "add
// AmigaVision to the emulators page"). The config path
// is read live from the library, never copied into
// Rust storage; the tab still bundles/copies/modifies
// nothing.

/// One Emulators-tab entry, as data: display text plus how
/// to find the program among the profiles. A fourth
/// emulator later is one more row in EMU_TAB_SPECS.
struct EmuTabSpec {
    /// Key for the manual-override map and messages.
    key: &'static str,
    label: &'static str,
    blurb: &'static str,
    /// Fragment the standalone profile's NAME must carry —
    /// the same matching pick_profile uses for these picks.
    frag: &'static str,
    /// Profile systems to prefer, in order (Dolphin serves
    /// two systems; its GameCube profile wins).
    systems: &'static [&'static str],
    /// For a row that launches content rather than the
    /// bare program: the registered library game's name
    /// whose path becomes the launch argument, expanded
    /// through the profile's args template (the
    /// AmigaVision row — its path IS the .uae config).
    /// None = the bare program, no argument.
    arg_game: Option<&'static str>,
}

const EMU_TAB_SPECS: &[EmuTabSpec] = &[
    EmuTabSpec {
        key: "PCSX2",
        label: "PCSX2",
        blurb: "PlayStation 2 emulator",
        frag: "PCSX2",
        systems: &["Sony PlayStation 2"],
        arg_game: None,
    },
    EmuTabSpec {
        key: "Dolphin",
        label: "Dolphin",
        blurb: "GameCube / Wii emulator",
        frag: "Dolphin",
        systems: &["Nintendo GameCube", "Nintendo Wii"],
        arg_game: None,
    },
    EmuTabSpec {
        key: "DuckStation",
        label: "DuckStation",
        blurb: "PlayStation emulator",
        frag: "DuckStation",
        systems: &["Sony PlayStation"],
        arg_game: None,
    },
    EmuTabSpec {
        key: "AmigaVision",
        label: "AmigaVision",
        blurb: "Amiga games & demos (AGS)",
        frag: "Amiberry",
        systems: &["Commodore Amiga"],
        arg_game: Some("AmigaVision (AGS)"),
    },
];

/// The standalone profiles that could BE this emulator,
/// best first: profiles whose name carries the fragment
/// AND whose core is empty. A profile WITH a core points
/// at RetroArch itself — starting that path bare would
/// open RetroArch, not the emulator the user asked for.
/// Preferred systems come first (in spec order), then
/// any remaining standalone matches in config order;
/// duplicate paths (one Dolphin filed under both
/// systems) collapse. emu_tab_profile_paths is built on
/// this; the launch-argument logic reads the resolved
/// profile's args template from here.
fn emu_tab_profiles_ordered<'a>(
    profiles: &'a [Profile],
    spec: &EmuTabSpec,
) -> Vec<&'a Profile> {
    let cands: Vec<&Profile> = profiles
        .iter()
        .filter(|p| {
            p.core.is_empty() && !p.path.trim().is_empty() && p.name.contains(spec.frag)
        })
        .collect();
    let mut out: Vec<&Profile> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    for sys in spec.systems {
        for p in cands.iter().filter(|p| p.system == *sys) {
            if seen.insert(p.path.as_str()) {
                out.push(p);
            }
        }
    }
    for p in &cands {
        if seen.insert(p.path.as_str()) {
            out.push(p);
        }
    }
    out
}

/// The profile paths that could BE this emulator, best
/// first (see emu_tab_profiles_ordered).
fn emu_tab_profile_paths(profiles: &[Profile], spec: &EmuTabSpec) -> Vec<String> {
    emu_tab_profiles_ordered(profiles, spec)
        .into_iter()
        .map(|p| p.path.clone())
        .collect()
}

/// What the tab resolved for one entry.
#[derive(Clone, PartialEq, Debug)]
enum EmuTabState {
    /// Runnable: the exe Run will start. `manual` = the
    /// path is the user's own pick (it wins over profiles).
    Ready { path: String, manual: bool },
    /// Not runnable: the entry offers "Select executable…"
    /// instead of a dead Run button. `stale` names a
    /// configured path (his pick first, else the profile's)
    /// whose file is gone, so he can see what broke.
    Missing { stale: Option<String> },
    /// The exe resolved, but this row launches content
    /// that isn't there — the reason names what's
    /// missing. No Run and no Select: registering the
    /// content is a library action, not a tab action.
    Blocked { reason: String },
}

/// Resolve one entry: the user's saved pick first, then the
/// standalone profile paths in preference order; a
/// candidate whose file is gone is skipped (and the first
/// such is remembered as `stale`) so a deleted or moved
/// install falls back to the Select state rather than a
/// Run button that fails. `exists` is injected so
/// --selftest can pin the whole precedence table.
fn resolve_emu_tab(
    spec: &EmuTabSpec,
    profiles: &[Profile],
    manual: Option<&str>,
    exists: &dyn Fn(&str) -> bool,
) -> EmuTabState {
    let mut stale: Option<String> = None;
    if let Some(m) = manual {
        if !m.trim().is_empty() {
            if exists(m) {
                return EmuTabState::Ready {
                    path: m.to_string(),
                    manual: true,
                };
            }
            stale = Some(m.to_string());
        }
    }
    for p in emu_tab_profile_paths(profiles, spec) {
        if exists(&p) {
            return EmuTabState::Ready {
                path: p,
                manual: false,
            };
        }
        if stale.is_none() {
            stale = Some(p);
        }
    }
    EmuTabState::Missing { stale }
}

/// Expand a profile args template around a file path —
/// launch_with's own rule: {rom} is substituted in, and
/// a template that never names the file gets it appended
/// bare; an empty template is just the bare path. Pure.
fn emu_tab_expand_args(template: &str, path: &str) -> Vec<String> {
    if template.trim().is_empty() {
        return vec![path.to_string()];
    }
    let mut out = split_args(&template.replace("{rom}", path));
    if !template.contains("{rom}") {
        out.push(path.to_string());
    }
    out
}

/// Full row resolution for the tab: the exe state
/// (resolve_emu_tab, unchanged) plus, for a row with an
/// arg_game, the launch argument built from the library.
/// Returns the state and the argv tail Run will pass
/// (empty unless the state is Ready). Pure — --selftest
/// pins the AmigaVision row end to end (resolution,
/// override precedence, template expansion, and the
/// Blocked state when the library entry is gone).
fn resolve_emu_tab_row(
    spec: &EmuTabSpec,
    profiles: &[Profile],
    games: &[Game],
    manual: Option<&str>,
    exists: &dyn Fn(&str) -> bool,
) -> (EmuTabState, Vec<String>) {
    let state = resolve_emu_tab(spec, profiles, manual, exists);
    let EmuTabState::Ready { path, .. } = &state else {
        // No exe: the Missing/select flow stands exactly
        // as shipped, whatever the row's content wants.
        return (state, Vec::new());
    };
    let Some(want) = spec.arg_game else {
        return (state, Vec::new());
    };
    let Some(game) = games.iter().find(|g| g.name == want) else {
        return (
            EmuTabState::Blocked {
                reason: format!(
                    "\"{want}\" isn't in the library — this entry starts {} \
                     with that entry's config file, so there is nothing to load.",
                    spec.label
                ),
            },
            Vec::new(),
        );
    };
    // The args template comes from the profile the exe
    // resolved from. A manual pick usually has no
    // profile: the bare path is the argument then (the
    // form the AmigaVision install already proved
    // Amiberry accepts as its file argument). If the
    // manual path happens to BE a candidate profile's
    // path, that profile's template applies.
    let profile = emu_tab_profiles_ordered(profiles, spec)
        .into_iter()
        .find(|p| &p.path == path);
    let template = profile.map(|p| p.args.as_str()).unwrap_or("");
    (state, emu_tab_expand_args(template, &game.path))
}

/// Is this a file the user may pick as an emulator
/// executable? It must exist and be an .exe — the file
/// picker's own filter, re-checked here so a typed path
/// or a same-named text file can't sneak in as a program.
fn emu_pick_ok(path: &str, exists: &dyn Fn(&str) -> bool) -> bool {
    !path.trim().is_empty() && path.to_lowercase().ends_with(".exe") && exists(path)
}

/// Start the emulator program itself: the launch path's
/// own shape (spawn from the exe's folder, so portable
/// installs find their config; spawn() returns at once —
/// the app never waits on the emulator). `args` is the
/// row's launch argument — empty for the bare-program
/// rows; the AmigaVision row passes its config path,
/// expanded through the profile template by
/// resolve_emu_tab_row.
fn run_emulator_exe(label: &str, path: &str, args: &[String]) -> Result<String, String> {
    let exe = Path::new(path);
    let mut cmd = Command::new(exe);
    if let Some(dir) = exe.parent() {
        if !dir.as_os_str().is_empty() {
            cmd.current_dir(dir);
        }
    }
    for a in args {
        cmd.arg(a);
    }
    cmd.spawn()
        .map(|_| format!("▶ {label} started — its window should open on this PC."))
        .map_err(|e| format!("⚠ Couldn't start {label} ({path}): {e}"))
}

// ---------------- RetroArch channel (Nightly / Stable) ----------------
// the user's rule: RetroArch stays on the LATEST build of the chosen
// channel at all times. EmuFront Rust asks at startup (picker),
// then checks buildbot and updates the RetroArch install in place when
// a newer build exists. Its own settings live beside the exe —
// the shared emufront_config.json is never touched by this.

const RA_DIR: &str = r"C:\RetroArch";
const SEVEN_ZIP: &str = r"C:\Program Files\7-Zip\7z.exe";

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn settings_path() -> PathBuf {
    exe_dir().join("emufront_rust.json")
}

/// One emulator's update-check result (the user's ask, Oct 20): the
/// version installed on this PC vs the latest official release.
#[derive(Clone, Default)]
struct EmuUpdateInfo {
    installed: String,
    latest: String,
    /// "update" | "current" | "unknown" | "manual" | "info"
    status: String,
    url: String,
}

#[derive(Clone)]
struct Settings {
    ra_channel: String, // "nightly" | "stable"
    ra_ask: bool,       // show the picker at startup
    /// Music player volume 0.0..=1.0 (design sec. 4.2).
    music_volume: f32,
    /// SoundFont override: any .sf2 instead of the provisioned
    /// GeneralUser GS. Empty = default lookup.
    soundfont_path: String,
    /// Emulator update-check cache (Rust-side settings only — never
    /// the shared config).
    emu_updates: BTreeMap<String, EmuUpdateInfo>,
    emu_updates_at: u64, // epoch seconds of the last check
    /// Emulators-tab manual executable picks (the user, Oct 24):
    /// tab spec key -> the .exe he chose in the file picker
    /// when profile resolution couldn't find the emulator.
    /// Rust-side settings only — the shared config's schema
    /// is Python's and never gains Rust-only fields.
    emu_overrides: BTreeMap<String, String>,
    /// Decoded-cover memory budget preset (perf Slice 1,
    /// RELEASE-PERF-DESIGN §2.3): "low" | "standard" |
    /// "high" — the texture LRU's byte cap. The settings
    /// FILE key works now; the in-app control arrives with
    /// Slice 2's cache-controls UI (§2.7).
    cover_memory: String,
    /// Low-spec machine mode (OLD-SYSTEMS study §3): the
    /// resolved machine profile reads this flag. Written
    /// two ways only — the first-run detection's default
    /// (new installs only) and the user's explicit
    /// toggle in ⚙ Cache & Performance.
    low_spec: bool,
    /// True once the user has touched the low-spec toggle
    /// themselves. Recorded so the UI can say whose choice
    /// the current state is; detection never runs again
    /// either way (it is first-run only, keyed on the
    /// settings file not existing).
    low_spec_user_set: bool,
    /// First-run probe results, cached forever (study
    /// §3): total RAM in GB and the boot drive's media
    /// type ("ssd" | "hdd" | "" when unknown). Detection
    /// runs once ever — when no settings file exists —
    /// so an existing install's values here are history,
    /// never a live re-flip.
    detected_ram_gb: Option<f64>,
    detected_disk: String,
    /// Discord Rich Presence (v1.07, feature #3): while
    /// a tracked game runs, show it in the Discord
    /// status. The toggle is the plan's Settings switch
    /// (default on). The application ID is NOT a secret
    /// — it is the public identifier of a Discord
    /// developer-portal app — and it lives here, never
    /// hard-coded, so it can be the user's own registration
    /// today and the project's at public release with no
    /// code change. Empty ID = presence off whatever the
    /// toggle says (there is nothing to connect as), at
    /// zero cost.
    discord_presence: bool,
    discord_app_id: String,
}

/// The texture LRU's byte budget for a cover-memory
/// preset: Low 128 MB for 8 GB / integrated-GPU machines,
/// Standard 384 MB (the default — ~1,400 covers), High
/// 1 GB. An unrecognized value is Standard. Pure;
/// --selftest pins the table.
fn cover_budget_bytes(setting: &str) -> usize {
    match setting {
        "low" => 128 * 1024 * 1024,
        "high" => 1024 * 1024 * 1024,
        _ => 384 * 1024 * 1024,
    }
}

fn settings_blank() -> Settings {
    Settings {
        ra_channel: "nightly".into(),
        ra_ask: true,
        music_volume: 0.8,
        soundfont_path: String::new(),
        emu_updates: BTreeMap::new(),
        emu_updates_at: 0,
        emu_overrides: BTreeMap::new(),
        cover_memory: "standard".to_string(),
        low_spec: false,
        low_spec_user_set: false,
        detected_ram_gb: None,
        detected_disk: String::new(),
        discord_presence: true,
        discord_app_id: String::new(),
    }
}

/// The settings file's JSON -> Settings. Pure (the file
/// read lives in load_settings) so --selftest can pin the
/// round-trip, including that a pre-v1.05 file — none of
/// the new keys — reads as their defaults.
fn settings_from_json(v: &serde_json::Value) -> Settings {
    let mut emu_updates = BTreeMap::new();
    if let Some(obj) = v.get("emu_updates").and_then(|x| x.as_object()) {
        for (name, arr) in obj {
            if let Some(a) = arr.as_array() {
                let g = |i: usize| {
                    a.get(i).and_then(|x| x.as_str()).unwrap_or("").to_string()
                };
                emu_updates.insert(
                    name.clone(),
                    EmuUpdateInfo {
                        installed: g(0),
                        latest: g(1),
                        status: g(2),
                        url: g(3),
                    },
                );
            }
        }
    }
    Settings {
        ra_channel: v
            .get("ra_channel")
            .and_then(|x| x.as_str())
            .unwrap_or("nightly")
            .to_string(),
        ra_ask: v
            .get("ra_ask")
            .and_then(|x| x.as_bool())
            .unwrap_or(true),
        music_volume: v
            .get("music_volume")
            .and_then(|x| x.as_f64())
            .map(|f| f as f32)
            .unwrap_or(0.8)
            .clamp(0.0, 1.0),
        soundfont_path: v
            .get("soundfont_path")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        emu_updates,
        emu_updates_at: v
            .get("emu_updates_at")
            .and_then(|x| x.as_u64())
            .unwrap_or(0),
        emu_overrides: parse_emu_overrides(v),
        cover_memory: v
            .get("cover_memory")
            .and_then(|x| x.as_str())
            .unwrap_or("standard")
            .to_string(),
        low_spec: v
            .get("low_spec")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        low_spec_user_set: v
            .get("low_spec_user_set")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        detected_ram_gb: v
            .get("detected_ram_gb")
            .and_then(|x| x.as_f64()),
        detected_disk: v
            .get("detected_disk")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        discord_presence: v
            .get("discord_presence")
            .and_then(|x| x.as_bool())
            .unwrap_or(true),
        discord_app_id: v
            .get("discord_app_id")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .to_string(),
    }
}

/// Settings -> the settings file's JSON. Pure; the
/// round-trip partner of settings_from_json.
fn settings_to_json(s: &Settings) -> serde_json::Value {
    let mut updates = serde_json::Map::new();
    for (name, u) in &s.emu_updates {
        updates.insert(
            name.clone(),
            serde_json::json!([u.installed, u.latest, u.status, u.url]),
        );
    }
    serde_json::json!({
        "ra_channel": s.ra_channel,
        "ra_ask": s.ra_ask,
        "music_volume": s.music_volume,
        "soundfont_path": s.soundfont_path,
        "emu_updates": updates,
        "emu_updates_at": s.emu_updates_at,
        "emu_overrides": emu_overrides_json(&s.emu_overrides),
        "cover_memory": s.cover_memory,
        "low_spec": s.low_spec,
        "low_spec_user_set": s.low_spec_user_set,
        "detected_ram_gb": s.detected_ram_gb,
        "detected_disk": s.detected_disk,
        "discord_presence": s.discord_presence,
        "discord_app_id": s.discord_app_id,
    })
}

fn load_settings() -> Settings {
    if let Ok(text) = std::fs::read_to_string(settings_path()) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            return settings_from_json(&v);
        }
    }
    settings_blank()
}

fn save_settings(s: &Settings) {
    let _ = std::fs::write(settings_path(), settings_to_json(s).to_string());
}

/// The Discord application ID presence would run under,
/// if any: it needs BOTH the toggle on and a non-empty
/// ID in settings. Pure — --selftest pins the table.
fn presence_app_id(s: &Settings) -> Option<&str> {
    let id = s.discord_app_id.trim();
    if s.discord_presence && !id.is_empty() {
        Some(id)
    } else {
        None
    }
}

/// The settings-file shape of the Emulators tab's manual
/// picks: a plain JSON object { tab key: exe path }. Blank
/// paths are dropped on the way out — an empty pick is no
/// pick. Pure; --selftest pins the round-trip.
fn emu_overrides_json(m: &BTreeMap<String, String>) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    for (k, p) in m {
        if !p.trim().is_empty() {
            obj.insert(k.clone(), serde_json::Value::String(p.clone()));
        }
    }
    serde_json::Value::Object(obj)
}

fn parse_emu_overrides(v: &serde_json::Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Some(obj) = v.get("emu_overrides").and_then(|x| x.as_object()) {
        for (k, p) in obj {
            if let Some(p) = p.as_str() {
                if !p.trim().is_empty() {
                    out.insert(k.clone(), p.to_string());
                }
            }
        }
    }
    out
}


// ---------------- Rust sidecar store (v1.07) -------------------
// emufront_rust_data.json, beside the exe next to
// emufront_rust.json: the Rust-only feature store the
// Oct 19 roadmap prep (§0) decided on. The shared Python
// config NEVER gains feature fields — Python's persist()
// rebuilds games[] from a fixed 7-field schema and would
// silently destroy them. Schema v1 holds the play map
// and the first-seen stamps; later batches (favorites,
// ratings, journal…) add their keys under this same
// version discipline. Entries are keyed by game path —
// the app's universal identity. An entry whose path is
// no longer in the library is hidden by every UI and
// never deleted: the data is the user's, and pruning is
// a manual act, never a side effect.
//
// Write discipline: debounced, serialized on the
// caller's thread and written on a worker (tmp +
// rename, the snapshot pattern), with the previous good
// file kept as .bak so a torn write can never lose the
// store. A synchronous final flush runs on window
// close / drop.

const RUST_DATA_VERSION: u32 = 1;

fn rust_data_path() -> PathBuf {
    exe_dir().join("emufront_rust_data.json")
}

fn rust_data_bak_path() -> PathBuf {
    exe_dir().join("emufront_rust_data.json.bak")
}

/// One game's play record: `last` = when the most
/// recent session ended (epoch secs), `secs` = total
/// counted play time, `sessions` = how many sessions
/// counted (see PLAY_MIN_SECS for what counts).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct PlayRec {
    last: u64,
    secs: u64,
    sessions: u64,
}

/// The sidecar in memory. `dirty`/`dirty_since` are the
/// debounce bookkeeping and `frozen` the newer-schema
/// guard; none of the three is serialized.
#[derive(Clone, Default)]
struct RustData {
    play: BTreeMap<String, PlayRec>,
    first_seen: BTreeMap<String, u64>,
    first_seen_baseline: u64,
    dirty: bool,
    dirty_since: u64,
    frozen: bool,
}

impl RustData {
    fn mark_dirty(&mut self, now: u64) {
        if !self.dirty {
            self.dirty = true;
            self.dirty_since = now;
        }
    }

    /// Observe the library's current paths (the startup
    /// load and every reload call this — never a frame).
    /// Day one (no baseline yet): stamp every present
    /// path with `now` and set the baseline — the whole
    /// existing library is "old" by definition, so New
    /// This Week starts honest (empty). Later: stamp
    /// only paths never seen before, at the moment they
    /// appear. An empty library never sets the baseline:
    /// a failed load must not make the real library
    /// "new" when it finally arrives. Returns whether
    /// anything changed.
    fn observe_first_seen<'a>(
        &mut self,
        paths: impl Iterator<Item = &'a str>,
        now: u64,
    ) -> bool {
        if self.first_seen_baseline == 0 {
            let mut any = false;
            for p in paths {
                self.first_seen.entry(p.to_string()).or_insert(now);
                any = true;
            }
            if any {
                self.first_seen_baseline = now;
                self.mark_dirty(now);
            }
            return any;
        }
        let mut changed = false;
        for p in paths {
            if !self.first_seen.contains_key(p) {
                self.first_seen.insert(p.to_string(), now);
                changed = true;
            }
        }
        if changed {
            self.mark_dirty(now);
        }
        changed
    }
}

/// The sidecar file's JSON -> RustData. Lenient like
/// the settings walk (missing/wrong-typed pieces read
/// as defaults) — a hand-edited or partially written
/// file degrades to what parses, never to a panic.
/// A file stamped with a NEWER schema version is read
/// for what v1 understands but marked frozen: a
/// downgrade must never save over a newer store.
/// Pure — --selftest pins the round-trip + the guard.
fn rust_data_from_json(v: &serde_json::Value) -> RustData {
    let mut d = RustData::default();
    if v.get("version").and_then(|x| x.as_u64()).unwrap_or(0)
        > RUST_DATA_VERSION as u64
    {
        d.frozen = true;
    }
    if let Some(obj) = v.get("play").and_then(|x| x.as_object()) {
        for (path, rec) in obj {
            let g = |k: &str| {
                rec.get(k).and_then(|x| x.as_u64()).unwrap_or(0)
            };
            d.play.insert(
                path.clone(),
                PlayRec {
                    last: g("last"),
                    secs: g("secs"),
                    sessions: g("sessions"),
                },
            );
        }
    }
    if let Some(obj) = v.get("first_seen").and_then(|x| x.as_object()) {
        for (path, ts) in obj {
            if let Some(ts) = ts.as_u64() {
                d.first_seen.insert(path.clone(), ts);
            }
        }
    }
    d.first_seen_baseline = v
        .get("first_seen_baseline")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    d
}

/// RustData -> the sidecar file's JSON. Pure; the
/// round-trip partner of rust_data_from_json.
fn rust_data_to_json(d: &RustData) -> serde_json::Value {
    let mut play = serde_json::Map::new();
    for (path, r) in &d.play {
        play.insert(
            path.clone(),
            serde_json::json!({
                "last": r.last,
                "secs": r.secs,
                "sessions": r.sessions,
            }),
        );
    }
    let mut seen = serde_json::Map::new();
    for (path, ts) in &d.first_seen {
        seen.insert(path.clone(), serde_json::Value::from(*ts));
    }
    serde_json::json!({
        "version": RUST_DATA_VERSION,
        "play": play,
        "first_seen": seen,
        "first_seen_baseline": d.first_seen_baseline,
    })
}

fn load_rust_data() -> RustData {
    for p in [rust_data_path(), rust_data_bak_path()] {
        if let Ok(text) = std::fs::read_to_string(&p) {
            if let Ok(v) =
                serde_json::from_str::<serde_json::Value>(&text)
            {
                return rust_data_from_json(&v);
            }
        }
    }
    RustData::default()
}

/// Serializes saves so two writers can never interleave
/// the shared tmp file.
static RUST_DATA_WRITE_LOCK: std::sync::Mutex<()> =
    std::sync::Mutex::new(());

/// The actual write: tmp + rename, with the outgoing
/// file first copied to .bak (the previous good store
/// survives a torn write). Callers serialize the text.
fn write_rust_data(text: &str) {
    let _guard = RUST_DATA_WRITE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let path = rust_data_path();
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, text).is_ok() {
        if path.exists() {
            let _ = std::fs::copy(&path, rust_data_bak_path());
        }
        let _ = std::fs::rename(&tmp, &path);
    }
}

/// Debounced flush: serialize now (the store is small
/// next to the library), write on a thread.
fn spawn_data_save(d: &RustData) {
    if d.frozen {
        return;
    }
    let text = rust_data_to_json(d).to_string();
    std::thread::spawn(move || write_rust_data(&text));
}

/// Final flush (window close / drop): the same write,
/// synchronous — the process is ending.
fn save_rust_data_now(d: &RustData) {
    if d.frozen {
        return;
    }
    write_rust_data(&rust_data_to_json(d).to_string());
}

/// Sessions shorter than this still move last-played
/// but add no seconds and no session count: some
/// standalones hand off to a child process and exit at
/// once, and a launcher stub's instant exit shouldn't
/// mint a "session" (roadmap §1.3 — the number stays
/// honest rather than flattering).
const PLAY_MIN_SECS: u64 = 5;

/// Fold one finished session into the play map. `last`
/// moves to the session's end; time and the session
/// count accrue only past PLAY_MIN_SECS. Pure.
fn fold_session(d: &mut RustData, path: &str, ended: u64, secs: u64) {
    let e = d.play.entry(path.to_string()).or_default();
    e.last = e.last.max(ended);
    if secs >= PLAY_MIN_SECS {
        e.secs += secs;
        e.sessions += 1;
    }
    d.mark_dirty(ended);
}

const NEW_WEEK_SECS: u64 = 7 * 24 * 3600;

/// New This Week membership (feature #11): first seen
/// strictly after the baseline (the day-one library is
/// never "new") and within the last 7 days. A stamp in
/// the future (the clock stepped back) counts as new
/// rather than vanishing — the baseline is the honesty
/// guard, not the clock. Pure; --selftest pins the
/// edges.
fn is_new_this_week(first_seen: u64, baseline: u64, now: u64) -> bool {
    baseline > 0
        && first_seen > baseline
        && (first_seen >= now || now - first_seen <= NEW_WEEK_SECS)
}

/// Indices into `games` for the Continue Playing wall /
/// strip: games with a play record, most recently
/// played first; ties break on total time, then name,
/// then index — a total order, so the strip never
/// reshuffles between frames. The search matcher is the
/// walls' own. Pure.
fn continue_indices(
    games: &[Game],
    play: &BTreeMap<String, PlayRec>,
    lower_names: &[String],
    q: &str,
) -> Vec<usize> {
    let mut idx: Vec<usize> = games
        .iter()
        .enumerate()
        .filter(|(i, g)| {
            play.contains_key(&g.path)
                && wall_game_matches(&lower_names[*i], q)
        })
        .map(|(i, _)| i)
        .collect();
    idx.sort_by(|&a, &b| {
        let pa = &play[&games[a].path];
        let pb = &play[&games[b].path];
        pb.last
            .cmp(&pa.last)
            .then_with(|| pb.secs.cmp(&pa.secs))
            .then_with(|| games[a].name.cmp(&games[b].name))
            .then_with(|| a.cmp(&b))
    });
    idx
}

/// Indices into `games` for the New This Week wall /
/// strip: newest first-seen first, name then index as
/// the tiebreaks. Pure.
fn new_indices(
    games: &[Game],
    d: &RustData,
    lower_names: &[String],
    q: &str,
    now: u64,
) -> Vec<usize> {
    let mut idx: Vec<usize> = games
        .iter()
        .enumerate()
        .filter(|(i, g)| {
            d.first_seen
                .get(&g.path)
                .map(|&ts| {
                    is_new_this_week(ts, d.first_seen_baseline, now)
                })
                .unwrap_or(false)
                && wall_game_matches(&lower_names[*i], q)
        })
        .map(|(i, _)| i)
        .collect();
    idx.sort_by(|&a, &b| {
        let ta = d.first_seen[&games[a].path];
        let tb = d.first_seen[&games[b].path];
        tb.cmp(&ta)
            .then_with(|| games[a].name.cmp(&games[b].name))
            .then_with(|| a.cmp(&b))
    });
    idx
}

/// Playtime in the strip/panel shape: "3h 12m" once
/// there's an hour, "12m" under it, "45s" under a
/// minute. Pure.
fn fmt_playtime(secs: u64) -> String {
    if secs >= 3600 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m", secs / 60)
    } else {
        format!("{secs}s")
    }
}

/// "Oct 12" for an epoch second, UTC (the app keeps no
/// local-time source; a session ending near midnight
/// can read as the adjacent day — a known cosmetic
/// limit). Days→civil is Howard Hinnant's public-domain
/// algorithm. Pure; --selftest pins known dates.
fn fmt_day(epoch: u64) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug",
        "Sep", "Oct", "Nov", "Dec",
    ];
    let days = (epoch / 86_400) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    format!("{} {}", MONTHS[(month - 1) as usize], day)
}

// ---------------- Discord presence (v1.07, feature #3) ---------
// While a tracked game runs, the Discord status shows
// it: "Playing <game>" + the system, elapsed counting
// from the launch. Local IPC only — the
// discord-rich-presence crate talks to the Discord
// client's named pipe: no tokens, no account, no
// network. ALL of it runs on one worker thread fed by
// an mpsc channel: the UI thread only sends small
// messages, so presence costs nothing while Discord is
// absent (a connect attempt per launch, failing
// silently) and can never error the app. The worker is
// spawned only when presence is configured (toggle on +
// an app ID — presence_app_id); with no ID there is no
// worker at all.

enum PresenceMsg {
    Set {
        name: String,
        system: String,
        start_ms: i64,
    },
    Clear,
}

/// One live tracked play session: the launched process,
/// retained so update() can watch it exit. Elapsed time
/// is measured on the monotonic clock.
struct NowPlaying {
    path: String,
    started: Instant,
    child: std::process::Child,
}

fn spawn_presence_worker(app_id: String) -> Sender<PresenceMsg> {
    use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
    let (tx, rx) = channel::<PresenceMsg>();
    std::thread::spawn(move || {
        let mut client: Option<DiscordIpcClient> = None;
        while let Ok(msg) = rx.recv() {
            // A failed IPC op drops the client (after the
            // borrow ends); the next Set reconnects.
            let mut dead = false;
            match msg {
                PresenceMsg::Set {
                    name,
                    system,
                    start_ms,
                } => {
                    if client.is_none() {
                        let mut c = DiscordIpcClient::new(&app_id);
                        if c.connect().is_ok() {
                            client = Some(c);
                        }
                    }
                    if let Some(c) = client.as_mut() {
                        let act = activity::Activity::new()
                            .details(&name)
                            .state(&system)
                            .timestamps(
                                activity::Timestamps::new()
                                    .start(start_ms),
                            );
                        if c.set_activity(act).is_err() {
                            dead = true;
                        }
                    }
                }
                PresenceMsg::Clear => {
                    if let Some(c) = client.as_mut() {
                        if c.clear_activity().is_err() {
                            dead = true;
                        }
                    }
                }
            }
            if dead {
                if let Some(mut c) = client.take() {
                    let _ = c.close();
                }
            }
        }
        // Every sender is gone (the app is exiting, or
        // the settings changed and a new worker replaced
        // this one): leave no stale status behind.
        if let Some(mut c) = client {
            let _ = c.clear_activity();
            let _ = c.close();
        }
    });
    tx
}

// ---------------- machine profile & first-run detection -------------
// The resolved performance posture of this install
// (OLD-SYSTEMS study §3): every perf lever in one place,
// computed from the settings by a pure function so the
// whole table is --selftest-pinnable. Low-spec mode
// changes how fast pixels arrive — never what the app
// decides: launch resolution, search results, config
// discipline, original art and music quality are the
// same in every profile.

struct MachineProfile {
    low_spec: bool,
    /// Texture LRU budget in bytes.
    cover_budget: usize,
    /// Cover rendition bounding box (w, h).
    rendition: (u32, u32),
    /// Art worker pool size.
    art_workers: usize,
    /// Wall prefetch on/off.
    prefetch: bool,
    /// Max new art requests accepted per frame
    /// (usize::MAX = uncapped).
    frame_art_cap: usize,
    /// Pin pixels_per_point to 1.0.
    scale_lock: bool,
    /// Initial window size.
    window: [f32; 2],
}

/// The study §3 low-spec budget: 48 MB — at 220x300 a
/// cover is ~0.26 MB decoded, so this still holds
/// several screenfuls, and on a 4 GB / iGPU machine it
/// leaves the OS its RAM.
const LOW_SPEC_BUDGET: usize = 48 * 1024 * 1024;

/// Resolve the profile from the settings + the core
/// count. Pure. The authority order is the study's:
/// the manual low-spec flag in the settings wins; the
/// first-run detection results only ever appear here as
/// recorded values (an HDD's shallower pool/prefetch),
/// never as a mode flip — detection wrote its defaults
/// into `low_spec` itself, once, at first run.
fn resolve_profile(s: &Settings, cores: usize) -> MachineProfile {
    if s.low_spec {
        return MachineProfile {
            low_spec: true,
            cover_budget: LOW_SPEC_BUDGET,
            rendition: (176, 240),
            art_workers: 1,
            prefetch: false,
            frame_art_cap: 8,
            scale_lock: true,
            window: [1280.0, 800.0],
        };
    }
    let mut workers = art_worker_count(cores);
    let mut prefetch = true;
    if s.detected_disk == "hdd" {
        // One spinning disk: parallel readers are
        // parallel seeks, and speculative reads are the
        // first thing it can't afford (study §3 rule:
        // HDD -> pool <= 2 and prefetch off, even
        // outside low-spec mode).
        workers = workers.min(2);
        prefetch = false;
    }
    MachineProfile {
        low_spec: false,
        cover_budget: cover_budget_bytes(&s.cover_memory),
        rendition: (220, 300),
        art_workers: workers,
        prefetch,
        frame_art_cap: usize::MAX,
        scale_lock: false,
        window: [1560.0, 980.0],
    }
}

fn cores_now() -> usize {
    std::thread::available_parallelism()
        .map(|v| v.get())
        .unwrap_or(2)
}

/// The live profile for this process.
fn machine_profile() -> MachineProfile {
    resolve_profile(&load_settings(), cores_now())
}

/// The boot drive's media class as the probe can know
/// it. Stored in settings as "ssd" / "hdd" / "" — the
/// string form is the settings-file shape.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DiskKind {
    Ssd,
    Hdd,
    Unknown,
}

fn disk_kind_str(k: DiskKind) -> &'static str {
    match k {
        DiskKind::Ssd => "ssd",
        DiskKind::Hdd => "hdd",
        DiskKind::Unknown => "",
    }
}

/// What the one-time probe learned about the machine.
struct Detection {
    cores: usize,
    ram_gb: Option<f64>,
    disk: DiskKind,
}

/// Does detection say this machine wants low-spec
/// defaults? Pure: <= 2 cores, or <= 4 GB of RAM when
/// the RAM probe answered at all.
fn detection_low_spec(cores: usize, ram_gb: Option<f64>) -> bool {
    cores <= 2 || ram_gb.map(|r| r <= 4.0).unwrap_or(false)
}

/// Parse the probe script's output — two lines,
/// "RAMGB=<n>" and "DISK=<text>". Pure; tolerant of
/// missing lines, extra output, decimal commas (a
/// comma-locale PowerShell prints 3,9), and junk.
fn parse_probe_output(text: &str) -> (Option<f64>, DiskKind) {
    let mut ram = None;
    let mut disk = DiskKind::Unknown;
    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("RAMGB=") {
            let v = v.trim().replace(',', ".");
            if let Ok(n) = v.parse::<f64>() {
                if n > 0.0 {
                    ram = Some(n);
                }
            }
        } else if let Some(v) = line.strip_prefix("DISK=") {
            disk = match v.trim().to_ascii_uppercase().as_str() {
                "SSD" => DiskKind::Ssd,
                "HDD" => DiskKind::Hdd,
                _ => DiskKind::Unknown,
            };
        }
    }
    (ram, disk)
}

/// The one-time machine probe (study §3): total RAM and
/// the boot drive's media type, asked once through the
/// app's existing PowerShell pattern. Bounded — a hung
/// or absent PowerShell costs a few seconds of the
/// FIRST run ever and answers "unknown", which resolves
/// to standard defaults. Every failure is silent by
/// design: detection is a default-setter, not a feature.
fn probe_machine() -> Detection {
    let cores = cores_now();
    let (ram_gb, disk) = ps_probe()
        .map(|out| parse_probe_output(&out))
        .unwrap_or((None, DiskKind::Unknown));
    Detection {
        cores,
        ram_gb,
        disk,
    }
}

fn ps_probe() -> Option<String> {
    let script = "$ram=''; $disk=''; \
        try { $ram = [math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory/1GB,1) } catch {}; \
        try { $n = (Get-Partition -DriveLetter C -ErrorAction Stop).DiskNumber; \
              $disk = (Get-PhysicalDisk | Where-Object { $_.DeviceId -eq [string]$n } | Select-Object -First 1).MediaType } catch {}; \
        Write-Output \"RAMGB=$ram\"; Write-Output \"DISK=$disk\"";
    let mut child = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .creation_flags(0x0800_0000)
        .spawn()
        .ok()?;
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let mut out = String::new();
                if let Some(mut so) = child.stdout.take() {
                    let _ = so.read_to_string(&mut out);
                }
                return Some(out);
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return None,
        }
    }
}

/// First-run setup, called from main() before the window
/// exists — and ONLY there. When no settings file
/// exists this is a brand-new install: run the probe,
/// record what it found, set the low-spec default if
/// the machine calls for it, and save — so the file
/// exists from then on and detection never runs again.
/// When a settings file DOES exist, this is a pure
/// read: an existing install (the user's above all) can
/// never be auto-flipped by anything in this function.
fn first_run_profile() -> MachineProfile {
    if !settings_path().exists() {
        let det = probe_machine();
        let mut s = settings_blank();
        s.detected_ram_gb = det.ram_gb;
        s.detected_disk = disk_kind_str(det.disk).to_string();
        if detection_low_spec(det.cores, det.ram_gb) {
            s.low_spec = true;
        }
        save_settings(&s);
        return resolve_profile(&s, det.cores);
    }
    machine_profile()
}

// ---------------- emulator update checks ---------------------------
// the user (Oct 20): EmuFront should check whether the installed
// emulators themselves have newer versions available. For each
// emulator with a known official source: probe the installed version
// (the exe's own version resource; RetroArch answers --version) and
// compare with the project's latest GitHub release. CHECK + DISPLAY
// only — updating stays a deliberate click on the release page.

struct EmuSource {
    name: &'static str,
    exe: &'static str, // lowercase exe file name, matched against profiles
    repo: Option<&'static str>,
    page: &'static str,
}

const EMU_SOURCES: &[EmuSource] = &[
    EmuSource { name: "Dolphin", exe: "dolphin.exe", repo: None, page: "https://dolphin-emu.org/download/" },
    EmuSource { name: "PCSX2", exe: "pcsx2-qt.exe", repo: Some("PCSX2/pcsx2"), page: "https://github.com/PCSX2/pcsx2/releases" },
    EmuSource { name: "DuckStation", exe: "duckstation-qt-x64-releaseltcg.exe", repo: Some("stenzek/duckstation"), page: "https://github.com/stenzek/duckstation/releases" },
    EmuSource { name: "Mesen", exe: "mesen.exe", repo: Some("SourMesen/Mesen2"), page: "https://github.com/SourMesen/Mesen2/releases" },
    EmuSource { name: "SNES9x", exe: "snes9x-x64.exe", repo: Some("snes9xgit/snes9x"), page: "https://github.com/snes9xgit/snes9x/releases" },
    EmuSource { name: "Project64", exe: "project64.exe", repo: None, page: "https://www.project64.eu/" },
    EmuSource { name: "mGBA", exe: "mgba.exe", repo: Some("mgba-emu/mgba"), page: "https://github.com/mgba-emu/mgba/releases" },
    EmuSource { name: "PPSSPP", exe: "ppssppwindows64.exe", repo: Some("hrydgard/ppsspp"), page: "https://github.com/hrydgard/ppsspp/releases" },
    EmuSource { name: "melonDS", exe: "melonds.exe", repo: Some("melonDS-emu/melonDS"), page: "https://github.com/melonDS-emu/melonDS/releases" },
    EmuSource { name: "Azahar", exe: "azahar.exe", repo: Some("azahar-emu/azahar"), page: "https://github.com/azahar-emu/azahar/releases" },
    EmuSource { name: "RPCS3", exe: "rpcs3.exe", repo: Some("RPCS3/rpcs3"), page: "https://github.com/RPCS3/rpcs3/releases" },
    EmuSource { name: "Cemu", exe: "cemu.exe", repo: Some("cemu-project/Cemu"), page: "https://github.com/cemu-project/Cemu/releases" },
    EmuSource { name: "MAME", exe: "mameui.exe", repo: Some("mamedev/mame"), page: "https://github.com/mamedev/mame/releases" },
    EmuSource { name: "FBNeo", exe: "fbneo64.exe", repo: Some("finalburnneo/FBNeo"), page: "https://github.com/finalburnneo/FBNeo/releases" },
    EmuSource { name: "Flycast", exe: "flycast.exe", repo: Some("flyinghead/flycast"), page: "https://github.com/flyinghead/flycast/releases" },
    EmuSource { name: "xemu", exe: "xemu.exe", repo: Some("xemu-project/xemu"), page: "https://github.com/xemu-project/xemu/releases" },
    EmuSource { name: "Stella", exe: "stella.exe", repo: Some("stella-emu/stella"), page: "https://github.com/stella-emu/stella/releases" },
    EmuSource { name: "BizHawk", exe: "emuhawk.exe", repo: Some("TASEmulators/BizHawk"), page: "https://github.com/TASEmulators/BizHawk/releases" },
    EmuSource { name: "VBA-M", exe: "visualboyadvance-m.exe", repo: Some("visualboyadvance-m/visualboyadvance-m"), page: "https://github.com/visualboyadvance-m/visualboyadvance-m/releases" },
    EmuSource { name: "Mednafen", exe: "mednafen.exe", repo: None, page: "https://mednafen.github.io/" },
];

/// Read a PE exe's version-resource string ("FileVersion", falling
/// back to "ProductVersion"). The version resource stores each entry
/// as: WORD length, WORD value-length, WORD type(=1), UTF-16 key,
/// DWORD-aligned UTF-16 value.
fn pe_version_string(path: &Path) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    fn read_entry(data: &[u8], key: &str) -> Option<String> {
        let mut pat: Vec<u8> = key
            .encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect();
        pat.extend_from_slice(&[0, 0]);
        let mut i = 0usize;
        while i + pat.len() + 6 <= data.len() {
            if &data[i..i + pat.len()] == pat.as_slice() && i >= 6 {
                let vallen =
                    u16::from_le_bytes([data[i - 4], data[i - 3]]) as usize;
                let wtype = u16::from_le_bytes([data[i - 2], data[i - 1]]);
                if wtype == 1 && (1..=64).contains(&vallen) {
                    let mut off = i + pat.len();
                    while off % 4 != 0 {
                        off += 1;
                    }
                    if off + vallen * 2 <= data.len() {
                        let mut s = String::new();
                        for k in 0..vallen {
                            let c = u16::from_le_bytes([
                                data[off + k * 2],
                                data[off + k * 2 + 1],
                            ]);
                            if c == 0 {
                                break;
                            }
                            s.push(char::from_u32(c as u32).unwrap_or('?'));
                        }
                        let s = s.trim().to_string();
                        if !s.is_empty() {
                            return Some(s);
                        }
                    }
                }
            }
            i += 2;
        }
        None
    }
    read_entry(&data, "FileVersion")
        .or_else(|| read_entry(&data, "ProductVersion"))
}

/// "v2.1.8093" / "0.1-12307" / "2609.104" -> numeric tuple for compare.
fn ver_tuple(s: &str) -> Vec<u64> {
    s.split(|c: char| !c.is_ascii_digit())
        .filter(|p| !p.is_empty())
        .map(|p| p.parse::<u64>().unwrap_or(0))
        .collect()
}

/// True when `latest` is a strictly newer version than `installed`.
fn ver_is_newer(latest: &[u64], installed: &[u64]) -> bool {
    let n = latest.len().max(installed.len());
    for i in 0..n {
        let l = latest.get(i).copied().unwrap_or(0);
        let v = installed.get(i).copied().unwrap_or(0);
        if l != v {
            return l > v;
        }
    }
    false
}

/// Latest release (tag, release-page URL) from a project's official
/// GitHub. Falls back to the newest release incl. prereleases for
/// projects that only publish those (DuckStation-style).
fn github_latest(repo: &str) -> Option<(String, String)> {
    let get = |url: &str| -> Option<serde_json::Value> {
        let resp = ureq::get(url)
            .set(
                "User-Agent",
                &format!("EmuFrontRust/{APP_VERSION} (emulator update check)"),
            )
            .timeout(std::time::Duration::from_secs(25))
            .call()
            .ok()?;
        resp.into_json().ok()
    };
    if let Some(v) = get(&format!(
        "https://api.github.com/repos/{repo}/releases/latest"
    )) {
        if let (Some(t), Some(h)) = (
            v.get("tag_name").and_then(|x| x.as_str()),
            v.get("html_url").and_then(|x| x.as_str()),
        ) {
            return Some((t.to_string(), h.to_string()));
        }
    }
    if let Some(v) = get(&format!(
        "https://api.github.com/repos/{repo}/releases?per_page=1"
    )) {
        if let Some(first) = v.as_array().and_then(|a| a.first()) {
            if let (Some(t), Some(h)) = (
                first.get("tag_name").and_then(|x| x.as_str()),
                first.get("html_url").and_then(|x| x.as_str()),
            ) {
                return Some((t.to_string(), h.to_string()));
            }
        }
    }
    None
}

/// RetroArch reports itself: "RetroArch: Frontend for libretro --
/// v1.21.1 -- <git> --" on stdout.
fn retroarch_version(exe: &Path) -> Option<String> {
    let out = Command::new(exe)
        .arg("--version")
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    for tok in text.split_whitespace() {
        // "-- v1.21.1 --" style, or "Version: 1.21.0" style
        let cand = tok.strip_prefix('v').unwrap_or(tok);
        if cand.chars().next().is_some_and(|c| c.is_ascii_digit())
            && cand.contains('.')
            && cand.chars().all(|c| c.is_ascii_digit() || c == '.')
        {
            return Some(cand.to_string());
        }
    }
    None
}

fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Run the full check against the emulators the profiles point at.
/// Blocking (network) — call from a worker thread or the CLI.
fn check_emu_updates(profiles: &[Profile]) -> BTreeMap<String, EmuUpdateInfo> {
    let mut by_exe: HashMap<String, PathBuf> = HashMap::new();
    for p in profiles {
        if let Some(f) = Path::new(&p.path).file_name() {
            by_exe
                .entry(f.to_string_lossy().to_lowercase())
                .or_insert_with(|| PathBuf::from(&p.path));
        }
    }
    let mut out: BTreeMap<String, EmuUpdateInfo> = BTreeMap::new();
    // RetroArch: version is real, but updates flow through the app's
    // channel machinery at startup (buildbot) — informational row.
    if let Some(exe) = by_exe.get("retroarch.exe") {
        let installed = retroarch_version(exe)
            .unwrap_or_else(|| "unknown".into());
        out.insert(
            "RetroArch".into(),
            EmuUpdateInfo {
                installed,
                latest: "via buildbot channel".into(),
                status: "info".into(),
                url: "https://buildbot.libretro.com/stable/".into(),
            },
        );
    }
    for src in EMU_SOURCES {
        let Some(exe) = by_exe.get(src.exe) else {
            continue;
        };
        let installed = pe_version_string(exe);
        let Some(repo) = src.repo else {
            out.insert(
                src.name.into(),
                EmuUpdateInfo {
                    installed: installed.unwrap_or_else(|| "unknown".into()),
                    latest: "check the official site".into(),
                    status: "manual".into(),
                    url: src.page.into(),
                },
            );
            continue;
        };
        let info = match (installed, github_latest(repo)) {
            (Some(inst), Some((tag, url))) => {
                let mut latest = tag.trim_start_matches('v').to_string();
                // mamedev tags look like "mame0280" for version 0.280
                if src.name == "MAME" && latest.starts_with("mame0") {
                    latest = format!("0.{}", &latest[5..]);
                }
                let lt = ver_tuple(&latest);
                let it = ver_tuple(&inst);
                let status = if lt.is_empty() || it.is_empty() {
                    "unknown"
                } else if ver_is_newer(&lt, &it) {
                    "update"
                } else {
                    "current"
                };
                EmuUpdateInfo {
                    installed: inst,
                    latest,
                    status: status.into(),
                    url,
                }
            }
            (inst, _) => EmuUpdateInfo {
                installed: inst.unwrap_or_else(|| "unknown".into()),
                latest: "check failed (offline?)".into(),
                status: "unknown".into(),
                url: src.page.into(),
            },
        };
        out.insert(src.name.into(), info);
    }
    out
}

fn ra_stamp_path() -> PathBuf {
    Path::new(RA_DIR).join("emufront_ra_stamp.json")
}

fn ra_stamp() -> (String, String) {
    if let Ok(text) = std::fs::read_to_string(ra_stamp_path()) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            return (
                v.get("channel")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                v.get("label")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
            );
        }
    }
    (String::new(), String::new())
}

fn ra_running() -> bool {
    Command::new("tasklist")
        .creation_flags(0x0800_0000)
        .args(["/FI", "IMAGENAME eq retroarch.exe"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("retroarch.exe"))
        .unwrap_or(false)
}

/// (download url, version label) for the newest build of a channel.
fn ra_remote(channel: &str) -> Option<(String, String)> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    if channel == "stable" {
        let text = agent
            .get("https://buildbot.libretro.com/stable/")
            .call()
            .ok()?
            .into_string()
            .ok()?;
        let mut best: Vec<u32> = vec![];
        let mut best_s = String::new();
        for part in text.split("href=\"") {
            if let Some(end) = part.find('/') {
                let cand = &part[..end];
                let nums: Vec<u32> = cand
                    .split('.')
                    .filter_map(|x| x.parse::<u32>().ok())
                    .collect();
                if nums.len() >= 2 && nums > best {
                    best = nums;
                    best_s = cand.to_string();
                }
            }
        }
        if best_s.is_empty() {
            return None;
        }
        Some((
            format!(
                "https://buildbot.libretro.com/stable/{best_s}/windows/x86_64/RetroArch.7z"
            ),
            format!("stable {best_s}"),
        ))
    } else {
        let url =
            "https://buildbot.libretro.com/nightly/windows/x86_64/RetroArch.7z"
                .to_string();
        let resp = agent.head(&url).call().ok()?;
        let label = resp
            .header("last-modified")
            .unwrap_or("latest")
            .to_string();
        Some((url, format!("nightly ({label})")))
    }
}

/// Check the channel and update RetroArch in place if newer.
/// Blocking — call from a worker thread.
fn ra_ensure(channel: &str) -> String {
    let (have_ch, have_label) = ra_stamp();
    let Some((url, label)) = ra_remote(channel) else {
        return "RetroArch: couldn't reach buildbot — keeping current build"
            .to_string();
    };
    if have_ch == channel && have_label == label {
        return format!("RetroArch {label} — already the latest");
    }
    if ra_running() {
        return format!(
            "RetroArch {label} is available, but RetroArch is running — update deferred"
        );
    }
    let tmp = exe_dir().join("ra-update.7z");
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(600))
        .build();
    let Ok(resp) = agent.get(&url).call() else {
        return "RetroArch download failed — keeping current build"
            .to_string();
    };
    let mut bytes = Vec::new();
    if resp.into_reader().read_to_end(&mut bytes).is_err() || bytes.len() < 1_000_000 {
        return "RetroArch download failed — keeping current build"
            .to_string();
    }
    if std::fs::write(&tmp, &bytes).is_err() {
        return "RetroArch update failed (couldn't stage download)"
            .to_string();
    }
    // The buildbot archive holds a top-level RetroArch folder:
    // extract over the RetroArch folder's parent so it merges
    // into the install in place, preserving cores/system/saves
    // that aren't in the archive.
    let ra_parent = Path::new(RA_DIR)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("C:\\"));
    let out = Command::new(SEVEN_ZIP)
        .creation_flags(0x0800_0000)
        .args([
            "x",
            "-y",
            &format!("-o{}", ra_parent.to_string_lossy()),
            tmp.to_string_lossy().as_ref(),
        ])
        .output();
    let _ = std::fs::remove_file(&tmp);
    match out {
        Ok(o) if o.status.success() => {
            let v = serde_json::json!({
                "channel": channel,
                "label": label,
            });
            let _ = std::fs::write(ra_stamp_path(), v.to_string());
            format!("RetroArch updated to {label}")
        }
        _ => "RetroArch update failed during extraction — current build kept"
            .to_string(),
    }
}


// ---------------- art fetching (worker pool) ----------------
// Perf Slice 1 (RELEASE-PERF-DESIGN §2.3): a fixed pool of
// background workers replaces the thread-per-cover spawns
// and the UI-thread decode. Workers fetch AND decode +
// resize; the UI thread only uploads finished pixels to
// the GPU. Requests carry a generation stamp — when the
// wall changes, queued work for the old wall is dropped
// instead of decoded.

// ---------------- wall prefetch tuning (v1.06) -----------
// Slice 3 tuning, now that the pool has shipped behavior
// to tune against. The v1.04 prefetch was symmetric: one
// viewport of speculative covers above AND below, all
// submitted the same frame — half of it behind the user
// when they're scrolling one way, and a wall-open burst
// that floods the pool ahead of the covers actually on
// screen. The tuning keeps the same total speculation
// (two viewport depths) and spends it deliberately:
// biased toward the scroll direction (§2.2: "the next
// screenful IN THE SCROLL DIRECTION"), and paced — at
// most PREFETCH_QUEUE_PER_FRAME new requests per frame,
// nearest the viewport first, so visible tiles always
// win the workers. Profiles with prefetch off
// (low-spec, spinning disks) get empty ranges and stay
// exactly as v1.05 left them.

/// Which way the wall is travelling, from the previous
/// frame's top row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ScrollDir {
    Up,
    Down,
    Still,
}

/// Pure: no previous frame (a fresh wall) reads as Still.
fn scroll_dir(prev_top: Option<usize>, top: usize) -> ScrollDir {
    match prev_top {
        None => ScrollDir::Still,
        Some(p) if top > p => ScrollDir::Down,
        Some(p) if top < p => ScrollDir::Up,
        Some(_) => ScrollDir::Still,
    }
}

/// The most new prefetch requests one frame may queue,
/// in columns: two rows' worth. A full window fills in
/// over a few frames instead of one burst.
const PREFETCH_QUEUE_PER_FRAME_COLS: usize = 2;

/// The prefetch index ranges for one frame —
/// (scroll-direction range first, the other second) —
/// as wall indices, clamped to the wall. Pure; the
/// profile tiers pin in --selftest: disabled → both
/// empty; Still reproduces the v1.04 ranges exactly
/// (one viewport each way); Down/Up split the same two
/// viewports 1.5 / 0.5 toward the travel direction.
fn prefetch_ranges(
    first_row: usize,
    last_row: usize,
    cols: usize,
    wall_len: usize,
    dir: ScrollDir,
    enabled: bool,
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
    let empty = 0..0;
    if !enabled || cols == 0 || wall_len == 0 {
        return (empty.clone(), empty);
    }
    let v = last_row.saturating_sub(first_row).max(1);
    let (ahead, behind) = match dir {
        ScrollDir::Still => (v, v),
        ScrollDir::Down => (v + v / 2, v - v / 2),
        ScrollDir::Up => (v - v / 2, v + v / 2),
    };
    let below_start = (last_row * cols).min(wall_len);
    let below_end = ((last_row + ahead) * cols).min(wall_len).max(below_start);
    let above_end = (first_row * cols).min(wall_len);
    let above_start = above_end.saturating_sub(behind * cols);
    let below = below_start..below_end;
    let above = above_start..above_end;
    match dir {
        ScrollDir::Up => (above, below),
        _ => (below, above),
    }
}

enum ArtMsg {
    /// Decoded cover pixels, ready to upload.
    Ready(String, egui::ColorImage),
    /// Bytes arrived but would not decode as an image.
    Failed(String),
    Missing(String),
}

/// Decode + downscale raw art bytes to the wall's cover
/// size — the exact work drain_art used to do on the UI
/// thread, now a pure function the pool workers run (and
/// --selftest can round-trip).
fn decode_cover(bytes: &[u8]) -> Result<egui::ColorImage, ()> {
    decode_cover_to(bytes, 220, 300)
}

/// decode_cover for a rendition tier's bounding box:
/// (220, 300) normally, (176, 240) in low-spec mode
/// (study §3 — decode cost scales with pixels, ~0.55x).
fn decode_cover_to(
    bytes: &[u8],
    w: u32,
    h: u32,
) -> Result<egui::ColorImage, ()> {
    let img = image::load_from_memory(bytes).map_err(|_| ())?;
    let img = img.resize(w, h, image::imageops::FilterType::Triangle);
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        rgba.as_raw(),
    ))
}

// ---------------- cover renditions (v1.05, design §2.2) --------------
// The shared box-art originals are the user's collection
// (and the Python app's cache) — never evicted, never
// re-encoded. What the wall actually displays, though, is
// a 220x300 downscale, and re-decoding a full-size
// original for every display is the recurring cost on a
// slow disk/CPU. So the first display of a cover also
// writes a small rendition PNG into the private cache
// tree (cache\thumbs), keyed by art key + the source
// file's fingerprint (size + mtime — a re-scrape changes
// it and the stale rendition simply stops matching).
// Later displays decode the rendition instead. The tree
// is capped at 2 GB with LRU eviction; every byte of it
// is derived data, so losing any of it costs one
// re-decode of an original that's still on disk.

const REND_CAP_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Clone)]
struct RendCfg {
    w: u32,
    h: u32,
    dir: PathBuf,
    cap: u64,
}

#[derive(Default)]
struct RendState {
    /// Tracked rendition bytes (seeded by the startup
    /// walk, then kept current by writes/sweeps).
    total: std::sync::Mutex<u64>,
    /// Last eviction sweep — sweeps are throttled so a
    /// long scroll across the cap can't turn into a
    /// directory walk per cover.
    last_sweep: std::sync::Mutex<Option<Instant>>,
}

/// The rendition file name for one cover at one tier:
/// key + tier box + the source file's fingerprint.
/// Pure — --selftest pins the round-trip.
fn rendition_name(
    key: &str,
    w: u32,
    h: u32,
    src_size: u64,
    src_mtime: u64,
) -> String {
    format!("{key}_{w}x{h}_{src_size}_{src_mtime}.png")
}

/// Parse a rendition name back: (w, h, src_size,
/// src_mtime). Anything that isn't one of ours (wrong
/// suffix, bad numbers) is None — the accounting walks
/// must never count foreign files as renditions.
fn parse_rendition_name(
    name: &str,
) -> Option<(u32, u32, u64, u64)> {
    let stem = name.strip_suffix(".png")?;
    let mut parts = stem.rsplit('_');
    let mtime: u64 = parts.next()?.parse().ok()?;
    let size: u64 = parts.next()?.parse().ok()?;
    let dims = parts.next()?;
    let (w, h) = dims.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?, size, mtime))
}

/// Who goes when the rendition tree is over its cap:
/// least-recently-used (oldest mtime) first, until the
/// total fits. Pure — --selftest pins the order and the
/// accounting.
fn eviction_plan(
    entries: &[(String, u64, u64)],
    total: u64,
    cap: u64,
) -> Vec<String> {
    let mut by_age: Vec<&(String, u64, u64)> = entries.iter().collect();
    by_age.sort_by_key(|e| e.1);
    let mut out = Vec::new();
    let mut remaining = total;
    for e in by_age {
        if remaining <= cap {
            break;
        }
        out.push(e.0.clone());
        remaining = remaining.saturating_sub(e.2);
    }
    out
}

/// (size, mtime secs) of a file — the rendition key's
/// source fingerprint.
fn file_fingerprint(p: &Path) -> Option<(u64, u64)> {
    let m = std::fs::metadata(p).ok()?;
    let mt = m
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    Some((m.len(), mt.as_secs()))
}

/// The local file fetch_art_bytes would read first, in
/// its exact candidate order (per-system cache, shared
/// flat cache, prototype cache, then the playlist alt
/// keys in the same order) — or None when only the
/// network can answer. Locating it separately is what
/// lets the worker fingerprint the source before
/// reading a byte of it.
fn locate_art_source(game: &Game) -> Option<PathBuf> {
    let key = art_key(&game.path);
    let mut cands = vec![
        boxart_sys_file(game),
        boxart_file(&key),
        proto_cache().join(format!("{key}.png")),
    ];
    for alt in &art_alt_paths(game) {
        let ak = art_key(alt);
        cands.push(boxart_sys_file_key(&game.system, &ak));
        cands.push(boxart_file(&ak));
        cands.push(proto_cache().join(format!("{ak}.png")));
    }
    cands.into_iter().find(|p| {
        std::fs::metadata(p)
            .map(|m| m.is_file() && m.len() > 0)
            .unwrap_or(false)
    })
}

/// Decode a stored rendition. No resize: what's stored
/// is already at the tier's box (decode_cover_to made
/// it), and re-resizing would just burn the CPU the
/// rendition exists to save.
fn decode_rendition(bytes: &[u8]) -> Result<egui::ColorImage, ()> {
    let img = image::load_from_memory(bytes).map_err(|_| ())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        rgba.as_raw(),
    ))
}

fn encode_rendition(img: &egui::ColorImage) -> Option<Vec<u8>> {
    let (w, h) = (img.size[0] as u32, img.size[1] as u32);
    let mut bytes = Vec::with_capacity(img.size[0] * img.size[1] * 4);
    for px in &img.pixels {
        bytes.extend_from_slice(&px.to_array());
    }
    let rgba = image::RgbaImage::from_raw(w, h, bytes)?;
    let mut cur = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(rgba)
        .write_to(&mut cur, image::ImageFormat::Png)
        .ok()?;
    Some(cur.into_inner())
}

/// Refresh a hit rendition's mtime (that's the LRU's
/// last-used) — but only when it's gone stale (> 6 h),
/// so a browsing session isn't a write per cover.
fn touch_rendition(path: &Path, bytes: &[u8]) {
    let stale = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| {
            t.elapsed()
                .map(|d| d.as_secs() > 6 * 3600)
                .unwrap_or(false)
        })
        .unwrap_or(false);
    if stale {
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, bytes).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }
}

fn write_rendition(
    path: &Path,
    img: &egui::ColorImage,
    state: &Arc<RendState>,
    cfg: &RendCfg,
) {
    let Some(bytes) = encode_rendition(img) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, &bytes).is_ok()
        && std::fs::rename(&tmp, path).is_ok()
    {
        let over = {
            let mut total = state.total.lock().unwrap();
            *total += bytes.len() as u64;
            *total > cfg.cap
        };
        if over {
            maybe_sweep_renditions(state, cfg);
        }
    }
}

fn maybe_sweep_renditions(state: &Arc<RendState>, cfg: &RendCfg) {
    {
        let mut last = state.last_sweep.lock().unwrap();
        if let Some(t) = *last {
            if t.elapsed() < Duration::from_secs(30) {
                return;
            }
        }
        *last = Some(Instant::now());
    }
    sweep_renditions(state, cfg);
}

/// Walk the rendition tree, recount it honestly, and
/// delete LRU files until it's under the cap.
fn sweep_renditions(state: &Arc<RendState>, cfg: &RendCfg) {
    let mut entries: Vec<(String, u64, u64)> = Vec::new();
    let mut total = 0u64;
    if let Ok(rd) = std::fs::read_dir(&cfg.dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if parse_rendition_name(&name).is_none() {
                continue;
            }
            if let Ok(m) = e.metadata() {
                let mtime = m
                    .modified()
                    .ok()
                    .and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH).ok()
                    })
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                total += m.len();
                entries.push((name, mtime, m.len()));
            }
        }
    }
    let plan = eviction_plan(&entries, total, cfg.cap);
    let mut freed = 0u64;
    for name in &plan {
        let p = cfg.dir.join(name);
        if let Ok(m) = std::fs::metadata(&p) {
            freed += m.len();
        }
        let _ = std::fs::remove_file(&p);
    }
    *state.total.lock().unwrap() = total.saturating_sub(freed);
}

/// The pool worker's whole job (v1.05 shape): rendition
/// first when a local source exists, the original
/// fetch path otherwise — and a rendition is written
/// whenever a full-size decode happened, whichever
/// source served it.
fn art_fetch_render(
    game: &Game,
    key: &str,
    cfg: &RendCfg,
    state: &Arc<RendState>,
) -> ArtMsg {
    if let Some(src) = locate_art_source(game) {
        if let Some((size, mtime)) = file_fingerprint(&src) {
            let rp = cfg
                .dir
                .join(rendition_name(key, cfg.w, cfg.h, size, mtime));
            if let Ok(bytes) = std::fs::read(&rp) {
                if !bytes.is_empty() {
                    match decode_rendition(&bytes) {
                        Ok(img) => {
                            touch_rendition(&rp, &bytes);
                            return ArtMsg::Ready(key.to_string(), img);
                        }
                        Err(()) => {
                            // A corrupt rendition heals
                            // itself: delete it and fall
                            // through to the source.
                            let _ = std::fs::remove_file(&rp);
                        }
                    }
                }
            }
            if let Ok(bytes) = std::fs::read(&src) {
                if !bytes.is_empty() {
                    return match decode_cover_to(&bytes, cfg.w, cfg.h) {
                        Ok(img) => {
                            write_rendition(&rp, &img, state, cfg);
                            ArtMsg::Ready(key.to_string(), img)
                        }
                        Err(()) => ArtMsg::Failed(key.to_string()),
                    };
                }
            }
        }
    }
    match fetch_art_bytes(game) {
        Some(bytes) => match decode_cover_to(&bytes, cfg.w, cfg.h) {
            Ok(img) => {
                // The network path just saved its download
                // to the prototype cache file; if it's
                // there, it gets a rendition too.
                let local = proto_cache().join(format!("{key}.png"));
                if let Some((size, mtime)) = file_fingerprint(&local) {
                    let rp = cfg.dir.join(rendition_name(
                        key, cfg.w, cfg.h, size, mtime,
                    ));
                    write_rendition(&rp, &img, state, cfg);
                }
                ArtMsg::Ready(key.to_string(), img)
            }
            Err(()) => ArtMsg::Failed(key.to_string()),
        },
        None => ArtMsg::Missing(key.to_string()),
    }
}

/// LRU bookkeeping for the decoded-cover cache: key
/// order (front = least recently used) plus per-key
/// decoded byte sizes, against a byte budget. Pure —
/// the TextureHandles themselves live in Proto; this
/// ledger only decides what stays. --selftest pins the
/// eviction order and the accounting.
struct TexLedger {
    order: VecDeque<String>,
    sizes: HashMap<String, usize>,
    bytes: usize,
    budget: usize,
}

impl TexLedger {
    fn new(budget: usize) -> Self {
        TexLedger {
            order: VecDeque::new(),
            sizes: HashMap::new(),
            bytes: 0,
            budget,
        }
    }

    fn contains(&self, key: &str) -> bool {
        self.sizes.contains_key(key)
    }

    /// Mark a key as just-used: it moves to the back.
    fn touch(&mut self, key: &str) {
        if self.sizes.contains_key(key) {
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
                self.order.push_back(key.to_string());
            }
        }
    }

    /// Record a decoded cover of `size` bytes; returns
    /// the keys evicted to fit the budget, LRU first.
    /// The inserted key is never evicted by its own
    /// insert — one oversized cover may exceed the
    /// budget alone, and it still stays.
    fn insert(&mut self, key: &str, size: usize) -> Vec<String> {
        self.remove(key);
        self.sizes.insert(key.to_string(), size);
        self.bytes += size;
        self.order.push_back(key.to_string());
        self.evict_over_budget()
    }

    fn remove(&mut self, key: &str) -> bool {
        if let Some(sz) = self.sizes.remove(key) {
            self.bytes -= sz;
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
            }
            true
        } else {
            false
        }
    }

    fn clear(&mut self) {
        self.order.clear();
        self.sizes.clear();
        self.bytes = 0;
    }

    /// Change the budget live (the ⚙ Cache & Performance
    /// controls): returns the keys evicted to fit it.
    fn set_budget(&mut self, budget: usize) -> Vec<String> {
        self.budget = budget;
        self.evict_over_budget()
    }

    fn evict_over_budget(&mut self) -> Vec<String> {
        let mut evicted = Vec::new();
        while self.bytes > self.budget && self.order.len() > 1 {
            let front = self.order.pop_front().expect("len > 1");
            if let Some(sz) = self.sizes.remove(&front) {
                self.bytes -= sz;
                evicted.push(front);
            }
        }
        evicted
    }
}

/// One unit of art work for the pool.
struct ArtJob {
    key: String,
    game: Game,
    gen: u64,
}

/// The pool's shared queue: visible tiles before
/// prefetch, duplicate keys refused, stale generations
/// dropped on pop. Pure bookkeeping — --selftest pins
/// the semantics without spawning a thread.
#[derive(Default)]
struct ArtQueue {
    hi: VecDeque<ArtJob>,
    lo: VecDeque<ArtJob>,
    queued: HashSet<String>,
}

impl ArtQueue {
    /// Queue a job; false when the key is already
    /// queued (the render loop asks every frame — the
    /// queue, like the in-flight set, takes a key once).
    fn push(&mut self, job: ArtJob, high: bool) -> bool {
        if !self.queued.insert(job.key.clone()) {
            return false;
        }
        if high {
            self.hi.push_back(job);
        } else {
            self.lo.push_back(job);
        }
        true
    }

    /// Take the next job worth doing: high priority
    /// first; jobs stamped with an old generation are
    /// dropped on the way (their wall is gone).
    fn pop(&mut self, current_gen: u64) -> Option<ArtJob> {
        loop {
            let job = match self.hi.pop_front() {
                Some(j) => j,
                None => self.lo.pop_front()?,
            };
            self.queued.remove(&job.key);
            if job.gen == current_gen {
                return Some(job);
            }
        }
    }

    /// Drop every job not stamped `current_gen`
    /// (runs when the generation bumps).
    fn purge(&mut self, current_gen: u64) {
        self.hi.retain(|j| j.gen == current_gen);
        self.lo.retain(|j| j.gen == current_gen);
        self.queued = self
            .hi
            .iter()
            .chain(self.lo.iter())
            .map(|j| j.key.clone())
            .collect();
    }
}

/// Pool size from the machine's parallelism, clamped
/// 2–4 (design §2.3: extra threads on a spinning disk
/// just queue seeks). Pure given the count, so
/// --selftest can pin the clamp.
fn art_worker_count(parallelism: usize) -> usize {
    parallelism.clamp(2, 4)
}

/// The art worker pool: background threads pulling
/// from the shared queue, fetching + decoding, and
/// posting finished pixels to the UI thread's channel.
/// Worker count and the rendition tier come from the
/// resolved machine profile (v1.05) — the profile, not
/// the pool, owns the machine policy.
struct ArtPool {
    queue: Arc<(std::sync::Mutex<ArtQueue>, std::sync::Condvar)>,
    gen: Arc<AtomicU64>,
    notify: Arc<std::sync::Mutex<Option<egui::Context>>>,
    rend: Arc<RendState>,
}

impl ArtPool {
    fn start(
        tx: Sender<ArtMsg>,
        workers: usize,
        rend_cfg: RendCfg,
    ) -> ArtPool {
        let pool = ArtPool {
            queue: Arc::new((
                std::sync::Mutex::new(ArtQueue::default()),
                std::sync::Condvar::new(),
            )),
            gen: Arc::new(AtomicU64::new(0)),
            notify: Arc::new(std::sync::Mutex::new(None)),
            rend: Arc::new(RendState::default()),
        };
        // Rendition accounting: seed the tracked total
        // with one walk, and sweep at startup if the
        // tree is found over its cap (design §2.2).
        {
            let state = pool.rend.clone();
            let cfg = rend_cfg.clone();
            std::thread::spawn(move || {
                let mut total = 0u64;
                if let Ok(rd) = std::fs::read_dir(&cfg.dir) {
                    for e in rd.flatten() {
                        let name =
                            e.file_name().to_string_lossy().to_string();
                        if parse_rendition_name(&name).is_some() {
                            if let Ok(m) = e.metadata() {
                                total += m.len();
                            }
                        }
                    }
                }
                *state.total.lock().unwrap() = total;
                if total > cfg.cap {
                    sweep_renditions(&state, &cfg);
                }
            });
        }
        for _ in 0..workers.max(1) {
            let queue = pool.queue.clone();
            let gen = pool.gen.clone();
            let notify = pool.notify.clone();
            let tx = tx.clone();
            let cfg = rend_cfg.clone();
            let state = pool.rend.clone();
            std::thread::spawn(move || loop {
                let job = {
                    let (lock, cvar) = &*queue;
                    let mut q = lock.lock().unwrap();
                    loop {
                        if let Some(j) = q.pop(gen.load(Ordering::SeqCst)) {
                            break j;
                        }
                        q = cvar.wait(q).unwrap();
                    }
                };
                let msg = art_fetch_render(&job.game, &job.key, &cfg, &state);
                let _ = tx.send(msg);
                if let Ok(n) = notify.lock() {
                    if let Some(ctx) = &*n {
                        ctx.request_repaint();
                    }
                }
            });
        }
        pool
    }

    /// Forget the tracked rendition total (Clear Cache
    /// just deleted the files under it).
    fn reset_renditions(&self) {
        *self.rend.total.lock().unwrap() = 0;
    }

    /// Offer work to the pool. `high` = a tile on
    /// screen is waiting for it; false = prefetch.
    fn submit(&self, job: ArtJob, high: bool) {
        let (lock, cvar) = &*self.queue;
        if let Ok(mut q) = lock.lock() {
            if q.push(job, high) {
                cvar.notify_one();
            }
        }
    }

    /// Point the pool's repaint notices at the live UI
    /// context (refreshed on every art request).
    fn set_notify(&self, ctx: &egui::Context) {
        if let Ok(mut n) = self.notify.lock() {
            *n = Some(ctx.clone());
        }
    }

    /// Advance the generation: queued work stamped
    /// older is dropped. Fetches already running finish
    /// — their covers still land in the texture cache,
    /// which is global — but nothing stale gets decoded
    /// from the queue.
    fn set_gen(&self, gen: u64) {
        self.gen.store(gen, Ordering::SeqCst);
        let (lock, cvar) = &*self.queue;
        if let Ok(mut q) = lock.lock() {
            q.purge(gen);
        }
        cvar.notify_all();
    }
}

/// Remove every [...] group (Python: re.sub(r"\s*\[[^\]]*\]", "")).
fn art_strip_square(s: &str) -> String {
    let mut out = String::new();
    let mut depth: i32 = 0;
    for c in s.chars() {
        match c {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// Drop ONE trailing (...) or [...] group, if the name ends
/// with one (Python: re.sub(r"\s*\([^)]*\)\s*$", "") and the
/// bracket twin).
fn art_drop_trailing_group(s: &str) -> Option<String> {
    let t = s.trim_end();
    let (open, close) = if t.ends_with(')') {
        ('(', ')')
    } else if t.ends_with(']') {
        ('[', ']')
    } else {
        return None;
    };
    let mut depth: i32 = 0;
    let mut cut: Option<usize> = None;
    for (i, c) in t.char_indices().rev() {
        if c == close {
            depth += 1;
        } else if c == open {
            depth -= 1;
            if depth == 0 {
                cut = Some(i);
                break;
            }
        }
    }
    cut.map(|i| t[..i].trim_end().to_string())
}

fn art_push_cand(out: &mut Vec<String>, s: &str) {
    let v = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if !v.is_empty() && !out.contains(&v) {
        out.push(v);
    }
}

fn art_candidates(game: &Game) -> Vec<String> {
    // Mirror Python's art_name_candidates (emu_front.py) exactly —
    // it is the candidate set the Python app demonstrably fetches
    // art with: exact stem, catalog-prefix-stripped stem,
    // square-bracket-stripped variants, and variants missing ONE
    // trailing (...) or [...] group. The old Rust set stripped
    // ALL bracketed groups instead, producing bare names like
    // "Chrono Cross" that the libretro repos don't carry (their
    // files keep the (USA) region tag) — and it never tried
    // dropping just a trailing "(Track 1)"/"(Disc 1)" group,
    // which is the form PS1 disc-image filenames take.
    let stem = Path::new(&game.path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut out: Vec<String> = Vec::new();
    art_push_cand(&mut out, &stem);
    art_push_cand(&mut out, &strip_catalog_prefix_full(&stem));
    let base = out.clone();
    for s in &base {
        art_push_cand(&mut out, &art_strip_square(s));
    }
    let base = out.clone();
    for s in &base {
        if let Some(d) = art_drop_trailing_group(s) {
            art_push_cand(&mut out, &d);
        }
    }
    out.truncate(6);
    out
}

fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.'
            | b'~' | b'(' | b')' | b'\'' | b'!' | b',' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn fetch_art_bytes(game: &Game) -> Option<Vec<u8>> {
    let key = art_key(&game.path);
    // 0) the per-system organized cache (new scrapes land here)
    let sysf = boxart_sys_file(game);
    if let Ok(bytes) = std::fs::read(&sysf) {
        if !bytes.is_empty() {
            return Some(bytes);
        }
    }
    // 1) the Python app's own artwork cache (shared, read-only)
    let shared = boxart_file(&key);
    if let Ok(bytes) = std::fs::read(&shared) {
        if !bytes.is_empty() {
            return Some(bytes);
        }
    }
    // 2) prototype-local cache
    let local = proto_cache().join(format!("{key}.png"));
    if let Ok(bytes) = std::fs::read(&local) {
        if !bytes.is_empty() {
            return Some(bytes);
        }
    }
    // 2b) playlist games: the cover may be filed under the disc
    // image's cache key (the .cue, or the .bin it points at)
    let alts = art_alt_paths(game);
    for alt in &alts {
        let ak = art_key(alt);
        let ss = boxart_sys_file_key(&game.system, &ak);
        if let Ok(bytes) = std::fs::read(&ss) {
            if !bytes.is_empty() {
                return Some(bytes);
            }
        }
        let s = boxart_file(&ak);
        if let Ok(bytes) = std::fs::read(&s) {
            if !bytes.is_empty() {
                return Some(bytes);
            }
        }
        let l = proto_cache().join(format!("{ak}.png"));
        if let Ok(bytes) = std::fs::read(&l) {
            if !bytes.is_empty() {
                return Some(bytes);
            }
        }
    }
    // 3) libretro-thumbnails over HTTP
    let repo = libretro_repo(&game.system)?;
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(20))
        .build();
    let mut cands = art_candidates(game);
    for alt in &alts {
        let ag = Game {
            path: alt.clone(),
            ..game.clone()
        };
        for c in art_candidates(&ag) {
            if !cands.contains(&c) {
                cands.push(c);
            }
        }
    }
    for cand in cands {
        let url = format!(
            "https://thumbnails.libretro.com/{}/Named_Boxarts/{}.png",
            url_encode(repo),
            url_encode(&cand)
        );
        if let Ok(resp) = agent.get(&url).call() {
            let mut bytes = Vec::new();
            if resp.into_reader().read_to_end(&mut bytes).is_ok()
                && bytes.len() > 64
            {
                let _ = std::fs::write(&local, &bytes);
                log_art_fetch(game, true);
                return Some(bytes);
            }
        }
    }
    log_art_fetch(game, false);
    None
}

// ---------------- icon painting (egui shapes) ----------------

fn draw_icon(p: &egui::Painter, r: Rect, system: &str) {
    if system == "star" {
        let c = r.center();
        let mut pts = Vec::new();
        for i in 0..10 {
            let ang = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
            let rad = if i % 2 == 0 { 44.0 } else { 18.0 };
            pts.push(Pos2::new(c.x + rad * ang.cos(), c.y + rad * ang.sin()));
        }
        p.add(egui::Shape::convex_polygon(pts, GOLD, egui::Stroke::NONE));
        return;
    }
    let cx = r.center().x;
    let cy = r.center().y - 6.0;
    let w = 150.0;
    let h = 62.0;
    let body = Rect::from_center_size(Pos2::new(cx, cy), Vec2::new(w, h));
    let gray = Color32::from_rgb(203, 203, 212);
    let dark = Color32::from_rgb(24, 24, 28);
    let white = Color32::from_rgb(238, 238, 240);
    let purple = Color32::from_rgb(126, 92, 199);
    let red = Color32::from_rgb(214, 48, 49);
    match system {
        "Super Nintendo" => {
            p.rect_filled(body, 12.0, gray);
            p.rect_filled(
                Rect::from_min_max(
                    Pos2::new(body.min.x, body.max.y - 16.0),
                    body.max,
                ),
                6.0,
                Color32::from_rgb(168, 168, 182),
            );
            for (dx, dy, c) in [
                (38.0, -12.0, purple),
                (56.0, -12.0, Color32::from_rgb(170, 152, 235)),
                (38.0, 6.0, Color32::from_rgb(170, 152, 235)),
                (56.0, 6.0, purple),
            ] {
                p.circle_filled(Pos2::new(cx + dx, cy + dy), 8.0, c);
            }
        }
        "Nintendo Entertainment System" => {
            p.rect_filled(body, 6.0, Color32::from_rgb(196, 196, 200));
            p.rect_filled(
                Rect::from_center_size(
                    Pos2::new(cx, cy),
                    Vec2::new(w, 14.0),
                ),
                2.0,
                Color32::from_rgb(40, 40, 44),
            );
            p.rect_filled(
                Rect::from_center_size(
                    Pos2::new(cx, cy + 3.0),
                    Vec2::new(w, 3.0),
                ),
                1.0,
                Color32::from_rgb(198, 40, 40),
            );
        }
        "Sega Genesis" => {
            p.rect_filled(body, 12.0, dark);
            p.circle_stroke(
                Pos2::new(cx, cy - 2.0),
                22.0,
                egui::Stroke::new(3.0_f32, white),
            );
            p.circle_filled(Pos2::new(cx + 58.0, cy + 18.0), 4.0, red);
        }
        "Nintendo GameCube" => {
            let cube = Rect::from_center_size(
                Pos2::new(cx, cy),
                Vec2::new(74.0, 66.0),
            );
            p.rect_filled(cube, 8.0, Color32::from_rgb(92, 84, 186));
            p.circle_stroke(
                Pos2::new(cx, cy),
                22.0,
                egui::Stroke::new(3.0_f32, Color32::from_rgb(52, 48, 120)),
            );
        }
        "Nintendo Game Boy" | "Nintendo Game Boy Color" => {
            let tint = if system.contains("Color") {
                Color32::from_rgb(64, 176, 186)
            } else {
                Color32::from_rgb(203, 203, 203)
            };
            let tall = Rect::from_center_size(
                Pos2::new(cx, cy),
                Vec2::new(62.0, 80.0),
            );
            p.rect_filled(tall, 8.0, tint);
            p.rect_filled(
                Rect::from_center_size(
                    Pos2::new(cx, cy - 16.0),
                    Vec2::new(44.0, 32.0),
                ),
                2.0,
                Color32::from_rgb(168, 184, 140),
            );
            p.circle_filled(
                Pos2::new(cx + 14.0, cy + 18.0),
                6.0,
                Color32::from_rgb(198, 40, 88),
            );
            p.circle_filled(
                Pos2::new(cx + 24.0, cy + 27.0),
                5.0,
                Color32::from_rgb(198, 40, 88),
            );
        }
        "Sony PlayStation 2" => {
            let slab = Rect::from_center_size(
                Pos2::new(cx, cy),
                Vec2::new(110.0, 70.0),
            );
            p.rect_filled(slab, 6.0, Color32::from_rgb(18, 18, 22));
            let mut x = slab.min.x + 10.0;
            while x < slab.max.x - 6.0 {
                p.line_segment(
                    [
                        Pos2::new(x, slab.min.y + 6.0),
                        Pos2::new(x, slab.max.y - 6.0),
                    ],
                    egui::Stroke::new(3.0_f32, Color32::from_rgb(70, 70, 84)),
                );
                x += 11.0;
            }
        }
        "Sony PlayStation" | "Sega Saturn" | "3DO" => {
            p.rect_filled(body, 10.0, gray);
            p.circle_stroke(
                Pos2::new(cx, cy),
                22.0,
                egui::Stroke::new(3.0_f32, Color32::from_rgb(90, 90, 98)),
            );
        }
        "Sony PlayStation 3" => {
            p.rect_filled(body, 14.0, Color32::from_rgb(16, 16, 20));
            p.rect_filled(
                Rect::from_center_size(
                    Pos2::new(cx, cy - 8.0),
                    Vec2::new(w, 6.0),
                ),
                2.0,
                Color32::from_rgb(170, 172, 180),
            );
        }
        "Microsoft Xbox" => {
            p.rect_filled(body, 10.0, Color32::from_rgb(20, 20, 22));
            let green = Color32::from_rgb(52, 178, 68);
            p.circle_stroke(Pos2::new(cx, cy), 20.0,
                            egui::Stroke::new(4.0_f32, green));
            p.line_segment(
                [Pos2::new(cx - 12.0, cy - 12.0),
                 Pos2::new(cx + 12.0, cy + 12.0)],
                egui::Stroke::new(4.0_f32, green),
            );
            p.line_segment(
                [Pos2::new(cx + 12.0, cy - 12.0),
                 Pos2::new(cx - 12.0, cy + 12.0)],
                egui::Stroke::new(4.0_f32, green),
            );
        }
        "Nintendo 64" => {
            p.rect_filled(body, 10.0, Color32::from_rgb(52, 52, 60));
            let cols = [
                Color32::from_rgb(66, 175, 84),
                Color32::from_rgb(230, 199, 62),
                Color32::from_rgb(62, 110, 220),
                Color32::from_rgb(214, 64, 64),
            ];
            for (i, c) in cols.iter().enumerate() {
                p.circle_filled(
                    Pos2::new(cx - 45.0 + i as f32 * 30.0, cy + 14.0),
                    6.0,
                    *c,
                );
            }
        }
        "Arcade" => {
            p.rect_filled(
                Rect::from_center_size(Pos2::new(cx, cy), Vec2::new(80.0, 82.0)),
                6.0,
                Color32::from_rgb(40, 40, 90),
            );
            p.rect_filled(
                Rect::from_center_size(
                    Pos2::new(cx, cy - 28.0),
                    Vec2::new(66.0, 16.0),
                ),
                2.0,
                Color32::from_rgb(240, 190, 60),
            );
            p.rect_filled(
                Rect::from_center_size(
                    Pos2::new(cx, cy + 2.0),
                    Vec2::new(58.0, 26.0),
                ),
                2.0,
                Color32::from_rgb(90, 200, 220),
            );
            p.circle_filled(Pos2::new(cx - 20.0, cy + 26.0), 5.0, red);
        }
        "Commodore 64" | "Commodore Amiga" => {
            p.rect_filled(
                body,
                6.0,
                Color32::from_rgb(216, 200, 168),
            );
            for r in 0..3 {
                for c in 0..9 {
                    p.rect_filled(
                        Rect::from_min_size(
                            Pos2::new(
                                body.min.x + 12.0 + c as f32 * 15.0,
                                body.min.y + 12.0 + r as f32 * 13.0,
                            ),
                            Vec2::new(9.0, 7.0),
                        ),
                        1.0,
                        Color32::from_rgb(120, 106, 84),
                    );
                }
            }
        }
        "PC Ports" => {
            // monitor with a play screen
            p.rect_filled(body, 10.0, dark);
            p.rect_filled(
                Rect::from_center_size(
                    Pos2::new(cx, cy - 7.0),
                    Vec2::new(126.0, 40.0),
                ),
                4.0,
                purple,
            );
            let tri = vec![
                Pos2::new(cx - 8.0, cy - 18.0),
                Pos2::new(cx - 8.0, cy + 4.0),
                Pos2::new(cx + 13.0, cy - 7.0),
            ];
            p.add(egui::Shape::convex_polygon(
                tri,
                white,
                egui::Stroke::NONE,
            ));
            p.rect_filled(
                Rect::from_center_size(
                    Pos2::new(cx, cy + 19.0),
                    Vec2::new(34.0, 5.0),
                ),
                2.0,
                gray,
            );
        }
        _ => {
            // generic gamepad tile in a per-system color
            let mut hasher = Sha1::new();
            hasher.update(system.as_bytes());
            let h = hasher.finalize();
            let col = Color32::from_rgb(
                90 + h[0] % 120,
                80 + h[1] % 110,
                120 + h[2] % 110,
            );
            p.rect_filled(body, 14.0, col);
            let pad = Rect::from_center_size(
                Pos2::new(cx, cy),
                Vec2::new(76.0, 34.0),
            );
            p.rect_filled(pad, 14.0, white);
            p.circle_filled(Pos2::new(cx + 20.0, cy - 4.0), 5.0, col);
            p.circle_filled(Pos2::new(cx + 30.0, cy + 4.0), 5.0, col);
        }
    }
    p.text(
        Pos2::new(cx, r.max.y - 16.0),
        egui::Align2::CENTER_CENTER,
        short_name(system),
        egui::FontId::proportional(17.0),
        Color32::from_rgb(240, 240, 248),
    );
}

// ---------------- config read/write (shared with the Python app) ----
fn cfg_path() -> PathBuf {
    data_dir().join("emufront_config.json")
}

fn load_cfg() -> serde_json::Value {
    let text = std::fs::read_to_string(cfg_path()).unwrap_or_default();
    serde_json::from_str(&text).unwrap_or(serde_json::json!({}))
}

fn save_cfg(cfg: &serde_json::Value) -> Result<(), String> {
    let tmp = data_dir().join("emufront_config.json.tmp");
    let text = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, cfg_path()).map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------- config writes: queued + guarded (v1.05) ------------
// Design §2.5: the write DISCIPLINE is unchanged — every
// write still starts from a fresh read of the authoritative
// file and lands via save_cfg's tmp+rename; no in-RAM copy
// ever becomes authoritative. What changed is WHEN and
// WHERE: all field-level writes now go through one
// background worker, so a pick never freezes the UI while
// a tens-of-MB file is read, pretty-printed and written
// on a slow disk — and each job runs the changed-under-us
// guard below, so a Python-side edit landing mid-write is
// re-read and re-applied onto, never blindly overwritten.
// (The one direct writer left is the topbar's config
// IMPORT: an explicit, user-driven wholesale replacement
// with its own backup — a different operation from this
// field-mutation family, and queued jobs are guarded
// against it exactly like against a Python edit.)

/// One config mutation, in data form. `apply` carries
/// exactly the semantics the old synchronous helpers
/// had — --selftest pins each one against fixtures.
enum CfgMutation {
    SetGameField {
        path: String,
        key: String,
        val: serde_json::Value,
    },
    /// A game's "Launch with:" pick: the emulator name
    /// plus the hand-lock flag in one write. A non-empty
    /// pick is a deliberate hand choice (locked, and
    /// honored first by pick_profile); "" is Auto
    /// (unlocked). Python parity: without the flag, Rust
    /// hand-picks would be indistinguishable from stale
    /// scan bakes.
    SetGameEmulator {
        path: String,
        name: String,
    },
    RegisterManual {
        entry: serde_json::Value,
    },
    ReplaceGames {
        games: serde_json::Value,
    },
    ReplaceEmulators {
        profiles: serde_json::Value,
    },
    ReplaceLibraryDirs {
        dirs: serde_json::Value,
    },
}

impl CfgMutation {
    fn apply(
        &self,
        cfg: &mut serde_json::Value,
    ) -> Result<(), String> {
        match self {
            CfgMutation::SetGameField { path, key, val } => {
                for arr_key in ["games", "manual_games"] {
                    if let Some(arr) =
                        cfg.get_mut(arr_key).and_then(|x| x.as_array_mut())
                    {
                        for g in arr.iter_mut() {
                            if g.get("path").and_then(|x| x.as_str())
                                == Some(path.as_str())
                            {
                                g[key.as_str()] = val.clone();
                            }
                        }
                    }
                }
                Ok(())
            }
            CfgMutation::SetGameEmulator { path, name } => {
                for arr_key in ["games", "manual_games"] {
                    if let Some(arr) =
                        cfg.get_mut(arr_key).and_then(|x| x.as_array_mut())
                    {
                        for g in arr.iter_mut() {
                            if g.get("path").and_then(|x| x.as_str())
                                == Some(path.as_str())
                            {
                                g["emulator"] =
                                    serde_json::Value::String(
                                        name.clone(),
                                    );
                                g["emulator_locked"] =
                                    serde_json::Value::Bool(
                                        !name.is_empty(),
                                    );
                            }
                        }
                    }
                }
                Ok(())
            }
            CfgMutation::RegisterManual { entry } => {
                let path = entry
                    .get("path")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                for arr_key in ["games", "manual_games"] {
                    let arr = cfg
                        .get_mut(arr_key)
                        .and_then(|x| x.as_array_mut())
                        .ok_or("config missing games list")?;
                    if !arr.iter().any(|g| {
                        g.get("path").and_then(|x| x.as_str())
                            == Some(path.as_str())
                    }) {
                        arr.push(entry.clone());
                    }
                }
                Ok(())
            }
            CfgMutation::ReplaceGames { games } => {
                cfg["games"] = games.clone();
                Ok(())
            }
            CfgMutation::ReplaceEmulators { profiles } => {
                cfg["emulators"] = profiles.clone();
                Ok(())
            }
            CfgMutation::ReplaceLibraryDirs { dirs } => {
                cfg["library_dirs"] = dirs.clone();
                Ok(())
            }
        }
    }
}

/// The guard's decision for one write attempt: the file
/// didn't move — write; it moved on the first attempt —
/// reload, re-apply, retry once; it moved again — stop
/// and report honestly instead of forcing the write.
/// Pure — --selftest pins the table.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WriteDecision {
    Write,
    Retry,
    GiveUp,
}

fn write_decide(attempt: u32, moved: bool) -> WriteDecision {
    if !moved {
        WriteDecision::Write
    } else if attempt == 0 {
        WriteDecision::Retry
    } else {
        WriteDecision::GiveUp
    }
}

/// One guarded write, end to end: read the file (stamping
/// it before the read), apply the mutation, stamp again
/// before the rename — if size/mtime moved since the
/// read, someone else (the Python app, an import) edited
/// it in between: reload, re-apply onto the fresh file,
/// retry once. A second collision reports honestly and
/// writes nothing.
fn cfg_write_guarded(m: &CfgMutation) -> Result<String, String> {
    let mut attempt = 0u32;
    loop {
        let before = cfg_stat();
        let mut cfg = load_cfg();
        // Semantic errors (a missing games list) surface
        // at once, exactly as the old helpers reported
        // them — retrying can't fix them.
        m.apply(&mut cfg)?;
        let pre_rename = cfg_stat();
        match write_decide(attempt, before != pre_rename) {
            WriteDecision::Write => {
                save_cfg(&cfg)?;
                return Ok(if attempt == 0 {
                    "saved".to_string()
                } else {
                    "saved (re-applied onto the changed config)"
                        .to_string()
                });
            }
            WriteDecision::Retry => attempt += 1,
            WriteDecision::GiveUp => {
                return Err(
                    "the config changed again while saving (the other app is editing it) — this change was not saved; please try again"
                        .to_string(),
                );
            }
        }
    }
}

/// A queued write. UI submissions carry a tag and get
/// their result on the results channel; thread/CLI
/// callers carry a one-shot reply instead.
struct CfgJob {
    m: CfgMutation,
    reply: Option<Sender<Result<String, String>>>,
    tag: Option<CfgTag>,
}

/// What a UI-queued write was for — drain_cfg keys its
/// follow-up (reload, messages, rescan) off this.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CfgTag {
    /// Panel picks and cheat saves: silent on success,
    /// the status line reports a failure.
    Quiet,
    /// Manage Emulators save: reload on success.
    EmulatorProfiles,
    /// Library Folders save: message, plus the rescan
    /// the user asked for, on success.
    LibraryDirs { rescan: bool },
}

struct CfgResult {
    tag: CfgTag,
    result: Result<String, String>,
}

static CFG_JOB_TX: std::sync::OnceLock<Sender<CfgJob>> =
    std::sync::OnceLock::new();
static CFG_RESULT_TX: std::sync::OnceLock<Sender<CfgResult>> =
    std::sync::OnceLock::new();

/// The write worker: one thread, jobs in submission
/// order (two halves of a cheats save can never
/// interleave with another write — an improvement on
/// the old two-separate-saves behavior, and the reason
/// they stay ordered).
fn cfg_submit(job: CfgJob) {
    let tx = CFG_JOB_TX.get_or_init(|| {
        let (tx, rx) = channel::<CfgJob>();
        std::thread::spawn(move || {
            while let Ok(job) = rx.recv() {
                let result = cfg_write_guarded(&job.m);
                if let Some(reply) = job.reply {
                    let _ = reply.send(result);
                } else if let Some(tag) = job.tag {
                    if let Some(rtx) = CFG_RESULT_TX.get() {
                        let _ = rtx.send(CfgResult { tag, result });
                    }
                }
            }
        });
        tx
    });
    let _ = tx.send(job);
}

/// Proto registers its results receiver once, at startup.
fn cfg_register_results(tx: Sender<CfgResult>) {
    let _ = CFG_RESULT_TX.set(tx);
}

/// Blocking form for background threads and the CLI
/// (their callers were already off the UI thread or
/// headless): enqueue and wait for the worker's verdict.
fn cfg_write_sync(m: CfgMutation) -> Result<String, String> {
    let (rtx, rrx) = channel::<Result<String, String>>();
    cfg_submit(CfgJob {
        m,
        reply: Some(rtx),
        tag: None,
    });
    rrx.recv()
        .unwrap_or_else(|_| Err("config write queue closed".to_string()))
}

fn register_manual_game(
    entry: serde_json::Value,
) -> Result<(), String> {
    cfg_write_sync(CfgMutation::RegisterManual { entry }).map(|_| ())
}

// ---------------- naming (ports of the Python rules) ----------------
pub fn strip_catalog_prefix_full(name: &str) -> String {
    let s = name;
    let b = s.as_bytes();
    let digit = |c: u8| c.is_ascii_digit();
    // 'NN-NN Title'
    {
        let mut i = 0;
        while i < b.len() && digit(b[i]) && i < 4 {
            i += 1;
        }
        if i >= 1 && i < s.len() && b[i] == b'-' {
            let mut j = i + 1;
            let start = j;
            while j < b.len() && digit(b[j]) && j - start < 4 {
                j += 1;
            }
            if j > start && j < s.len() && b[j] == b' ' {
                let rest = &s[j + 1..];
                if rest.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
                    return rest.to_string();
                }
            }
        }
    }
    // up to two groups of 'NNNN ' or 'NNNN. '
    let mut idx = 0;
    for _ in 0..2 {
        let mut i = idx;
        let mut n = 0;
        while i < b.len() && digit(b[i]) && n < 4 {
            i += 1;
            n += 1;
        }
        if n == 0 {
            break;
        }
        if i < s.len() && b[i] == b'.' && i + 1 < s.len() && b[i + 1] == b' ' {
            let after = i + 2;
            if s[after..]
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or(false)
            {
                idx = after;
                continue;
            }
        }
        if i < s.len() && b[i] == b' ' {
            let after = i + 1;
            if s[after..]
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or(false)
            {
                idx = after;
                continue;
            }
        }
        break;
    }
    if idx > 0 {
        return s[idx..].to_string();
    }
    if (s.starts_with("007") || s.starts_with("1942")) && idx == 0 {
        return s.to_string();
    }
    s.to_string()
}

fn clean_game_name(filename: &str) -> String {
    let base = filename.replace('\\', "/");
    let base = base.rsplit('/').next().unwrap_or("").to_string();
    let stem = match base.rfind('.') {
        Some(i) if i > 0 => base[..i].to_string(),
        _ => base,
    };
    let mut name = strip_catalog_prefix_full(&stem);
    // [tags] removed
    let mut out = String::new();
    let mut depth = 0i32;
    for c in name.chars() {
        match c {
            '[' => depth += 1,
            ']' => depth = (depth - 1).max(0),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    name = out;
    // (tags): region/rev/year groups dropped, others kept inline
    let mut out = String::new();
    let mut rest = name.as_str();
    while let Some(open) = rest.find('(') {
        out.push_str(&rest[..open]);
        if let Some(close) = rest[open..].find(')') {
            let inner = &rest[open + 1..open + close];
            let drop_it = ["USA", "EUR", "JPN", "Japan", "Europe", "World",
                "Rev", "Beta", "Proto", "Demo"]
                .iter()
                .any(|k| inner.contains(k))
                || inner.chars().any(|c| c.is_ascii_digit())
                    && inner.len() <= 6
                    && inner.chars().all(|c| c.is_ascii_digit() || c == ' ');
            if !drop_it {
                out.push(' ');
                out.push_str(inner);
                out.push(' ');
            } else {
                out.push(' ');
            }
            rest = &rest[open + close + 1..];
        } else {
            break;
        }
    }
    out.push_str(rest);
    let cleaned: String = out
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '\'' || c == '-' || c == '.' || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ").trim_matches(|c| c == ' ' || c == '.').to_string()
}

fn safe_folder(name: &str) -> String {
    name.chars()
        .filter(|c| !"<>:\"/\\|?*".contains(*c))
        .collect::<String>()
        .trim()
        .to_string()
}

fn hacked_dir(game: &Game, enhanced: bool) -> PathBuf {
    let base = data_dir().join(if enhanced { "Enhanced ROMs" } else { "Hacked ROMs" });
    let sys = safe_folder(&game.system);
    let g = safe_folder(&game.name);
    if sys.is_empty() {
        base.join(if g.is_empty() { "Game".into() } else { g })
    } else {
        base.join(sys).join(if g.is_empty() { "Game".into() } else { g })
    }
}

// ---------------- CRC32 + IPS/UPS/BPS patchers ----------------------
fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for i in 0..256u32 {
        let mut c = i;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB88320 ^ (c >> 1) } else { c >> 1 };
        }
        table[i as usize] = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc = table[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

fn apply_ips(rom: &[u8], patch: &[u8]) -> Result<Vec<u8>, String> {
    if patch.len() < 5 || &patch[..5] != b"PATCH" {
        return Err("Not an IPS patch.".into());
    }
    let mut out: Vec<u8> = rom.to_vec();
    let mut i = 5usize;
    let mut truncate: Option<usize> = None;
    loop {
        if patch[i..].starts_with(b"EOF") {
            i += 3;
            if i + 3 <= patch.len() {
                truncate = Some(
                    ((patch[i] as usize) << 16)
                        | ((patch[i + 1] as usize) << 8)
                        | patch[i + 2] as usize,
                );
            }
            break;
        }
        if i + 5 > patch.len() {
            return Err("Corrupt IPS patch (record overrun).".into());
        }
        let offset = ((patch[i] as usize) << 16)
            | ((patch[i + 1] as usize) << 8)
            | patch[i + 2] as usize;
        let size = ((patch[i + 3] as usize) << 8) | patch[i + 4] as usize;
        i += 5;
        let data: Vec<u8>;
        if size == 0 {
            if i + 3 > patch.len() {
                return Err("Corrupt IPS patch (RLE overrun).".into());
            }
            let count = ((patch[i] as usize) << 8) | patch[i + 1] as usize;
            data = vec![patch[i + 2]; count];
            i += 3;
        } else {
            if i + size > patch.len() {
                return Err("Corrupt IPS patch (data overrun).".into());
        }
            data = patch[i..i + size].to_vec();
            i += size;
        }
        if offset + data.len() > out.len() {
            out.resize(offset + data.len(), 0);
        }
        out[offset..offset + data.len()].copy_from_slice(&data);
    }
    if let Some(t) = truncate {
        out.truncate(t);
    }
    Ok(out)
}

fn ups_varint(buf: &[u8], mut i: usize) -> (usize, usize) {
    let mut value: usize = 0;
    let mut shift: usize = 1;
    loop {
        let x = buf[i] as usize;
        i += 1;
        value += (x & 0x7F) * shift;
        if x & 0x80 != 0 {
            break;
        }
        shift <<= 7;
        value += shift;
    }
    (value, i)
}

fn apply_ups(rom: &[u8], patch: &[u8]) -> Result<Vec<u8>, String> {
    if patch.len() < 16 || &patch[..4] != b"UPS1" {
        return Err("Not a UPS patch.".into());
    }
    let mut i = 4usize;
    let (src_size, ni) = ups_varint(patch, i);
    i = ni;
    let (dst_size, ni) = ups_varint(patch, i);
    i = ni;
    if src_size != rom.len() {
        return Err(format!(
            "UPS patch expects a {src_size}-byte ROM, yours is {} bytes (wrong ROM version).",
            rom.len()
        ));
    }
    let mut out = vec![0u8; dst_size];
    let mut pos = 0usize;
    while i < patch.len().saturating_sub(12) {
        let (skip, ni) = ups_varint(patch, i);
        i = ni;
        for _ in 0..skip {
            if pos >= dst_size {
                break;
            }
            out[pos] = if pos < rom.len() { rom[pos] } else { 0 };
            pos += 1;
        }
        if pos > dst_size {
            break;
        }
        let start = i;
        while i < patch.len() && patch[i] != 0 {
            i += 1;
        }
        i += 1;
        let block = &patch[start..i.min(patch.len())];
        for (j, b) in block.iter().enumerate() {
            if pos + j >= dst_size {
                break;
            }
            let orig = if pos + j < rom.len() { rom[pos + j] } else { 0 };
            out[pos + j] = orig ^ b;
        }
        pos += block.len();
    }
    for k in pos..dst_size.min(rom.len()) {
        out[k] = rom[k];
    }
    let tail = &patch[patch.len() - 12..];
    let src_crc = u32::from_le_bytes([tail[0], tail[1], tail[2], tail[3]]);
    let dst_crc = u32::from_le_bytes([tail[4], tail[5], tail[6], tail[7]]);
    if crc32(rom) != src_crc {
        return Err("UPS source CRC mismatch (wrong ROM version).".into());
    }
    if crc32(&out) != dst_crc {
        return Err("UPS result CRC mismatch (patch failed).".into());
    }
    Ok(out)
}

fn bps_signed(buf: &[u8], i: usize) -> (i64, usize) {
    let (raw, ni) = ups_varint(buf, i);
    let v = if raw & 1 == 1 {
        -((raw >> 1) as i64)
    } else {
        (raw >> 1) as i64
    };
    (v, ni)
}

fn apply_bps(rom: &[u8], patch: &[u8]) -> Result<Vec<u8>, String> {
    if patch.len() < 16 || &patch[..4] != b"BPS1" {
        return Err("Not a BPS patch.".into());
    }
    let mut i = 4usize;
    let (src_size, ni) = ups_varint(patch, i);
    i = ni;
    let (dst_size, ni) = ups_varint(patch, i);
    i = ni;
    let (meta_len, ni) = ups_varint(patch, i);
    i = ni + meta_len;
    if src_size != rom.len() {
        return Err(format!(
            "BPS patch expects a {src_size}-byte ROM, yours is {} bytes (wrong ROM version).",
            rom.len()
        ));
    }
    let mut out: Vec<u8> = Vec::with_capacity(dst_size);
    let mut src_rel: i64 = 0;
    let mut dst_rel: i64 = 0;
    let end = patch.len() - 12;
    while i < end {
        let (cmd, ni) = ups_varint(patch, i);
        i = ni;
        let action = cmd & 3;
        let length = (cmd >> 2) + 1;
        match action {
            0 => {
                for _ in 0..length {
                    let idx = out.len();
                    out.push(if idx < rom.len() { rom[idx] } else { 0 });
                }
            }
            1 => {
                out.extend_from_slice(&patch[i..(i + length).min(patch.len())]);
                i += length;
            }
            2 => {
                let (off, ni) = bps_signed(patch, i);
                i = ni;
                src_rel += off;
                for _ in 0..length {
                    out.push(if src_rel >= 0 && (src_rel as usize) < rom.len() {
                        rom[src_rel as usize]
                    } else {
                        0
                    });
                    src_rel += 1;
                }
            }
            _ => {
                let (off, ni) = bps_signed(patch, i);
                i = ni;
                dst_rel += off;
                for _ in 0..length {
                    out.push(if dst_rel >= 0 && (dst_rel as usize) < out.len() {
                        out[dst_rel as usize]
                    } else {
                        0
                    });
                    dst_rel += 1;
                }
            }
        }
        if out.len() > dst_size {
            return Err("BPS patch produced too much data.".into());
        }
    }
    let tail = &patch[patch.len() - 12..];
    let src_crc = u32::from_le_bytes([tail[0], tail[1], tail[2], tail[3]]);
    let dst_crc = u32::from_le_bytes([tail[4], tail[5], tail[6], tail[7]]);
    if crc32(rom) != src_crc {
        return Err("BPS source CRC mismatch (wrong ROM version).".into());
    }
    if out.len() != dst_size || crc32(&out) != dst_crc {
        return Err("BPS result CRC mismatch (patch failed).".into());
    }
    Ok(out)
}

fn apply_patch(rom: &[u8], patch: &[u8], ext: &str) -> Result<Vec<u8>, String> {
    match ext.to_ascii_lowercase().as_str() {
        ".ips" => apply_ips(rom, patch),
        ".ups" => apply_ups(rom, patch),
        ".bps" => apply_bps(rom, patch),
        other => Err(format!("{other} patches aren't supported yet.")),
    }
}

// ---------------- library scan (port of scan_library) ---------------
const ROM_EXTS: &[&str] = &[
    ".nes", ".smc", ".sfc", ".fig", ".gb", ".gbc", ".gba", ".n64", ".z64",
    ".v64", ".md", ".gen", ".smd", ".sms", ".gg", ".pce", ".a26", ".a78",
    ".lnx", ".ws", ".wsc", ".ngp", ".ngc", ".iso", ".bin", ".cue", ".chd",
    ".pbp", ".cso", ".nds", ".3ds", ".cci", ".wad", ".wbfs", ".xbe", ".nsp",
    ".xci", ".adf", ".d64", ".tap", ".tzx", ".col", ".int", ".vec", ".rom",
    ".zip", ".7z", ".m3u",
];

fn system_from_path(path: &Path, root: &Path) -> String {
    if let Ok(rel) = path.parent().unwrap_or(Path::new("")).strip_prefix(root) {
        let comps: Vec<String> = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        if !comps.is_empty() {
            return comps[0].clone();
        }
    }
    "Unknown".to_string()
}

fn scan_library() -> Result<Vec<serde_json::Value>, String> {
    let cfg = load_cfg();
    let old: std::collections::HashMap<String, serde_json::Value> = cfg
        .get("games")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|g| {
                    g.get("path")
                        .and_then(|x| x.as_str())
                        .map(|p| (p.to_lowercase(), g.clone()))
                })
                .collect()
        })
        .unwrap_or_default();
    let mut games: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();
    let roots: Vec<String> = cfg
        .get("library_dirs")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    for root_s in &roots {
        let root = Path::new(root_s);
        if !root.is_dir() {
            continue;
        }
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else {
                continue;
            };
            let mut files: Vec<PathBuf> = Vec::new();
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else {
                    files.push(p);
                }
            }
            // .m3u playlists collapse their discs into one entry
            let mut covered: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for f in &files {
                if f.extension().map(|x| x.to_string_lossy().to_lowercase())
                    == Some("m3u".into())
                {
                    if let Ok(text) = std::fs::read_to_string(f) {
                        for ln in text.lines() {
                            let ln = ln.trim();
                            if !ln.is_empty() {
                                covered.insert(
                                    ln.replace('\\', "/")
                                        .rsplit('/')
                                        .next()
                                        .unwrap_or("")
                                        .to_lowercase(),
                                );
                            }
                        }
                    }
                    let key = f.to_string_lossy().to_lowercase();
                    games.insert(
                        key,
                        serde_json::json!({
                            "name": clean_game_name(&f.file_name().unwrap_or_default().to_string_lossy()),
                            "path": f.to_string_lossy(),
                            "system": system_from_path(f, root),
                            "emulator": "",
                            "patch": "",
                        }),
                    );
                }
            }
            for f in &files {
                let ext = f
                    .extension()
                    .map(|x| format!(".{}", x.to_string_lossy().to_lowercase()))
                    .unwrap_or_default();
                let fname = f
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();
                if ROM_EXTS.contains(&ext.as_str())
                    && ext != ".m3u"
                    && !covered.contains(&fname)
                {
                    let key = f.to_string_lossy().to_lowercase();
                    games.insert(
                        key,
                        serde_json::json!({
                            "name": clean_game_name(&f.file_name().unwrap_or_default().to_string_lossy()),
                            "path": f.to_string_lossy(),
                            "system": system_from_path(f, root),
                            "emulator": "",
                            "patch": "",
                        }),
                    );
                }
            }
        }
    }
    if let Some(arr) = cfg.get("manual_games").and_then(|x| x.as_array()) {
        for g in arr {
            if let Some(p) = g.get("path").and_then(|x| x.as_str()) {
                if !p.is_empty() {
                    games.insert(p.to_lowercase(), g.clone());
                }
            }
        }
    }
    let mut out: Vec<serde_json::Value> = games.into_values().collect();
    for g in out.iter_mut() {
        let key = g
            .get("path")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_lowercase();
        if let Some(o) = old.get(&key) {
            for k in ["emulator", "emulator_locked", "patch", "cheats_text", "cheats_enabled", "name"] {
                if let Some(v) = o.get(k) {
                    if k == "name" && v.as_str().unwrap_or("").is_empty() {
                        continue;
                    }
                    g[k] = v.clone();
                }
            }
        }
    }
    out.sort_by(|a, b| {
        let ka = format!(
            "{}{}",
            a.get("system").and_then(|x| x.as_str()).unwrap_or(""),
            sort_key(a.get("name").and_then(|x| x.as_str()).unwrap_or(""))
        );
        let kb = format!(
            "{}{}",
            b.get("system").and_then(|x| x.as_str()).unwrap_or(""),
            sort_key(b.get("name").and_then(|x| x.as_str()).unwrap_or(""))
        );
        ka.cmp(&kb)
    });
    Ok(out)
}

// ---------------- compatibility (subset of EMU_COMPAT) ---------------
fn compat_for(profile: &Profile) -> Option<(u8, &'static str)> {
    let hay = format!("{} {}", profile.name, profile.core).to_lowercase();
    let table: &[(&str, u8, &str)] = &[
        ("bsnes", 5, "bsnes — the most accurate SNES emulation"),
        ("mesen", 5, "Mesen — best-in-class NES accuracy"),
        ("duckstation", 5, "DuckStation — best-in-class PlayStation"),
        ("mednafen_psx", 5, "Beetle PSX — near-perfect PlayStation"),
        ("beetle_psx", 5, "Beetle PSX — near-perfect PlayStation"),
        ("mednafen_saturn", 5, "Mednafen — the best Saturn emulation"),
        ("beetle_saturn", 5, "Beetle Saturn — the best Saturn emulation"),
        ("sameboy", 5, "SameBoy — extremely accurate Game Boy"),
        ("mgba", 5, "mGBA — excellent Game Boy Advance"),
        ("snes9x", 4, "Snes9x — fast, very compatible SNES"),
        ("genesis_plus_gx", 5, "Genesis Plus GX — excellent Genesis/Sega CD"),
        ("blastem", 4, "BlastEm — very accurate Genesis"),
        ("fceumm", 4, "FCEUmm — solid all-round NES"),
        ("nestopia", 4, "Nestopia — accurate NES"),
        ("gambatte", 4, "Gambatte — accurate Game Boy Color"),
        ("melonds", 4, "melonDS — strong Nintendo DS"),
        ("desmume", 3, "DeSmuME — older DS emulator"),
        ("mupen64plus", 4, "Mupen64Plus — solid Nintendo 64"),
        ("parallel_n64", 4, "ParaLLEl N64 — accuracy-focused N64"),
        ("project64", 3, "Project64 — easy N64, some accuracy gaps"),
        ("dolphin", 4, "Dolphin — excellent GameCube/Wii (standalone recommended)"),
        ("pcsx2", 4, "PCSX2 — the PlayStation 2 standard"),
        ("rpcs3", 3, "RPCS3 — PS3 emulation is demanding + partial"),
        ("ppsspp", 5, "PPSSPP — excellent PSP"),
        ("fbneo", 4, "FinalBurn Neo — strong arcade/Neo Geo"),
        ("mame", 4, "MAME — the arcade reference"),
        ("stella", 5, "Stella — the Atari 2600 standard"),
        ("mednafen", 4, "Mednafen — accurate multi-system"),
        ("kega", 4, "Kega Fusion — classic Sega all-rounder"),
        ("zsnes", 2, "ZSNES — beloved but ancient; accuracy issues today"),
        ("ares", 5, "ares — top-tier multi-system accuracy"),
        ("azahar", 4, "Azahar — leading 3DS emulator"),
        ("citra", 4, "Citra — solid 3DS"),
        ("vice", 4, "VICE — the Commodore 64 standard"),
        ("winuae", 4, "WinUAE — the Amiga standard"),
        ("opera", 4, "Opera — solid 3DO"),
        ("4do", 3, "4DO — decent 3DO"),
        ("redream", 4, "Redream — strong Dreamcast"),
        ("flycast", 4, "Flycast — strong Dreamcast"),
        ("xemu", 3, "xemu — Xbox emulation still maturing"),
    ];
    for (frag, stars, note) in table {
        if hay.contains(frag) {
            return Some((*stars, note));
        }
    }
    None
}

// ---------------- cheats --------------------------------------------
fn parse_cheat_lines(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (mut desc, mut code) = (String::new(), line.to_string());
        if let Some(i) = line.find('|') {
            desc = line[..i].trim().to_string();
            code = line[i + 1..].trim().to_string();
        } else if let Some(i) = line.find('=') {
            let head = line[..i].trim();
            let codey = !head.is_empty()
                && head
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() || ":+- ".contains(c));
            if !codey {
                desc = head.to_string();
                code = line[i + 1..].trim().to_string();
            }
        }
        if desc.is_empty() {
            desc = format!("Cheat {}", out.len() + 1);
        }
        if !code.is_empty() {
            out.push((desc, code));
        }
    }
    out
}

fn write_cht(game: &Game) -> Option<PathBuf> {
    let cheats = parse_cheat_lines(&game.cheats_text);
    if cheats.is_empty() {
        return None;
    }
    let dir = data_dir().join("Cheats");
    let _ = std::fs::create_dir_all(&dir);
    let stem = Path::new(&game.path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let path = dir.join(format!("{stem}.cht"));
    let mut lines = vec![format!("cheats = {}", cheats.len())];
    for (i, (d, c)) in cheats.iter().enumerate() {
        lines.push(format!(
            "cheat{i}_desc = \"{}\"",
            d.replace('"', "'")
        ));
        lines.push(format!(
            "cheat{i}_code = \"{}\"",
            c.replace('"', "")
        ));
        lines.push(format!("cheat{i}_enable = true"));
    }
    std::fs::write(&path, lines.join("\n") + "\n").ok()?;
    Some(path)
}

fn parse_libretro_cht(text: &str) -> Vec<(String, String)> {
    let mut descs: std::collections::HashMap<usize, String> = Default::default();
    let mut codes: std::collections::HashMap<usize, String> = Default::default();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("cheat") {
            if let Some(eq) = rest.find('=') {
                let lhs = rest[..eq].trim();
                let val = rest[eq + 1..]
                    .trim()
                    .trim_matches('"')
                    .to_string();
                let mut parts = lhs.splitn(2, '_');
                if let (Some(n), Some(kind)) = (parts.next(), parts.next()) {
                    if let Ok(idx) = n.parse::<usize>() {
                        match kind {
                            "desc" => {
                                descs.insert(idx, val);
                            }
                            "code" => {
                                codes.insert(idx, val);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    let mut keys: Vec<usize> = codes.keys().cloned().collect();
    keys.sort();
    keys.iter()
        .filter_map(|i| {
            let code = codes.get(i)?.replace(' ', "");
            if code.is_empty() {
                None
            } else {
                Some((
                    descs
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| format!("Cheat {}", i + 1)),
                    code,
                ))
            }
        })
        .collect()
}

fn fetch_libretro_cheats(game: &Game) -> Vec<(String, String)> {
    let Some(repo) = libretro_repo(&game.system) else {
        return vec![];
    };
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(20))
        .build();
    for cand in art_candidates(game) {
        let url = format!(
            "https://raw.githubusercontent.com/libretro/libretro-database/master/cht/{}/{}.cht",
            url_encode(repo),
            url_encode(&cand)
        );
        if let Ok(resp) = agent.get(&url).call() {
            if let Ok(text) = resp.into_string() {
                let pairs = parse_libretro_cht(&text);
                if !pairs.is_empty() {
                    return pairs;
                }
            }
        }
    }
    vec![]
}


// ---------------- hack search + download/build chain -----------------
#[derive(Clone)]
struct HackEntry {
    title: String,
    url: String,
}

/// One shared ureq agent per timeout value, process-wide
/// (perf Slice 1, RELEASE-PERF-DESIGN §2.3/§2.6): an
/// Agent owns a connection pool, so building one per call
/// — the old shape — threw the pool away every time and
/// paid a fresh connection for nearly every fetch on slow
/// networks. Agents are cheap to clone and safe to share;
/// the per-call timeout policy is unchanged because the
/// cache is keyed by the timeout itself.
fn http_agent(secs: u64) -> ureq::Agent {
    static AGENTS: std::sync::OnceLock<
        std::sync::Mutex<HashMap<u64, ureq::Agent>>,
    > = std::sync::OnceLock::new();
    let cell = AGENTS.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    if let Ok(g) = cell.lock() {
        if let Some(a) = g.get(&secs) {
            return a.clone();
        }
    }
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(secs))
        .build();
    if let Ok(mut g) = cell.lock() {
        g.entry(secs).or_insert_with(|| agent.clone());
    }
    agent
}

// ---------------- session caches (perf Slice 1) -----------
// RELEASE-PERF-DESIGN §2.6, session tier: within one app
// run, a repeated Find hunt for the same game and a
// repeat open of the Enhanced catalog reuse the results
// already fetched instead of paying the network again.
// Session ONLY — nothing is persisted, so a hunt's
// freshness (its whole value; the Phantasy Star II
// lesson) survives across runs. Only NON-EMPTY results
// are stored: an empty result can mean "no such thing"
// or "the network hiccuped", and pinning the second
// reading for a whole session would hide real content
// behind one transient failure — empty fetches live
// again on the next ask.

fn session_cache_lookup<T: Clone>(
    cell: &std::sync::OnceLock<std::sync::Mutex<HashMap<String, Vec<T>>>>,
    key: &str,
) -> Option<Vec<T>> {
    let map = cell.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    map.lock().ok()?.get(key).cloned()
}

fn session_cache_store<T: Clone>(
    cell: &std::sync::OnceLock<std::sync::Mutex<HashMap<String, Vec<T>>>>,
    key: String,
    val: &[T],
) {
    if val.is_empty() {
        return;
    }
    let map = cell.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    if let Ok(mut g) = map.lock() {
        g.insert(key, val.to_vec());
    }
}

/// Hunts are keyed by the game's path: unique per
/// library entry, stable for the whole run.
fn hunt_cache_key(game: &Game) -> String {
    game.path.clone()
}

static HUNT_MSU_CACHE: std::sync::OnceLock<
    std::sync::Mutex<HashMap<String, Vec<HackEntry>>>,
> = std::sync::OnceLock::new();

static HUNT_PLAZA_CACHE: std::sync::OnceLock<
    std::sync::Mutex<HashMap<String, Vec<HackEntry>>>,
> = std::sync::OnceLock::new();

static CATALOG_CACHE: std::sync::OnceLock<
    std::sync::Mutex<HashMap<String, Vec<CatalogEntry>>>,
> = std::sync::OnceLock::new();

fn html_unescape(t: &str) -> String {
    t.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&apos;", "'")
}

fn strip_tags(t: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in t.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    html_unescape(out.trim())
}

fn search_plaza(game: &Game) -> Vec<HackEntry> {
    let key = hunt_cache_key(game);
    if let Some(v) = session_cache_lookup(&HUNT_PLAZA_CACHE, &key) {
        return v;
    }
    let out = search_plaza_query(&clean_game_name(&game.name));
    session_cache_store(&HUNT_PLAZA_CACHE, key, &out);
    out
}

fn search_plaza_query(q: &str) -> Vec<HackEntry> {
    let url = format!(
        "https://romhackplaza.org/database?s={}",
        url_encode(q)
    );
    let agent = http_agent(25);
    let Ok(resp) = agent
        .get(&url)
        .set(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36",
        )
        .call()
    else {
        return vec![];
    };
    let Ok(html) = resp.into_string() else {
        return vec![];
    };
    // anchors: href="https://romhackplaza.org/(romhacks|translations)/..."
    // title = the anchor's full inner HTML, tags stripped (mirrors
    // the Python parser, which is proven against this site)
    let mut out: Vec<HackEntry> = Vec::new();
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let needle = "href=\"https://romhackplaza.org/";
    let mut rest = html.as_str();
    while let Some(pos) = rest.find(needle) {
        let after = &rest[pos + needle.len()..];
        let Some(endq) = after.find('"') else { break };
        let path = &after[..endq];
        if !(path.starts_with("romhacks/") || path.starts_with("translations/")) {
            rest = &after[endq + 1..];
            continue;
        }
        let full = format!("https://romhackplaza.org/{path}");
        let tail = &after[endq + 1..];
        let title = tail
            .find('>')
            .and_then(|gt| {
                let inner = &tail[gt + 1..];
                inner.find("</a>").map(|lt| {
                    strip_tags(&inner[..lt])
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                })
            })
            .unwrap_or_default();
        if title.len() > 2 {
            // each entry appears twice: a cover anchor (title = the
            // system badge) then the real title anchor. Like the
            // Python parser's dict, the later anchor wins.
            if let Some(&idx) = seen.get(&full) {
                out[idx].title = title;
            } else if seen.insert(full.clone(), out.len()).is_none() {
                out.push(HackEntry {
                    title,
                    url: full,
                });
            }
        }
        rest = tail;
    }
    out
}

/// archive.org hunt for a CD-audio version of a game. Port
/// of Python's `search_archive_msu`: Genesis accepts MSU /
/// MD+ / MSU-MD titles, everything else MSU. The docs filter
/// is pure so --selftest can pin it down.
fn filter_archive_docs(docs: &[serde_json::Value], genesis: bool) -> Vec<HackEntry> {
    let mut out = Vec::new();
    for d in docs {
        let title = d
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let ident = d
            .get("identifier")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        if !title.is_empty() && !ident.is_empty() && msu_title_ok(&title, genesis) {
            out.push(HackEntry {
                title,
                url: format!("https://archive.org/details/{ident}"),
            });
        }
    }
    out
}

fn search_archive_msu(name: &str, genesis: bool) -> Vec<HackEntry> {
    let q = if genesis {
        format!("title:({name}) AND (msu OR \"md+\" OR msu-md)")
    } else {
        format!("title:({name}) AND (msu)")
    };
    let url = format!(
        "https://archive.org/advancedsearch.php?q={}&fl[]=identifier&fl[]=title&rows=20&output=json",
        url_encode(&q)
    );
    let agent = http_agent(30);
    let Ok(resp) = agent.get(&url).call() else {
        return vec![];
    };
    let Ok(v) = resp.into_json::<serde_json::Value>() else {
        return vec![];
    };
    let docs = v
        .get("response")
        .and_then(|r| r.get("docs"))
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    filter_archive_docs(&docs, genesis)
}

/// ---- GitHub hunt source (v1.03) ----
// the user, Oct 24: the Find hunt must also search GitHub --
// real MD+/MSU-1 patches live there as repos (infinest's
// Phantasy Star II MD+ patch is only on GitHub and in
// Zeldix's link list; the archive.org/Plaza hunt found
// nothing for that game). Unauthenticated REST API:
// repository search, then the repo's latest release assets
// (fetch side, below). GitHub requires a User-Agent on API
// calls; the unauthenticated budgets (60 core requests per
// hour, 10 search requests per minute, per IP) sit far
// above what one user-initiated hunt spends (at most 2
// searches per name variant).

fn github_api_get(agent: &ureq::Agent, url: &str) -> Option<serde_json::Value> {
    let resp = agent
        .get(url)
        .set("Accept", "application/vnd.github+json")
        .set(
            "User-Agent",
            &format!("EmuFrontRust/{APP_VERSION} (enhanced hunt)"),
        )
        .call()
        .ok()?;
    resp.into_json().ok()
}

/// The search words of a game name that a GitHub repo must
/// answer for: lowercase alphanumeric tokens, stopwords
/// gone, roman numerals folded to digits (repo names spell
/// it "Phantasy Star 2" as often as "Phantasy Star II").
/// Pure -- --selftest pins the matching built on it.
fn game_tokens(name: &str) -> Vec<String> {
    const STOP: [&str; 5] = ["the", "of", "and", "a", "an"];
    let mut out: Vec<String> = Vec::new();
    for raw in name.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
        if raw.is_empty() || STOP.contains(&raw) {
            continue;
        }
        let t = match raw {
            "ii" => "2",
            "iii" => "3",
            "iv" => "4",
            "vi" => "6",
            _ => raw,
        }
        .to_string();
        if !out.contains(&t) {
            out.push(t);
        }
    }
    out
}

/// Does one repository-search item answer for this game
/// AND carry the CD-audio marker (in its name, description
/// or topics)? Pure; the --selftest fixture is shaped like
/// the real API response (Infinest/Phantasy-Star-II-MD-
/// Plus, verified live Oct 24 2026). The marker test runs
/// on a dash-spaced copy so "md-plus" and "msu-md" spell
/// the way msu_title_ok expects.
fn github_repo_matches(
    item: &serde_json::Value,
    tokens: &[String],
    genesis: bool,
) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let mut hay = String::new();
    for key in ["name", "full_name", "description"] {
        if let Some(s) = item.get(key).and_then(|v| v.as_str()) {
            hay.push_str(s);
            hay.push(' ');
        }
    }
    if let Some(topics) = item.get("topics").and_then(|v| v.as_array()) {
        for t in topics {
            if let Some(s) = t.as_str() {
                hay.push_str(s);
                hay.push(' ');
            }
        }
    }
    if !msu_title_ok(&hay.replace('-', " "), genesis) {
        return false;
    }
    let hay_tokens = game_tokens(&hay);
    tokens.iter().all(|t| hay_tokens.contains(t))
}

/// Repository-search items -> hunt results: matched repos
/// only, each tagged with its source in the title (the
/// Zeldix results set that pattern). Pure.
fn filter_github_repos(
    items: &[serde_json::Value],
    name: &str,
    genesis: bool,
) -> Vec<HackEntry> {
    let tokens = game_tokens(name);
    let term = if genesis { "MD+" } else { "MSU-1" };
    let mut out: Vec<HackEntry> = Vec::new();
    for it in items {
        if !github_repo_matches(it, &tokens, genesis) {
            continue;
        }
        let full = it.get("full_name").and_then(|v| v.as_str()).unwrap_or("");
        let url = it.get("html_url").and_then(|v| v.as_str()).unwrap_or("");
        if full.is_empty() || url.is_empty() {
            continue;
        }
        if out.iter().any(|e| e.url == url) {
            continue;
        }
        out.push(HackEntry {
            title: format!("{full} — {term} patch · GitHub"),
            url: url.to_string(),
        });
    }
    out
}

/// GitHub hunt: two repository searches per name (title +
/// term, then the same under the topic: qualifier), the
/// results filtered to repos that answer for the game.
/// The request count is capped here on purpose -- see the
/// budget note above.
fn search_github(name: &str, genesis: bool) -> Vec<HackEntry> {
    let term = if genesis { "md-plus" } else { "msu-1" };
    let agent = http_agent(20);
    let mut items: Vec<serde_json::Value> = Vec::new();
    for q in [format!("{name} {term}"), format!("{name} topic:{term}")] {
        let url = format!(
            "https://api.github.com/search/repositories?q={}&per_page=10",
            url_encode(&q)
        );
        if let Some(v) = github_api_get(&agent, &url) {
            if let Some(arr) = v.get("items").and_then(|i| i.as_array()) {
                items.extend(arr.iter().cloned());
            }
        }
    }
    filter_github_repos(&items, name, genesis)
}

/// The Minerva mirror of the Complete Sega MD+ Collection:
/// one fixed directory listing in Minerva's Internet
/// Archive subtree (uploader folder `md_`). Verified live
/// Oct 24 2026: 139 per-game MD+/32X+ packs in .7z form,
/// e.g. "MD+ Phantasy Star II (PS2 Arranged).7z" (116.87
/// MB, file id 2250802). Minerva's own search API and
/// file pages ride on a database that was down that day;
/// the /browse/ listings are static and work regardless,
/// which is why the hunt walks this listing.
const MINERVA_MDPLUS_LISTING: &str =
    "https://minerva-archive.org/browse/Internet%20Archive/md_/mdplus_collection_22_04_16/";

/// Parse one Minerva directory listing into (pack
/// filename, file-page URL) pairs. Rows are anchors to
/// "/rom?id=<digits>" whose text is the filename; only
/// archive files count (the listing also carries the
/// collection's .dat file and thumbnail images). Pure.
fn parse_minerva_listing(html: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let needle = "href=\"/rom?id=";
    let mut rest = html;
    while let Some(pos) = rest.find(needle) {
        let after = &rest[pos + needle.len()..];
        let Some(endq) = after.find('"') else { break };
        let id = &after[..endq];
        let tail = &after[endq + 1..];
        let name = tail
            .find('>')
            .and_then(|gt| {
                let inner = &tail[gt + 1..];
                inner
                    .find("</a>")
                    .map(|lt| strip_tags(&inner[..lt]).trim().to_string())
            })
            .unwrap_or_default();
        let lower = name.to_lowercase();
        if !id.is_empty()
            && id.chars().all(|c| c.is_ascii_digit())
            && [".7z", ".zip", ".rar"].iter().any(|e| lower.ends_with(*e))
        {
            let url = format!("https://minerva-archive.org/rom?id={id}");
            if !out.iter().any(|(_, u)| *u == url) {
                out.push((name, url));
            }
        }
        rest = tail;
    }
    out
}

/// The Minerva MD+ index: (pack filename, file-page URL)
/// pairs from the collection listing, fetched once per
/// run and cached -- the ZELDIX_INDEX pattern.
static MINERVA_INDEX: std::sync::OnceLock<
    std::sync::Mutex<Option<Vec<(String, String)>>>,
> = std::sync::OnceLock::new();

fn minerva_index() -> Vec<(String, String)> {
    let cell = MINERVA_INDEX.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(g) = cell.lock() {
        if let Some(v) = &*g {
            return v.clone();
        }
    }
    let mut out: Vec<(String, String)> = Vec::new();
    let agent = http_agent(40);
    if let Ok(resp) = agent
        .get(MINERVA_MDPLUS_LISTING)
        .set(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36",
        )
        .call()
    {
        if let Ok(html) = resp.into_string() {
            out = parse_minerva_listing(&html);
        }
    }
    if let Ok(mut g) = cell.lock() {
        *g = Some(out.clone());
    }
    out
}

/// Minerva index rows that answer for one game name: the
/// filename must carry the CD-audio marker AND contain
/// every token of the game's name (the GitHub filter's
/// rule, reused). One guard of its own: a bare number in
/// the pack's core title (the part before the first "(")
/// that the game's name doesn't have rejects the row, so
/// "Golden Axe" never claims the "Golden Axe II" pack --
/// and pack-version numbers inside the parentheses
/// ("Sega Ages 2500 Vol. 7") never block a match. Pure.
fn filter_minerva_index(rows: &[(String, String)], name: &str) -> Vec<HackEntry> {
    let tokens = game_tokens(name);
    let mut out: Vec<HackEntry> = Vec::new();
    if tokens.is_empty() {
        return out;
    }
    let game_digits: Vec<String> = tokens
        .iter()
        .filter(|t| t.chars().all(|c| c.is_ascii_digit()))
        .cloned()
        .collect();
    for (fname, url) in rows {
        if !msu_title_ok(fname, true) {
            continue;
        }
        let stem = Path::new(fname)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let core = stem.split('(').next().unwrap_or("").trim();
        let row_tokens = game_tokens(core);
        if !tokens.iter().all(|t| row_tokens.contains(t)) {
            continue;
        }
        let row_digits_bad = row_tokens
            .iter()
            .filter(|t| t.chars().all(|c| c.is_ascii_digit()))
            .any(|d| !game_digits.contains(d));
        if row_digits_bad {
            continue;
        }
        out.push(HackEntry {
            title: format!("{fname} · Minerva"),
            url: url.clone(),
        });
    }
    out
}

/// Minerva hunt (Genesis only): the indexed subtree is
/// the Sega MD+ collection, so there is nothing for the
/// SNES MSU-1 hunt here.
fn search_minerva(name: &str, genesis: bool) -> Vec<HackEntry> {
    if !genesis {
        return vec![];
    }
    let rows = minerva_index();
    filter_minerva_index(&rows, name)
}

/// The Zeldix MSU-1 thread index: (title, thread URL) pairs,
/// fetched once per run and cached. Port of Python's
/// `_zeldix_threads` (SNES only -- Zeldix has no Genesis
/// section; Genesis results come from archive.org + Plaza).
static ZELDIX_INDEX: std::sync::OnceLock<
    std::sync::Mutex<Option<Vec<(String, String)>>>,
> = std::sync::OnceLock::new();

fn zeldix_threads() -> Vec<(String, String)> {
    let cell = ZELDIX_INDEX.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(g) = cell.lock() {
        if let Some(v) = &*g {
            return v.clone();
        }
    }
    let mut out: Vec<(String, String)> = Vec::new();
    let agent = http_agent(40);
    if let Ok(resp) = agent
        .get("https://www.zeldix.net/t2684-alphabetical-list-every-snes-msu1-hack")
        .set(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36",
        )
        .call()
    {
        if let Ok(html) = resp.into_string() {
            // WANTED threads are titled in red; each title is
            // followed by its thread link (/t<digits>...).
            let mut titles: Vec<(usize, String)> = Vec::new();
            let mut rest = html.as_str();
            let mut base = 0usize;
            while let Some(pos) = rest.find("<font color=\"#FF0000\">") {
                let after = &rest[pos + 22..];
                let Some(end) = after.find("</font>") else { break };
                let t = strip_tags(&after[..end]).trim().to_string();
                if !t.is_empty() {
                    titles.push((base + pos, t));
                }
                base += pos + 22 + end + 7;
                rest = &after[end + 7..];
            }
            let mut links: Vec<(usize, String)> = Vec::new();
            let mut rest = html.as_str();
            let mut base = 0usize;
            while let Some(pos) = rest.find("href=\"/t") {
                let after = &rest[pos + 8..];
                if after.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    if let Some(end) = after.find('"') {
                        links.push((
                            base + pos,
                            format!("https://www.zeldix.net/t{}", &after[..end]),
                        ));
                    }
                }
                base += pos + 8;
                rest = after;
            }
            for (pos, title) in &titles {
                if let Some((_, url)) = links.iter().find(|(lp, _)| lp > pos) {
                    out.push((title.clone(), url.clone()));
                }
            }
        }
    }
    if let Ok(mut g) = cell.lock() {
        *g = Some(out.clone());
    }
    out
}

/// Extract direct patch/archive links from a Zeldix thread
/// page: the leftmost match wins, files whose URL mentions
/// MSU are preferred. Port of Python's thread-page regex.
fn parse_zeldix_thread(page: &str) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < page.len() {
        let Some(rel) = page[i..].find("http") else { break };
        let start = i + rel;
        if !(page[start..].starts_with("http://")
            || page[start..].starts_with("https://"))
        {
            i = start + 4;
            continue;
        }
        let mut end = start;
        for c in page[start..].chars() {
            if c.is_whitespace() || c == '"' || c == '\'' || c == '<' || c == '>' {
                break;
            }
            end += c.len_utf8();
        }
        let run = &page[start..end];
        let lower = run.to_lowercase();
        let mut best: Option<usize> = None;
        for ext in [".zip", ".ips", ".bps", ".rar", ".7z"] {
            if let Some(p) = lower.find(ext) {
                let e = p + ext.len();
                if best.map_or(true, |b| e < b) {
                    best = Some(e);
                }
            }
        }
        if let Some(e) = best {
            let link = run[..e].to_string();
            if !links.contains(&link) {
                links.push(link);
            }
        }
        i = end.max(start + 4);
    }
    let msu: Vec<String> = links
        .iter()
        .filter(|u| u.to_lowercase().contains("msu"))
        .cloned()
        .collect();
    if !msu.is_empty() {
        msu
    } else {
        links
    }
}

fn search_zeldix(game: &Game) -> Vec<HackEntry> {
    let mut names = name_variants(game);
    if let Some(first) = names.first().cloned() {
        if let Some(a) = search_alias(&first) {
            names.push(a.to_string());
        }
    }
    if names.is_empty() {
        return vec![];
    }
    let threads = zeldix_threads();
    let mut matched: Vec<&(String, String)> = Vec::new();
    for pair in &threads {
        let t = pair.0.to_lowercase();
        let hit = names.iter().any(|n| {
            let nl = n.to_lowercase();
            nl.len() >= 4 && (t.contains(&nl) || nl.contains(&t))
        });
        if hit {
            matched.push(pair);
        }
    }
    let agent = http_agent(40);
    let mut out = Vec::new();
    for (title, url) in matched.iter().take(4) {
        let Ok(resp) = agent
            .get(url)
            .set(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36",
            )
            .call()
        else {
            continue;
        };
        let Ok(page) = resp.into_string() else { continue };
        for link in parse_zeldix_thread(&page) {
            out.push(HackEntry {
                title: format!("{title} — MSU-1 patch"),
                url: link,
            });
        }
    }
    out
}

/// Hunt sources, and how a new one joins (v1.03 note): a
/// source is (a) a search fn returning HackEntry rows --
/// the title carries the source tag -- and (b) when its
/// entry URL is a page rather than a file, a branch in
/// fetch_unpacked resolving the entry to an Unpacked.
/// Everything downstream (classify -> patch -> stage ->
/// register) is source-blind, which is what keeps the user's
/// rule: his only choice is WHICH result.
///
/// MINERVA (minerva-archive.org), wired Oct 24 2026 on
/// a second, deeper look (the user asked for it twice): it
/// is the community rescue mirror of the Myrient archive
/// -- a browse-only tree of whole ROM collections -- BUT
/// its Internet Archive subtree (sorted by uploader)
/// carries the Complete Sega MD+ Collection itself,
/// per-game packs and all, under the `md_` uploader
/// folder. search_minerva() walks that one listing; see
/// MINERVA_MDPLUS_LISTING. Acquisition is the honest
/// limit: Minerva hands its files out as torrents (its
/// own FAQ says so) and its file pages were erroring the
/// day this was wired, so fetch_minerva_pack tries the
/// file page directly and otherwise says plainly that
/// the pack is torrent-only. Full research:
/// SOURCES-PREP-REPORT-20261024.md + MINERVA-ADDENDUM.md
/// (session-20261024/).
///
/// Session-cached entry to the live hunt below (§2.6):
/// a second Find for the same game in the same run is
/// instant and costs no source its rate limit.
fn search_msu1(game: &Game) -> Vec<HackEntry> {
    let key = hunt_cache_key(game);
    if let Some(v) = session_cache_lookup(&HUNT_MSU_CACHE, &key) {
        return v;
    }
    let out = search_msu1_live(game);
    session_cache_store(&HUNT_MSU_CACHE, key, &out);
    out
}

/// Find a CD-audio (MSU-1 / MD+) version of a game online:
/// archive.org first, RomHack Plaza second (its results
/// filtered to plausible packs), GitHub third (v1.03),
/// Minerva fourth (Genesis only, v1.03), Zeldix
/// prepended for SNES. Stops at the first name
/// variant that yields anything.
/// Port of Python's `search_msu1`.
fn search_msu1_live(game: &Game) -> Vec<HackEntry> {
    let genesis = game.system == "Sega Genesis";
    let mut names = name_variants(game);
    if let Some(first) = names.first().cloned() {
        if let Some(a) = search_alias(&first) {
            names.push(a.to_string());
        }
    }
    for (i, nm) in names.iter().enumerate() {
        let mut found = search_archive_msu(nm, genesis);
        if i < 2 {
            for h in search_plaza_query(nm) {
                if msu_title_ok(&h.title, genesis) {
                    found.push(h);
                }
            }
            found.extend(search_github(nm, genesis));
            found.extend(search_minerva(nm, genesis));
        }
        if !found.is_empty() {
            if game.system.is_empty() || game.system == "Super Nintendo" {
                let mut z = search_zeldix(game);
                z.extend(found);
                return z;
            }
            return found;
        }
    }
    if game.system.is_empty() || game.system == "Super Nintendo" {
        return search_zeldix(game);
    }
    vec![]
}

// ---------------- Enhanced Games catalog -----------------
// The "🎵 Enhanced Games" browser — Python EmuFront Box
// v1.05–1.08 parity, the Rust rewrite's last big catalog
// gap. the user's rule for this window, in his words: the
// Super Nintendo enhanced games and the Sega Genesis
// enhanced games stay SEPARATE. The window therefore
// holds exactly ONE system's catalog at a time, chosen
// with the two labeled buttons at its top; entries are
// assembled per system from per-system sources, and rows
// are matched only against that system's games, so a
// Genesis row can never render — or match — inside the
// Nintendo catalog, nor the reverse.
//
// Sources (fetch budget: one cached fetch per source per
// catalog load; the Zeldix and Minerva indexes are the
// process-wide caches the Find hunt already keeps):
//   Super Nintendo: Zeldix thread index (patches) +
//     archive.org title:(MSU-1) (packs) + GitHub repos —
//     1 archive query + 2 GitHub searches per load.
//   Sega Genesis: archive.org MSU-MD queries (packs) +
//     the Minerva MD+ index (packs) + GitHub repos —
//     2 archive queries + 2 GitHub searches per load.
// Row rebuilds after a build reuse the fetched entries
// and cost no network at all.
//
// A row's build is the Find window's own chain
// (download_and_build_msu): the pick is the user's only
// manual step. STAGING DECISION (flagged for the user):
// Python stages Enhanced installs into an "Enhanced
// ROMs" tree; this window passes enhanced=false, so
// catalog builds land in Hacked ROMs like every other
// Rust build — one staging tree and one detection story
// (game_has_msu and the ⭐ shelf already read it). The
// chain's enhanced=true path still exists if he ever
// wants the trees split.

/// One catalog entry: a title, where it downloads from,
/// who listed it, and whether it is a full pack or a
/// patch (the 📦/🧩 section split reads the kind string,
/// exactly like Python's `is_patch`). The serde derives
/// exist for the v1.06 disk cache (cache/catalogs) —
/// the entry itself is unchanged.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct CatalogEntry {
    title: String,
    url: String,
    source: String,
    kind: String,
}

fn entry_is_patch(e: &CatalogEntry) -> bool {
    e.kind.to_lowercase().contains("patch")
}

/// Drop "[...]" / "(...)" groups (with any space before
/// them) from a title — catalog_norm's first step.
fn drop_title_groups(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '[' || c == '(' {
            let close = if c == '[' { ']' } else { ')' };
            while out.ends_with(' ') {
                out.pop();
            }
            for c2 in chars.by_ref() {
                if c2 == close {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The word tokens of a catalog/game title: catalog
/// prefix stripped, bracket/paren groups dropped,
/// lowercased, split on anything that is not
/// alphanumeric, '-' or '+' (so "msu-1" and "md+" stay
/// whole words, the way Python's regex sees them).
fn catalog_tokens(title: &str) -> Vec<String> {
    let t = strip_catalog_prefix_full(title);
    let t = drop_title_groups(&t);
    t.to_lowercase()
        .split(|c: char| !(c.is_alphanumeric() || c == '-' || c == '+'))
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

/// The format/packaging words catalog_norm erases
/// (Python's two word lists in `_catalog_norm`).
const CATALOG_DROP: [&str; 10] = [
    "msu-1", "msu-md", "msu", "md+", "md", "patch", "pack", "music",
    "soundtrack", "arranged",
];

/// Reduce a catalog or game title to a comparable core:
/// brackets/regions off, MSU/MD+ and pack/patch words
/// off, punctuation off. Port of Python's `_catalog_norm`.
fn catalog_norm(title: &str) -> String {
    let mut out = String::new();
    for tok in catalog_tokens(title) {
        if CATALOG_DROP.contains(&tok.as_str()) {
            continue;
        }
        out.extend(tok.chars().filter(|c| c.is_alphanumeric()));
    }
    out
}

/// The bare-number tokens of a title, roman numerals
/// folded to digits (game_tokens' map): the sequel
/// guard's currency. The format words are erased first,
/// so the "1" of "MSU-1" never poisons the set, and
/// "PS2" / "7z" are not bare numbers.
fn catalog_digit_set(title: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for tok in catalog_tokens(title) {
        if CATALOG_DROP.contains(&tok.as_str()) {
            continue;
        }
        let t = match tok.as_str() {
            "ii" => "2",
            "iii" => "3",
            "iv" => "4",
            "vi" => "6",
            _ => tok.as_str(),
        };
        if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
            out.push(t.to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

/// A library game pre-digested for catalog matching:
/// the normalized cores and number sets of its name
/// variants, plus the search alias as an extra variant
/// (Final Fantasy IV for Final Fantasy II — the MSU-1
/// scene labels SNES games by their Japanese numbering).
struct CatalogGamePrep {
    game: Game,
    norms: Vec<String>,
    digits: Vec<Vec<String>>,
}

fn catalog_game_prep(game: &Game) -> CatalogGamePrep {
    let mut names = name_variants(game);
    if let Some(first) = names.first().cloned() {
        if let Some(a) = search_alias(&first) {
            names.push(a.to_string());
        }
    }
    let norms: Vec<String> = names.iter().map(|n| catalog_norm(n)).collect();
    let digits: Vec<Vec<String>> =
        names.iter().map(|n| catalog_digit_set(n)).collect();
    CatalogGamePrep {
        game: game.clone(),
        norms,
        digits,
    }
}

/// Does this catalog entry belong to this game? Python's
/// match_catalog_entry rule (either normalized core
/// contains the other), plus the sequel guard the
/// Minerva hunt already uses, generalized: when either
/// side carries bare-number tokens, the number sets must
/// agree — otherwise "Final Fantasy II" would claim the
/// "Final Fantasy III" packs whose cores merely start
/// with its own, and "Golden Axe" the "Golden Axe II"
/// pack. Pure — --selftest pins it, Python cases included.
fn catalog_entry_matches(entry_title: &str, prep: &CatalogGamePrep) -> bool {
    let en = catalog_norm(entry_title);
    if en.len() < 4 {
        return false;
    }
    let e_digits = catalog_digit_set(entry_title);
    for (vn, vd) in prep.norms.iter().zip(prep.digits.iter()) {
        if vn.len() >= 4
            && (en.contains(vn.as_str()) || vn.contains(en.as_str()))
            && *vd == e_digits
        {
            return true;
        }
    }
    false
}

/// A manual_games entry from the shared config: the
/// registered builds. Python keeps these apart from the
/// scanned library; Rust dual-writes them into "games",
/// and this reads the authoritative manual list back.
#[derive(Clone)]
struct ManualRef {
    name: String,
    system: String,
    path: String,
}

fn manual_game_refs() -> Vec<ManualRef> {
    let cfg = load_cfg();
    let mut out = Vec::new();
    if let Some(arr) = cfg.get("manual_games").and_then(|x| x.as_array()) {
        for g in arr {
            let s = |k: &str| {
                g.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string()
            };
            out.push(ManualRef {
                name: s("name"),
                system: s("system"),
                path: s("path"),
            });
        }
    }
    out
}

/// The installed enhanced build for a library game, if
/// one is registered (the catalog's ✔ Installed):
/// (A) Python's rule — a manual entry for the same
/// system named "<game> [<tag>]", the shape
/// build_hacked_from registers; or (B) a manual entry
/// staged under the data dir whose name catalog-matches
/// the game AND whose folder really carries staged
/// audio — the shape install_pack_rom_from registers
/// (the pack ROM's own cleaned name, no bracket tag).
fn catalog_installed(
    prep: &CatalogGamePrep,
    manuals: &[ManualRef],
    staged_prefix: &str,
) -> Option<ManualRef> {
    let game = &prep.game;
    let prefix = format!("{} [", game.name);
    for m in manuals {
        if m.system == game.system && m.name.starts_with(&prefix) {
            return Some(m.clone());
        }
    }
    for m in manuals {
        if m.system != game.system
            || m.path == game.path
            || !m.path.starts_with(staged_prefix)
        {
            continue;
        }
        if !catalog_entry_matches(&m.name, prep) {
            continue;
        }
        let mg = Game {
            name: m.name.clone(),
            path: m.path.clone(),
            system: m.system.clone(),
            emulator: String::new(),
            emulator_locked: false,
            patch: String::new(),
            cheats_text: String::new(),
            cheats_enabled: false,
        };
        if game_has_msu(&mg) {
            return Some(m.clone());
        }
    }
    None
}

/// A catalog row ready for the window: the entry, the
/// library game it matched (None = not in his library,
/// so it can't be built), its installed build when one
/// is registered (Launch's target), and the hint that
/// the same game also appears in the other section.
#[derive(Clone)]
struct EnhRow {
    entry: CatalogEntry,
    game: Option<Game>,
    installed: Option<Game>,
    also_other: bool,
}

/// Build one system's rows from its entries. Pure apart
/// from catalog_installed's folder read. Separation
/// invariant: only this system's BASE games can match —
/// staged builds (their paths live under the data dir)
/// are installed-state, never match targets.
fn build_catalog_rows(
    system: &str,
    entries: &[CatalogEntry],
    games: &[Game],
    manuals: &[ManualRef],
    staged_prefix: &str,
) -> Vec<EnhRow> {
    let preps: Vec<CatalogGamePrep> = games
        .iter()
        .filter(|g| g.system == system && !g.path.starts_with(staged_prefix))
        .map(catalog_game_prep)
        .collect();
    let mut rows: Vec<EnhRow> = Vec::new();
    for e in entries {
        let hit = preps.iter().find(|p| catalog_entry_matches(&e.title, p));
        let (game, installed) = match hit {
            Some(p) => {
                let inst =
                    catalog_installed(p, manuals, staged_prefix).and_then(|m| {
                        games
                            .iter()
                            .find(|g| g.path == m.path)
                            .cloned()
                            .or_else(|| {
                                Some(Game {
                                    name: m.name.clone(),
                                    path: m.path.clone(),
                                    system: m.system.clone(),
                                    emulator: String::new(),
                                    emulator_locked: false,
                                    patch: String::new(),
                                    cheats_text: String::new(),
                                    cheats_enabled: false,
                                })
                            })
                    });
                (Some(p.game.clone()), inst)
            }
            None => (None, None),
        };
        rows.push(EnhRow {
            entry: e.clone(),
            game,
            installed,
            also_other: false,
        });
    }
    // Python's hint data: a game with rows in BOTH
    // sections can be had either way — pack or patch.
    let mut kinds: HashMap<String, (bool, bool)> = HashMap::new();
    for r in &rows {
        if let Some(g) = &r.game {
            let k = kinds.entry(g.path.clone()).or_insert((false, false));
            if entry_is_patch(&r.entry) {
                k.1 = true;
            } else {
                k.0 = true;
            }
        }
    }
    let both: HashSet<String> = kinds
        .into_iter()
        .filter(|(_, (pack, patch))| *pack && *patch)
        .map(|(path, _)| path)
        .collect();
    for r in &mut rows {
        if let Some(g) = &r.game {
            r.also_other = both.contains(&g.path);
        }
    }
    // Python's row order: matched first, then by title.
    rows.sort_by(|a, b| {
        b.game
            .is_some()
            .cmp(&a.game.is_some())
            .then_with(|| {
                a.entry
                    .title
                    .to_lowercase()
                    .cmp(&b.entry.title.to_lowercase())
            })
    });
    rows
}

/// Merge per-source entry lists into one system's
/// catalog: first occurrence of a title wins, then sort
/// by title. Port of Python catalog_entries' dedup.
fn merge_catalog_entries(parts: Vec<Vec<CatalogEntry>>) -> Vec<CatalogEntry> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<CatalogEntry> = Vec::new();
    for part in parts {
        for e in part {
            let key: String = e
                .title
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>()
                .to_lowercase();
            if seen.insert(key) {
                out.push(e);
            }
        }
    }
    out.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    out
}

/// Broad archive.org title catalog (Python's
/// `_archive_catalog`): the query itself is the filter —
/// every doc it returns becomes a row of the given kind.
fn fetch_archive_catalog(queries: &[&str], kind: &str) -> Vec<CatalogEntry> {
    let agent = http_agent(40);
    let mut out: Vec<CatalogEntry> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for q in queries {
        let url = format!(
            "https://archive.org/advancedsearch.php?q={}&fl[]=identifier&fl[]=title&rows=300&output=json",
            url_encode(q)
        );
        let Ok(resp) = agent.get(&url).call() else {
            continue;
        };
        let Ok(v) = resp.into_json::<serde_json::Value>() else {
            continue;
        };
        let docs = v
            .get("response")
            .and_then(|r| r.get("docs"))
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();
        for d in &docs {
            let title = d
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let ident = d
                .get("identifier")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            if title.is_empty() || ident.is_empty() || !seen.insert(ident.clone()) {
                continue;
            }
            out.push(CatalogEntry {
                title,
                url: format!("https://archive.org/details/{ident}"),
                source: "archive.org".to_string(),
                kind: kind.to_string(),
            });
        }
    }
    out
}

/// GitHub's catalog slice, pure part: repository-search
/// items -> catalog rows. The marker rule is the Find
/// filter's (github_repo_matches' haystack test), without
/// a game to answer for — rows that match no library
/// game simply show as not owned, like Python's
/// unmatched archive rows.
fn filter_github_catalog(
    items: &[serde_json::Value],
    genesis: bool,
) -> Vec<CatalogEntry> {
    let term = if genesis { "MD+" } else { "MSU-1" };
    let mut out: Vec<CatalogEntry> = Vec::new();
    for it in items {
        let mut hay = String::new();
        for key in ["name", "full_name", "description"] {
            if let Some(s) = it.get(key).and_then(|v| v.as_str()) {
                hay.push_str(s);
                hay.push(' ');
            }
        }
        if let Some(topics) = it.get("topics").and_then(|v| v.as_array()) {
            for t in topics {
                if let Some(s) = t.as_str() {
                    hay.push_str(s);
                    hay.push(' ');
                }
            }
        }
        // Catalog marker test — stricter than the Find
        // filter's msu_title_ok: the two catalogs are kept
        // strictly separate, so a Genesis row must carry an
        // MD+ marker of its own. A bare "msu" mention is an
        // SNES/MSU-1 item and never a Genesis one (the Find
        // hunt can afford the loose rule — its queries are
        // term-scoped and name-checked; the catalog is not).
        let spaced = hay.replace('-', " ").to_lowercase();
        let marked = if genesis {
            spaced.contains("md+")
                || spaced.contains("md plus")
                || spaced.contains("msu md")
        } else {
            msu_title_ok(&spaced, false)
        };
        if !marked {
            continue;
        }
        let full = it.get("full_name").and_then(|v| v.as_str()).unwrap_or("");
        let url = it.get("html_url").and_then(|v| v.as_str()).unwrap_or("");
        if full.is_empty()
            || url.is_empty()
            || out.iter().any(|e| e.url == url)
        {
            continue;
        }
        out.push(CatalogEntry {
            title: full.to_string(),
            url: url.to_string(),
            source: "GitHub".to_string(),
            kind: format!("{term} patch"),
        });
    }
    out
}

/// GitHub catalog fetch: one bare-term and one topic:
/// repository search (the Find hunt's query shapes,
/// widened to a catalog page of 50).
fn fetch_github_catalog(genesis: bool) -> Vec<CatalogEntry> {
    let term = if genesis { "md-plus" } else { "msu-1" };
    let agent = http_agent(20);
    let mut items: Vec<serde_json::Value> = Vec::new();
    for q in [term.to_string(), format!("topic:{term}")] {
        let url = format!(
            "https://api.github.com/search/repositories?q={}&per_page=50",
            url_encode(&q)
        );
        if let Some(v) = github_api_get(&agent, &url) {
            if let Some(arr) = v.get("items").and_then(|i| i.as_array()) {
                items.extend(arr.iter().cloned());
            }
        }
    }
    filter_github_catalog(&items, genesis)
}

/// The live assembly with both cache tiers fed
/// (§2.6): a successful live fetch is the only event
/// allowed to stamp a disk copy, so a saved copy's age
/// always means "when these entries came off the
/// network", never "when we last looked at them". Only
/// non-empty results land in either tier (the Slice 1
/// session rule, for the same reason: an empty fetch
/// can be a network hiccup, and pinning it — on disk,
/// for a day — would hide real content behind it).
fn catalog_entries_live_cached(system: &str) -> Vec<CatalogEntry> {
    let out = catalog_entries_live(system);
    if !out.is_empty() {
        write_catalog_disk(system, &out, now_ts());
        session_cache_store(&CATALOG_CACHE, system.to_string(), &out);
    }
    out
}

// ---------------- catalog disk cache (Slice 3) -----------
// RELEASE-PERF-DESIGN §2.6, disk tier: the assembled
// catalogs persist in the app's private cache tree
// (cache\catalogs — never the shared config, never the
// shared box-art tree) with a 24-hour TTL. Catalogs are
// large, slow to fetch and change slowly; per-game Find
// results are the opposite and stay session-only above.
// The two systems keep separate files, as they keep
// separate everything. A saved copy is derived data in
// the snapshot's family: any doubt about the file —
// missing, unreadable, wrong magic/schema/system —
// reads as "absent" and the live assembly runs instead.

/// How long a saved catalog is served as-is.
const CATALOG_TTL_SECS: i64 = 24 * 3600;
const CATALOG_DISK_MAGIC: &str = "emufront-rust-catalog";
const CATALOG_DISK_SCHEMA: u32 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
struct CatalogDiskFile {
    magic: String,
    schema: u32,
    system: String,
    fetched_at: i64,
    entries: Vec<CatalogEntry>,
}

fn catalog_cache_path(system: &str) -> PathBuf {
    let slug = match system {
        "Super Nintendo" => "super-nintendo".to_string(),
        "Sega Genesis" => "sega-genesis".to_string(),
        _ => system
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect(),
    };
    catalogs_dir().join(format!("{slug}-v{CATALOG_DISK_SCHEMA}.json"))
}

/// Parse a saved catalog file. Pure — the corruption
/// cases are --selftest fixtures. None means "treat as
/// absent": bad JSON, wrong magic, wrong schema.
fn parse_catalog_disk(
    text: &str,
) -> Option<(String, i64, Vec<CatalogEntry>)> {
    let f: CatalogDiskFile = serde_json::from_str(text).ok()?;
    if f.magic != CATALOG_DISK_MAGIC || f.schema != CATALOG_DISK_SCHEMA {
        return None;
    }
    Some((f.system, f.fetched_at, f.entries))
}

/// Read one system's saved catalog, if a usable one
/// exists. A file naming a different system than the
/// one asked for is as absent as a corrupt one.
fn read_catalog_disk(system: &str) -> Option<(i64, Vec<CatalogEntry>)> {
    let text = std::fs::read_to_string(catalog_cache_path(system)).ok()?;
    let (sys, fetched_at, entries) = parse_catalog_disk(&text)?;
    if sys != system {
        return None;
    }
    Some((fetched_at, entries))
}

/// Persist a catalog atomically (tmp + rename, the
/// snapshot's pattern). Best-effort: a failed write
/// costs the next open a refetch, nothing more.
fn write_catalog_disk(system: &str, entries: &[CatalogEntry], fetched_at: i64) {
    let f = CatalogDiskFile {
        magic: CATALOG_DISK_MAGIC.to_string(),
        schema: CATALOG_DISK_SCHEMA,
        system: system.to_string(),
        fetched_at,
        entries: entries.to_vec(),
    };
    let dir = catalogs_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(text) = serde_json::to_string(&f) {
        let path = catalog_cache_path(system);
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

/// Is a copy fetched at `fetched_at` past its TTL at
/// `now`? Pure. A missing/zero stamp is stale by
/// definition; a stamp in the future (the clock stepped
/// back) is trusted rather than punished with a
/// refetch loop.
fn catalog_is_stale(fetched_at: i64, now: i64) -> bool {
    if fetched_at <= 0 {
        return true;
    }
    let age = now - fetched_at;
    if age < 0 {
        return false;
    }
    age >= CATALOG_TTL_SECS
}

/// The window's honest age line for a saved copy.
fn catalog_age_label(age_secs: i64) -> String {
    if age_secs < 90 {
        "just now".to_string()
    } else if age_secs < 3600 {
        format!("{} min ago", age_secs / 60)
    } else if age_secs < 86_400 {
        format!("{} h ago", age_secs / 3600)
    } else {
        format!("{} d ago", age_secs / 86_400)
    }
}

/// The state of one system's saved catalog at a moment
/// in time — the model's whole input.
enum CatalogDisk {
    Absent,
    /// Within TTL: serve as-is, no network.
    Fresh(Vec<CatalogEntry>, i64),
    /// Past TTL: serve immediately AND refresh behind it.
    Stale(Vec<CatalogEntry>, i64),
}

impl CatalogDisk {
    fn entries(&self) -> Option<&Vec<CatalogEntry>> {
        match self {
            CatalogDisk::Absent => None,
            CatalogDisk::Fresh(e, _) | CatalogDisk::Stale(e, _) => Some(e),
        }
    }
    fn age_label(&self, now: i64) -> String {
        match self {
            CatalogDisk::Absent => String::new(),
            CatalogDisk::Fresh(_, at) | CatalogDisk::Stale(_, at) => {
                catalog_age_label(now - *at)
            }
        }
    }
}

/// Decide the state from a read result. Pure — the
/// TTL boundary cases are --selftest fixtures.
fn catalog_disk_decide(
    parsed: Option<(i64, Vec<CatalogEntry>)>,
    now: i64,
) -> CatalogDisk {
    match parsed {
        None => CatalogDisk::Absent,
        Some((at, entries)) => {
            if catalog_is_stale(at, now) {
                CatalogDisk::Stale(entries, at)
            } else {
                CatalogDisk::Fresh(entries, at)
            }
        }
    }
}

fn catalog_disk_state(system: &str, now: i64) -> CatalogDisk {
    catalog_disk_decide(read_catalog_disk(system), now)
}

/// The load plan for opening (or refreshing) a catalog:
/// which entries, if any, render immediately, and
/// whether a live fetch follows. Pure. `force` is the
/// Refresh button: it never renders the saved copy
/// first (the window already shows rows) and always
/// goes live — the copy survives in the state as the
/// fallback if the fetch comes back empty.
fn catalog_plan(
    disk: &CatalogDisk,
    force: bool,
) -> (Option<Vec<CatalogEntry>>, bool) {
    if force {
        return (None, true);
    }
    match disk {
        CatalogDisk::Absent => (None, true),
        CatalogDisk::Fresh(e, _) => (Some(e.clone()), false),
        CatalogDisk::Stale(e, _) => (Some(e.clone()), true),
    }
}

/// Everything downloadable for one enhanced system.
/// The two systems' sources are assembled separately
/// and never merged — the separation starts here.
fn catalog_entries_live(system: &str) -> Vec<CatalogEntry> {
    match system {
        "Super Nintendo" => {
            let zeldix: Vec<CatalogEntry> = zeldix_threads()
                .into_iter()
                .map(|(title, url)| CatalogEntry {
                    title,
                    url,
                    source: "Zeldix".to_string(),
                    kind: "MSU-1 patch".to_string(),
                })
                .collect();
            merge_catalog_entries(vec![
                zeldix,
                fetch_archive_catalog(&["title:(MSU-1)"], "MSU-1 pack"),
                fetch_github_catalog(false),
            ])
        }
        "Sega Genesis" => {
            let minerva: Vec<CatalogEntry> = minerva_index()
                .into_iter()
                .map(|(fname, url)| CatalogEntry {
                    title: Path::new(&fname)
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or(fname),
                    url,
                    source: "Minerva".to_string(),
                    kind: "MD+ pack".to_string(),
                })
                .collect();
            merge_catalog_entries(vec![
                fetch_archive_catalog(
                    &["title:(msu-md)", "(msu-md) AND mediatype:software"],
                    "MD+ pack",
                ),
                minerva,
                fetch_github_catalog(true),
            ])
        }
        _ => vec![],
    }
}

/// The HackEntry a catalog row builds from. Zeldix rows
/// point at thread pages; like search_zeldix, the thread
/// is resolved to its direct patch link first (MSU links
/// preferred), so the build chain sees exactly the entry
/// shape the Find window builds from.
fn catalog_build_entry(entry: &CatalogEntry) -> Result<HackEntry, String> {
    if entry.source == "Zeldix" {
        let agent = http_agent(40);
        let page = agent
            .get(&entry.url)
            .set(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36",
            )
            .call()
            .map_err(|e| format!("Couldn't open the Zeldix thread: {e}"))?
            .into_string()
            .map_err(|e| e.to_string())?;
        let link = parse_zeldix_thread(&page)
            .into_iter()
            .next()
            .ok_or("The Zeldix thread for this one lists no direct patch link EmuFront can fetch.")?;
        return Ok(HackEntry {
            title: format!("{} — MSU-1 patch", entry.title),
            url: link,
        });
    }
    Ok(HackEntry {
        title: entry.title.clone(),
        url: entry.url.clone(),
    })
}

fn extract_archive(archive: &Path, dest: &Path) -> Result<Vec<PathBuf>, String> {
    let _ = std::fs::create_dir_all(dest);
    let st = Command::new(SEVEN_ZIP)
        .creation_flags(0x0800_0000)
        .args([
            "x",
            "-y",
            &format!("-o{}", dest.to_string_lossy()),
            archive.to_string_lossy().as_ref(),
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if !st.status.success() {
        return Err("Couldn't extract the download (7-Zip failed).".into());
    }
    let mut files = Vec::new();
    let mut stack = vec![dest.to_path_buf()];
    while let Some(d) = stack.pop() {
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else {
                    files.push(p);
                }
            }
        }
    }
    Ok(files)
}

fn stage_msu_assets(src_dir: &Path, dest: &Path, rom_stem: &str) {
    let Ok(rd) = std::fs::read_dir(src_dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
        let lower = name.to_lowercase();
        let new_name = if lower.ends_with(".msu") {
            format!("{rom_stem}.msu")
        } else if lower.ends_with(".pcm") {
            // Python's rule: a trailing track number becomes
            // "<rom stem>-<n>.pcm"; a numberless file keeps
            // its own name.
            pcm_staged_name(&name, rom_stem)
        } else if lower.ends_with(".bml") {
            name.clone()
        } else if lower.ends_with(".cue") {
            // MD+/MSU-MD: the cue MUST carry the ROM's stem
            // (Genesis Plus GX matches them by basename).
            format!("{rom_stem}.cue")
        } else if lower.ends_with(".wav") || lower.ends_with(".ogg") {
            // Cue sheets reference audio files by their
            // original names -- never rename these.
            name.clone()
        } else {
            continue;
        };
        let _ = std::fs::copy(&p, dest.join(new_name));
    }
}

fn pick_patch_for_rom(patches: &[PathBuf], rom_len: usize) -> Option<PathBuf> {
    if patches.len() == 1 {
        return patches.first().cloned();
    }
    let mb = rom_len as f64 / 1_048_576.0;
    let tokens: &[&str] = if mb <= 1.05 {
        &["1mb"]
    } else if mb <= 1.55 {
        &["1.5mb", "15mb", "1_5mb"]
    } else if mb <= 2.1 {
        &["2mb"]
    } else if mb <= 3.1 {
        &["3mb"]
    } else {
        &["4mb"]
    };
    for t in tokens {
        for p in patches {
            if p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
                .replace(' ', "")
                .contains(t)
            {
                return Some(p.clone());
            }
        }
    }
    patches.first().cloned()
}

fn watch_downloads_for_patch() -> Option<PathBuf> {
    // The signed-in user's Downloads folder.
    let dl_buf = std::env::var("USERPROFILE")
        .map(|h| PathBuf::from(h).join("Downloads"))
        .unwrap_or_else(|_| PathBuf::from("Downloads"));
    let dl = dl_buf.as_path();
    let snapshot = |dir: &Path| -> std::collections::HashMap<String, std::time::SystemTime> {
        let mut m: std::collections::HashMap<String, std::time::SystemTime> = Default::default();
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                if let Ok(md) = e.metadata() {
                    m.insert(
                        e.file_name().to_string_lossy().to_string(),
                        md.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                    );
                }
            }
        }
        m
    };
    let before = snapshot(dl);
    for _ in 0..120 {
        std::thread::sleep(std::time::Duration::from_secs(2));
        if let Ok(rd) = std::fs::read_dir(dl) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                let lower = name.to_lowercase();
                if !(lower.ends_with(".ips")
                    || lower.ends_with(".ups")
                    || lower.ends_with(".bps")
                    || lower.ends_with(".zip")
                    || lower.ends_with(".rar")
                    || lower.ends_with(".7z"))
                {
                    continue;
                }
                if lower.contains(".crdownload") || lower.contains(".part") {
                    continue;
                }
                let is_new = match before.get(&name) {
                    None => true,
                    Some(t0) => e
                        .metadata()
                        .and_then(|m| m.modified())
                        .map(|t| t > *t0)
                        .unwrap_or(false),
                };
                if is_new {
                    return Some(e.path());
                }
            }
        }
    }
    None
}

/// Download a Plaza entry and build/install it for `game`.
/// Blocking — run on a worker thread.
// ---------------- MSU-1 / MD+ download chain -----------------
// Port of the Python build's run_download_chain / download_hack:
// resolve an entry to files, classify them (patch / bundled
// ROM / music / audio companions), then build, install, or
// save-and-hunt. Used by the "Find MSU-1/MD+ version..." flow
// and by the generic hacks flow (which previously had no
// music-only path at all).

const PATCH_EXTS: [&str; 3] = [".ips", ".ups", ".bps"];
const MSU_MUSIC_EXTS: [&str; 3] = [".msu", ".pcm", ".bml"];
const MSU_AUDIO_EXTS: [&str; 3] = [".cue", ".wav", ".ogg"];
const PACK_ROM_EXTS: [&str; 12] = [
    ".sfc", ".smc", ".bin", ".md", ".gen", ".nes", ".gb", ".gbc", ".gba", ".n64",
    ".z64", ".v64",
];

fn ext_of(p: &Path) -> String {
    p.extension()
        .map(|x| format!(".{}", x.to_string_lossy().to_lowercase()))
        .unwrap_or_default()
}

/// A resolved download, classified. `dirs` holds every folder
/// the files came from (extraction root + each file's parent)
/// so companion staging sees the whole pack, not one level.
#[derive(Default)]
struct Unpacked {
    patches: Vec<PathBuf>,
    roms: Vec<PathBuf>,
    music: Vec<PathBuf>,
    audio: Vec<PathBuf>,
    dirs: Vec<PathBuf>,
    /// Set when this pack was fetched by the loose-file
    /// fallback after the item's bundled archive failed:
    /// surfaced with the build result so the detour is
    /// never silent. Not carried by merge/classify.
    note: Option<String>,
}

impl Unpacked {
    fn has_audio(&self) -> bool {
        !self.music.is_empty() || !self.audio.is_empty()
    }
    fn merge(&mut self, other: Unpacked) {
        self.patches.extend(other.patches);
        self.roms.extend(other.roms);
        self.music.extend(other.music);
        self.audio.extend(other.audio);
        for d in other.dirs {
            if !self.dirs.contains(&d) {
                self.dirs.push(d);
            }
        }
    }
}

/// Classify a set of loose files (an extraction's contents, or
/// an archive.org item's own files) into an Unpacked.
fn classify_files(files: Vec<PathBuf>, root: &Path) -> Unpacked {
    let mut out = Unpacked {
        dirs: vec![root.to_path_buf()],
        ..Default::default()
    };
    let add_dir = |out: &mut Unpacked, f: &Path| {
        if let Some(p) = f.parent() {
            let p = p.to_path_buf();
            if !out.dirs.contains(&p) {
                out.dirs.push(p);
            }
        }
    };
    for f in files {
        let e = ext_of(&f);
        if PATCH_EXTS.contains(&e.as_str()) {
            add_dir(&mut out, &f);
            out.patches.push(f);
        } else if MSU_MUSIC_EXTS.contains(&e.as_str()) {
            add_dir(&mut out, &f);
            out.music.push(f);
        } else if MSU_AUDIO_EXTS.contains(&e.as_str()) {
            add_dir(&mut out, &f);
            out.audio.push(f);
        } else if PACK_ROM_EXTS.contains(&e.as_str()) {
            add_dir(&mut out, &f);
            out.roms.push(f);
        } else if e == ".bsz" {
            // bsnes patch container (a zip by another name):
            // open it and classify what is inside. A bare
            // program.rom/rom.rom takes the pack's stem as a
            // .sfc, like the Python extractor does.
            let stem = f
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "bsz".into());
            let sub = root.join(format!("bsz-{stem}"));
            if let Ok(inner) = extract_archive(&f, &sub) {
                let mut inner = inner;
                for p in inner.iter_mut() {
                    let n = p
                        .file_name()
                        .map(|x| x.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    if n == "program.rom" || n == "rom.rom" {
                        let np = p.with_file_name(format!("{stem}.sfc"));
                        if std::fs::rename(&*p, &np).is_ok() {
                            *p = np;
                        }
                    }
                }
                out.merge(classify_files(inner, &sub));
            }
        }
    }
    // Python's ROM guard: a bundled ROM counts only when the
    // pack carries music, or the ROM's own name marks it as
    // the CD-audio build (a stray ROM in a random archive is
    // not a pack).
    if out.music.is_empty() {
        out.roms.retain(|r| msu_nameish(r));
    }
    out
}

/// Turn a downloaded payload file into an Unpacked.
fn unpack_payload(payload: &Path, work: &Path) -> Result<Unpacked, String> {
    let ext = ext_of(payload);
    let parent = payload
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    if PATCH_EXTS.contains(&ext.as_str()) {
        return Ok(Unpacked {
            patches: vec![payload.to_path_buf()],
            dirs: vec![parent],
            ..Default::default()
        });
    }
    if MSU_MUSIC_EXTS.contains(&ext.as_str()) {
        return Ok(Unpacked {
            music: vec![payload.to_path_buf()],
            dirs: vec![parent],
            ..Default::default()
        });
    }
    if [".zip", ".rar", ".7z", ".bsz"].contains(&ext.as_str()) {
        let out_dir = work.join("x");
        let mut files = extract_archive(payload, &out_dir)?;
        if ext == ".bsz" {
            let stem = payload
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            for p in files.iter_mut() {
                let n = p
                    .file_name()
                    .map(|x| x.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if n == "program.rom" || n == "rom.rom" {
                    let np = p.with_file_name(format!("{stem}.sfc"));
                    if std::fs::rename(&*p, &np).is_ok() {
                        *p = np;
                    }
                }
            }
        }
        return Ok(classify_files(files, &out_dir));
    }
    // Anything else (a lone cue sheet, a bare ROM link...) is
    // not a usable pack on its own -- same call the Python
    // extractor makes.
    Ok(Unpacked {
        dirs: vec![parent],
        ..Default::default()
    })
}

/// Download a file URL into `work`; returns the local path.
/// (Named fetch_to_file: download_to_file(url, dest) already exists
/// for the texture-pack flow and returns a byte count.)
fn fetch_to_file(
    agent: &ureq::Agent,
    url: &str,
    work: &Path,
) -> Result<PathBuf, String> {
    let resp = agent
        .get(url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36")
        .call()
        .map_err(|e| format!("Download failed: {e}"))?;
    let fname = url
        .rsplit('/')
        .next()
        .unwrap_or("pack.bin")
        .split('?')
        .next()
        .unwrap_or("pack.bin")
        .to_string();
    let dest = work.join(if fname.is_empty() {
        "pack.bin".to_string()
    } else {
        fname
    });
    let mut data = Vec::new();
    resp.into_reader()
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() < 16 {
        return Err("The download came back empty.".into());
    }
    std::fs::write(&dest, &data).map_err(|e| e.to_string())?;
    Ok(dest)
}

/// Resolve a possibly-relative href against a page URL
/// (the slice of RFC 3986 joining these sites need).
fn resolve_url(base: &str, href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }
    let origin = match base.find("://") {
        Some(i) => {
            let rest = &base[i + 3..];
            match rest.find('/') {
                Some(j) => base[..i + 3 + j].to_string(),
                None => base.trim_end_matches('/').to_string(),
            }
        }
        None => return href.to_string(),
    };
    if let Some(stripped) = href.strip_prefix('/') {
        return format!("{origin}/{stripped}");
    }
    match base.rfind('/') {
        Some(i) if i > origin.len() => format!("{}{}", &base[..=i], href),
        _ => format!("{origin}/{href}"),
    }
}

/// One page fetch, shared by the link finder and the
/// --plaza-probe classification.
fn fetch_page_html(url: &str) -> Result<String, String> {
    let agent = http_agent(40);
    agent
        .get(url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36")
        .call()
        .map_err(|e| format!("Couldn't open the page: {e}"))?
        .into_string()
        .map_err(|e| e.to_string())
}

// ---------------- download link finder (v1.06) -----------
// IMPROVEMENT-REVIEW §4.3: the shipped finder read one
// page, double-quoted hrefs only, and stopped at the
// best score — so three real page shapes defeated it:
// a "download page" whose own page holds the file, a
// download offered as a form button instead of a link,
// and single-quoted/unquoted hrefs. All three are
// handled below inside the same ureq fetch path, and
// every fixture is pinned in --selftest. Pages gated
// by JavaScript or bot checks still defeat any fetcher;
// those keep the browser fallback, unchanged.

/// The extensions that make a URL a file rather than a
/// page — the set the shipped finder scored +5 for.
const DL_FILE_EXTS: [&str; 6] =
    [".zip", ".rar", ".7z", ".ips", ".ups", ".bps"];

/// Does this URL point at a downloadable file (query
/// and fragment don't count)? Pure.
fn is_file_url(u: &str) -> bool {
    let path = u.split(['?', '#']).next().unwrap_or("");
    let low = path.to_lowercase();
    DL_FILE_EXTS.iter().any(|e| low.ends_with(e))
}

/// Score one candidate link. Pure — the fixtures pin
/// the order: a file (+5) beats a download page (+4
/// for "download" in the URL, the shipped rule), and
/// "download" in the visible text (+3, new) rescues the
/// classic Plaza shape of an opaque URL behind a
/// "Download" label, where the label is the only signal.
fn score_page_link(url: &str, text: &str) -> i32 {
    let mut score = 0;
    if is_file_url(url) {
        score += 5;
    }
    if url.to_lowercase().contains("download") {
        score += 4;
    }
    if text.to_lowercase().contains("download") {
        score += 3;
    }
    score
}

/// Find ASCII `needle` in `hay` at or after byte `from`,
/// case-insensitively; the index is into the original
/// string (searching a lowercased copy would drift on
/// non-ASCII text).
fn find_ascii_ci(hay: &str, needle: &str, from: usize) -> Option<usize> {
    let h = hay.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() || h.len() < n.len() || from > h.len() - n.len() {
        return None;
    }
    let last = h.len() - n.len();
    let mut i = from;
    while i <= last {
        if h[i..i + n.len()].eq_ignore_ascii_case(n) {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// True when the character before byte `pos` can't be
/// part of a longer attribute name (so `href` found at
/// `pos` is the attribute, not the tail of `data-href`).
fn attr_name_boundary(html: &str, pos: usize) -> bool {
    if pos == 0 {
        return true;
    }
    let c = html.as_bytes()[pos - 1] as char;
    !(c.is_alphanumeric() || c == '-' || c == '_')
}

/// Read one attribute's value starting just after its
/// name: `="v"`, `='v'` or `=v`, with any spacing.
/// Returns the value and the index just past it.
fn attr_value(html: &str, at: usize) -> Option<(String, usize)> {
    let b = html.as_bytes();
    let mut i = at;
    while i < b.len() && (b[i] as char).is_whitespace() {
        i += 1;
    }
    if i >= b.len() || b[i] != b'=' {
        return None;
    }
    i += 1;
    while i < b.len() && (b[i] as char).is_whitespace() {
        i += 1;
    }
    if i >= b.len() {
        return None;
    }
    if b[i] == b'"' || b[i] == b'\'' {
        let q = b[i];
        let start = i + 1;
        let mut j = start;
        while j < b.len() && b[j] != q {
            j += 1;
        }
        Some((html[start..j].to_string(), (j + 1).min(html.len())))
    } else {
        let start = i;
        let mut j = start;
        while j < b.len() && !(b[j] as char).is_whitespace() && b[j] != b'>' {
            j += 1;
        }
        if j == start {
            return None;
        }
        Some((html[start..j].to_string(), j))
    }
}

/// Every candidate link a page offers, unresolved:
/// anchors in any quote style (the shipped finder saw
/// only href="...") with their visible text, plus form
/// actions with the form's text — including submit
/// `value=` labels, since a WordPress download button
/// is a form, not a link, and its label lives in the
/// attribute. Pure.
fn extract_page_links(html: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut i = 0;
    while let Some(pos) = find_ascii_ci(html, "href", i) {
        let parsed = if attr_name_boundary(html, pos) {
            attr_value(html, pos + 4)
        } else {
            None
        };
        let Some((href, past)) = parsed else {
            i = pos + 4;
            continue;
        };
        let text = match html[past..].find('>') {
            Some(gt) => {
                let inner_at = past + gt + 1;
                match find_ascii_ci(html, "</a>", inner_at) {
                    Some(end) => strip_tags(&html[inner_at..end]),
                    None => String::new(),
                }
            }
            None => String::new(),
        };
        out.push((href, text));
        i = past;
    }
    let mut i = 0;
    while let Some(pos) = find_ascii_ci(html, "<form", i) {
        let tag_end = html[pos..]
            .find('>')
            .map(|g| pos + g)
            .unwrap_or(html.len());
        if tag_end >= html.len() {
            break;
        }
        let mut action: Option<String> = None;
        if let Some(ap) = find_ascii_ci(&html[pos..tag_end], "action", 0) {
            if attr_name_boundary(html, pos + ap) {
                if let Some((v, _)) = attr_value(html, pos + ap + 6) {
                    action = Some(v);
                }
            }
        }
        if let Some(a) = action {
            if !a.trim().is_empty() {
                let close = find_ascii_ci(html, "</form>", tag_end)
                    .unwrap_or_else(|| (tag_end + 4096).min(html.len()));
                let inner = &html[tag_end + 1..close];
                let mut text = strip_tags(inner);
                let mut vi = 0;
                while let Some(vp) = find_ascii_ci(inner, "value", vi) {
                    match attr_value(inner, vp + 5) {
                        Some((v, past)) => {
                            if !v.is_empty() {
                                text.push(' ');
                                text.push_str(&v);
                            }
                            vi = past;
                        }
                        None => break,
                    }
                }
                out.push((a, text));
            }
        }
        i = tag_end + 1;
    }
    out
}

/// The best download candidate on one page, resolved
/// against the page's URL. Pure. Candidates compete in
/// two classes — files and pages — highest score wins
/// within a class, first one takes ties, and a file
/// winner always beats a page winner: that is the
/// shipped order (a file's +5 over a page's +4) kept
/// safe from the new text points, which exist to pick
/// the right PAGE among pages, not to outrank a file
/// that's already here. Non-links (javascript:,
/// mailto:, bare fragments) and a link back to the page
/// itself never compete.
fn pick_download_candidate(base: &str, html: &str) -> Option<String> {
    let mut best_file: Option<(i32, String)> = None;
    let mut best_page: Option<(i32, String)> = None;
    for (href, text) in extract_page_links(html) {
        let h = href.trim();
        let low = h.to_lowercase();
        if h.is_empty()
            || h.starts_with('#')
            || low.starts_with("javascript:")
            || low.starts_with("mailto:")
            || low.starts_with("data:")
            || low.starts_with("tel:")
        {
            continue;
        }
        let url = resolve_url(base, h);
        if url == base {
            continue;
        }
        let score = score_page_link(&url, &text);
        if score == 0 {
            continue;
        }
        let slot = if is_file_url(&url) {
            &mut best_file
        } else {
            &mut best_page
        };
        if slot.as_ref().map_or(true, |(s, _)| score > *s) {
            *slot = Some((score, url));
        }
    }
    best_file.or(best_page).map(|(_, u)| u)
}

/// The two-page resolution, with the fetch injected so
/// --selftest can pin the hop behavior — including the
/// cap — against fixture pages. Returns the winner and
/// how many pages were fetched (0, 1 or 2).
fn resolve_download<F: FnMut(&str) -> Result<String, String>>(
    entry_url: &str,
    mut fetch: F,
) -> Result<(Option<String>, u8), String> {
    if is_file_url(entry_url) {
        return Ok((Some(entry_url.to_string()), 0));
    }
    let html = fetch(entry_url)?;
    let Some(winner) = pick_download_candidate(entry_url, &html) else {
        return Ok((None, 1));
    };
    if is_file_url(&winner) {
        return Ok((Some(winner), 1));
    }
    // The one extra hop. Its failure is swallowed: the
    // first winner is still exactly what the pre-v1.06
    // finder would have returned for this page.
    match fetch(&winner) {
        Ok(html2) => {
            if let Some(w2) = pick_download_candidate(&winner, &html2) {
                if is_file_url(&w2) {
                    return Ok((Some(w2), 2));
                }
            }
            Ok((Some(winner), 2))
        }
        Err(_) => Ok((Some(winner), 1)),
    }
}

/// Find the real download link behind an entry page.
/// Port of Python's `find_download_on_page`, widened in
/// v1.06 (IMPROVEMENT-REVIEW §4.3): a direct file URL is
/// returned as-is; otherwise the best-scoring candidate
/// wins — and when the winner is another PAGE (the
/// classic "download page" shape), exactly one more hop
/// follows it to the file behind it. A chain needing a
/// third page returns the first page winner, exactly
/// what the old finder returned. Every URL returned
/// still passes through the caller's existing
/// fetch/unpack/classify path: finding more links never
/// loosens what counts as a valid download.
fn find_download_on_page(entry_url: &str) -> Result<Option<String>, String> {
    let (url, _) = resolve_download(entry_url, fetch_page_html)?;
    Ok(url)
}

/// archive.org/details/<ident> -> ident, when the URL is one.
fn archive_ident(url: &str) -> Option<String> {
    let rest = url.strip_prefix("https://archive.org/details/")?;
    let ident: String = rest
        .chars()
        .take_while(|c| *c != '/' && *c != '?' && *c != '#')
        .collect();
    if ident.is_empty() {
        None
    } else {
        Some(ident)
    }
}

fn archive_file_url(ident: &str, name: &str) -> String {
    let enc = name
        .split('/')
        .map(url_encode)
        .collect::<Vec<_>>()
        .join("/");
    format!("https://archive.org/download/{ident}/{enc}")
}

/// archive.org items are resolved through the metadata API
/// instead of page scraping: many real MSU-MD packs are
/// loose files (patch + cue + WAV tracks), not one archive.
/// Only the item's own "original" files count -- archive.org
/// also lists generated derivatives (.mp3/.flac previews of
/// the tracks), which must never be staged.

/// The names of an item's own files, from its metadata file
/// list: only entries marked source="original" count, so the
/// generated derivatives are excluded here, once, for every
/// route below. Pure -- --selftest pins it against a Shadow
/// Dancer-shaped listing.
fn original_file_names(files: &[serde_json::Value]) -> Vec<String> {
    files
        .iter()
        .filter(|f| f.get("source").and_then(|s| s.as_str()) == Some("original"))
        .filter_map(|f| {
            f.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())
        })
        .collect()
}

/// The bundled archive among an item's files, if it offers
/// one: a single archive beats fetching the set file by
/// file -- whenever the transfer actually completes.
fn bundled_archive_name(names: &[String]) -> Option<&str> {
    names.iter().map(|n| n.as_str()).find(|n| {
        let l = n.to_lowercase();
        l.ends_with(".zip") || l.ends_with(".7z") || l.ends_with(".rar")
    })
}

/// The item's loose files worth fetching one by one: patch,
/// MSU music, audio companions (cue/WAV/OGG), bundled ROMs
/// and .bsz containers. The bundled archive itself never
/// qualifies (its extensions match none of the sets).
fn loose_file_names(names: &[String]) -> Vec<&str> {
    names
        .iter()
        .map(|n| n.as_str())
        .filter(|n| {
            let e = ext_of(Path::new(n));
            PATCH_EXTS.contains(&e.as_str())
                || MSU_MUSIC_EXTS.contains(&e.as_str())
                || MSU_AUDIO_EXTS.contains(&e.as_str())
                || PACK_ROM_EXTS.contains(&e.as_str())
                || e == ".bsz"
        })
        .collect()
}

/// True when an Unpacked holds anything the build chain can
/// use. An archive that downloads but unpacks to one of
/// these is a corrupt/undersized result, not a pack.
fn unpack_has_content(u: &Unpacked) -> bool {
    !u.patches.is_empty() || !u.roms.is_empty() || u.has_audio()
}

/// What the archive.org metadata route produced for an item.
enum ArchiveOutcome {
    /// A classified pack. Unpacked.note is set when the
    /// loose-file fallback produced it.
    Got(Unpacked),
    /// Metadata unreachable, or nothing usable listed and no
    /// download attempted: the caller falls through to the
    /// page-link chain, as before.
    Miss,
    /// The bundled archive failed AND the loose-file
    /// fallback failed too. The message names both failures;
    /// the caller surfaces it rather than re-trying the same
    /// archive through the page chain.
    Failed(String),
}

fn fetch_archive_item(
    agent: &ureq::Agent,
    ident: &str,
    work: &Path,
) -> ArchiveOutcome {
    let resp = match agent
        .get(&format!("https://archive.org/metadata/{ident}"))
        .call()
    {
        Ok(r) => r,
        Err(_) => return ArchiveOutcome::Miss,
    };
    let v: serde_json::Value = match resp.into_json() {
        Ok(v) => v,
        Err(_) => return ArchiveOutcome::Miss,
    };
    let Some(files) = v.get("files").and_then(|f| f.as_array()) else {
        return ArchiveOutcome::Miss;
    };
    let originals = original_file_names(files);
    // Preferred route: the bundled archive, when the item
    // offers one. Its failure used to end the fetch here
    // (v1.02 as first shipped returned None): the 307 MB
    // Shadow Dancer zip stalled or timed out three times in
    // the ship session while the item's loose files -- the
    // same pack, one file at a time -- sat unusable. A
    // failed archive (network error, timeout, HTTP error
    // status, or a download that unpacks to nothing usable)
    // now falls through to the loose files below.
    let mut archive_err: Option<String> = None;
    if let Some(name) = bundled_archive_name(&originals) {
        match fetch_to_file(agent, &archive_file_url(ident, name), work)
            .and_then(|p| unpack_payload(&p, work))
        {
            Ok(u) if unpack_has_content(&u) => return ArchiveOutcome::Got(u),
            Ok(_) => {
                archive_err = Some(format!(
                    "the bundled archive ({name}) unpacked, but held no patch, ROM or music files"
                ));
            }
            Err(e) => {
                archive_err = Some(format!(
                    "the bundled archive ({name}) failed: {e}"
                ));
            }
        }
    }
    let interesting = loose_file_names(&originals);
    if interesting.is_empty() {
        return match archive_err {
            Some(e) => ArchiveOutcome::Failed(format!(
                "{e}; and the item lists no loose pack files to fall back to."
            )),
            None => ArchiveOutcome::Miss,
        };
    }
    match fetch_loose_files(agent, ident, work, &interesting) {
        Ok(mut u) => {
            if let Some(e) = &archive_err {
                u.note = Some(format!(
                    "Note: archive download failed — fetching the pack's loose files instead (this build used those). Archive error: {e}"
                ));
            }
            ArchiveOutcome::Got(u)
        }
        Err(loose_err) => match archive_err {
            Some(e) => ArchiveOutcome::Failed(format!(
                "Both download routes failed for this item. {e} The loose-file fallback also failed: {loose_err}"
            )),
            // No archive was offered, so a loose-file failure
            // is the metadata route simply missing, exactly
            // as before: the page-link chain gets its turn.
            None => ArchiveOutcome::Miss,
        },
    }
}

/// Download an item's loose files one by one into
/// work/item and classify the set with the same
/// classify_files the archive route feeds. All-or-nothing:
/// one failed file fails the set (a pack missing tracks is
/// worse than a clean error), and the error names the file.
/// No size floor on purpose: a legit 0-byte .msu marker is
/// part of real packs.
fn fetch_loose_files(
    agent: &ureq::Agent,
    ident: &str,
    work: &Path,
    names: &[&str],
) -> Result<Unpacked, String> {
    let dir = work.join("item");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut paths = Vec::new();
    for name in names {
        let local = dir.join(
            Path::new(name)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default(),
        );
        let resp = agent
            .get(&archive_file_url(ident, name))
            .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36")
            .call()
            .map_err(|e| format!("'{name}': {e}"))?;
        let mut data = Vec::new();
        resp.into_reader()
            .read_to_end(&mut data)
            .map_err(|e| format!("'{name}': {e}"))?;
        std::fs::write(&local, &data).map_err(|e| format!("'{name}': {e}"))?;
        paths.push(local);
    }
    if paths.is_empty() {
        return Err("no files were downloaded".into());
    }
    Ok(classify_files(paths, &dir))
}

/// https://github.com/<owner>/<repo>[/...] -> (owner, repo),
/// when the URL is one. Pure -- --selftest pins it.
fn github_repo_of(url: &str) -> Option<(String, String)> {
    let rest = url.strip_prefix("https://github.com/")?;
    let mut parts = rest.split('/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    let ok = |s: &str| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    if ok(owner) && ok(repo) {
        Some((owner.to_string(), repo.to_string()))
    } else {
        None
    }
}

/// A GitHub release asset worth downloading into the pack
/// pipeline: patches, MSU music, audio companions, bundled
/// ROMs, and archives that might hold any of those -- the
/// file families the archive.org loose-file route accepts,
/// plus the archives themselves. Pure.
fn github_asset_ok(name: &str) -> bool {
    let e = ext_of(Path::new(name));
    PATCH_EXTS.contains(&e.as_str())
        || MSU_MUSIC_EXTS.contains(&e.as_str())
        || MSU_AUDIO_EXTS.contains(&e.as_str())
        || PACK_ROM_EXTS.contains(&e.as_str())
        || [".zip", ".rar", ".7z", ".bsz"].contains(&e.as_str())
}

/// The (name, download URL) assets of a release that the
/// pipeline can use, in listed order. Pure -- --selftest
/// pins it against the real Release 1.0 shape (a single
/// patch.ips asset) and a mixed pack-shaped release.
fn github_pick_assets(release: &serde_json::Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(assets) = release.get("assets").and_then(|a| a.as_array()) {
        for a in assets {
            let name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let url = a
                .get("browser_download_url")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !name.is_empty() && !url.is_empty() && github_asset_ok(name) {
                out.push((name.to_string(), url.to_string()));
            }
        }
    }
    out
}

/// The repo-archive URL of a release (its source zipball):
/// the fallback when no asset qualifies. Pure.
fn github_release_zipball(release: &serde_json::Value) -> Option<String> {
    release
        .get("zipball_url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// The repo's latest usable release JSON: /releases/latest,
/// else the first entry of /releases (a repo whose newest
/// release is a prerelease has no /latest). None when the
/// repo has no releases at all.
fn github_latest_release(
    agent: &ureq::Agent,
    owner: &str,
    repo: &str,
) -> Option<serde_json::Value> {
    if let Some(v) = github_api_get(
        agent,
        &format!("https://api.github.com/repos/{owner}/{repo}/releases/latest"),
    ) {
        if v.get("tag_name").is_some() {
            return Some(v);
        }
    }
    let v = github_api_get(
        agent,
        &format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=5"),
    )?;
    v.as_array()?.first().cloned()
}

/// Raw download to an explicit path (the GitHub route
/// names its own files: fetch_to_file derives the name
/// from the URL, which fails for zipball endpoints whose
/// URL ends in "zipball"). No size floor, like
/// fetch_loose_files: a 0-byte .msu marker is a legit
/// pack member.
fn fetch_url_to(agent: &ureq::Agent, url: &str, dest: &Path) -> Result<(), String> {
    let resp = agent
        .get(url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36")
        .call()
        .map_err(|e| format!("Download failed: {e}"))?;
    let mut data = Vec::new();
    resp.into_reader()
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    std::fs::write(dest, &data).map_err(|e| e.to_string())?;
    Ok(())
}

/// Resolve a GitHub hunt result to an Unpacked: the latest
/// release's usable assets (patch / audio / archives) when
/// it has any; otherwise the repo archive, whose tree the
/// classifier scans for pack files (patch repos keep their
/// .ips/.bps in PATCH/-style folders -- infinest's does).
/// Everything lands in the same classify -> patch -> stage
/// pipeline as every other source, so the pick stays the
/// only manual step.
fn fetch_github_repo(
    agent: &ureq::Agent,
    owner: &str,
    repo: &str,
    work: &Path,
) -> Result<Unpacked, String> {
    let dir = work.join("github");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let release = github_latest_release(agent, owner, repo);
    let mut out = Unpacked::default();
    let mut loose: Vec<PathBuf> = Vec::new();
    if let Some(rel) = &release {
        for (name, url) in github_pick_assets(rel) {
            let local = dir.join(
                Path::new(&name)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default(),
            );
            fetch_url_to(agent, &url, &local)?;
            if [".zip", ".rar", ".7z", ".bsz"].contains(&ext_of(&local).as_str()) {
                out.merge(unpack_payload(&local, &dir)?);
            } else {
                loose.push(local);
            }
        }
    }
    if !loose.is_empty() {
        out.merge(classify_files(loose, &dir));
    }
    if unpack_has_content(&out) {
        return Ok(out);
    }
    // No usable release assets (or no release at all): the
    // repo archive itself is the download.
    let zip_url = release
        .as_ref()
        .and_then(github_release_zipball)
        .unwrap_or_else(|| format!("https://api.github.com/repos/{owner}/{repo}/zipball"));
    let zp = dir.join("repo.zip");
    fetch_url_to(agent, &zip_url, &zp)?;
    let u = unpack_payload(&zp, &dir)?;
    if unpack_has_content(&u) {
        Ok(u)
    } else {
        Err(format!(
            "The GitHub repo {owner}/{repo} held no patch, ROM or music files EmuFront can use."
        ))
    }
}

/// Resolve a search entry to an Unpacked: the payload
/// download, in this chain's priority order (archive.org
/// metadata -> GitHub repo -> best page link ->, when
/// allowed, the browser download watch the generic flow
/// uses).
/// Minerva entry URLs are the archive's file pages:
/// "https://minerva-archive.org/rom?id=<digits>". Pure.
fn minerva_rom_of(url: &str) -> Option<String> {
    let rest = url.strip_prefix("https://minerva-archive.org/rom?id=")?;
    let id: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

/// Minerva fetch: the file page is asked for the pack
/// directly, and only a real archive payload (zip / 7z /
/// rar magic) may continue into unpack_payload. As wired
/// (Oct 24 2026) Minerva's file pages were erroring --
/// its database backend was down -- and Minerva's own
/// FAQ says its files go out as torrents, which EmuFront
/// cannot fetch; both cases land on the same plain
/// explanation instead of the page scrape or the browser
/// watch other sources fall back to. If Minerva ever
/// serves the file itself at that URL, the magic check
/// lets it through and the normal chain takes over.
fn fetch_minerva_pack(
    agent: &ureq::Agent,
    entry: &HackEntry,
    work: &Path,
) -> Result<Unpacked, String> {
    const TORRENT_ONLY: &str = "Minerva lists this pack, but Minerva only hands its files out as torrents — there is no direct download for EmuFront to fetch, so this one can't be built automatically. (The collection it belongs to is also on archive.org, which EmuFront can download.)";
    // The pack filename rides in the entry title, minus
    // the source tag; keep just the final path segment.
    let titled = entry
        .title
        .strip_suffix(" · Minerva")
        .unwrap_or(&entry.title);
    let fname = titled
        .rsplit(|c| c == '/' || c == '\\')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    let fname = if fname.is_empty() {
        "minerva-pack.7z".to_string()
    } else {
        fname
    };
    let dest = work.join(&fname);
    fetch_url_to(agent, &entry.url, &dest).map_err(|_| TORRENT_ONLY.to_string())?;
    let mut head = [0u8; 8];
    let got = std::fs::File::open(&dest)
        .and_then(|mut f| IoRead::read(&mut f, &mut head))
        .unwrap_or(0);
    let is_archive = got >= 4
        && (head.starts_with(b"PK\x03\x04")
            || head.starts_with(b"Rar!")
            || head.starts_with(b"7z\xbc\xaf\x27\x1c"));
    if !is_archive {
        let _ = std::fs::remove_file(&dest);
        return Err(TORRENT_ONLY.to_string());
    }
    unpack_payload(&dest, work)
}

fn fetch_unpacked(
    entry: &HackEntry,
    work: &Path,
    allow_browser: bool,
) -> Result<Unpacked, String> {
    let _ = std::fs::create_dir_all(work);
    let agent = http_agent(60);
    if let Some(ident) = archive_ident(&entry.url) {
        match fetch_archive_item(&agent, &ident, work) {
            ArchiveOutcome::Got(u) => return Ok(u),
            // Both routes were tried and failed; surface the
            // combined error instead of falling through to a
            // page chain that would re-pick the same archive.
            ArchiveOutcome::Failed(msg) => return Err(msg),
            ArchiveOutcome::Miss => {}
        }
    }
    if let Some((owner, repo)) = github_repo_of(&entry.url) {
        return fetch_github_repo(&agent, &owner, &repo, work);
    }
    if minerva_rom_of(&entry.url).is_some() {
        return fetch_minerva_pack(&agent, entry, work);
    }
    match find_download_on_page(&entry.url)? {
        Some(u) => {
            let p = fetch_to_file(&agent, &u, work)?;
            unpack_payload(&p, work)
        }
        None => {
            if !allow_browser {
                return Err(
                    "That result needs a browser download, so it can't be fetched automatically."
                        .into(),
                );
            }
            let _ = Command::new("cmd")
                .args(["/C", "start", "", &entry.url])
        .creation_flags(0x0800_0000)
                .spawn();
            let p = watch_downloads_for_patch().ok_or(
                "No download appeared in your Downloads folder (waited 4 minutes).",
            )?;
            unpack_payload(&p, work)
        }
    }
}

/// Copy a pack's music + audio companion files into the
/// game's Patches folder (Python's flow keeps them there
/// until a patch build stages them next to the ROM).
fn save_companions(u: &Unpacked, pdir: &Path) {
    let _ = std::fs::create_dir_all(pdir);
    for f in u.music.iter().chain(u.audio.iter()) {
        if let Some(name) = f.file_name() {
            let dest = pdir.join(name);
            if dest != *f {
                let _ = std::fs::copy(f, &dest);
            }
        }
    }
}

/// Patch flow shared by both chains: copy the patch into the
/// game's Patches folder, apply it to a COPY of the ROM,
/// write "<stem> [<title>].<ext>" into the hacked/enhanced
/// folder, and stage the pack's music + companions against
/// the BUILT file's stem -- so the ROM and its .cue always
/// share a name (the MD+/MSU-MD hard requirement), and music
/// saved by an earlier music-only download is picked up from
/// the Patches folder.
fn build_hacked_from(
    game: &Game,
    got: &Unpacked,
    title: &str,
    enhanced: bool,
) -> Result<String, String> {
    let dest_dir = hacked_dir(game, enhanced);
    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    let rom_bytes = std::fs::read(&game.path).map_err(|e| e.to_string())?;
    let patch_path = pick_patch_for_rom(&got.patches, rom_bytes.len())
        .ok_or("No patch file found.")?;
    let pdir = data_dir().join("Patches").join(safe_folder(&game.name));
    let _ = std::fs::create_dir_all(&pdir);
    let _ = std::fs::copy(
        &patch_path,
        pdir.join(patch_path.file_name().unwrap_or_default()),
    );
    let patch_bytes = std::fs::read(&patch_path).map_err(|e| e.to_string())?;
    let built = apply_patch(&rom_bytes, &patch_bytes, &ext_of(&patch_path))?;
    let rom_stem = Path::new(&game.path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let rom_ext = Path::new(&game.path)
        .extension()
        .map(|x| x.to_string_lossy().to_string())
        .unwrap_or_default();
    // The [<title>] tag goes into a filename: keep it to
    // characters Windows accepts (Python sanitizes the same
    // way in make_patched_copy).
    let tag: String = title
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || ['_', '-', ' '].contains(c))
        .take(60)
        .collect::<String>()
        .trim()
        .to_string();
    let tag = if tag.is_empty() { "hack".to_string() } else { tag };
    let out_name = format!("{rom_stem} [{tag}].{rom_ext}");
    let out_path = dest_dir.join(&out_name);
    std::fs::write(&out_path, &built).map_err(|e| e.to_string())?;
    let out_stem = out_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    for d in &got.dirs {
        stage_msu_assets(d, &dest_dir, &out_stem);
    }
    stage_msu_assets(&pdir, &dest_dir, &out_stem);
    let staged_audio = std::fs::read_dir(&dest_dir)
        .map(|rd| {
            rd.flatten().any(|e| {
                let n = e.file_name().to_string_lossy().to_lowercase();
                n.ends_with(".msu") || n.ends_with(".pcm") || n.ends_with(".cue")
            })
        })
        .unwrap_or(false);
    let new_name = format!("{} [{}]", game.name, title);
    register_manual_game(serde_json::json!({
        "name": new_name,
        "path": out_path.to_string_lossy(),
        "system": game.system,
        "emulator": "",
        "patch": patch_path.file_name().unwrap_or_default().to_string_lossy(),
        "cheats_text": "",
        "cheats_enabled": false,
    }))?;
    Ok(format!(
        "✅ Done: built '{new_name}' into {} — original ROM untouched. Just hit LAUNCH.{}",
        if enhanced { "Enhanced ROMs" } else { "Hacked ROMs" },
        if staged_audio {
            " 🎵 Music staged next to it."
        } else {
            ""
        }
    ))
}

/// Complete-pack flow: the download already holds the
/// pre-patched ROM -- install it as-is and stage its audio.
fn install_pack_rom_from(
    game: &Game,
    got: &Unpacked,
    enhanced: bool,
) -> Result<String, String> {
    let dest_dir = hacked_dir(game, enhanced);
    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    let rom = got.roms.first().ok_or("No bundled ROM found.")?;
    let stem = rom
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = rom
        .extension()
        .map(|x| x.to_string_lossy().to_string())
        .unwrap_or_default();
    let dest = dest_dir.join(format!("{stem}.{ext}"));
    std::fs::copy(rom, &dest).map_err(|e| e.to_string())?;
    for d in &got.dirs {
        stage_msu_assets(d, &dest_dir, &stem);
    }
    let name = clean_game_name(&rom.file_name().unwrap_or_default().to_string_lossy());
    register_manual_game(serde_json::json!({
        "name": name,
        "path": dest.to_string_lossy(),
        "system": game.system,
        "emulator": "",
        "patch": "",
        "cheats_text": "",
        "cheats_enabled": false,
    }))?;
    Ok(format!(
        "✅ Pack installed: {name} — find it in the wall and hit LAUNCH."
    ))
}

/// Music-only outcome: the pack's audio is saved, so hunt the
/// same sources for the matching patch and build with it if
/// one turns up (Python's two-step design: music first, the
/// patch attaches when found -- including by a later build).
fn music_only_flow(
    game: &Game,
    got: &Unpacked,
    entry: &HackEntry,
    enhanced: bool,
    work: &Path,
) -> Result<String, String> {
    let term = msu_term(game);
    let pdir = data_dir().join("Patches").join(safe_folder(&game.name));
    save_companions(got, &pdir);
    let saved = got.music.len() + got.audio.len();
    for (i, cand) in search_msu1(game).iter().enumerate() {
        if cand.url == entry.url {
            continue;
        }
        let w2 = work.join(format!("hunt{i}"));
        if let Ok(g2) = fetch_unpacked(cand, &w2, false) {
            save_companions(&g2, &pdir);
            if !g2.patches.is_empty() {
                return build_hacked_from(game, &g2, &cand.title, enhanced);
            }
        }
    }
    Ok(format!(
        "🎵 {term} music pack downloaded — {saved} music file(s) saved for {}. I hunted for the matching {term} patch too, but couldn't find one online; once the patch is attached, building the hacked ROM adds this music automatically.",
        game.name
    ))
}

/// The "Find MSU-1 / MD+ version..." worker: resolve the
/// chosen entry and run the chain.
fn download_and_build_msu(
    game: &Game,
    entry: &HackEntry,
    enhanced: bool,
) -> Result<String, String> {
    let work = std::env::temp_dir().join(format!("efr-msu-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&work);
    let got = fetch_unpacked(entry, &work, true)?;
    // Set when the pack arrived via the loose-file fallback:
    // appended to the result so the user sees the detour.
    let note = got.note.clone();
    let result = if !got.patches.is_empty() {
        build_hacked_from(game, &got, &entry.title, enhanced)
    } else if !got.roms.is_empty() {
        install_pack_rom_from(game, &got, enhanced)
    } else if got.has_audio() {
        music_only_flow(game, &got, entry, enhanced, &work)
    } else {
        Err(format!(
            "Downloaded it, but found no .ips/.ups/.bps patch inside, and no {} music files either.",
            msu_term(game)
        ))
    };
    match (result, note) {
        (Ok(msg), Some(n)) => Ok(format!("{msg}\n{n}")),
        (r, _) => r,
    }
}

fn download_and_build(
    game: &Game,
    entry: &HackEntry,
    enhanced: bool,
) -> Result<String, String> {
    let agent = http_agent(60);
    // 1) get the entry page and look for direct file links
    let html = agent
        .get(&entry.url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36")
        .call()
        .map_err(|e| format!("Couldn't open the hack page: {e}"))?
        .into_string()
        .map_err(|e| e.to_string())?;
    let mut file_url: Option<String> = None;
    for part in html.split("href=\"").skip(1) {
        let Some(end) = part.find('"') else { continue };
        let href = &part[..end];
        let lower = href.to_lowercase();
        let is_file = lower.contains("/patch/")
            || [".ips", ".ups", ".bps", ".zip", ".rar", ".7z"]
                .iter()
                .any(|x| lower.ends_with(x));
        if is_file && (href.starts_with("http") || href.starts_with('/')) {
            file_url = Some(if href.starts_with("http") {
                href.to_string()
            } else {
                format!("https://romhackplaza.org{href}")
            });
            break;
        }
    }
    let work = std::env::temp_dir().join(format!("efr-hack-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&work);
    let payload: PathBuf = match file_url {
        Some(u) => {
            let resp = agent
                .get(&u)
                .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36")
                .call()
                .map_err(|e| format!("Download failed: {e}"))?;
            let fname = u
                .rsplit('/')
                .next()
                .unwrap_or("patch.bin")
                .split('?')
                .next()
                .unwrap_or("patch.bin")
                .to_string();
            let dest = work.join(if fname.is_empty() {
                "patch.bin".into()
            } else {
                fname
            });
            let mut data = Vec::new();
            resp.into_reader()
                .read_to_end(&mut data)
                .map_err(|e| e.to_string())?;
            if data.len() < 16 {
                return Err("The download came back empty.".into());
            }
            std::fs::write(&dest, &data).map_err(|e| e.to_string())?;
            dest
        }
        None => {
            // Livewire click-through: open the page in his browser
            // and watch Downloads for the file, like the Python app.
            let _ = Command::new("cmd")
                .args(["/C", "start", "", &entry.url])
        .creation_flags(0x0800_0000)
                .spawn();
            watch_downloads_for_patch()
                .ok_or("No download appeared in your Downloads folder (waited 4 minutes).")?
        }
    };
    // 2) unpack + classify, then run the shared chain (the
    // same one the MSU/MD+ flow uses): patch -> build;
    // bundled ROM -> install; audio only -> save + patch hunt.
    let got = unpack_payload(&payload, &work)?;
    if !got.patches.is_empty() {
        return build_hacked_from(game, &got, &entry.title, enhanced);
    }
    if !got.roms.is_empty() {
        return install_pack_rom_from(game, &got, enhanced);
    }
    if got.has_audio() {
        return music_only_flow(game, &got, entry, enhanced, &work);
    }
    Err(format!(
        "That download held no patch file and no ready ROM, and no {} music files either.",
        msu_term(game)
    ))
}

// ---------------- game info (Wikipedia) ------------------------------
#[derive(Clone)]
struct GameInfo {
    title: String,
    extract: String,
}

/// Remove "[...]" groups (with any space before them) from a
/// name. Helper for name_variants; mirrors the regex the
/// Python build applies to raw file stems.
fn drop_bracket_groups(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '[' {
            while out.ends_with(' ') {
                out.pop();
            }
            for c2 in chars.by_ref() {
                if c2 == ']' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Well-known title mix-ups for hack searches (the MSU-1
/// scene labels the SNES games by their Japanese numbering).
/// Mirrors Python's SEARCH_ALIASES.
fn search_alias(name: &str) -> Option<&'static str> {
    match name {
        "Final Fantasy II" => Some("Final Fantasy IV"),
        "Final Fantasy III" => Some("Final Fantasy VI"),
        _ => None,
    }
}

/// Candidate search titles for a game, best first, max 3:
/// display name, cleaned file name, then the raw stem (with
/// and without its archive catalog prefix). Port of Python's
/// `name_variants`.
fn name_variants(game: &Game) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |v: &str| {
        let v = v.replace('_', " ");
        let v = v.split_whitespace().collect::<Vec<_>>().join(" ");
        if v.len() >= 2
            && !out
                .iter()
                .any(|x: &String| x.to_lowercase() == v.to_lowercase())
        {
            out.push(v);
        }
    };
    push(&game.name);
    push(&clean_game_name(&game.path));
    let stem = Path::new(&game.path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let raw = drop_bracket_groups(&stem);
    push(&strip_catalog_prefix_full(&raw));
    push(&raw);
    out.truncate(3);
    out
}

fn wiki_summary(title: &str) -> Option<GameInfo> {
    let agent = http_agent(20);
    let url = format!(
        "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
        url_encode(&title.replace(' ', "_"))
    );
    let resp = agent.get(&url).call().ok()?;
    let v: serde_json::Value = resp.into_json().ok()?;
    if v.get("type").and_then(|x| x.as_str()) == Some("disambiguation") {
        return None;
    }
    let extract = v.get("extract")?.as_str()?.to_string();
    if extract.len() < 60 {
        return None;
    }
    Some(GameInfo {
        title: v
            .get("title")
            .and_then(|x| x.as_str())
            .unwrap_or(title)
            .to_string(),
        extract,
    })
}

fn fetch_game_info(game: &Game) -> Option<GameInfo> {
    let name = clean_game_name(&game.name);
    for cand in [
        name.clone(),
        format!("{name} (video game)"),
    ] {
        if let Some(info) = wiki_summary(&cand) {
            let low = info.extract.to_lowercase();
            if low.contains("video game") || low.contains("arcade game") {
                return Some(info);
            }
        }
    }
    // search fallback
    let agent = http_agent(20);
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=opensearch&search={}&limit=3&namespace=0&format=json",
        url_encode(&format!("{name} video game"))
    );
    let resp = agent.get(&url).call().ok()?;
    let v: serde_json::Value = resp.into_json().ok()?;
    let titles = v.get(1)?.as_array()?;
    for t in titles {
        if let Some(ts) = t.as_str() {
            if let Some(info) = wiki_summary(ts) {
                return Some(info);
            }
        }
    }
    None
}

// ---------------- texture packs ------------------------------------
// Real downloading. A curated catalog of verified packs plus a
// generic "install from URL" path, per-emulator folder layouts:
//   Dolphin  -> <user>/Load/Textures/<GAMEID>/
//   PCSX2    -> <pcsx2>/textures/<SERIAL>/replacements/
//   GLideN64 -> loose PNGs in Plugin/GFX/hires_texture/<ROM NAME>/,
//               .hts/.htc cache packs in Plugin/GFX/cache/
//   Mesen    -> <mesen>/HdPacks/<ROM file stem>/
//   DuckStation -> <user dir>/textures/<SERIAL>/replacements/
// PS1 packs target DuckStation; its serial comes from the disc's
// SYSTEM.CNF when the image can be read, else from the game name.

#[derive(Clone, Copy, PartialEq, Debug)]
enum TexKind {
    Dolphin,
    Pcsx2,
    GlideN64,
    Mesen,
    DuckStation,
    Unsupported,
}

#[derive(Clone)]
struct TexTarget {
    status: String,
    folder: Option<PathBuf>,
    search: Option<String>,
    kind: TexKind,
    key: String,
    gfx_dir: Option<PathBuf>,
    /// Dolphin only: the user root Dolphin itself reads
    /// (resolved by dolphin_user_root), when one could be
    /// confirmed. The installer switches Load Custom Textures
    /// on in that root's Config\GFX.ini after a pack install.
    /// None = no root confirmed; the install falls back to the
    /// legacy exe-folder layout and leaves the setting alone.
    dolphin_user: Option<PathBuf>,
}

fn game_id_from_rom(game: &Game) -> Option<String> {
    use std::io::Read as _;
    // A playlist entry resolves to its first disc, like a launch.
    let mut path = game.path.clone();
    if path.to_lowercase().ends_with(".m3u") {
        path = m3u_first_entry(Path::new(&path))?;
    }
    let ext = Path::new(&path)
        .extension()
        .map(|x| x.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    // Raw disc images only: the 6-byte game ID sits at offset 0.
    // (.rvz/.chd/.wbfs are containers — their IDs can't be read
    // this way, so those keep the name-folder fallback.)
    if ext != "iso" && ext != "gcm" {
        return None;
    }
    let mut f = std::fs::File::open(&path).ok()?;
    let mut data = [0u8; 6];
    f.read_exact(&mut data).ok()?;
    let id = String::from_utf8_lossy(&data).to_string();
    if id.chars().all(|c| c.is_ascii_alphanumeric()) {
        Some(id)
    } else {
        None
    }
}

fn count_files_recursive(dir: &Path) -> usize {
    let mut n = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else {
                    n += 1;
                }
            }
        }
    }
    n
}

/// "SLUS_203.12" / "SLUS20312" / "slus-20312" -> "SLUS-20312"
fn ps2_serial_normalize(raw: &str) -> Option<String> {
    let up: String = raw
        .trim()
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    if up.len() != 9 {
        return None;
    }
    let (a, b) = up.split_at(4);
    if !a.chars().all(|c| c.is_ascii_alphabetic())
        || !b.chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    Some(format!("{a}-{b}"))
}

/// Read the game serial out of a PS2 ISO's SYSTEM.CNF (ISO9660 walk).
fn ps2_serial_from_iso(path: &Path) -> Option<String> {
    use std::io::{Read as _, Seek as _, SeekFrom};
    let mut f = std::fs::File::open(path).ok()?;
    let mut pvd = [0u8; 2048];
    f.seek(SeekFrom::Start(32768)).ok()?;
    f.read_exact(&mut pvd).ok()?;
    if &pvd[1..6] != b"CD001" {
        return None;
    }
    let root = &pvd[156..190];
    let root_lba =
        u32::from_le_bytes(root[2..6].try_into().ok()?) as u64;
    let root_size =
        u32::from_le_bytes(root[10..14].try_into().ok()?) as usize;
    let mut read_extent = |lba: u64, size: usize| -> Option<Vec<u8>> {
        let mut buf = vec![0u8; size];
        f.seek(SeekFrom::Start(lba * 2048)).ok()?;
        f.read_exact(&mut buf).ok()?;
        Some(buf)
    };
    let dir = read_extent(root_lba, root_size.min(2048 * 32))?;
    let mut off = 0usize;
    while off + 34 <= dir.len() {
        let len = dir[off] as usize;
        if len < 34 {
            break;
        }
        let name_len = dir[off + 32] as usize;
        if off + 33 + name_len <= dir.len() {
            let name = String::from_utf8_lossy(
                &dir[off + 33..off + 33 + name_len],
            )
            .to_string();
            if name.to_uppercase().starts_with("SYSTEM.CNF") {
                let lba = u32::from_le_bytes(
                    dir[off + 2..off + 6].try_into().ok()?,
                ) as u64;
                let size = u32::from_le_bytes(
                    dir[off + 10..off + 14].try_into().ok()?,
                ) as usize;
                let data = read_extent(lba, size.min(4096))?;
                let text = String::from_utf8_lossy(&data);
                for part in text.split(|c: char| {
                    c == '\\' || c == '/' || c == ';' || c == '='
                        || c.is_whitespace()
                }) {
                    if let Some(s) = ps2_serial_normalize(part) {
                        return Some(s);
                    }
                }
                return None;
            }
        }
        off += len;
    }
    None
}

/// Read SYSTEM.CNF out of an ISO9660 image with an explicit
/// sector geometry (raw PS1 bins are 2352-byte sectors whose
/// 2048-byte user data starts 16 or 24 bytes in; plain ISOs are
/// flat 2048) and normalize the serial from its BOOT line
/// ("BOOT = cdrom:\SLUS_005.94;1" -> "SLUS-00594").
fn iso_system_cnf_serial(
    path: &Path,
    sector: u64,
    data_off: u64,
) -> Option<String> {
    use std::io::{Read as _, Seek as _, SeekFrom};
    let mut f = std::fs::File::open(path).ok()?;
    // read `size` user-data bytes starting at `lba`, sector by
    // sector (user data is not contiguous in raw-sector images)
    let read_user = |f: &mut std::fs::File,
                         lba: u64,
                         size: usize|
     -> Option<Vec<u8>> {
        let mut out = Vec::with_capacity(size);
        let mut cur = lba;
        while out.len() < size {
            let want = (size - out.len()).min(2048);
            let mut buf = vec![0u8; want];
            f.seek(SeekFrom::Start(cur * sector + data_off)).ok()?;
            f.read_exact(&mut buf).ok()?;
            out.extend_from_slice(&buf);
            cur += 1;
        }
        Some(out)
    };
    let pvd = read_user(&mut f, 16, 2048)?;
    if &pvd[1..6] != b"CD001" {
        return None;
    }
    let root = &pvd[156..190];
    let root_lba =
        u32::from_le_bytes(root[2..6].try_into().ok()?) as u64;
    let root_size =
        u32::from_le_bytes(root[10..14].try_into().ok()?) as usize;
    let dir = read_user(&mut f, root_lba, root_size.min(2048 * 8))?;
    let mut off = 0usize;
    while off + 34 <= dir.len() {
        let len = dir[off] as usize;
        if len < 34 {
            break;
        }
        let name_len = dir[off + 32] as usize;
        if off + 33 + name_len <= dir.len() {
            let name = String::from_utf8_lossy(
                &dir[off + 33..off + 33 + name_len],
            )
            .to_string();
            if name.to_uppercase().starts_with("SYSTEM.CNF") {
                let lba = u32::from_le_bytes(
                    dir[off + 2..off + 6].try_into().ok()?,
                ) as u64;
                let size = u32::from_le_bytes(
                    dir[off + 10..off + 14].try_into().ok()?,
                ) as usize;
                let data = read_user(&mut f, lba, size.min(4096))?;
                let text = String::from_utf8_lossy(&data);
                // BOOT lines vary: "cdrom:\SLUS_004.22" and
                // "cdrom:SLUS_000.67" both occur on real discs —
                // scan the text for a serial-shaped token rather
                // than splitting on one fixed separator set.
                return serial_token_in(&text);
            }
        }
        off += len;
    }
    None
}

/// A serial printed in a game or file name ("[SLPS-01868]",
/// "[SCUS-94221]") — the fallback for formats whose disc image
/// can't be read directly (this library's .chd files often carry
/// the serial in the folder name).
fn serial_token_in(text: &str) -> Option<String> {
    let mut run = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
            run.push(c);
        } else {
            if run.len() >= 8 {
                if let Some(s) = ps2_serial_normalize(&run) {
                    return Some(s);
                }
            }
            run.clear();
        }
    }
    if run.len() >= 8 {
        return ps2_serial_normalize(&run);
    }
    None
}

/// The first FILE entry of a .cue sheet (the data track's image).
fn cue_first_file(cue: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(cue).ok()?;
    let dir = cue.parent().unwrap_or(Path::new(""));
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("FILE") {
            let rest = rest.trim();
            let name = if let Some(q) = rest.strip_prefix('"') {
                q.split('"').next().unwrap_or("").to_string()
            } else {
                rest.split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string()
            };
            if !name.is_empty() {
                return Some(dir.join(name));
            }
        }
    }
    None
}

fn ps1_serial_from_image(path: &Path) -> Option<String> {
    for (sector, off) in
        [(2048u64, 0u64), (2352, 24), (2352, 16), (2336, 8)]
    {
        if let Some(s) = iso_system_cnf_serial(path, sector, off) {
            return Some(s);
        }
    }
    None
}

fn ps1_serial_from_path(path: &Path, depth: u32) -> Option<String> {
    if depth > 3 {
        return None;
    }
    let ext = path
        .extension()
        .map(|x| x.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "m3u" => {
            let text = std::fs::read_to_string(path).ok()?;
            let dir = path.parent().unwrap_or(Path::new(""));
            let first = text
                .lines()
                .map(|l| l.trim())
                .find(|l| !l.is_empty() && !l.starts_with('#'))?;
            ps1_serial_from_path(&dir.join(first), depth + 1)
        }
        "cue" => {
            let img = cue_first_file(path)?;
            ps1_serial_from_image(&img)
        }
        "iso" | "bin" | "img" => ps1_serial_from_image(path),
        _ => None,
    }
}

/// PS1 serial for a library game: SYSTEM.CNF inside the disc
/// image when it can be read (.m3u/.cue/.bin/.iso), else a
/// serial printed in the game name, file name, or folder name
/// (the .chd fallback).
fn ps1_serial_for_game(game: &Game) -> Option<String> {
    let p = Path::new(&game.path);
    ps1_serial_from_path(p, 0).or_else(|| {
        serial_token_in(&game.name)
            .or_else(|| {
                p.file_name()
                    .and_then(|n| serial_token_in(&n.to_string_lossy()))
            })
            .or_else(|| {
                p.parent()
                    .and_then(|d| d.file_name())
                    .and_then(|n| serial_token_in(&n.to_string_lossy()))
            })
    })
}

/// Where DuckStation keeps its user data for this install.
/// Current builds use the Documents folder — which on many
/// PCs is redirected into OneDrive, so USERPROFILE\Documents
/// does NOT resolve there; check the OneDrive location first. Older
/// portable installs keep settings beside the exe instead.
fn duckstation_user_dir(exe_dir: &Path) -> PathBuf {
    if let Ok(home) = std::env::var("USERPROFILE") {
        let one = Path::new(&home)
            .join("OneDrive")
            .join("Documents")
            .join("DuckStation");
        if one.is_dir() {
            return one;
        }
        let docs = Path::new(&home).join("Documents").join("DuckStation");
        if docs.is_dir() {
            return docs;
        }
    }
    if exe_dir.join("settings.ini").is_file()
        || exe_dir.join("database").is_dir()
    {
        return exe_dir.to_path_buf();
    }
    exe_dir.to_path_buf()
}

/// GameIndex region check. Beware: "NTSC-Unk" (unknown) CONTAINS
/// the substring "NTSC-U" — a plain contains() match would crown
/// an unknown-region serial as the US one. Compare whole tokens.
fn is_ntsc_u(region: &str) -> bool {
    let parts: Vec<&str> = region.trim().split('-').collect();
    parts.len() == 2 && parts[0] == "NTSC" && parts[1] == "U"
}

fn normalize_title(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

/// Serial lookup in PCSX2's own GameIndex.yaml (name -> serial).
/// Prefers an NTSC-U entry when several regions share a title.
fn ps2_serial_from_gameindex(pcsx2_dir: &Path, game: &Game) -> Option<String> {
    let text =
        std::fs::read_to_string(pcsx2_dir.join("GameIndex.yaml")).ok()?;
    let want = normalize_title(&clean_game_name(&game.name));
    if want.is_empty() {
        return None;
    }
    // parse entries: "SLUS-20312:" then indented name:/region: lines
    let mut entries: Vec<(String, String, String)> = Vec::new();
    let mut cur: Option<(String, String, String)> = None;
    for line in text.lines() {
        if !line.starts_with(|c: char| c.is_whitespace())
            && line.trim_end().ends_with(':')
        {
            if let Some(e) = cur.take() {
                entries.push(e);
            }
            cur = ps2_serial_normalize(line.trim_end_matches(':'))
                .map(|s| (s, String::new(), String::new()));
            continue;
        }
        if let Some(e) = &mut cur {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("name:") {
                e.1 = rest.trim().trim_matches('"').to_string();
            } else if let Some(rest) = t.strip_prefix("region:") {
                e.2 = rest.trim().trim_matches('"').to_string();
            }
        }
    }
    if let Some(e) = cur.take() {
        entries.push(e);
    }
    let mut prefix_hit: Option<&(String, String, String)> = None;
    for e in &entries {
        let n = normalize_title(&e.1);
        if n == want {
            if is_ntsc_u(&e.2) {
                return Some(e.0.clone());
            }
            if prefix_hit.is_none() {
                prefix_hit = Some(e);
            }
        }
    }
    if let Some(e) = prefix_hit {
        return Some(e.0.clone());
    }
    // fallback: one title is a prefix of the other (editions,
    // subtitles) — prefer the LONGEST matching title (most
    // specific: "Final Fantasy X-2 International" must beat plain
    // "Final Fantasy X"), then NTSC-U, then file order.
    let mut best: Option<&(String, String, String)> = None;
    for e in &entries {
        let n = normalize_title(&e.1);
        if n.len() >= 6 && (n.starts_with(&want) || want.starts_with(&n)) {
            let better = match best {
                None => true,
                Some(b) => {
                    let bn = normalize_title(&b.1);
                    n.len() > bn.len()
                        || (n.len() == bn.len()
                            && is_ntsc_u(&e.2)
                            && !is_ntsc_u(&b.2))
                }
            };
            if better {
                best = Some(e);
            }
        }
    }
    best.map(|e| e.0.clone())
}

/// N64 header name (bytes 0x20..0x34). Byte order depends on the
/// dump format: .z64 is big-endian (plain), .v64 swaps byte pairs,
/// and .n64 files in this library are pair-swapped too (verified
/// against the real Paper Mario dump). Try the extension's own
/// layout first, then the others; the first fully printable
/// result wins.
fn n64_name_from_bytes(raw: &[u8], ext: &str) -> String {
    let clean = |b: &[u8]| {
        String::from_utf8_lossy(b)
            .trim_matches(|c: char| c == '\0' || c.is_whitespace())
            .to_string()
    };
    let printable = |s: &str| {
        !s.is_empty()
            && s.chars().all(|c| (' '..='~').contains(&c))
    };
    let plain = clean(raw);
    let mut pair_b = raw.to_vec();
    for c in pair_b.chunks_exact_mut(2) {
        c.swap(0, 1);
    }
    let pair = clean(&pair_b);
    let mut word_b = raw.to_vec();
    for c in word_b.chunks_exact_mut(4) {
        c.reverse();
    }
    let word = clean(&word_b);
    let order: Vec<&String> = match ext {
        "v64" => vec![&pair, &plain, &word],
        "n64" => vec![&pair, &word, &plain],
        _ => vec![&plain, &pair, &word],
    };
    for cand in order {
        if printable(cand) {
            return cand.clone();
        }
    }
    plain
}

fn n64_internal_name(game: &Game) -> Option<String> {
    use std::io::Read as _;
    let ext = Path::new(&game.path)
        .extension()
        .map(|x| x.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if !matches!(ext.as_str(), "z64" | "v64" | "n64") {
        return None;
    }
    let mut f = std::fs::File::open(&game.path).ok()?;
    let mut buf = [0u8; 0x34];
    f.read_exact(&mut buf).ok()?;
    let name = n64_name_from_bytes(&buf[0x20..0x34], &ext);
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// The folder Dolphin itself uses for user data on this
/// install, resolved the way Dolphin resolves it:
/// portable.txt next to the exe forces the exe-adjacent User
/// folder; otherwise Dolphin uses the Windows Documents
/// folder — which on some machines is redirected into
/// OneDrive (...\OneDrive\Documents\...), the same trap that
/// hid DuckStation's real data root in Oct 2026. A candidate
/// wins outright when Dolphin's own config files are in it;
/// otherwise the first existing candidate wins (the pre-fix
/// order), and if none exist the caller falls back to the
/// legacy exe-folder layout with no confirmed root.
fn dolphin_user_root(exe_dir: &Path) -> Option<PathBuf> {
    if exe_dir.join("portable.txt").exists() {
        return Some(exe_dir.join("User"));
    }
    let mut cands: Vec<PathBuf> = vec![exe_dir.join("User")];
    if let Ok(od) = std::env::var("OneDrive") {
        cands.push(
            Path::new(&od)
                .join("Documents")
                .join("Dolphin Emulator"),
        );
    }
    if let Ok(up) = std::env::var("USERPROFILE") {
        cands.push(
            Path::new(&up)
                .join("Documents")
                .join("Dolphin Emulator"),
        );
    }
    for c in &cands {
        if c.join("Config").join("Dolphin.ini").exists()
            || c.join("Config").join("GFX.ini").exists()
        {
            return Some(c.clone());
        }
    }
    cands.into_iter().find(|c| c.is_dir())
}

/// Where Dolphin keeps Load\Textures for this install, plus
/// the confirmed user root when there is one.
fn dolphin_textures_root(exe_dir: &Path) -> (PathBuf, Option<PathBuf>) {
    match dolphin_user_root(exe_dir) {
        Some(user) => (
            user.join("Load").join("Textures"),
            Some(user),
        ),
        None => (exe_dir.join("Load").join("Textures"), None),
    }
}

fn texture_target(game: &Game, profiles: &[Profile]) -> TexTarget {
    let search = Some(format!(
        "https://www.google.com/search?q={}",
        url_encode(&format!(
            "\"{}\" HD texture pack {}",
            clean_game_name(&game.name),
            game.system
        ))
    ));
    let find_prof = |frag: &str| {
        profiles.iter().find(|p| {
            p.name.to_lowercase().contains(frag)
                || p.path.to_lowercase().contains(frag)
        })
    };
    let mk = |status: String,
              folder: Option<PathBuf>,
              kind: TexKind,
              key: String,
              gfx_dir: Option<PathBuf>| TexTarget {
        status,
        folder,
        search: search.clone(),
        kind,
        key,
        gfx_dir,
        dolphin_user: None,
    };
    if game.system.contains("GameCube") || game.system.contains("Wii") {
        let Some(d) = find_prof("dolphin") else {
            return mk(
                "No Dolphin profile found.".into(),
                None,
                TexKind::Unsupported,
                String::new(),
                None,
            );
        };
        let exe_dir = Path::new(&d.path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        let id = game_id_from_rom(game)
            .unwrap_or_else(|| safe_folder(&game.name));
        let (tex_root, user_root) = dolphin_textures_root(&exe_dir);
        let folder = tex_root.join(&id);
        let count = count_files_recursive(&folder);
        let status = if count > 0 {
            format!(
                "Texture pack installed for {id} ({count} files in the pack folder)."
            )
        } else {
            format!(
                "No pack installed yet. Dolphin reads packs from this folder (game ID {id}):"
            )
        };
        let mut t = mk(status, Some(folder), TexKind::Dolphin, id, None);
        t.dolphin_user = user_root;
        return t;
    }
    if game.system.contains("PlayStation 2") {
        let Some(p) = find_prof("pcsx2") else {
            return mk(
                "No PCSX2 profile found.".into(),
                None,
                TexKind::Unsupported,
                String::new(),
                None,
            );
        };
        let exe_dir = Path::new(&p.path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        let ext = Path::new(&game.path)
            .extension()
            .map(|x| x.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let serial = if ext == "iso" {
            ps2_serial_from_iso(Path::new(&game.path))
        } else {
            None
        }
        .or_else(|| ps2_serial_from_gameindex(&exe_dir, game))
        .or_else(|| {
            // last resort: a serial printed in the file name itself
            let stem = Path::new(&game.path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            stem.split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
                .find_map(ps2_serial_normalize)
        });
        let Some(serial) = serial else {
            return mk(
                "Couldn't identify this disc's serial number, so the PCSX2 pack folder can't be named. PCSX2 names pack folders by serial (e.g. SLUS-20312).".into(),
                None,
                TexKind::Unsupported,
                String::new(),
                None,
            );
        };
        let folder = exe_dir.join("textures").join(&serial);
        let count = count_files_recursive(&folder.join("replacements"));
        let status = if count > 0 {
            format!(
                "Texture pack installed for {serial} ({count} replacement files)."
            )
        } else {
            format!(
                "No pack installed yet. PCSX2 reads replacement textures from this folder (serial {serial}):"
            )
        };
        return mk(status, Some(folder), TexKind::Pcsx2, serial, None);
    }
    if game.system.contains("Nintendo 64") {
        let Some(p) = find_prof("project64") else {
            return mk(
                "No Project64 profile found.".into(),
                None,
                TexKind::Unsupported,
                String::new(),
                None,
            );
        };
        let exe_dir = Path::new(&p.path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        let gfx_dir = exe_dir.join("Plugin").join("GFX");
        let Some(iname) = n64_internal_name(game) else {
            return mk(
                "Couldn't read this ROM's internal name, so the GLideN64 pack folder can't be named.".into(),
                None,
                TexKind::Unsupported,
                String::new(),
                Some(gfx_dir),
            );
        };
        let folder = gfx_dir.join("hires_texture").join(&iname);
        let count = count_files_recursive(&folder);
        let cache_hit = std::fs::read_dir(gfx_dir.join("cache"))
            .map(|rd| {
                rd.flatten().any(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .to_uppercase()
                        .starts_with(&iname.to_uppercase())
                })
            })
            .unwrap_or(false);
        let status = if count > 0 {
            format!(
                "Texture pack installed for {iname} ({count} files in the pack folder)."
            )
        } else if cache_hit {
            format!(
                "A GLideN64 cache pack for {iname} is installed (Plugin\\GFX\\cache)."
            )
        } else {
            format!(
                "No pack installed yet. GLideN64 reads loose packs from this folder (ROM name {iname}):"
            )
        };
        return mk(status, Some(folder), TexKind::GlideN64, iname, Some(gfx_dir));
    }
    if game.system.contains("Nintendo Entertainment System") {
        let Some(p) = find_prof("mesen") else {
            return mk(
                "No Mesen profile found.".into(),
                None,
                TexKind::Unsupported,
                String::new(),
                None,
            );
        };
        let exe_dir = Path::new(&p.path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        let stem = Path::new(&game.path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let folder = exe_dir.join("HdPacks").join(&stem);
        let count = count_files_recursive(&folder);
        let has_core = folder.join("hires.txt").is_file();
        let status = if count > 0 && has_core {
            format!(
                "HD pack installed for {stem} ({count} files)."
            )
        } else if count > 0 {
            format!(
                "Files are present for {stem} ({count}) but no hires.txt — this may not be a complete Mesen HD pack."
            )
        } else {
            format!(
                "No HD pack installed yet. Mesen reads packs from this folder (named after the ROM file):"
            )
        };
        return mk(status, Some(folder), TexKind::Mesen, stem, None);
    }
    if game.system.contains("PlayStation")
        && !game.system.contains("PlayStation 2")
        && !game.system.contains("PlayStation 3")
        && !game.system.contains("Portable")
    {
        let Some(d) = find_prof("duckstation") else {
            return mk(
                "No DuckStation profile found. DuckStation is the PS1 emulator with texture-replacement support — the other PS1 emulators in your profiles don't load texture packs. Add DuckStation in Manage Emulators and this window can target it.".into(),
                None,
                TexKind::Unsupported,
                String::new(),
                None,
            );
        };
        let exe_dir = Path::new(&d.path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        let Some(serial) = ps1_serial_for_game(game) else {
            return mk(
                "Couldn't identify this disc's serial number, so the DuckStation pack folder can't be named. DuckStation names pack folders by serial (e.g. SLUS-00594).".into(),
                None,
                TexKind::Unsupported,
                String::new(),
                None,
            );
        };
        let folder = duckstation_user_dir(&exe_dir)
            .join("textures")
            .join(&serial);
        let count = count_files_recursive(&folder.join("replacements"));
        let status = if count > 0 {
            format!(
                "Texture pack installed for {serial} ({count} replacement files)."
            )
        } else {
            format!(
                "No pack installed yet. DuckStation reads replacement textures from this folder (serial {serial}):"
            )
        };
        return mk(status, Some(folder), TexKind::DuckStation, serial, None);
    }
    mk(
        "Texture packs aren't supported by this system's emulator. They are available for GameCube/Wii (Dolphin), PlayStation 2 (PCSX2), PlayStation (DuckStation), Nintendo 64 (Project64/GLideN64) and NES (Mesen HD packs)."
            .into(),
        None,
        TexKind::Unsupported,
        String::new(),
        None,
    )
}

// ---------------- curated pack catalog -------------------------------
// Every entry's download URL was verified live (HTTP 200, archive
// content type) on 2026-10-18. Keys: Dolphin = game ID, PCSX2 =
// serial, DuckStation = serial, GLideN64 = ROM internal name
// (uppercased), Mesen = ROM file stem (lowercased).
// PS1/DuckStation: no entries yet — as of Oct 2026 no
// DuckStation-format pack with a directly downloadable archive
// could be verified (the format is new; packs circulate via
// Discord/forums). PS1 games use Install-from-URL meanwhile.
#[derive(Clone)]
struct TexPack {
    title: &'static str,
    kind: TexKind,
    key: &'static str,
    url: &'static str,
    page: &'static str,
    size: &'static str,
}

const TEX_PACKS: &[TexPack] = &[
    // ---- Dolphin (GameCube / Wii) ----
    TexPack {
        title: "Super Mario Sunshine UHD (1080p variant) — qashto",
        kind: TexKind::Dolphin,
        key: "GMSE01",
        url: "https://github.com/qashto/Super_Mario_Sunshine_UHD_Texture_Pack/releases/download/2.1.1/GMS_1080.zip",
        page: "https://github.com/qashto/Super_Mario_Sunshine_UHD_Texture_Pack",
        size: "149 MB",
    },
    TexPack {
        title: "Super Mario Sunshine UHD (full 4K, DDS) — qashto",
        kind: TexKind::Dolphin,
        key: "GMSE01",
        url: "https://github.com/qashto/Super_Mario_Sunshine_UHD_Texture_Pack/releases/download/2.1.1/GMS.7z",
        page: "https://github.com/qashto/Super_Mario_Sunshine_UHD_Texture_Pack",
        size: "940 MB",
    },
    // ---- PCSX2 (PlayStation 2) ----
    TexPack {
        title: "Mortal Kombat: Deception Remastered 1.13 — vStar925",
        kind: TexKind::Pcsx2,
        key: "SLUS-20881",
        url: "https://github.com/vStar925/Mortal-Kombat-Deception-Remastered/releases/download/PCSX2-1.13/MKDeception.Remastered.1.13.7z",
        page: "https://github.com/vStar925/Mortal-Kombat-Deception-Remastered",
        size: "543 MB",
    },
    // ---- Project64 / GLideN64 (Nintendo 64) — evilgames.eu ----
    TexPack {
        title: "Banjo-Tooie Reloaded (HTS cache pack) — GhostlyDark",
        kind: TexKind::GlideN64,
        key: "BANJO TOOIE",
        url: "https://evilgames.eu/files/texture-packs/bt-reloaded-v0.1.1-gliden64-hts-hd.7z",
        page: "https://evilgames.eu/texture-packs/bt-reloaded.htm",
        size: "18 MB",
    },
    TexPack {
        title: "Banjo-Kazooie Reloaded (HTS cache pack) — GhostlyDark",
        kind: TexKind::GlideN64,
        key: "BANJO-KAZOOIE",
        url: "https://evilgames.eu/files/texture-packs/bk-reloaded-v0.2.1-gliden64-hts-hd.7z",
        page: "https://evilgames.eu/texture-packs/bk-reloaded.htm",
        size: "34 MB",
    },
    TexPack {
        title: "GoldenEye 007 HD (HTS cache pack) — intermissionfb",
        kind: TexKind::GlideN64,
        key: "GOLDENEYE",
        url: "https://evilgames.eu/files/texture-packs/ge007-hd-v2025.12.30-gliden64-hts-hd.7z",
        page: "https://evilgames.eu/texture-packs/ge007-hd.htm",
        size: "77 MB",
    },
    TexPack {
        title: "Super Mario 64 Reloaded (HTS cache pack) — GhostlyDark",
        kind: TexKind::GlideN64,
        key: "SUPER MARIO 64",
        url: "https://evilgames.eu/files/texture-packs/sm64-reloaded-v2.6.0-gliden64-hts-hd.7z",
        page: "https://evilgames.eu/texture-packs/sm64-reloaded.htm",
        size: "147 MB",
    },
    TexPack {
        title: "Mario Kart 64 Reloaded (HTS cache pack) — GhostlyDark",
        kind: TexKind::GlideN64,
        key: "MARIOKART64",
        url: "https://evilgames.eu/files/texture-packs/mk64-reloaded-v2026.04.03-gliden64-hts-hd.7z",
        page: "https://evilgames.eu/texture-packs/mk64-reloaded.htm",
        size: "379 MB",
    },
    TexPack {
        title: "Majora's Mask Reloaded (HTS cache pack) — GhostlyDark",
        kind: TexKind::GlideN64,
        key: "ZELDA MAJORA'S MASK",
        url: "https://evilgames.eu/files/texture-packs/mm-reloaded-v11.0.2-gliden64-hts-hd.7z",
        page: "https://evilgames.eu/texture-packs/mm-reloaded.htm",
        size: "658 MB",
    },
    TexPack {
        title: "Ocarina of Time Reloaded (HTS cache pack) — GhostlyDark",
        kind: TexKind::GlideN64,
        key: "THE LEGEND OF ZELDA",
        url: "https://evilgames.eu/files/texture-packs/oot-reloaded-v11.0.0-gliden64-hts-hd.7z",
        page: "https://evilgames.eu/texture-packs/oot-reloaded.htm",
        size: "1.25 GB",
    },
    // ---- Mesen HD packs (NES) — github.com/TasticHacks ----
    TexPack {
        title: "Contra 80s HD Pack 1.4.2 — Tastic",
        kind: TexKind::Mesen,
        key: "contra (usa)",
        url: "https://github.com/TasticHacks/Contra80s/releases/download/1.4.2/Contra80s.zip",
        page: "https://github.com/TasticHacks/Contra80s",
        size: "99 MB",
    },
];

fn packs_for(target: &TexTarget) -> Vec<TexPack> {
    TEX_PACKS
        .iter()
        .filter(|p| {
            p.kind == target.kind
                && match target.kind {
                    TexKind::Dolphin | TexKind::Pcsx2
                    | TexKind::DuckStation => p.key == target.key,
                    TexKind::GlideN64 => {
                        p.key.eq_ignore_ascii_case(&target.key)
                    }
                    TexKind::Mesen => {
                        p.key.eq_ignore_ascii_case(&target.key)
                    }
                    TexKind::Unsupported => false,
                }
        })
        .cloned()
        .collect()
}

// ---------------- pack install (worker thread) -----------------------
fn download_to_file(url: &str, dest: &Path) -> Result<u64, String> {
    let agent = http_agent(1200);
    let resp = agent
        .get(url)
        .set(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36",
        )
        .call()
        .map_err(|e| format!("Download failed: {e}"))?;
    let mut file =
        std::fs::File::create(dest).map_err(|e| e.to_string())?;
    let mut reader = resp.into_reader();
    std::io::copy(&mut reader, &mut file)
        .map_err(|e| format!("Download failed: {e}"))
}

/// Point GLideN64 at this PC's real texture folders and switch
/// hi-res textures on. Its ini still carries the old P: drive paths,
/// so packs would never be found without this repair.
fn fix_gliden64_ini(gfx_dir: &Path, file_storage: bool) -> bool {
    let ini = gfx_dir.join("GLideN64.ini");
    let Ok(text) = std::fs::read_to_string(&ini) else {
        return false;
    };
    let hires = gfx_dir
        .join("hires_texture")
        .to_string_lossy()
        .replace('\\', "/");
    let cache = gfx_dir
        .join("cache")
        .to_string_lossy()
        .replace('\\', "/");
    let mut changed = false;
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let new = if line.starts_with("textureFilter\\txPath=") {
            format!("textureFilter\\txPath={hires}")
        } else if line.starts_with("textureFilter\\txCachePath=") {
            format!("textureFilter\\txCachePath={cache}")
        } else if line.starts_with("textureFilter\\txHiresEnable=") {
            "textureFilter\\txHiresEnable=1".to_string()
        } else if file_storage
            && line.starts_with("textureFilter\\txHiresTextureFileStorage=")
        {
            "textureFilter\\txHiresTextureFileStorage=1".to_string()
        } else {
            line.to_string()
        };
        if new != line {
            changed = true;
        }
        out.push(new);
    }
    if changed {
        let _ = std::fs::write(&ini, out.join("\r\n") + "\r\n");
    }
    changed
}

fn dolphin_running() -> bool {
    Command::new("tasklist")
        .creation_flags(0x0800_0000)
        .args(["/FI", "IMAGENAME eq Dolphin.exe"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("Dolphin.exe"))
        .unwrap_or(false)
}

/// Switch on Dolphin's "Load Custom Textures" — GFX.ini,
/// [Settings] HiresTextures = True — in the user root Dolphin
/// actually reads. The app previously installed packs and left
/// this step to the user ("Enable Graphics > Advanced > Load
/// Custom Textures"), which is the main reason installed
/// GameCube packs never appeared in-game. Backup-first (one
/// GFX.ini.emufront-bak), idempotent, and skipped while
/// Dolphin is running because Dolphin rewrites its config on
/// exit. Returns a sentence for the install result message.
fn dolphin_enable_hires_textures(user_root: &Path) -> String {
    if dolphin_running() {
        return " Note: Dolphin is running, so its Load Custom Textures setting was left alone (Dolphin rewrites its config when it closes) — close Dolphin and install the pack again to switch it on automatically, or enable Graphics > Advanced > Load Custom Textures by hand.".to_string();
    }
    let cfg_dir = user_root.join("Config");
    let ini = cfg_dir.join("GFX.ini");
    let existed = ini.exists();
    let text = std::fs::read_to_string(&ini).unwrap_or_default();
    let mut section = String::new();
    let mut found = false;
    let mut changed = false;
    let mut out: Vec<String> = Vec::new();
    let mut insert_at: Option<usize> = None;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            section = t[1..t.len() - 1].to_lowercase();
            out.push(line.to_string());
            if section == "settings" {
                insert_at = Some(out.len());
            }
            continue;
        }
        if section == "settings"
            && t.to_lowercase().starts_with("hirestextures")
        {
            found = true;
            if t != "HiresTextures = True" {
                changed = true;
            }
            out.push("HiresTextures = True".to_string());
            continue;
        }
        out.push(line.to_string());
    }
    if !found {
        changed = true;
        match insert_at {
            Some(i) => out.insert(i, "HiresTextures = True".to_string()),
            None => {
                out.push("[Settings]".to_string());
                out.push("HiresTextures = True".to_string());
            }
        }
    }
    if !changed {
        return " Load Custom Textures is already switched on in Dolphin."
            .to_string();
    }
    if existed {
        let bak = ini.with_extension("ini.emufront-bak");
        if !bak.exists() {
            let _ = std::fs::copy(&ini, &bak);
        }
    }
    if std::fs::create_dir_all(&cfg_dir).is_err()
        || std::fs::write(&ini, out.join("\r\n") + "\r\n").is_err()
    {
        return " Note: the pack installed, but EmuFront could not update Dolphin's GFX.ini — enable Graphics > Advanced > Load Custom Textures in Dolphin by hand.".to_string();
    }
    " Load Custom Textures was switched ON in Dolphin (GFX.ini updated; backup kept beside it).".to_string()
}

/// Download a pack (archive or single file) and install it into the
/// folder layout the target emulator actually reads. Blocking.
fn install_texture_pack(
    url: &str,
    target: &TexTarget,
) -> Result<String, String> {
    let Some(dest_root) = &target.folder else {
        return Err("No texture folder is known for this game.".into());
    };
    let tmp = std::env::temp_dir()
        .join(format!("emufront-tex-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let result = install_texture_pack_inner(url, target, dest_root, &tmp);
    let _ = std::fs::remove_dir_all(&tmp);
    result
}

fn install_texture_pack_inner(
    url: &str,
    target: &TexTarget,
    dest_root: &Path,
    tmp: &Path,
) -> Result<String, String> {
    let dl = tmp.join("pack-download");
    let bytes = download_to_file(url, &dl)?;
    if bytes < 64 {
        return Err(
            "That download was nearly empty — not a texture pack.".into(),
        );
    }
    // sniff what actually arrived
    let mut head = [0u8; 16];
    {
        use std::io::Read as _;
        if let Ok(mut f) = std::fs::File::open(&dl) {
            let _ = f.read(&mut head);
        }
    }
    let first = head
        .iter()
        .copied()
        .find(|b| !b.is_ascii_whitespace())
        .unwrap_or(0);
    if first == b'<' {
        return Err(
            "That link returned a web page, not a pack file. Use a direct download link (a .zip/.7z/.rar file URL)."
                .into(),
        );
    }
    let url_name = url
        .split('/')
        .next_back()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("")
        .to_string();
    let url_ext = Path::new(&url_name)
        .extension()
        .map(|x| x.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let is_archive = head[..4] == [0x50, 0x4B, 0x03, 0x04]
        || head[..6] == [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]
        || head[..4] == [0x52, 0x61, 0x72, 0x21]
        || matches!(url_ext.as_str(), "zip" | "7z" | "rar");
    // collect (source file, relative components) pairs
    let mut files: Vec<(PathBuf, Vec<String>)> = Vec::new();
    if is_archive {
        let ext_dir = tmp.join("extract");
        let found = extract_archive(&dl, &ext_dir)?;
        for p in found {
            if let Ok(rel) = p.strip_prefix(&ext_dir) {
                let comps: Vec<String> = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect();
                if !comps.is_empty() {
                    files.push((p, comps));
                }
            }
        }
    } else {
        let ext = if head[..4] == [0x89, 0x50, 0x4E, 0x47] {
            "png"
        } else if &head[..4] == b"DDS " {
            "dds"
        } else {
            url_ext.as_str()
        };
        if !matches!(
            ext,
            "png" | "dds" | "htc" | "hts" | "zip" | "7z" | "rar"
        ) {
            return Err(format!(
                "That download isn't a texture pack archive or texture file (type: {}).",
                if ext.is_empty() { "unknown" } else { ext }
            ));
        }
        let name = if url_name.is_empty() {
            format!("pack.{ext}")
        } else {
            url_name.clone()
        };
        files.push((dl.clone(), vec![name]));
    }
    if files.is_empty() {
        return Err("The pack archive was empty.".into());
    }
    // Mesen: root the copy at the folder that contains hires.txt
    // (packs often ship beside repo extras — READMEs, art folders —
    // which must NOT land in the pack folder).
    if target.kind == TexKind::Mesen {
        let mut prefix: Option<Vec<String>> = None;
        for (_, comps) in &files {
            let is_hires = comps
                .last()
                .map(|c| c.eq_ignore_ascii_case("hires.txt"))
                .unwrap_or(false);
            if is_hires {
                let p: Vec<String> = comps[..comps.len() - 1].to_vec();
                let better = match &prefix {
                    None => true,
                    Some(cur) => p.len() < cur.len(),
                };
                if better {
                    prefix = Some(p);
                }
            }
        }
        if let Some(p) = prefix {
            if !p.is_empty() {
                files.retain(|(_, comps)| {
                    comps.len() > p.len()
                        && comps[..p.len()] == p[..]
                });
                for (_, comps) in files.iter_mut() {
                    comps.drain(0..p.len());
                }
            }
        }
    }
    // PCSX2 / DuckStation: if the pack carries a "replacements"
    // folder, root there.
    if matches!(target.kind, TexKind::Pcsx2 | TexKind::DuckStation) {
        for (_, comps) in files.iter_mut() {
            if let Some(pos) = comps
                .iter()
                .position(|c| c.eq_ignore_ascii_case("replacements"))
            {
                *comps = comps.split_off(pos + 1);
            }
        }
        files.retain(|(_, c)| !c.is_empty());
    }
    // Dolphin: if files sit under a folder named by the game ID
    // (possibly nested, e.g. Load/Textures/<ID>), root below it.
    if target.kind == TexKind::Dolphin {
        for (_, comps) in files.iter_mut() {
            if let Some(pos) =
                comps.iter().rposition(|c| c == &target.key)
            {
                if pos + 1 < comps.len() {
                    *comps = comps.split_off(pos + 1);
                }
            }
        }
        files.retain(|(_, c)| !c.is_empty());
    }
    // Any kind: strip wrapper folders shared by every file
    // (a pack-name folder, a GitHub "<repo>-<branch>" root, ...),
    // but never strip a Mesen pack's own content folders.
    loop {
        if files.iter().any(|(_, c)| c.len() < 2) {
            break;
        }
        let first = files[0].1[0].clone();
        if files.iter().all(|(_, c)| c[0] == first) {
            // for PCSX2/Dolphin a lone wrapper is always safe to drop;
            // elsewhere drop it too — the destination folder itself
            // carries the game name.
            for (_, comps) in files.iter_mut() {
                comps.remove(0);
            }
        } else {
            break;
        }
    }
    let want_ext = |e: &str| -> bool {
        match target.kind {
            TexKind::Dolphin | TexKind::Pcsx2 => {
                matches!(e, "png" | "dds")
            }
            TexKind::DuckStation => e == "png",
            TexKind::GlideN64 => matches!(e, "png" | "dds" | "htc" | "hts"),
            TexKind::Mesen => true,
            TexKind::Unsupported => false,
        }
    };
    let install_dest =
        if matches!(target.kind, TexKind::Pcsx2 | TexKind::DuckStation) {
            dest_root.join("replacements")
        } else {
            dest_root.to_path_buf()
        };
    let mut copied = 0usize;
    let mut cache_copied = 0usize;
    let mut saw_hires_txt = false;
    for (src, comps) in &files {
        let fname = comps.last().cloned().unwrap_or_default();
        let ext = Path::new(&fname)
            .extension()
            .map(|x| x.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if !want_ext(&ext) {
            continue;
        }
        if comps.len() == 1 && fname.eq_ignore_ascii_case("hires.txt") {
            saw_hires_txt = true;
        }
        // GLideN64 cache packs (.hts/.htc) belong in the cache folder
        if target.kind == TexKind::GlideN64
            && matches!(ext.as_str(), "htc" | "hts")
        {
            let Some(gfx) = &target.gfx_dir else {
                continue;
            };
            let cache = gfx.join("cache");
            let _ = std::fs::create_dir_all(&cache);
            if std::fs::copy(src, cache.join(&fname)).is_ok() {
                cache_copied += 1;
            }
            continue;
        }
        let mut dst = install_dest.clone();
        for c in comps {
            dst = dst.join(c);
        }
        if let Some(parent) = dst.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::copy(src, &dst).is_ok() {
            copied += 1;
        }
    }
    let total = copied + cache_copied;
    if total == 0 {
        return Err(
            "No usable texture files were found in that download.".into(),
        );
    }
    let mb = bytes as f64 / 1_048_576.0;
    let mut msg = match target.kind {
        TexKind::Dolphin => format!(
            "Installed {copied} texture files for {} ({mb:.0} MB download).",
            target.key
        ),
        TexKind::Pcsx2 => format!(
            "Installed {copied} replacement textures for {} ({mb:.0} MB download). PCSX2 already has texture replacements enabled.",
            target.key
        ),
        TexKind::DuckStation => format!(
            "Installed {copied} replacement textures for {} ({mb:.0} MB download). DuckStation loads them automatically — texture replacements are enabled in its settings.",
            target.key
        ),
        TexKind::GlideN64 => {
            let mut m = String::new();
            if cache_copied > 0 {
                m.push_str(&format!(
                    "Installed {cache_copied} GLideN64 cache pack file(s) for {}. ",
                    target.key
                ));
            }
            if copied > 0 {
                m.push_str(&format!(
                    "Installed {copied} loose texture files for {}. ",
                    target.key
                ));
            }
            m.push_str(&format!("({mb:.0} MB download)"));
            m
        }
        TexKind::Mesen => format!(
            "Installed {copied} HD pack files for {} ({mb:.0} MB download).",
            target.key
        ),
        TexKind::Unsupported => format!("Installed {total} files."),
    };
    if target.kind == TexKind::GlideN64 {
        if let Some(gfx) = &target.gfx_dir {
            if fix_gliden64_ini(gfx, cache_copied > 0) {
                msg.push_str(
                    " GLideN64 was pointed at this PC's texture folders and hi-res textures were switched on.",
                );
            }
        }
    }
    if target.kind == TexKind::Dolphin {
        match &target.dolphin_user {
            Some(user) => {
                msg.push_str(&dolphin_enable_hires_textures(user))
            }
            None => msg.push_str(
                " Note: EmuFront could not confirm which settings folder your Dolphin uses, so its Load Custom Textures setting was left alone — enable Graphics > Advanced > Load Custom Textures in Dolphin by hand.",
            ),
        }
    }
    if target.kind == TexKind::Mesen && !saw_hires_txt {
        msg.push_str(
            " Note: no hires.txt in that pack — it may not be a complete Mesen HD pack.",
        );
    }
    Ok(msg)
}
// ---------------- file dialogs (via PowerShell) ---------------------
fn ps_dialog(script: &str) -> Option<String> {
    let out = Command::new("powershell")
        .creation_flags(0x0800_0000)
        .args(["-STA", "-NoProfile", "-Command", script])
        .output()
        .ok()?;
    let t = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

fn pick_file_dialog() -> Option<String> {
    ps_dialog(
        "Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.OpenFileDialog; $d.Filter = 'Configuration (*.json)|*.json|All files (*.*)|*.*'; if ($d.ShowDialog() -eq 'OK') { $d.FileName }",
    )
}

/// The Emulators tab's picker (the user, Oct 24): point at
/// the executable that corresponds to an emulator when
/// the tab can't find it on its own.
fn pick_exe_dialog() -> Option<String> {
    ps_dialog(
        "Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.OpenFileDialog; $d.Filter = 'Programs (*.exe)|*.exe|All files (*.*)|*.*'; $d.Title = 'Select the emulator executable'; if ($d.ShowDialog() -eq 'OK') { $d.FileName }",
    )
}

fn save_file_dialog(def: &str) -> Option<String> {
    ps_dialog(&format!(
        "Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.SaveFileDialog; $d.Filter = 'Configuration (*.json)|*.json'; $d.FileName = '{def}'; if ($d.ShowDialog() -eq 'OK') {{ $d.FileName }}"
    ))
}

fn pick_folder_dialog() -> Option<String> {
    ps_dialog(
        "Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.FolderBrowserDialog; if ($d.ShowDialog() -eq 'OK') { $d.SelectedPath }",
    )
}


// ---------------- the app ----------------

#[derive(Clone)]
enum View {
    Picker,
    Home,
    Wall(String),
}

// ---------------- background work messages ----------------------------
enum WorkMsg {
    Status(String),
    JobDone(String),
    Reload,
    Hacks(Vec<HackEntry>),
    /// Enhanced catalog loaded: (system, fetched entries
    /// kept for build-free row rebuilds, built rows,
    /// provenance note for the meta line — None leaves
    /// the line as it was, e.g. a post-build row rebuild).
    Catalog(String, Vec<CatalogEntry>, Vec<EnhRow>, Option<String>),
    /// A catalog build finished; the text goes to the
    /// status line and un-blocks the window.
    CatalogBuilt(String),
    Info(Option<GameInfo>),
    CheatsFetched(Vec<(String, String)>),
    Scraped(String),
    TexDone(String),
    EmuUpdates(BTreeMap<String, EmuUpdateInfo>),
    MusicStatus(String),
    MusicImported(String),
    /// ⚙ Cache & Performance measurements: (app cache
    /// tree bytes, shared library box-art bytes).
    CacheSizes(u64, u64),
}

struct HackState {
    game: Game,
    entries: Option<Vec<HackEntry>>,
    status: String,
    building: bool,
    /// true when this window came from the "Find MSU-1 / MD+
    /// version..." button (MSU search + chain), false for the
    /// generic RomHack Plaza window.
    msu: bool,
}

/// The "🎵 Enhanced Games" window's state. Exactly one
/// system's catalog at a time — `system` is the system
/// whose rows `rows` holds; switching systems reloads
/// rather than mixing. `entries` keeps the fetched
/// catalog so rows can be rebuilt after a build with no
/// new fetching.
struct EnhCatalogState {
    system: String,
    entries: Option<Vec<CatalogEntry>>,
    rows: Option<Vec<EnhRow>>,
    status: String,
    /// The catalog's provenance line (v1.06): how old
    /// the saved copy on screen is, or that it just came
    /// live. Kept apart from `status` so builds and
    /// launches don't overwrite the honesty.
    meta: String,
    /// A live fetch is in flight (the Refresh button,
    /// or a stale copy refreshing behind itself).
    refreshing: bool,
    filter: String,
    owned_only: bool,
    building: bool,
}

struct CheatState {
    game_path: String,
    name: String,
    text: String,
    status: String,
}

#[derive(Clone)]
struct InfoState {
    name: String,
    info: Option<GameInfo>,
    loading: bool,
}

#[derive(Clone)]
struct TextureState {
    game: Game,
    name: String,
    status: String,
    note: String,
    folder: Option<PathBuf>,
    search: Option<String>,
    kind: TexKind,
    key: String,
    gfx_dir: Option<PathBuf>,
    dolphin_user: Option<PathBuf>,
    packs: Vec<TexPack>,
    url_text: String,
    busy: bool,
}

fn texture_state_for(game: &Game, profiles: &[Profile]) -> TextureState {
    let t = texture_target(game, profiles);
    TextureState {
        game: game.clone(),
        name: game.name.clone(),
        status: t.status.clone(),
        note: String::new(),
        folder: t.folder.clone(),
        search: t.search.clone(),
        kind: t.kind,
        key: t.key.clone(),
        gfx_dir: t.gfx_dir.clone(),
        dolphin_user: t.dolphin_user.clone(),
        packs: packs_for(&t),
        url_text: String::new(),
        busy: false,
    }
}

#[derive(Default, Clone)]
struct EmuEdit {
    name: String,
    path: String,
    system: String,
    core: String,
    args: String,
    extensions: String,
    folder: String,
}

const CREDITS: &str = "\
Developed by Huntergokuz & Maya

Built with:
• egui / eframe — MIT License, © Emil Ernerfeldt and contributors
• image, serde, serde_json, ureq, sha1 crates — MIT/Apache-2.0, Rust ecosystem

Formats & data (thank you):
• IPS patch format — community format
• UPS and BPS patch formats — by Near (byuu)
• Box art — the libretro-thumbnails project (libretro contributors)
• Cheat codes — the libretro-database project
• ROM hack listings & files — RomHack Plaza and the hack authors
• MSU-1 patch index — Zeldix (zeldix.net) and its thread authors
• MD+ / MSU-1 patch repos — the patch authors on GitHub (infinest and others)
• MD+ pack mirror — the MiNERVA Archive (the community rescue of Myrient) and the Complete Sega MD+ Collection contributors
• HD texture packs — the pack authors: GhostlyDark & intermissionfb (evilgames.eu), qashto, vStar925, and the Mesen HD pack creators
• Game descriptions — Wikipedia contributors, CC BY-SA 4.0
• RetroArch — the libretro team
• Emulator update checks — each project's official GitHub releases (Dolphin, PCSX2, DuckStation, Mesen, SNES9x, Project64, mGBA, PPSSPP, melonDS, Azahar, RPCS3, Cemu, MAME, FBNeo, Flycast, xemu, Stella, BizHawk, VBA-M) and buildbot.libretro.com (RetroArch)

• Game music playback — libgme (Game_Music_Emu) by Shay Green (blargg) and contributors, LGPL-2.1-or-later, via the game-music-emu crate (MIT) — source: https://github.com/libgme/game-music-emu
• MIDI playback — rustysynth by sinshu (MIT), with the GeneralUser GS SoundFont by S. Christian Collins
• Audio output — rodio (MIT/Apache-2.0) and cpal (Apache-2.0)
• Discord presence — the discord-rich-presence crate (MIT OR Apache-2.0); local IPC to the Discord app on this machine only, no data leaves the PC
• The EmuFront Lounge Loop — an original composition for EmuFront

No ROMs, no BIOS files, and no emulators are included with
EmuFront — you supply your own games and any BIOS files your
emulators require. EmuFront organizes and launches what you
already have; it does not provide game content.

Original ROMs are never modified; patches build new files.";


/// System order (the user, Oct 24 2026): wherever systems are
/// presented for selection they read strict A–Z — no system
/// pinned ahead of its alphabetical place. The comparison is
/// the music section's natural order (digit runs numeric, so
/// "3DO" leads and "System 2" precedes "System 10"; letters
/// case-insensitive, so "MSX" no longer jumps ahead of
/// "Microsoft …" the way the old byte sort put it), with the
/// plain byte order as the tiebreak for names that fold
/// equal — the result is a total order, never hash-map order.
fn system_name_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    natural_cmp(a, b).then_with(|| a.cmp(b))
}

/// THE systems list: per-system game counts in selection
/// order. `Proto::systems` is built here and only here (at
/// startup and on reload), and every surface that presents
/// systems for selection renders that list — so the order
/// cannot drift between screens. Presentation only: this
/// sorts an in-memory (name, count) list; the stored library
/// (the config's games/profiles order) is never rewritten.
fn sorted_systems(games: &[Game]) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for g in games {
        *counts.entry(g.system.clone()).or_insert(0) += 1;
    }
    let mut systems: Vec<(String, usize)> = counts.into_iter().collect();
    systems.sort_by(|a, b| system_name_cmp(&a.0, &b.0));
    systems
}

/// The in-RAM search index (perf Slice 1, design §2.4):
/// every game's lowercase name and wall sort key,
/// computed ONCE per library load, parallel to the
/// games vec. refresh_wall used to lowercase all ~200k
/// names on every typed character; now the query is
/// lowercased once and the scan reads precomputed
/// strings, and wall sorts compare precomputed keys
/// instead of allocating one per comparison.
fn build_search_index(games: &[Game]) -> (Vec<String>, Vec<String>) {
    let lower: Vec<String> =
        games.iter().map(|g| g.name.to_lowercase()).collect();
    let keys: Vec<String> = games.iter().map(|g| sort_key(&g.name)).collect();
    (lower, keys)
}

/// Index-form wall membership: bit-for-bit the old
/// `name.to_lowercase().contains(q)` rule with the
/// lowercasing prepaid by the index (q arrives already
/// lowercased). Pure — --selftest pins parity against
/// the naive per-keystroke form.
fn wall_game_matches(lower_name: &str, q: &str) -> bool {
    q.is_empty() || lower_name.contains(q)
}

struct Proto {
    games: Vec<Game>,
    profiles: Vec<Profile>,
    systems: Vec<(String, usize)>,
    view: View,
    wall: Vec<usize>,
    bucket: Option<u8>,
    search: String,
    selected: Option<usize>,
    status: String,
    textures: HashMap<String, egui::TextureHandle>,
    tex_ledger: TexLedger,
    in_flight: HashSet<String>,
    failed: HashSet<String>,
    art_misses: HashMap<String, i64>,
    scrape_cancel: Arc<AtomicBool>,
    scrape_system: Option<String>,
    rx: Receiver<ArtMsg>,
    art_pool: ArtPool,
    art_gen: u64,
    /// Startup load in flight: Some until the loader
    /// thread delivers the library (snapshot hit or full
    /// parse). While Some, update() paints only the
    /// loading state — the window is alive from the
    /// first frame even on a cold cache and a slow disk.
    loading: Option<Receiver<StartupLib>>,
    /// The library's derived bundle (search index, art
    /// keys, ⭐ flags, per-system wall lists + bucket
    /// membership) — computed by build_derived on a full
    /// parse, or loaded straight from the snapshot.
    derived: Derived,
    /// The resolved machine profile: budgets, pool
    /// size, rendition tier, prefetch, pacing — from
    /// the settings via resolve_profile; the ⚙ Cache &
    /// Performance window recomputes it on change.
    profile: MachineProfile,
    /// Queued config writes awaiting the worker's
    /// verdict, and their result channel.
    cfg_rx: Receiver<CfgResult>,
    cfg_pending: usize,
    /// Art requests accepted this frame (the low-spec
    /// pacing cap counts these).
    art_frame_count: usize,
    /// The wall's top row last frame — the prefetch
    /// direction's memory (v1.06). Reset whenever the
    /// wall's generation bumps: a new wall has no travel
    /// direction yet.
    wall_top_prev: Option<usize>,
    show_cache: bool,
    /// Cache-tree size once measured (the ⚙ window
    /// measures on a thread; None = measuring).
    cache_size: Option<u64>,
    boxart_size: Option<u64>,
    /// Per-game MSU-badge results, computed once per
    /// run (design Pain 7: game_has_msu does a folder
    /// read; the panel asked every frame). Cleared when
    /// a build finishes or the library reloads — the
    /// moments a game's folder can gain MSU content.
    msu_cache: HashMap<String, bool>,
    load_ms: f64,
    settings: Settings,
    ra_status: String,
    ra_rx: Receiver<String>,
    ra_tx: Sender<String>,
    ra_started: bool,
    work_tx: Sender<WorkMsg>,
    work_rx: Receiver<WorkMsg>,
    launch_override: String,
    show_emulators: bool,
    emu_list: Vec<serde_json::Value>,
    emu_sel: Option<usize>,
    emu_edit: EmuEdit,
    emu_msg: String,
    show_folders: bool,
    folder_list: Vec<String>,
    new_folder: String,
    folder_msg: String,
    show_emu_tab: bool,
    emu_tab_status: String,
    hacks: Option<HackState>,
    enhanced_catalog: Option<EnhCatalogState>,
    cheats: Option<CheatState>,
    info: Option<InfoState>,
    textures_w: Option<TextureState>,
    show_about: bool,
    busy: bool,
    panel_path: String,
    emu_checking: bool,
    emu_check_started: bool,
    music: Option<MusicUiState>,
    music_player: Option<MusicPlayer>,
    /// v1.07: the sidecar store and its loader channel.
    /// Until the store lands (data_rx is Some) the play
    /// features are inert; a late load never clobbers
    /// what was already recorded (drain_data).
    data: RustData,
    data_rx: Option<Receiver<RustData>>,
    /// The one live tracked play session, if any.
    now_playing: Option<NowPlaying>,
    /// Discord presence worker handle — Some only when
    /// presence is configured (toggle on + an app ID).
    /// Sending on it is a channel push into a worker
    /// that drops every error.
    presence_tx: Option<Sender<PresenceMsg>>,
    /// The ⚙ window's Discord app-ID edit-box text,
    /// committed to settings when the box loses focus
    /// (the window's deferred-apply idiom).
    discord_id_edit: String,
}

impl Proto {
    fn new(cc: &eframe::CreationContext) -> Self {
        let settings = load_settings();
        let profile = resolve_profile(&settings, cores_now());
        let (tx, rx) = channel();
        let rend_cfg = RendCfg {
            w: profile.rendition.0,
            h: profile.rendition.1,
            dir: thumbs_dir(),
            cap: REND_CAP_BYTES,
        };
        let art_pool = ArtPool::start(tx, profile.art_workers, rend_cfg);
        let (ra_tx, ra_rx) = channel::<String>();
        let (work_tx, work_rx) = channel::<WorkMsg>();
        let (cfg_tx, cfg_rx) = channel::<CfgResult>();
        cfg_register_results(cfg_tx);
        // The library loads on a thread — snapshot if it
        // verifies, the full typed parse otherwise (and
        // that thread rebuilds the snapshot for next
        // time; see load_startup_library).
        let (load_tx, load_rx) = channel::<StartupLib>();
        {
            let ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                let data = load_startup_library();
                let _ = load_tx.send(data);
                ctx.request_repaint();
            });
        }
        // The sidecar loads on its own thread too: a
        // day-one first_seen map is library-sized, and
        // parsing it must not gate the first frame any
        // more than the library parse does.
        let (data_tx, data_rx) = channel::<RustData>();
        {
            let ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                let _ = data_tx.send(load_rust_data());
                ctx.request_repaint();
            });
        }
        let discord_id_edit = settings.discord_app_id.clone();
        let presence_tx = presence_app_id(&settings)
            .map(|id| spawn_presence_worker(id.to_string()));
        let ask = settings.ra_ask;
        Self {
            games: Vec::new(),
            profiles: Vec::new(),
            systems: Vec::new(),
            view: if ask { View::Picker } else { View::Home },
            wall: Vec::new(),
            bucket: None,
            search: String::new(),
            selected: None,
            status: "Loading your library…".into(),
            textures: HashMap::new(),
            tex_ledger: TexLedger::new(profile.cover_budget),
            in_flight: HashSet::new(),
            failed: HashSet::new(),
            art_misses: load_art_misses(),
            scrape_cancel: Arc::new(AtomicBool::new(false)),
            scrape_system: None,
            rx,
            art_pool,
            art_gen: 0,
            loading: Some(load_rx),
            derived: Derived::default(),
            profile,
            cfg_rx,
            cfg_pending: 0,
            art_frame_count: 0,
            wall_top_prev: None,
            show_cache: false,
            cache_size: None,
            boxart_size: None,
            msu_cache: HashMap::new(),
            load_ms: 0.0,
            ra_status: "RetroArch: …".into(),
            ra_started: false,
            settings,
            ra_rx,
            ra_tx,
            work_tx,
            work_rx,
            launch_override: String::new(),
            show_emulators: false,
            emu_list: Vec::new(),
            emu_sel: None,
            emu_edit: EmuEdit::default(),
            emu_msg: String::new(),
            show_folders: false,
            folder_list: Vec::new(),
            new_folder: String::new(),
            folder_msg: String::new(),
            show_emu_tab: false,
            emu_tab_status: String::new(),
            hacks: None,
            enhanced_catalog: None,
            cheats: None,
            info: None,
            textures_w: None,
            show_about: false,
            busy: false,
            panel_path: String::new(),
            emu_checking: false,
            emu_check_started: false,
            music: None,
            music_player: None,
            data: RustData::default(),
            data_rx: Some(data_rx),
            now_playing: None,
            presence_tx,
            discord_id_edit,
        }
    }

    /// The loader thread delivered the library: swap it
    /// in and say so, in the status line's long-standing
    /// format.
    fn apply_loaded(&mut self, data: StartupLib) {
        let n = data.games.len();
        self.systems = data.derived.systems.clone();
        self.games = data.games;
        self.profiles = data.profiles;
        self.derived = data.derived;
        self.load_ms = data.load_ms;
        self.status = format!("Loaded {n} games in {:.0} ms", data.load_ms);
        self.observe_library();
    }

    /// The sidecar store landed (its own loader thread:
    /// parsing a library-sized first_seen map must not
    /// gate the first frame). If anything was somehow
    /// recorded before it arrived, the in-memory copy
    /// wins and the loaded one is dropped — on a first
    /// run there is no file at all, so that race is
    /// theoretical, and this keeps it lossless anyway.
    fn drain_data(&mut self) {
        let landed = match &self.data_rx {
            Some(rx) => match rx.try_recv() {
                Ok(d) => Some(Some(d)),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    Some(None)
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
            },
            None => None,
        };
        if let Some(maybe) = landed {
            self.data_rx = None;
            if let Some(d) = maybe {
                if !self.data.dirty {
                    self.data = d;
                }
            }
            self.observe_library();
        }
    }

    /// Feed the library's paths to the first-seen
    /// bookkeeping (the day-one baseline; later, the
    /// new-path stamps). Called when the store lands,
    /// when the library arrives, and on every reload —
    /// never per frame.
    fn observe_library(&mut self) {
        if self.data_rx.is_some() {
            return;
        }
        let now = now_epoch();
        let paths: Vec<&str> =
            self.games.iter().map(|g| g.path.as_str()).collect();
        self.data.observe_first_seen(paths.into_iter(), now);
    }

    /// A tracked launch succeeded: retain the session.
    /// Any session already live is finalized first —
    /// tracking is one game at a time (attribution
    /// honesty: with two emulators running, only the
    /// latest launch is "the game being played"; the
    /// first one's clock stops here — roadmap §1.3).
    fn begin_play(&mut self, game: &Game, child: std::process::Child) {
        self.end_play();
        self.now_playing = Some(NowPlaying {
            path: game.path.clone(),
            started: Instant::now(),
            child,
        });
        self.presence_set(game);
    }

    /// Fold the live session (if any) into the sidecar
    /// and clear the presence. Idempotent — it runs on
    /// emulator exit, on the next tracked launch, on
    /// window close, and on drop, and only the first of
    /// those does anything.
    fn end_play(&mut self) {
        if let Some(np) = self.now_playing.take() {
            let secs = np.started.elapsed().as_secs();
            fold_session(&mut self.data, &np.path, now_epoch(), secs);
            self.presence_clear();
        }
    }

    /// Watch the tracked session: when the emulator's
    /// process exits, fold it. While a session is live
    /// we keep a 1 s repaint cadence so the exit is
    /// noticed promptly even if the window sits idle;
    /// the fold itself is elapsed-time based, so a late
    /// notice never loses time.
    fn poll_play(&mut self, ctx: &egui::Context) {
        let exited = match &mut self.now_playing {
            Some(np) => !matches!(np.child.try_wait(), Ok(None)),
            None => false,
        };
        if exited {
            self.end_play();
        } else if self.now_playing.is_some() {
            ctx.request_repaint_after(Duration::from_millis(1000));
        }
    }

    fn presence_set(&self, game: &Game) {
        if let Some(tx) = &self.presence_tx {
            let _ = tx.send(PresenceMsg::Set {
                name: game.name.clone(),
                system: game.system.clone(),
                start_ms: now_epoch() as i64 * 1000,
            });
        }
    }

    fn presence_clear(&self) {
        if let Some(tx) = &self.presence_tx {
            let _ = tx.send(PresenceMsg::Clear);
        }
    }

    /// (Re)build the presence worker from the current
    /// settings (the ⚙ window calls this on a toggle or
    /// ID change). Dropping the old sender ends the old
    /// worker, which clears any live status on its way
    /// out — changes take effect immediately.
    fn reset_presence_worker(&mut self) {
        self.presence_tx = presence_app_id(&self.settings)
            .map(|id| spawn_presence_worker(id.to_string()));
    }

    /// The debounced sidecar flush: ~3 s after the first
    /// unflushed change, serialize and write on a thread.
    /// Called every frame; costs a flag check.
    fn flush_data(&mut self) {
        if self.data.dirty
            && now_epoch().saturating_sub(self.data.dirty_since) >= 3
        {
            self.data.dirty = false;
            spawn_data_save(&self.data);
        }
    }

    /// The final flush (window close / drop): write now,
    /// on this thread.
    fn flush_data_now(&mut self) {
        if self.data.dirty {
            self.data.dirty = false;
            save_rust_data_now(&self.data);
        }
    }

    /// Queue a config write for the background worker
    /// (§2.5). The UI never waits on the file again; the
    /// verdict comes back through drain_cfg.
    fn cfg_enqueue(&mut self, m: CfgMutation, tag: CfgTag) {
        self.cfg_pending += 1;
        cfg_submit(CfgJob {
            m,
            reply: None,
            tag: Some(tag),
        });
    }

    fn drain_cfg(&mut self, ctx: &egui::Context) {
        while let Ok(r) = self.cfg_rx.try_recv() {
            self.cfg_pending = self.cfg_pending.saturating_sub(1);
            match r.tag {
                CfgTag::Quiet => {
                    if let Err(e) = r.result {
                        self.status = format!("Save failed: {e}");
                    }
                }
                CfgTag::EmulatorProfiles => match r.result {
                    Ok(_) => {
                        self.reload_library();
                        self.emu_msg =
                            "Saved — library reloaded.".into();
                        self.status =
                            "Emulator profiles saved.".into();
                    }
                    Err(e) => {
                        self.emu_msg = format!("Save failed: {e}")
                    }
                },
                CfgTag::LibraryDirs { rescan } => match r.result {
                    Ok(_) => {
                        self.folder_msg = "Folders saved.".into();
                        if rescan {
                            self.spawn_rescan(ctx);
                        }
                    }
                    Err(e) => {
                        self.folder_msg = format!("Save failed: {e}")
                    }
                },
            }
        }
    }

    /// Measure the cache tree and the shared box-art
    /// folder on a thread (both walks can be slow on a
    /// spinning disk); the ⚙ window shows "Measuring…"
    /// until WorkMsg::CacheSizes lands.
    fn spawn_cache_measure(&mut self) {
        self.cache_size = None;
        self.boxart_size = None;
        let tx = self.work_tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(WorkMsg::CacheSizes(
                dir_size(&cache_dir()),
                dir_size(&art_dir()),
            ));
        });
    }

    /// Re-resolve the machine profile after a ⚙ control
    /// changed the settings, and apply the parts that can
    /// change live: the texture budget (evicting whatever
    /// no longer fits). Workers, rendition tier, window
    /// and scale follow at the next launch — the window
    /// says so.
    fn apply_profile_budgets(&mut self) {
        self.profile = resolve_profile(&self.settings, cores_now());
        let evicted = self
            .tex_ledger
            .set_budget(self.profile.cover_budget);
        for e in evicted {
            self.textures.remove(&e);
        }
    }

    fn start_ra_check(&self, ctx: &egui::Context) {
        let tx = self.ra_tx.clone();
        let channel = self.settings.ra_channel.clone();
        let ctx2 = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send("RetroArch: checking for the latest build…".into());
            let msg = ra_ensure(&channel);
            let _ = tx.send(msg);
            ctx2.request_repaint();
        });
    }

    /// Emulator update check on a worker thread — never blocks the UI.
    fn start_emu_check(&mut self, ctx: &egui::Context) {
        if self.emu_checking {
            return;
        }
        self.emu_checking = true;
        let tx = self.work_tx.clone();
        let profiles = self.profiles.clone();
        let ctx2 = ctx.clone();
        std::thread::spawn(move || {
            let map = check_emu_updates(&profiles);
            let _ = tx.send(WorkMsg::EmuUpdates(map));
            ctx2.request_repaint();
        });
    }

    fn drain_ra(&mut self) {
        while let Ok(m) = self.ra_rx.try_recv() {
            self.ra_status = m;
        }
    }

    fn enter_wall(&mut self, system: &str) {
        // Letter-range browsing: a freshly opened system starts
        // unbucketed (the chooser shows for large systems).
        self.bucket = None;
        self.bump_art_gen();
        let q = self.search.to_lowercase();
        // The derived per-system list is already in wall
        // order — build_derived sorts with the exact
        // comparator this fn used to re-apply on every
        // open (precomputed sort key, then name) — so
        // opening a wall is a lookup plus the search
        // filter, never a fresh sort of the whole system.
        let base = self
            .derived
            .system_list(system)
            .cloned()
            .unwrap_or_default();
        self.wall = base
            .into_iter()
            .filter(|&i| {
                wall_game_matches(&self.derived.lower_names[i], &q)
            })
            .collect();
        self.selected = self.wall.first().copied();
        self.view = View::Wall(system.to_string());
    }

    /// The wall (or the library under it) changed: advance
    /// the art generation so queued covers for the old
    /// wall are dropped instead of decoded, and forget
    /// what was in flight for it — the tiles re-ask as
    /// they render.
    fn bump_art_gen(&mut self) {
        self.art_gen += 1;
        self.in_flight.clear();
        self.art_pool.set_gen(self.art_gen);
        self.wall_top_prev = None;
    }

    /// Ask the art pool for a game's cover — a tile on
    /// screen is waiting for it.
    fn request_art(&mut self, ctx: &egui::Context, game: &Game) {
        self.request_art_prio(ctx, game, true);
    }

    /// Prefetch priority: decode before the tile scrolls
    /// into view, dropped first when the wall changes.
    /// The profile decides whether prefetch exists at
    /// all (low-spec machines and spinning disks don't
    /// speculate — study §3). Returns whether a job was
    /// actually queued — the wall's per-frame pacing
    /// (v1.06) counts real queues, not attempts.
    fn request_art_low(&mut self, ctx: &egui::Context, game: &Game) -> bool {
        if !self.profile.prefetch {
            return false;
        }
        self.request_art_prio(ctx, game, false)
    }

    fn request_art_prio(&mut self, ctx: &egui::Context, game: &Game, high: bool) -> bool {
        // Low-spec pacing (study §3): cap the new art
        // work one frame may start, so a fast scroll on a
        // single-core machine degrades to "covers fill
        // in" instead of frame hitches. A capped tile is
        // not forgotten — the render loop re-asks next
        // frame, since its texture is still missing.
        if self.art_frame_count >= self.profile.frame_art_cap {
            return false;
        }
        let key = art_key(&game.path);
        if self.textures.contains_key(&key)
            || self.in_flight.contains(&key)
            || self.failed.contains(&key)
        {
            return false;
        }
        // a recorded "no art found" is not retried on display
        // until it ages out (manual scrapes always retry)
        if let Some(ts) = self.art_misses.get(&key) {
            if miss_is_fresh(*ts, now_ts()) {
                return false;
            }
        }
        self.in_flight.insert(key.clone());
        self.art_frame_count += 1;
        self.art_pool.set_notify(ctx);
        self.art_pool.submit(
            ArtJob {
                key,
                game: game.clone(),
                gen: self.art_gen,
            },
            high,
        );
        true
    }

    /// The cached cover texture for a key, if decoded.
    /// Reading through here is what makes the LRU an
    /// LRU: the read marks the key as just-used.
    fn tex_handle(&mut self, key: &str) -> Option<egui::TextureHandle> {
        let h = self.textures.get(key)?.clone();
        self.tex_ledger.touch(key);
        Some(h)
    }

    /// The game's MSU/MD+ badge answer, computed once
    /// per run (the folder read the panel used to pay
    /// every frame).
    fn has_msu(&mut self, game: &Game) -> bool {
        if let Some(v) = self.msu_cache.get(&game.path) {
            return *v;
        }
        let v = game_has_msu(game);
        self.msu_cache.insert(game.path.clone(), v);
        v
    }

    fn drain_art(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                ArtMsg::Ready(key, color) => {
                    self.in_flight.remove(&key);
                    // Upload only — decode happened on a
                    // pool worker. The ledger accounts the
                    // decoded bytes and names any LRU
                    // evictions the budget forces.
                    let bytes = color.size[0] * color.size[1] * 4;
                    let tex = ctx.load_texture(
                        &key,
                        color,
                        egui::TextureOptions::LINEAR,
                    );
                    let evicted = self.tex_ledger.insert(&key, bytes);
                    for e in evicted {
                        self.textures.remove(&e);
                    }
                    self.textures.insert(key, tex);
                }
                ArtMsg::Failed(key) => {
                    self.in_flight.remove(&key);
                    self.failed.insert(key);
                }
                ArtMsg::Missing(key) => {
                    self.in_flight.remove(&key);
                    self.failed.insert(key.clone());
                    // persist the miss so the next launch doesn't
                    // retry the network lookup for this game
                    self.art_misses.insert(key, now_ts());
                    save_art_misses(&self.art_misses);
                }
            }
        }
    }

    fn picker(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical_centered(|ui| {
            ui.add_space(120.0);
            ui.heading(
                egui::RichText::new("🎮 EmuFront Rust")
                    .size(34.0)
                    .color(Color32::WHITE),
            );
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(
                    "Which RetroArch build should run your games?",
                )
                .size(17.0)
                .color(Color32::WHITE),
            );
            let (ch, label) = ra_stamp();
            if !label.is_empty() {
                ui.label(
                    egui::RichText::new(format!(
                        "Installed now: {ch} — {label}"
                    ))
                    .color(TEXT_DIM),
                );
            }
            ui.add_space(18.0);
            let mut pick: Option<&str> = None;
            if ui
                .add_sized(
                    [420.0, 58.0],
                    egui::Button::new(
                        egui::RichText::new(
                            "🌙  Nightly — always the latest build",
                        )
                        .size(18.0)
                        .color(Color32::WHITE),
                    )
                    .fill(ACCENT),
                )
                .clicked()
            {
                pick = Some("nightly");
            }
            ui.label(
                egui::RichText::new(
                    "Newest RetroArch, updated automatically at startup",
                )
                .color(TEXT_DIM),
            );
            ui.add_space(10.0);
            if ui
                .add_sized(
                    [420.0, 58.0],
                    egui::Button::new(
                        egui::RichText::new(
                            "🛡️  Stable — official releases only",
                        )
                        .size(18.0)
                        .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(70, 70, 96)),
                )
                .clicked()
            {
                pick = Some("stable");
            }
            ui.label(
                egui::RichText::new(
                    "Slower-moving, battle-tested RetroArch releases",
                )
                .color(TEXT_DIM),
            );
            ui.add_space(16.0);
            ui.checkbox(&mut self.settings.ra_ask, "Ask me at startup");
            if let Some(c) = pick {
                self.settings.ra_channel = c.to_string();
                save_settings(&self.settings);
                self.view = View::Home;
                self.ra_started = true;
                self.start_ra_check(ctx);
            }
        });
    }

    /// The v1.07 Home strips above the systems grid:
    /// ▶ Continue Playing and 🆕 New This Week. Each is a
    /// headed horizontal row of mini cover tiles fed by
    /// the same art pipeline as the walls; a strip with
    /// nothing to show doesn't render at all. (home()
    /// calls this inside its scroll area.)
    fn home_strips(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let cont = continue_indices(
            &self.games,
            &self.data.play,
            &self.derived.lower_names,
            "",
        );
        if !cont.is_empty() {
            let items: Vec<(usize, String)> = cont
                .iter()
                .map(|&gi| {
                    let r = &self.data.play[&self.games[gi].path];
                    (
                        gi,
                        format!(
                            "{} · {}",
                            fmt_playtime(r.secs),
                            fmt_day(r.last)
                        ),
                    )
                })
                .collect();
            self.shelf_strip(ui, ctx, CONTINUE_TITLE, &items, 12);
        }
        let new = new_indices(
            &self.games,
            &self.data,
            &self.derived.lower_names,
            "",
            now_epoch(),
        );
        if !new.is_empty() {
            let items: Vec<(usize, String)> = new
                .iter()
                .map(|&gi| {
                    let ts = self.data.first_seen[&self.games[gi].path];
                    (gi, format!("Added {}", fmt_day(ts)))
                })
                .collect();
            self.shelf_strip(ui, ctx, NEW_TITLE, &items, 24);
        }
    }

    /// One Home strip: heading + a horizontally scrolling
    /// row of up to `cap` mini tiles, plus a "See all ›"
    /// end-cap (only when the shelf holds more than the
    /// cap) that opens the whole shelf as a wall. A tile
    /// click jumps to the game's system wall with the
    /// game selected — on a big system it lands in the
    /// game's letter bucket, so the game is actually on
    /// screen; the tile's ▶ launches it directly
    /// (tracked, like every GUI launch).
    fn shelf_strip(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        title: &str,
        items: &[(usize, String)],
        cap: usize,
    ) {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(title)
                .size(17.0)
                .color(Color32::WHITE),
        );
        ui.add_space(4.0);
        let tile_w = 118.0;
        let tile_h = 158.0;
        let shown = items.len().min(cap);
        let mut jump: Option<usize> = None;
        let mut launch_gi: Option<usize> = None;
        let mut open_all = false;
        egui::ScrollArea::horizontal()
            .id_salt(title)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (gi, sub) in items.iter().take(shown) {
                        let gi = *gi;
                        let game = self.games[gi].clone();
                        let (rect, resp) = ui.allocate_exact_size(
                            Vec2::new(tile_w, tile_h),
                            Sense::click(),
                        );
                        let p = ui.painter_at(rect);
                        p.rect_filled(rect, 8.0, PANEL);
                        if resp.hovered() {
                            p.rect_stroke(
                                rect,
                                8.0,
                                egui::Stroke::new(2.0_f32, ACCENT),
                            );
                        }
                        let art_rect = Rect::from_min_size(
                            Pos2::new(rect.min.x + 9.0, rect.min.y + 7.0),
                            Vec2::new(100.0, 104.0),
                        );
                        p.rect_filled(art_rect, 0.0, BG);
                        let key = art_key(&game.path);
                        if let Some(tex) = self.tex_handle(&key) {
                            let ts = tex.size_vec2();
                            let s = (art_rect.width() / ts.x)
                                .min(art_rect.height() / ts.y)
                                .min(1.0);
                            let fit = Rect::from_center_size(
                                art_rect.center(),
                                Vec2::new(ts.x * s, ts.y * s),
                            );
                            p.image(
                                tex.id(),
                                fit,
                                Rect::from_min_max(
                                    Pos2::ZERO,
                                    Pos2::new(1.0, 1.0),
                                ),
                                Color32::WHITE,
                            );
                        } else {
                            self.request_art(ctx, &game);
                            let initial = game
                                .name
                                .chars()
                                .next()
                                .unwrap_or('?')
                                .to_string();
                            p.text(
                                art_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                initial,
                                egui::FontId::proportional(30.0),
                                TEXT_DIM,
                            );
                        }
                        // the direct-launch button, over
                        // the art's top-right corner
                        let play_rect = Rect::from_min_size(
                            Pos2::new(rect.max.x - 30.0, rect.min.y + 6.0),
                            Vec2::new(24.0, 24.0),
                        );
                        let play_resp = ui.interact(
                            play_rect,
                            egui::Id::new(("shelfplay", gi)),
                            Sense::click(),
                        );
                        let pp = ui.painter_at(play_rect);
                        pp.circle_filled(play_rect.center(), 11.0, ACCENT);
                        pp.text(
                            play_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "▶",
                            egui::FontId::proportional(11.0),
                            Color32::WHITE,
                        );
                        p.text(
                            Pos2::new(rect.center().x, rect.max.y - 30.0),
                            egui::Align2::CENTER_CENTER,
                            &game.name,
                            egui::FontId::proportional(11.5),
                            Color32::WHITE,
                        );
                        p.text(
                            Pos2::new(rect.center().x, rect.max.y - 14.0),
                            egui::Align2::CENTER_CENTER,
                            sub,
                            egui::FontId::proportional(10.5),
                            TEXT_DIM,
                        );
                        if play_resp.clicked() {
                            launch_gi = Some(gi);
                        } else if resp.clicked() {
                            jump = Some(gi);
                        }
                    }
                    if items.len() > shown {
                        let (rect, resp) = ui.allocate_exact_size(
                            Vec2::new(64.0, tile_h),
                            Sense::click(),
                        );
                        let p = ui.painter_at(rect);
                        p.rect_filled(rect, 8.0, PANEL);
                        if resp.hovered() {
                            p.rect_stroke(
                                rect,
                                8.0,
                                egui::Stroke::new(2.0_f32, ACCENT),
                            );
                        }
                        p.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "See\nall ›",
                            egui::FontId::proportional(13.0),
                            TEXT_DIM,
                        );
                        if resp.clicked() {
                            open_all = true;
                        }
                    }
                });
            });
        if let Some(gi) = jump {
            let sys = self.games[gi].system.clone();
            let total = self
                .systems
                .iter()
                .find(|(n, _)| n == &sys)
                .map(|(_, c)| *c)
                .unwrap_or(0);
            self.enter_wall(&sys);
            if total > BUCKET_MIN {
                // Land in the game's own letter bucket so
                // the jumped-to game is on screen, not
                // behind the chooser.
                self.bucket = Some(bucket_of(&self.games[gi].name));
                self.refresh_wall();
            }
            self.selected = Some(gi);
        }
        if let Some(gi) = launch_gi {
            let game = self.games[gi].clone();
            self.music_stop();
            match launch_started(&game, &self.profiles) {
                Ok((m, child)) => {
                    self.status = m;
                    self.begin_play(&game, child);
                }
                Err(m) => self.status = m,
            }
        }
        if open_all {
            self.view = View::Wall(title.to_string());
            self.refresh_wall();
        }
        ui.add_space(10.0);
    }

    fn home(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(6.0);
        // The ⭐ shelf tile leads (a shelf, not a system);
        // the systems follow in strict A–Z from self.systems
        // (the user, Oct 24 2026: no system pinned ahead of its
        // alphabetical place). The old pinned "Super Nintendo
        // — MSU-1 Enhanced" / "Sega Genesis — MD+ Enhanced"
        // tiles were duplicates — they entered the same walls
        // as the plain system tiles — so they are gone: Sega
        // Genesis and Super Nintendo appear once each, in
        // place, and the Enhanced catalog lives on the header
        // "🎵 Enhanced Games" button.
        let mut tiles: Vec<(String, String, String, bool)> = vec![
            (
                "star".into(),
                "⭐ Enhanced & Hacked".into(),
                format!(
                    "{} installed",
                    self.games
                        .iter()
                        .filter(|g| g.path.starts_with(data_dir_str()))
                        .count()
                ),
                true,
            ),
        ];
        for (name, count) in &self.systems {
            tiles.push((
                name.clone(),
                name.clone(),
                format!("{count} games"),
                false,
            ));
        }
        let avail = ui.available_width();
        let tile_w = 216.0;
        let tile_h = 172.0;
        let cols = ((avail / tile_w) as usize).max(1);
        let home_pad = ((avail - cols as f32 * tile_w) / 2.0).max(0.0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.home_strips(ui, ctx);
            let rows = tiles.len().div_ceil(cols);
            for r in 0..rows {
                ui.horizontal(|ui| {
                    ui.add_space(home_pad);
                    for c in 0..cols {
                        let i = r * cols + c;
                        if i >= tiles.len() {
                            break;
                        }
                        let (sys, label, sub, _gold) = &tiles[i];
                        let (rect, resp) = ui.allocate_exact_size(
                            Vec2::new(tile_w - 12.0, tile_h - 12.0),
                            Sense::click(),
                        );
                        let p = ui.painter_at(rect);
                        p.rect_filled(rect, 10.0, PANEL);
                        if resp.hovered() {
                            p.rect_stroke(
                                rect,
                                10.0,
                                egui::Stroke::new(2.0_f32, ACCENT),
                            );
                        }
                        let icon_rect = Rect::from_min_size(
                            Pos2::new(rect.min.x + 5.0, rect.min.y + 2.0),
                            Vec2::new(196.0, 112.0),
                        );
                        draw_icon(&p, icon_rect, sys);
                        p.text(
                            Pos2::new(rect.center().x, rect.max.y - 34.0),
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::proportional(13.5),
                            Color32::WHITE,
                        );
                        p.text(
                            Pos2::new(rect.center().x, rect.max.y - 15.0),
                            egui::Align2::CENTER_CENTER,
                            sub,
                            egui::FontId::proportional(11.5),
                            TEXT_DIM,
                        );
                        if resp.clicked() {
                            if i == 0 {
                                // enhanced shelf: games living under
                                // the EmuFront data dir
                                self.wall = self
                                    .games
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, g)| {
                                        g.path.starts_with(data_dir_str())
                                    })
                                    .map(|(ix, _)| ix)
                                    .collect();
                                self.selected =
                                    self.wall.first().copied();
                                self.view = View::Wall(
                                    "⭐ Enhanced & Hacked".into(),
                                );
                            } else {
                                let s = sys.clone();
                                self.enter_wall(&s);
                            }
                        }
                    }
                });
            }
        });
        let _ = ctx;
    }

    fn reload_library(&mut self) {
        let l = load_library_full();
        let derived = build_derived(&l.games);
        // The config just changed (our own queued write,
        // a rescan, an import, or the Python app): the
        // snapshot must follow it, or the next start
        // would rebuild from a fingerprint mismatch
        // anyway — rebuilding now, in the background,
        // keeps every start on the fast path.
        let snap = build_snapshot(
            &l.games,
            &l.profiles,
            &derived,
            &l.fingerprint,
        );
        std::thread::spawn(move || persist_snapshot(&snap));
        self.games = l.games;
        self.profiles = l.profiles;
        self.systems = derived.systems.clone();
        self.derived = derived;
        self.load_ms = l.load_ms;
        self.textures.clear();
        self.tex_ledger.clear();
        self.failed.clear();
        self.msu_cache.clear();
        self.bump_art_gen();
        if let View::Wall(sys) = self.view.clone() {
            self.refresh_wall();
            let _ = sys;
        }
        self.observe_library();
    }

    fn refresh_wall(&mut self) {
        let title = match &self.view {
            View::Wall(t) => t.clone(),
            _ => return,
        };
        self.bump_art_gen();
        let q = self.search.to_lowercase();
        // v1.07 shelves: membership AND order come from
        // the sidecar (recency, first-seen), not the
        // system predicate below — build them directly.
        // The search filter still applies, with the same
        // matcher as every wall.
        if title == CONTINUE_TITLE || title == NEW_TITLE {
            self.wall = if title == CONTINUE_TITLE {
                continue_indices(
                    &self.games,
                    &self.data.play,
                    &self.derived.lower_names,
                    &q,
                )
            } else {
                new_indices(
                    &self.games,
                    &self.data,
                    &self.derived.lower_names,
                    &q,
                    now_epoch(),
                )
            };
            if !self
                .selected
                .map(|s| self.wall.contains(&s))
                .unwrap_or(false)
            {
                self.selected = self.wall.first().copied();
            }
            return;
        }
        let enhanced = title.starts_with('⭐');
        // Letter-range browsing: the bucket predicate applies
        // only to real system walls (never the ⭐ shelf, never
        // PC Ports) and only while the search box is empty --
        // typing a title always searches the whole system.
        let bucket = if !enhanced && title != "PC Ports" && q.is_empty() {
            self.bucket
        } else {
            None
        };
        self.wall = self
            .games
            .iter()
            .enumerate()
            .filter(|(i, g)| {
                let in_shelf = if enhanced {
                    g.path.starts_with(data_dir_str())
                } else {
                    g.system == title
                };
                in_shelf
                    && wall_game_matches(&self.derived.lower_names[*i], &q)
                    && bucket.map_or(true, |b| bucket_of(&g.name) == b)
            })
            .map(|(i, _)| i)
            .collect();
        if !self
            .selected
            .map(|s| self.wall.contains(&s))
            .unwrap_or(false)
        {
            self.selected = self.wall.first().copied();
        }
    }

    fn open_enhanced(&mut self) {
        self.bucket = None;
        self.view = View::Wall("⭐ Enhanced & Hacked".into());
        self.refresh_wall();
    }

    fn effective_profile(&self, game: &Game) -> Option<Profile> {
        if !self.launch_override.is_empty() {
            if let Some(p) = self
                .profiles
                .iter()
                .find(|p| p.name == self.launch_override)
            {
                return Some(p.clone());
            }
        }
        pick_profile(game, &self.profiles).cloned()
    }

    fn spawn_rescan(&mut self, ctx: &egui::Context) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.status = "Rescanning library folders…".into();
        let tx = self.work_tx.clone();
        let ctx2 = ctx.clone();
        std::thread::spawn(move || {
            match scan_library() {
                Ok(games) => {
                    let n = games.len();
                    // Through the guarded write queue
                    // like every other config mutation:
                    // a rescan's games replace is applied
                    // onto a fresh read, never a stale one.
                    match cfg_write_sync(CfgMutation::ReplaceGames {
                        games: serde_json::Value::Array(games),
                    }) {
                        Ok(_) => {
                            let _ = tx.send(WorkMsg::Reload);
                            let _ = tx.send(WorkMsg::JobDone(format!(
                                "Rescan complete: {n} games in the library."
                            )));
                        }
                        Err(e) => {
                            let _ = tx
                                .send(WorkMsg::JobDone(format!("Rescan failed: {e}")));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(WorkMsg::JobDone(format!("Rescan failed: {e}")));
                }
            }
            ctx2.request_repaint();
        });
    }

    fn spawn_scrape_system(&mut self, ctx: &egui::Context, system: String) {
        if self.busy {
            return;
        }
        // PC Ports are never scraped: their art is the ports' own
        // icons, placed in the cache when the entry was registered.
        //
        // Scraping runs in batches of SCRAPE_BATCH games (the user's
        // rule: never the whole system at once). The queue is
        // pre-filtered to games with no cached cover and no fresh
        // recorded miss, so stopping a run and starting it again
        // RESUMES where it left off instead of starting over.
        // Refresh the miss index from disk first: scrape threads
        // record misses to the file as they go, and the filter
        // below must see them or it would re-hunt games already
        // known to have no cover.
        self.art_misses = load_art_misses();
        let now = now_ts();
        let games: Vec<Game> = self
            .games
            .iter()
            .filter(|g| g.system == system && !is_port(g))
            .filter(|g| {
                !boxart_file(&art_key(&g.path)).exists()
                    && !boxart_sys_file(g).exists()
            })
            .filter(|g| match self.art_misses.get(&art_key(&g.path)) {
                Some(ts) => !miss_is_fresh(*ts, now),
                None => true,
            })
            .cloned()
            .collect();
        if games.is_empty() {
            self.status = format!(
                "Box art for {system}: nothing to scrape -- every game already has a cover in the cache folder or a recent miss on record."
            );
            return;
        }
        let total = games.len();
        let batches = total.div_ceil(SCRAPE_BATCH);
        self.busy = true;
        self.scrape_system = Some(system.clone());
        self.scrape_cancel.store(false, Ordering::Relaxed);
        self.status = format!(
            "Scraping box art for {system} -- {total} games in {batches} batches of {SCRAPE_BATCH}…"
        );
        let tx = self.work_tx.clone();
        let ctx2 = ctx.clone();
        let cancel = self.scrape_cancel.clone();
        std::thread::spawn(move || {
            let mut got = 0usize;
            let mut done = 0usize;
            let mut stopped = false;
            'outer: for (bi, chunk) in games.chunks(SCRAPE_BATCH).enumerate() {
                let mut saved_keys: Vec<String> = Vec::new();
                let mut missed_keys: Vec<String> = Vec::new();
                for g in chunk {
                    if cancel.load(Ordering::Relaxed) {
                        stopped = true;
                        break 'outer;
                    }
                    let key = art_key(&g.path);
                    match fetch_art_bytes(g) {
                        Some(bytes) => {
                            if save_boxart(g, &bytes) {
                                got += 1;
                                saved_keys.push(key);
                            }
                        }
                        None => missed_keys.push(key),
                    }
                    done += 1;
                }
                // per-batch bookkeeping: new covers stop being
                // misses; new misses are recorded so a re-run (and
                // the display path) skips them for the TTL window
                if !saved_keys.is_empty() || !missed_keys.is_empty() {
                    let mut m = load_art_misses();
                    for k in &saved_keys {
                        m.remove(k);
                    }
                    let ts = now_ts();
                    for k in &missed_keys {
                        m.insert(k.clone(), ts);
                    }
                    save_art_misses(&m);
                }
                let _ = tx.send(WorkMsg::Status(format!(
                    "Scraping {system}: batch {}/{batches} done -- {done}/{total} games, {got} covers saved…",
                    bi + 1
                )));
                ctx2.request_repaint();
            }
            let _ = tx.send(WorkMsg::JobDone(if stopped {
                format!(
                    "Art scrape for {system} stopped after {done}/{total} games ({got} covers saved). Scrape again to pick up where it left off."
                )
            } else {
                format!(
                    "Art scrape for {system}: {got} covers saved ({total} games in batches of {SCRAPE_BATCH})."
                )
            }));
            ctx2.request_repaint();
        });
    }

    fn topbar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.heading(
                egui::RichText::new("🎮 EmuFront Rust")
                    .size(22.0)
                    .color(Color32::WHITE),
            );
            ui.label(
                egui::RichText::new(format!("v{APP_VERSION}"))
                    .color(GOLD),
            );
            ui.label(
                egui::RichText::new("developed by Huntergokuz & Maya")
                    .color(TEXT_DIM),
            );
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if ui.button("RetroArch build…").clicked() {
                        self.view = View::Picker;
                    }
                    ui.label(
                        egui::RichText::new(&self.ra_status).color(GOLD),
                    );
                },
            );
        });
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            let in_wall = matches!(self.view, View::Wall(_));
            if ui
                .add_enabled(in_wall, egui::Button::new("🏠 Systems"))
                .clicked()
            {
                self.view = View::Home;
            }
            if ui.button("🎵 Enhanced Games").clicked() {
                // Python parity: this button opens the
                // Enhanced catalog (on Super Nintendo, like
                // Python) — the ⭐ installed shelf stays on
                // the Home wall and in the window's footer.
                self.open_enhanced_catalog(ctx, "Super Nintendo");
            }
            if ui.button("🎮 Manage Emulators").clicked() {
                let cfg = load_cfg();
                self.emu_list = cfg
                    .get("emulators")
                    .and_then(|x| x.as_array())
                    .cloned()
                    .unwrap_or_default();
                self.emu_sel = None;
                self.emu_msg = String::new();
                self.show_emulators = true;
            }
            if ui.button("🕹 Emulators").clicked() {
                self.emu_tab_status = String::new();
                self.show_emu_tab = true;
            }
            if ui.button("📂 Library Folders…").clicked() {
                let cfg = load_cfg();
                self.folder_list = cfg
                    .get("library_dirs")
                    .and_then(|x| x.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                self.folder_msg = String::new();
                self.show_folders = true;
            }
            if ui.button("⚙ Cache & Performance").clicked() {
                self.show_cache = true;
                self.spawn_cache_measure();
            }
            if ui.button("🔄 Rescan Library").clicked() {
                self.spawn_rescan(ctx);
            }
            if in_wall {
                if self.scrape_system.is_some() {
                    if ui.button("⏹ Stop Art Scrape").clicked() {
                        self.scrape_cancel.store(true, Ordering::Relaxed);
                        self.status =
                            "Stopping the art scrape at the next game…"
                                .into();
                    }
                } else if let View::Wall(sys) = self.view.clone() {
                    if !sys.starts_with('⭐')
                        && sys != CONTINUE_TITLE
                        && sys != NEW_TITLE
                        && ui.button("⬇ Scrape Art for System…").clicked()
                    {
                        self.spawn_scrape_system(ctx, sys);
                    }
                }
            }
            ui.menu_button("📚 Library", |ui| {
                if ui.button("Export Configuration…").clicked() {
                    ui.close_menu();
                    if let Some(dest) =
                        save_file_dialog("emufront_config_export.json")
                    {
                        match std::fs::copy(cfg_path(), &dest) {
                            Ok(_) => {
                                self.status =
                                    format!("Configuration exported to {dest}");
                            }
                            Err(e) => {
                                self.status =
                                    format!("Export failed: {e}");
                            }
                        }
                    }
                }
                if ui.button("Import Configuration…").clicked() {
                    ui.close_menu();
                    if let Some(src) = pick_file_dialog() {
                        match std::fs::read_to_string(&src) {
                            Ok(text) => {
                                let valid =
                                    serde_json::from_str::<serde_json::Value>(
                                        &text,
                                    )
                                    .map(|v| {
                                        v.get("emulators").is_some()
                                            && v.get("games").is_some()
                                    })
                                    .unwrap_or(false);
                                if valid {
                                    let _ = std::fs::copy(
                                        cfg_path(),
                                        data_dir().join(
                                            "emufront_config.backup.json",
                                        ),
                                    );
                                    let tmp = data_dir()
                                        .join("emufront_config.json.tmp");
                                    if std::fs::write(&tmp, &text).is_ok()
                                        && std::fs::rename(&tmp, cfg_path())
                                            .is_ok()
                                    {
                                        self.reload_library();
                                        self.status = "Configuration imported (previous one backed up as emufront_config.backup.json).".into();
                                    } else {
                                        self.status =
                                            "Import failed writing the config.".into();
                                    }
                                } else {
                                    self.status = "That file doesn't look like an EmuFront configuration.".into();
                                }
                            }
                            Err(e) => {
                                self.status =
                                    format!("Import failed: {e}");
                            }
                        }
                    }
                }
            });
            if ui.button("ℹ About & Credits").clicked() {
                self.show_about = true;
            }
        });
    }

    fn wall_central(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let title = match &self.view {
            View::Wall(t) => t.clone(),
            _ => String::new(),
        };
        // Letter-range browsing: the header shows a breadcrumb
        // ("Super Nintendo › H–M") and an "All letters" button
        // while a bucket is active. A universal Back button
        // leads the header (the user, Oct 23): one step back --
        // bucket to chooser, everything else to the home grid
        // (see back_target).
        let active_bucket = self.bucket;
        let system_total = self
            .systems
            .iter()
            .find(|(n, _)| n == &title)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        let mut back_to_chooser = false;
        let mut go_back = false;
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("‹ Back").clicked() {
                go_back = true;
            }
            ui.heading(
                egui::RichText::new(&title)
                    .size(24.0)
                    .color(Color32::WHITE),
            );
            if let Some(b) = active_bucket {
                ui.label(
                    egui::RichText::new(format!(
                        "› {}",
                        BUCKET_LABELS[b as usize]
                    ))
                    .size(20.0)
                    .color(TEXT_DIM),
                );
                if ui.button("‹ All letters").clicked() {
                    back_to_chooser = true;
                }
            }
            ui.label(
                egui::RichText::new(format!(
                    "{} games on this shelf",
                    self.wall.len()
                ))
                .color(TEXT_DIM),
            );
            ui.add_space(16.0);
            ui.label("Search:");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.search)
                    .desired_width(260.0)
                    .hint_text("Filter this shelf…"),
            );
            if resp.changed() {
                self.refresh_wall();
            }
        });
        if go_back {
            match back_target(&title, active_bucket, system_total) {
                BackTarget::Chooser => {
                    self.bucket = None;
                    self.refresh_wall();
                }
                BackTarget::Home => {
                    self.bucket = None;
                    self.view = View::Home;
                }
            }
        } else if back_to_chooser {
            self.bucket = None;
            self.refresh_wall();
        }
        ui.add_space(6.0);
        // The chooser replaces the grid only for a large real
        // system with no bucket chosen and an empty search box
        // (never the ⭐ shelf, never PC Ports).
        let show_chooser = !is_shelf_title(&title)
            && self.bucket.is_none()
            && self.search.is_empty()
            && system_total > BUCKET_MIN;
        if show_chooser {
            self.bucket_chooser(ui, ctx, &title);
        } else {
            let width = ui.available_width();
            self.wall_grid(ui, ctx, width);
        }
    }

    /// Letter-range chooser: five tiles (0-9, A–G, H–M, N–T,
    /// U–Z) with per-bucket counts, painted in the home
    /// screen's tile idiom. Clicking a tile enters that range.
    fn bucket_chooser(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        system: &str,
    ) {
        // Membership comes from the derived bundle when
        // it's loaded (the snapshot carries it); the
        // scan is the fallback, and both agree by
        // construction (bucket_counts is bucket_of over
        // the same games).
        let counts = self
            .derived
            .bucket_counts(system)
            .unwrap_or_else(|| bucket_counts(&self.games, system));
        let total: usize = counts.iter().sum();
        ui.label(
            egui::RichText::new(format!(
                "{total} games — pick a letter range"
            ))
            .color(TEXT_DIM),
        );
        ui.add_space(8.0);
        let avail = ui.available_width();
        let tile_w = 216.0;
        let tile_h = 172.0;
        let cols = ((avail / tile_w) as usize).max(1).min(5);
        let pad = ((avail - cols as f32 * tile_w) / 2.0).max(0.0);
        let rows = BUCKET_LABELS.len().div_ceil(cols);
        let mut picked: Option<u8> = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for r in 0..rows {
                ui.horizontal(|ui| {
                    ui.add_space(pad);
                    for c in 0..cols {
                        let i = r * cols + c;
                        if i >= BUCKET_LABELS.len() {
                            break;
                        }
                        let (rect, resp) = ui.allocate_exact_size(
                            Vec2::new(tile_w - 12.0, tile_h - 12.0),
                            Sense::click(),
                        );
                        let p = ui.painter_at(rect);
                        p.rect_filled(rect, 10.0, PANEL);
                        if resp.hovered() {
                            p.rect_stroke(
                                rect,
                                10.0,
                                egui::Stroke::new(2.0_f32, ACCENT),
                            );
                        }
                        p.text(
                            Pos2::new(
                                rect.center().x,
                                rect.min.y + 62.0,
                            ),
                            egui::Align2::CENTER_CENTER,
                            BUCKET_LABELS[i],
                            egui::FontId::proportional(44.0),
                            Color32::WHITE,
                        );
                        p.text(
                            Pos2::new(
                                rect.center().x,
                                rect.max.y - 22.0,
                            ),
                            egui::Align2::CENTER_CENTER,
                            format!("{} games", counts[i]),
                            egui::FontId::proportional(12.5),
                            TEXT_DIM,
                        );
                        if resp.clicked() {
                            picked = Some(i as u8);
                        }
                    }
                });
            }
        });
        if let Some(b) = picked {
            self.bucket = Some(b);
            self.refresh_wall();
        }
        let _ = ctx;
    }



    fn wall_grid(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, width: f32) {
        // v1.02 tile alignment: metrics match the Python Box wall
        // (TILE_W x TILE_H = 168x236, ART_W x ART_H = 148x190).
        let tile_w = 168.0;
        let art_w = 148.0;
        let art_h = 190.0;
        let tile_h = 236.0;
        let cols = ((width / tile_w) as usize).max(1);
        let xoff = ((width - cols as f32 * tile_w) / 2.0).max(0.0);
        let rows = self.wall.len().div_ceil(cols);
        let total_h = rows as f32 * tile_h;
        let mut launch_req: Option<usize> = None;
        egui::ScrollArea::vertical()
            .id_salt("wall")
            .show_viewport(ui, |ui, viewport| {
                // v1.02 scroll fix: register the full virtual height with an
                // explicit allocation. set_min_height(total_h) alone left the
                // ScrollArea measuring content as viewport-height, which
                // pinned the max scroll offset at 0 (the wall never scrolled
                // and no scrollbar rendered). The allocated space makes the
                // content height rows*tile_h, so the offset can range over
                // the whole shelf; tiles below are positioned absolutely and
                // are unaffected by the allocation.
                let _ = ui.allocate_space(Vec2::new(width, total_h));
                let first_row =
                    ((viewport.min.y / tile_h).floor() as usize)
                        .saturating_sub(1);
                let last_row = ((viewport.max.y / tile_h).ceil() as usize
                    + 1)
                .min(rows);
                for r in first_row..last_row {
                    for c in 0..cols {
                        let i = r * cols + c;
                        if i >= self.wall.len() {
                            break;
                        }
                        let gi = self.wall[i];
                        let game = self.games[gi].clone();
                        // Python Box tile layout: no filled card —
                        // the art box sits at tile-local (+10, +8)
                        // on the wall background and the name label
                        // goes BELOW the art, not inside a card.
                        let tile_min = Pos2::new(
                            ui.min_rect().min.x + xoff + c as f32 * tile_w,
                            ui.min_rect().min.y + r as f32 * tile_h,
                        );
                        let rect = Rect::from_min_size(
                            tile_min,
                            Vec2::new(tile_w, tile_h),
                        );
                        let resp = ui.interact(
                            rect,
                            egui::Id::new(("tile", gi)),
                            Sense::click(),
                        );
                        let p = ui.painter_at(rect);
                        let is_sel = self.selected == Some(gi);
                        let art_rect = Rect::from_min_size(
                            Pos2::new(tile_min.x + 10.0, tile_min.y + 8.0),
                            Vec2::new(art_w, art_h),
                        );
                        // Python thumbs are letterboxed onto a PANEL
                        // backing that fills the whole art box.
                        p.rect_filled(art_rect, 0.0, PANEL);
                        let key = art_key(&game.path);
                        if let Some(tex) = self.tex_handle(&key) {
                            // Fit inside the box with aspect preserved
                            // (never stretched, never upscaled) and
                            // centered — the painter equivalent of
                            // Python's thumbnail() + centered paste.
                            let ts = tex.size_vec2();
                            let s = (art_w / ts.x).min(art_h / ts.y).min(1.0);
                            let fit = Rect::from_min_size(
                                Pos2::new(
                                    art_rect.center().x - ts.x * s / 2.0,
                                    art_rect.center().y - ts.y * s / 2.0,
                                ),
                                Vec2::new(ts.x * s, ts.y * s),
                            );
                            p.image(
                                tex.id(),
                                fit,
                                Rect::from_min_max(
                                    Pos2::ZERO,
                                    Pos2::new(1.0, 1.0),
                                ),
                                Color32::WHITE,
                            );
                        } else {
                            self.request_art(ctx, &game);
                            // Python placeholder: accent band across
                            // the top of the box, the game's first
                            // letter, then its name wrapped below.
                            p.rect_filled(
                                Rect::from_min_size(
                                    art_rect.min,
                                    Vec2::new(art_w, 46.0),
                                ),
                                0.0,
                                ACCENT,
                            );
                            let letter: String = game
                                .name
                                .chars()
                                .next()
                                .unwrap_or('?')
                                .to_uppercase()
                                .collect();
                            p.text(
                                Pos2::new(
                                    art_rect.center().x,
                                    art_rect.min.y + 96.0,
                                ),
                                egui::Align2::CENTER_CENTER,
                                letter,
                                egui::FontId::proportional(16.0),
                                Color32::WHITE,
                            );
                            for (li, line) in
                                wrap_chars(&game.name, 16)
                                    .iter()
                                    .take(3)
                                    .enumerate()
                            {
                                p.text(
                                    Pos2::new(
                                        art_rect.center().x,
                                        art_rect.min.y
                                            + 128.0
                                            + li as f32 * 16.0,
                                    ),
                                    egui::Align2::CENTER_CENTER,
                                    line,
                                    egui::FontId::proportional(11.0),
                                    LABEL_COL,
                                );
                            }
                        }
                        // Name label below the art (Python: canvas
                        // text at art bottom + 8, #cfcfe0, wrapped
                        // within ART_W + 8, name cut at 40 chars).
                        let label: String = {
                            let n = &game.name;
                            if n.chars().count() > 40 {
                                let t: String =
                                    n.chars().take(39).collect();
                                format!("{t}…")
                            } else {
                                n.clone()
                            }
                        };
                        let mut job =
                            egui::text::LayoutJob::single_section(
                                label,
                                egui::TextFormat {
                                    font_id:
                                        egui::FontId::proportional(12.0),
                                    color: LABEL_COL,
                                    ..Default::default()
                                },
                            );
                        job.wrap.max_width = art_w + 8.0;
                        job.halign = egui::Align::Center;
                        let galley = ctx.fonts(|f| f.layout_job(job));
                        p.galley(
                            Pos2::new(
                                tile_min.x + tile_w / 2.0
                                    - (art_w + 8.0) / 2.0,
                                art_rect.max.y + 8.0,
                            ),
                            galley,
                            LABEL_COL,
                        );
                        // Selection: Python draws only a 3px accent
                        // outline around art + label (no filled card).
                        if is_sel {
                            p.rect_stroke(
                                Rect::from_min_size(
                                    Pos2::new(
                                        tile_min.x + 4.0,
                                        tile_min.y + 2.0,
                                    ),
                                    Vec2::new(art_w + 12.0, art_h + 44.0),
                                ),
                                0.0,
                                egui::Stroke::new(3.0_f32, ACCENT),
                            );
                        }
                        if resp.clicked() {
                            self.selected = Some(gi);
                        }
                        if resp.double_clicked() {
                            self.selected = Some(gi);
                            launch_req = Some(gi);
                        }
                    }
                }
                // Prefetch (design §2.2; tuned v1.06):
                // queue covers just outside the viewport
                // at low priority, so scrolling arrives at
                // covers already decoded. The window is
                // biased toward the scroll direction and
                // the submissions are paced per frame —
                // prefetch_ranges holds the math. Prefetch
                // is the first work the generation drop
                // throws away when the wall changes.
                let dir = scroll_dir(self.wall_top_prev, first_row);
                self.wall_top_prev = Some(first_row);
                let (primary, secondary) = prefetch_ranges(
                    first_row,
                    last_row,
                    cols,
                    self.wall.len(),
                    dir,
                    self.profile.prefetch,
                );
                let frame_cap = (cols * PREFETCH_QUEUE_PER_FRAME_COLS).max(1);
                let mut queued = 0usize;
                for j in primary.chain(secondary) {
                    if queued >= frame_cap {
                        break;
                    }
                    let g = self.games[self.wall[j]].clone();
                    if self.request_art_low(ctx, &g) {
                        queued += 1;
                    }
                }
            });
        if let Some(gi) = launch_req {
            if gi < self.games.len() {
                let game = self.games[gi].clone();
                match launch_started(&game, &self.profiles) {
                    Ok((m, child)) => {
                        self.status = m;
                        self.begin_play(&game, child);
                    }
                    Err(m) => self.status = m,
                }
            }
        }
    }

    fn panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let Some(gi) = self.selected else {
            ui.label(
                egui::RichText::new("Pick a game.").color(TEXT_DIM),
            );
            return;
        };
        if gi >= self.games.len() {
            return;
        }
        let game = self.games[gi].clone();
        if self.panel_path != game.path {
            self.panel_path = game.path.clone();
            // The in-session picker starts from the game's true
            // hand-lock only; a stale baked `emulator` value is
            // not a pick, so the panel starts on Auto for it.
            self.launch_override = if game.emulator_locked {
                game.emulator.clone()
            } else {
                String::new()
            };
            // Music: a new game means a new soundtrack (design
            // sec. 5) -- stop whatever was playing and rebuild
            // the player state for this game. No autoplay, ever.
            self.music_stop();
            self.music = Some(build_music_state(&game, &self.settings));
        }
        egui::ScrollArea::vertical()
            .id_salt("panel")
            .show(ui, |ui| {
                ui.add_space(6.0);
                let key = art_key(&game.path);
                if let Some(tex) = self.tex_handle(&key) {
                    let id = tex.id();
                    ui.image((id, Vec2::new(236.0, 300.0)));
                } else {
                    self.request_art(ctx, &game);
                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::new(236.0, 300.0),
                        Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 8.0, PANEL);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "no art yet",
                        egui::FontId::proportional(13.0),
                        TEXT_DIM,
                    );
                }
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(&game.name)
                        .size(16.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    egui::RichText::new(&game.system).color(TEXT_DIM),
                );
                if !game.patch.is_empty() {
                    ui.label(
                        egui::RichText::new(format!(
                            "Built with patch: {}",
                            game.patch
                        ))
                        .size(11.5)
                        .color(TEXT_DIM),
                    );
                }
                if self.has_msu(&game) {
                    ui.label(
                        egui::RichText::new(format!(
                            "🎵 {} enhanced audio",
                            msu_term(&game)
                        ))
                        .color(GOLD),
                    );
                }
                let eff = self.effective_profile(&game);
                let port = is_port(&game);
                if port {
                    ui.label(
                        egui::RichText::new(
                            "Native PC port — launches directly",
                        )
                        .color(TEXT_DIM),
                    );
                } else if let Some(p) = &eff {
                    ui.label(
                        egui::RichText::new(engine_label_p(&game, p))
                            .color(TEXT_DIM),
                    );
                    if let Some((stars, note)) = compat_for(p) {
                        let star_s: String = (0..5)
                            .map(|i| if i < stars { '★' } else { '☆' })
                            .collect();
                        ui.label(
                            egui::RichText::new(format!(
                                "Compatibility: {star_s}"
                            ))
                            .color(Color32::from_rgb(0x9e, 0xd9, 0x57)),
                        );
                        ui.label(
                            egui::RichText::new(note)
                                .size(11.5)
                                .color(TEXT_DIM),
                        );
                    } else {
                        // Honest "no data": never show another
                        // emulator's rating for this pick.
                        ui.label(
                            egui::RichText::new(
                                "Compatibility: no data for this emulator",
                            )
                            .size(11.5)
                            .color(TEXT_DIM),
                        );
                    }
                } else {
                    ui.label(
                        egui::RichText::new("No emulator for this system yet")
                            .color(GOLD),
                    );
                }
                if let Some(r) = self.data.play.get(&game.path) {
                    // v1.07: this game's play record from
                    // the sidecar. Nothing shows when the
                    // game was never launched from this app
                    // (Python Box launches aren't recorded
                    // — the sidecar only knows Rust's).
                    let sess = if r.sessions == 1 {
                        "1 session".to_string()
                    } else {
                        format!("{} sessions", r.sessions)
                    };
                    ui.label(
                        egui::RichText::new(format!(
                            "Last played {} · {} total · {}",
                            fmt_day(r.last),
                            fmt_playtime(r.secs),
                            sess
                        ))
                        .size(11.5)
                        .color(TEXT_DIM),
                    );
                }
                ui.add_space(6.0);
                if !port {
                ui.label(
                    egui::RichText::new("Launch with:").color(TEXT_DIM),
                );
                let mut new_override: Option<String> = None;
                let cur_label = if self.launch_override.is_empty() {
                    "Auto (RetroArch default)".to_string()
                } else {
                    self.launch_override.clone()
                };
                egui::ComboBox::from_id_salt("launch_with")
                    .selected_text(cur_label)
                    .width(250.0)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(
                                &mut self.launch_override,
                                String::new(),
                                "Auto (RetroArch default)",
                            )
                            .changed()
                        {
                            new_override = Some(String::new());
                        }
                        for p in self
                            .profiles
                            .iter()
                            .filter(|p| p.system == game.system)
                        {
                            if ui
                                .selectable_value(
                                    &mut self.launch_override,
                                    p.name.clone(),
                                    &p.name,
                                )
                                .changed()
                            {
                                new_override =
                                    Some(self.launch_override.clone());
                            }
                        }
                    });
                if let Some(v) = new_override {
                    self.cfg_enqueue(
                        CfgMutation::SetGameEmulator {
                            path: game.path.clone(),
                            name: v.clone(),
                        },
                        CfgTag::Quiet,
                    );
                    if let Some(g) = self
                        .games
                        .iter_mut()
                        .find(|g| g.path == game.path)
                    {
                        g.emulator = v.clone();
                        g.emulator_locked = !v.is_empty();
                    }
                    // Repaint now so the engine label + compat
                    // stars re-key to the new pick immediately.
                    ctx.request_repaint();
                }
                }
                ui.add_space(8.0);
                if ui
                    .add_sized(
                        [252.0, 46.0],
                        egui::Button::new(
                            egui::RichText::new("▶  LAUNCH")
                                .size(19.0)
                                .color(Color32::WHITE),
                        )
                        .fill(ACCENT),
                    )
                    .clicked()
                {
                    // Music stops before any launch (design
                    // sec. 5): the game gets the audio device.
                    self.music_stop();
                    if port {
                        match launch_direct_started(&game) {
                            Ok((m, child)) => {
                                self.status = m;
                                self.begin_play(&game, child);
                            }
                            Err(m) => self.status = m,
                        }
                    } else {
                        match &eff {
                            Some(p) => match launch_with_started(&game, p) {
                                Ok((m, child)) => {
                                    self.status = m;
                                    self.begin_play(&game, child);
                                }
                                Err(m) => self.status = m,
                            },
                            None => {
                                self.status =
                                    "No emulator profile for this system.".into()
                            }
                        }
                    }
                }
                ui.add_space(10.0);
                if !port {
                    self.music_section(ui, ctx, &game);
                }
                if !port {
                if ui
                    .add_sized(
                        [252.0, 34.0],
                        egui::Button::new("🔍 Hacks & patches…"),
                    )
                    .clicked()
                {
                    self.open_hacks(ctx, game.clone());
                }
                if ui
                    .add_sized(
                        [252.0, 34.0],
                        egui::Button::new(format!(
                            "🎵 Find {} version…",
                            msu_term(&game)
                        )),
                    )
                    .clicked()
                {
                    self.open_msu_find(ctx, game.clone());
                }
                if ui
                    .add_sized(
                        [252.0, 34.0],
                        egui::Button::new("⬇ Scrape box art"),
                    )
                    .clicked()
                {
                    self.spawn_scrape_game(ctx, game.clone());
                }
                if ui
                    .add_sized(
                        [252.0, 34.0],
                        egui::Button::new("🎮 Cheats…"),
                    )
                    .clicked()
                {
                    self.cheats = Some(CheatState {
                        game_path: game.path.clone(),
                        name: game.name.clone(),
                        text: game.cheats_text.clone(),
                        status: String::new(),
                    });
                }
                if ui
                    .add_sized(
                        [252.0, 34.0],
                        egui::Button::new("🧩 Texture packs…"),
                    )
                    .clicked()
                {
                    self.textures_w =
                        Some(texture_state_for(&game, &self.profiles));
                }
                } else {
                    ui.label(
                        egui::RichText::new(
                            "ROM tools (hacks, cheats, texture packs) don't apply to PC ports.",
                        )
                        .size(11.5)
                        .color(TEXT_DIM),
                    );
                    ui.add_space(4.0);
                }
                if ui
                    .add_sized(
                        [252.0, 34.0],
                        egui::Button::new("ℹ Game info…"),
                    )
                    .clicked()
                {
                    self.info = Some(InfoState {
                        name: game.name.clone(),
                        info: None,
                        loading: true,
                    });
                    let tx = self.work_tx.clone();
                    let ctx2 = ctx.clone();
                    let g2 = game.clone();
                    std::thread::spawn(move || {
                        let _ = tx
                            .send(WorkMsg::Info(fetch_game_info(&g2)));
                        ctx2.request_repaint();
                    });
                }
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(&game.path)
                        .size(11.0)
                        .color(TEXT_DIM),
                );
            });
    }

    // ---------------- music player (panel) ---------------------

    /// Stop music playback (game switch, LAUNCH, Stop button).
    fn music_stop(&mut self) {
        if let Some(p) = &self.music_player {
            p.send(MusicCmd::Stop);
        }
        if let Some(st) = &mut self.music {
            st.playing = false;
            st.paused = false;
            st.shared = None;
        }
    }

    fn music_play(&mut self, ctx: &egui::Context, idx: usize) {
        if self.music_player.is_none() {
            self.music_player = Some(MusicPlayer::spawn(
                self.work_tx.clone(),
                ctx,
                self.settings.music_volume,
            ));
        }
        let Some(st) = &mut self.music else {
            return;
        };
        let Some(track) = st.tracks.get(idx).cloned() else {
            return;
        };
        match build_music_source(st, &track) {
            Ok((src, shared)) => {
                if let Some(p) = &self.music_player {
                    p.send(MusicCmd::Play(src));
                }
                st.index = Some(idx);
                st.playing = true;
                st.paused = false;
                st.shared = Some(shared);
                self.status = format!("🎵 Playing: {}", track.title);
            }
            Err(msg) => {
                self.status = format!("🎵 {msg}");
            }
        }
    }

    fn music_toggle_pause(&mut self) {
        let Some(st) = &mut self.music else {
            return;
        };
        if !st.playing {
            return;
        }
        if let Some(p) = &self.music_player {
            if st.paused {
                p.send(MusicCmd::Resume);
            } else {
                p.send(MusicCmd::Pause);
            }
        }
        st.paused = !st.paused;
    }

    fn music_step(&mut self, ctx: &egui::Context, delta: i32) {
        let next = {
            let Some(st) = &self.music else {
                return;
            };
            let n = st.tracks.len();
            if n == 0 {
                return;
            }
            let cur = st.index.unwrap_or(0) as i32;
            (cur + delta).rem_euclid(n as i32) as usize
        };
        self.music_play(ctx, next);
    }

    /// Track-end handling, checked once per panel render: a
    /// finished track auto-advances; the last track stops. The
    /// lounge loop loops inside its sequencer and never ends.
    fn music_tick(&mut self, ctx: &egui::Context) {
        let (ended, next_idx) = {
            let Some(st) = &self.music else {
                return;
            };
            let ended = st.playing
                && !st.paused
                && st
                    .shared
                    .as_ref()
                    .map(|s| s.ended.load(Ordering::SeqCst))
                    .unwrap_or(false);
            let next = st
                .index
                .and_then(|i| if i + 1 < st.tracks.len() { Some(i + 1) } else { None });
            (ended, next)
        };
        if !ended {
            return;
        }
        match next_idx {
            Some(i) => self.music_play(ctx, i),
            None => {
                if let Some(st) = &mut self.music {
                    st.playing = false;
                    st.paused = false;
                    st.shared = None;
                }
                self.status = "🎵 Soundtrack finished.".into();
            }
        }
    }

    /// "Add soundtrack...": file a picked archive / music file /
    /// folder of files into this game's Music folder on a
    /// worker thread; the panel rescans when the MusicImported
    /// message lands. The dialog itself runs on the UI thread,
    /// like the app's other pickers.
    fn music_import(&mut self, ctx: &egui::Context, game: &Game, from_folder: bool) {
        let picked = if from_folder {
            pick_folder_dialog()
        } else {
            pick_music_file_dialog()
        };
        let game = game.clone();
        let dest = music_dir_for(&game);
        let tx = self.work_tx.clone();
        let ctx2 = ctx.clone();
        self.status = "🎵 Adding soundtrack…".into();
        std::thread::spawn(move || {
            let msg = music_import_worker(picked, &dest);
            let _ = tx.send(WorkMsg::Status(msg));
            let _ = tx.send(WorkMsg::MusicImported(game.path.clone()));
            ctx2.request_repaint();
        });
    }

    /// The "🎵 Soundtrack" panel section (design sec. 5):
    /// header + track list + transport + progress + volume,
    /// inserted right after the LAUNCH button. Renders for
    /// every non-port game -- with the lounge fallback when the
    /// game has no set (design sec. 6).
    fn music_section(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, game: &Game) {
        // Keep the state keyed to the game on screen (the
        // panel-path hook normally rebuilds it; this also
        // covers the first render and post-import rescans).
        if self
            .music
            .as_ref()
            .map(|m| m.game_path != game.path)
            .unwrap_or(true)
        {
            self.music = Some(build_music_state(game, &self.settings));
        }
        self.music_tick(ctx);
        // Display state, copied out so the actions below can
        // borrow self freely (house pattern).
        let (lounge, sf_missing, has_midi, playing, paused, cur, summary) = {
            let Some(st) = &self.music else {
                return;
            };
            let mut kinds: Vec<&str> = Vec::new();
            for t in &st.tracks {
                let l = t.kind.label();
                if !kinds.contains(&l) {
                    kinds.push(l);
                }
            }
            let summary = if st.lounge {
                "no soundtrack set".to_string()
            } else {
                format!("{} tracks · {}", st.tracks.len(), kinds.join("/"))
            };
            (
                st.lounge,
                st.sf_path.is_none(),
                st.tracks.iter().any(|t| t.kind == MusicKind::Midi),
                st.playing,
                st.paused,
                st.index,
                summary,
            )
        };
        ui.add_space(2.0);
        ui.separator();
        ui.label(
            egui::RichText::new(format!("🎵 Soundtrack — {summary}"))
                .strong()
                .color(Color32::WHITE),
        );
        if lounge {
            ui.label(
                egui::RichText::new(
                    "No soundtrack found for this one — enjoy the ride.",
                )
                .size(11.5)
                .color(TEXT_DIM),
            );
            if sf_missing {
                ui.label(
                    egui::RichText::new(
                        "Lounge band couldn't make it (no SoundFont).",
                    )
                    .size(11.5)
                    .color(GOLD),
                );
            }
        } else if has_midi && sf_missing {
            ui.label(
                egui::RichText::new(
                    "MIDI tracks need a SoundFont — none found in Music\\_soundfonts yet.",
                )
                .size(11.5)
                .color(GOLD),
            );
        }
        // ---- track list (click = play that track)
        let mut play_idx: Option<usize> = None;
        {
            let st = self.music.as_ref().unwrap();
            egui::ScrollArea::vertical()
                .id_salt("musictracks")
                .max_height(150.0)
                .show(ui, |ui| {
                    for (i, t) in st.tracks.iter().enumerate() {
                        let is_cur = cur == Some(i);
                        let mark = if is_cur && (playing || paused) {
                            "▶ "
                        } else {
                            ""
                        };
                        let label = format!(
                            "{mark}{:02}. {} — {}",
                            i + 1,
                            t.title,
                            fmt_mmss(t.play_secs)
                        );
                        let mut rt = egui::RichText::new(label).size(12.5);
                        if is_cur {
                            rt = rt.color(ACCENT);
                        }
                        if ui.selectable_label(is_cur, rt).clicked() {
                            play_idx = Some(i);
                        }
                    }
                });
        }
        if let Some(i) = play_idx {
            self.music_play(ctx, i);
        }
        // ---- transport
        let mut do_prev = false;
        let mut do_toggle = false;
        let mut do_next = false;
        let mut do_stop = false;
        ui.horizontal(|ui| {
            if ui
                .add_sized([46.0, 30.0], egui::Button::new("◀◀"))
                .clicked()
            {
                do_prev = true;
            }
            let play_label = if playing && !paused { "‖" } else { "▶" };
            if ui
                .add_sized([46.0, 30.0], egui::Button::new(play_label))
                .clicked()
            {
                do_toggle = true;
            }
            if ui
                .add_sized([46.0, 30.0], egui::Button::new("▶▶"))
                .clicked()
            {
                do_next = true;
            }
            if ui
                .add_sized([46.0, 30.0], egui::Button::new("■"))
                .clicked()
            {
                do_stop = true;
            }
        });
        if do_prev {
            self.music_step(ctx, -1);
        }
        if do_next {
            self.music_step(ctx, 1);
        }
        if do_stop {
            self.music_stop();
        }
        if do_toggle {
            if playing {
                self.music_toggle_pause();
            } else {
                self.music_play(ctx, cur.unwrap_or(0));
            }
        }
        // ---- progress
        let (pos_s, tot_s) = {
            let st = self.music.as_ref().unwrap();
            match &st.shared {
                Some(sh) => (
                    sh.pos_frames.load(Ordering::SeqCst) as f64 / MUSIC_RATE as f64,
                    sh.total_frames.load(Ordering::SeqCst) as f64
                        / MUSIC_RATE as f64,
                ),
                None => (
                    0.0,
                    st.tracks
                        .get(cur.unwrap_or(0))
                        .map(|t| t.total_secs())
                        .unwrap_or(0.0),
                ),
            }
        };
        let frac = if tot_s > 0.0 {
            (pos_s / tot_s).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };
        ui.add(egui::ProgressBar::new(frac));
        ui.label(
            egui::RichText::new(format!(
                "{} / {}",
                fmt_mmss(pos_s),
                fmt_mmss(tot_s)
            ))
            .size(11.0)
            .color(TEXT_DIM),
        );
        if playing && !paused {
            // ~4 Hz progress refresh while playing (design
            // sec. 5: never per-frame)
            ctx.request_repaint_after(Duration::from_millis(250));
        }
        // ---- volume (persisted in Settings)
        let mut vol = self.settings.music_volume;
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🔊").color(TEXT_DIM));
            if ui
                .add(egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false))
                .changed()
            {
                self.settings.music_volume = vol;
                save_settings(&self.settings);
                if let Some(p) = &self.music_player {
                    p.send(MusicCmd::SetVolume(vol));
                }
            }
        });
        // ---- acquisition
        let mut add_file = false;
        let mut add_folder = false;
        if ui
            .add_sized([252.0, 30.0], egui::Button::new("🎵 Add soundtrack…"))
            .clicked()
        {
            add_file = true;
        }
        if ui
            .add_sized([252.0, 30.0], egui::Button::new("📂 Add from folder…"))
            .clicked()
        {
            add_folder = true;
        }
        if lounge {
            ui.hyperlink_to("🌐 Find this soundtrack", soundtrack_url(&game.system));
        }
        if add_file {
            self.music_import(ctx, game, false);
        }
        if add_folder {
            self.music_import(ctx, game, true);
        }
        ui.add_space(2.0);
    }

    fn open_hacks(&mut self, ctx: &egui::Context, game: Game) {
        self.hacks = Some(HackState {
            game: game.clone(),
            entries: None,
            status: "Searching RomHack Plaza…".into(),
            building: false,
            msu: false,
        });
        let tx = self.work_tx.clone();
        let ctx2 = ctx.clone();
        std::thread::spawn(move || {
            let entries = search_plaza(&game);
            let _ = tx.send(WorkMsg::Hacks(entries));
            ctx2.request_repaint();
        });
    }

    /// The "🎵 Find MSU-1 / MD+ version…" button: same window
    /// as Hacks & patches, but the search is the CD-audio hunt
    /// (search_msu1) and builds run the MSU chain.
    fn open_msu_find(&mut self, ctx: &egui::Context, game: Game) {
        self.hacks = Some(HackState {
            game: game.clone(),
            entries: None,
            status: format!(
                "⏳ Searching for a {} version of this game…",
                msu_term(&game)
            ),
            building: false,
            msu: true,
        });
        let tx = self.work_tx.clone();
        let ctx2 = ctx.clone();
        std::thread::spawn(move || {
            let entries = search_msu1(&game);
            let _ = tx.send(WorkMsg::Hacks(entries));
            ctx2.request_repaint();
        });
    }

    /// Open the "🎵 Enhanced Games" catalog on one
    /// system. The filter and owned-only toggle carry
    /// over when switching systems; entries and rows
    /// reload — the two catalogs never share a list.
    fn open_enhanced_catalog(&mut self, ctx: &egui::Context, system: &str) {
        let (filter, owned_only) = match &self.enhanced_catalog {
            Some(st) => (st.filter.clone(), st.owned_only),
            None => (String::new(), false),
        };
        self.enhanced_catalog = Some(EnhCatalogState {
            system: system.to_string(),
            entries: None,
            rows: None,
            status: format!("Loading the {system} enhanced catalog…"),
            meta: String::new(),
            refreshing: false,
            filter,
            owned_only,
            building: false,
        });
        self.spawn_catalog_load(ctx, system, false);
    }

    /// Load one system's catalog on a worker thread and
    /// match it against that system's games only. The
    /// v1.06 flow (Slice 3, catalog_plan): a saved copy
    /// renders immediately — within its TTL that's the
    /// whole load; past it, the copy renders AND a live
    /// fetch runs behind it, swapping fresh rows in when
    /// it lands, so the window never waits on the
    /// network. `force` (the Refresh button) skips the
    /// saved copy's turn and goes straight live; if the
    /// fetch then comes back empty, the saved copy (or
    /// this session's earlier load) is shown rather than
    /// an empty window.
    fn spawn_catalog_load(&mut self, ctx: &egui::Context, system: &str, force: bool) {
        let tx = self.work_tx.clone();
        let ctx2 = ctx.clone();
        let sys = system.to_string();
        let games: Vec<Game> = self
            .games
            .iter()
            .filter(|g| g.system == sys)
            .cloned()
            .collect();
        std::thread::spawn(move || {
            let send = |entries: Vec<CatalogEntry>, note: String| {
                let manuals = manual_game_refs();
                let rows = build_catalog_rows(
                    &sys, &entries, &games, &manuals, data_dir_str(),
                );
                let _ = tx.send(WorkMsg::Catalog(
                    sys.clone(),
                    entries,
                    rows,
                    Some(note),
                ));
                ctx2.request_repaint();
            };
            let now = now_ts();
            let disk = catalog_disk_state(&sys, now);
            let (immediate, fetch_live) = catalog_plan(&disk, force);
            if let Some(entries) = &immediate {
                session_cache_store(&CATALOG_CACHE, sys.clone(), entries);
                let note = if fetch_live {
                    format!(
                        "Saved copy · updated {} — checking for updates…",
                        disk.age_label(now)
                    )
                } else {
                    format!("Saved copy · updated {}", disk.age_label(now))
                };
                send(entries.clone(), note);
            }
            if !fetch_live {
                return;
            }
            if immediate.is_none() && !force {
                // No saved copy, but this session may
                // already hold the catalog (the cache was
                // cleared mid-run): serve it instead of
                // paying the network twice in one run.
                if let Some(sess) = session_cache_lookup(&CATALOG_CACHE, &sys) {
                    send(
                        sess,
                        "From this session's earlier load — Refresh re-checks the sources."
                            .to_string(),
                    );
                    return;
                }
            }
            let live = catalog_entries_live_cached(&sys);
            if !live.is_empty() {
                send(live, "Live · updated just now".to_string());
                return;
            }
            // The live fetch came back empty — a hiccup
            // reads exactly like "nothing exists", so an
            // empty window is the one wrong answer when a
            // copy exists anywhere.
            if let Some(fb) = disk.entries() {
                let note = if force {
                    format!(
                        "Saved copy · updated {} (refresh didn't come through — showing the saved copy)",
                        disk.age_label(now_ts())
                    )
                } else {
                    format!(
                        "Saved copy · updated {} (couldn't check for updates)",
                        disk.age_label(now_ts())
                    )
                };
                if immediate.is_none() {
                    send(fb.clone(), note);
                } else {
                    // The saved rows are already on screen;
                    // resending them with the new note only
                    // updates the provenance line.
                    send(immediate.unwrap(), note);
                }
                return;
            }
            if let Some(sess) = session_cache_lookup(&CATALOG_CACHE, &sys) {
                send(
                    sess,
                    "From this session's earlier load (the live check didn't come through)."
                        .to_string(),
                );
                return;
            }
            if immediate.is_none() {
                send(
                    vec![],
                    "Couldn't load the catalog — check your connection, then Refresh."
                        .to_string(),
                );
            }
        });
    }

    /// The catalog window's Refresh button: re-ask the
    /// sources for the system on screen, keeping the
    /// current rows up while the fetch runs.
    fn refresh_enhanced_catalog(&mut self, ctx: &egui::Context) {
        let system = match &self.enhanced_catalog {
            Some(st) if !st.refreshing && !st.building => st.system.clone(),
            _ => return,
        };
        if let Some(st) = &mut self.enhanced_catalog {
            st.refreshing = true;
            st.meta = "Checking for updates…".to_string();
        }
        self.spawn_catalog_load(ctx, &system, true);
    }

    fn spawn_scrape_game(&mut self, ctx: &egui::Context, game: Game) {
        self.status = format!("Scraping artwork for {}…", game.name);
        let tx = self.work_tx.clone();
        let ctx2 = ctx.clone();
        std::thread::spawn(move || {
            let mut n = 0;
            if let Some(bytes) = fetch_art_bytes(&game) {
                if save_boxart(&game, &bytes) {
                    n += 1;
                }
            }
            let _ = tx.send(WorkMsg::Scraped(game.path.clone()));
            let _ = tx.send(WorkMsg::Status(format!(
                "Artwork scrape for {}: {} cover saved.",
                game.name, n
            )));
            ctx2.request_repaint();
        });
    }

    fn drain_work(&mut self) {
        while let Ok(m) = self.work_rx.try_recv() {
            match m {
                WorkMsg::Status(t) => {
                    self.status = t.clone();
                    if let Some(st) = &mut self.hacks {
                        st.status = t;
                        st.building = false;
                    }
                }
                WorkMsg::JobDone(t) => {
                    self.status = t;
                    self.busy = false;
                    self.scrape_system = None;
                    // Pick up any misses the finished job recorded.
                    self.art_misses = load_art_misses();
                    self.failed.clear();
                    // A finished job may have staged MSU
                    // content: badge answers are stale now.
                    self.msu_cache.clear();
                }
                WorkMsg::Reload => self.reload_library(),
                WorkMsg::Hacks(entries) => {
                    if let Some(st) = &mut self.hacks {
                        st.status = if entries.is_empty() {
                            if st.msu {
                                // Python's find_msu1 wording,
                                // per system family.
                                if st.game.system == "Sega Genesis" {
                                    "No MD+ version found online for this game — MD+ (also called MSU-MD) is the Genesis form of MSU-1, and only some games have one.".into()
                                } else if st.game.system == "Super Nintendo" {
                                    "No MSU-1 version found online for this game — not every game has one.".into()
                                } else {
                                    "No MSU-1 version found online for this game (MSU-1 is mainly a Super Nintendo feature).".into()
                                }
                            } else {
                                "No hacks found on RomHack Plaza for this game."
                                    .into()
                            }
                        } else if st.msu {
                            format!("Found {} {} result(s) — pick one to download & build; the music files are set up automatically.", entries.len(), msu_term(&st.game))
                        } else {
                            format!("{} result(s) — pick one to download & build.", entries.len())
                        };
                        st.entries = Some(entries);
                    }
                }
                WorkMsg::Catalog(sys, entries, rows, note) => {
                    // One system at a time: a late load for a
                    // system the window has moved away from is
                    // dropped, never merged in.
                    if let Some(st) = &mut self.enhanced_catalog {
                        if st.system == sys {
                            let fresh = st.rows.is_none();
                            st.refreshing = false;
                            let total = rows.len();
                            let owned =
                                rows.iter().filter(|r| r.game.is_some()).count();
                            st.entries = Some(entries);
                            st.rows = Some(rows);
                            // The count line refreshes on a
                            // first load and whenever a load
                            // carries a provenance note (a
                            // background refresh landing);
                            // a note-less rebuild after a
                            // build leaves it alone.
                            if fresh || note.is_some() {
                                st.status = format!(
                                    "{total} enhanced downloads for {sys} — {owned} match games in your library."
                                );
                            }
                            if let Some(n) = note {
                                st.meta = n;
                            }
                        }
                    }
                }
                WorkMsg::CatalogBuilt(t) => {
                    self.status = t.clone();
                    // A finished build may have staged MSU
                    // content: badge answers are stale now.
                    self.msu_cache.clear();
                    if let Some(st) = &mut self.enhanced_catalog {
                        st.status = t;
                        st.building = false;
                    }
                }
                WorkMsg::Info(info) => {
                    if let Some(st) = &mut self.info {
                        st.info = info;
                        st.loading = false;
                    }
                }
                WorkMsg::CheatsFetched(pairs) => {
                    if let Some(st) = &mut self.cheats {
                        let mut added = 0;
                        for (d, c) in &pairs {
                            if !st.text.contains(c.as_str()) {
                                if !st.text.is_empty() && !st.text.ends_with('\n') {
                                    st.text.push('\n');
                                }
                                st.text.push_str(&format!("{d} | {c}\n"));
                                added += 1;
                            }
                        }
                        st.status = format!(
                            "Fetched {} cheat codes from libretro-database.",
                            added
                        );
                    }
                }
                WorkMsg::Scraped(path) => {
                    let key = art_key(&path);
                    self.textures.remove(&key);
                    self.failed.remove(&key);
                    // a manual scrape found art — forget any
                    // recorded miss for this game
                    if self.art_misses.remove(&key).is_some() {
                        save_art_misses(&self.art_misses);
                    }
                }
                WorkMsg::TexDone(msg) => {
                    self.status = msg.clone();
                    if let Some(st) = &mut self.textures_w {
                        // refresh status/folder/counts for the same game
                        let fresh =
                            texture_state_for(&st.game, &self.profiles);
                        st.status = fresh.status;
                        st.folder = fresh.folder;
                        st.kind = fresh.kind;
                        st.key = fresh.key;
                        st.gfx_dir = fresh.gfx_dir;
                        st.dolphin_user = fresh.dolphin_user;
                        st.packs = fresh.packs;
                        st.note = msg;
                        st.busy = false;
                    }
                }
                WorkMsg::MusicStatus(t) => {
                    self.status = t;
                }
                WorkMsg::CacheSizes(cache, art) => {
                    self.cache_size = Some(cache);
                    self.boxart_size = Some(art);
                }
                WorkMsg::MusicImported(path) => {
                    if path == self.panel_path {
                        // The panel is showing this game: rescan
                        // its soundtrack folder so the new set
                        // appears (playback stays stopped).
                        if let Some(g) = self
                            .games
                            .iter()
                            .find(|g| g.path == path)
                            .cloned()
                        {
                            self.music =
                                Some(build_music_state(&g, &self.settings));
                        }
                    }
                }
                WorkMsg::EmuUpdates(map) => {
                    self.emu_checking = false;
                    let n_upd = map
                        .values()
                        .filter(|u| u.status == "update")
                        .count();
                    self.settings.emu_updates = map;
                    self.settings.emu_updates_at = now_epoch();
                    save_settings(&self.settings);
                    self.status = if n_upd == 0 {
                        "Emulator update check complete: everything is up to date."
                            .into()
                    } else {
                        format!(
                            "Emulator update check complete: {n_upd} update(s) available — see Manage Emulators."
                        )
                    };
                }
            }
        }
    }


    fn windows(&mut self, ctx: &egui::Context) {
        // ---- About & Credits
        if self.show_about {
            let mut open = true;
            egui::Window::new("About EmuFront Rust")
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("EmuFront Rust v{APP_VERSION}"));
                    ui.label(CREDITS);
                });
            self.show_about = open;
        }
        // ---- Cache & Performance (design §2.7: the
        // minimal release control set — size line, Clear
        // Cache, cover-memory budget, low-spec toggle)
        if self.show_cache {
            let mut open = true;
            let mut clear_clicked = false;
            let mut measure_clicked = false;
            let mut new_cover: Option<String> = None;
            let mut new_low: Option<bool> = None;
            let mut new_discord: Option<bool> = None;
            let mut new_discord_id: Option<String> = None;
            egui::Window::new("⚙ Cache & Performance")
                .open(&mut open)
                .default_size([540.0, 430.0])
                .show(ctx, |ui| {
                    ui.heading("App cache");
                    match self.cache_size {
                        Some(n) => {
                            ui.label(format!(
                                "{:.1} MB — the startup snapshot, cover renditions and catalog data. All of it is rebuildable.",
                                n as f64 / 1_048_576.0
                            ));
                        }
                        None => {
                            ui.label("Measuring…");
                        }
                    }
                    match self.boxart_size {
                        Some(n) => {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Library box art: {:.1} MB — your collection, shared with EmuFront Box. Clear Cache never touches it.",
                                    n as f64 / 1_048_576.0
                                ))
                                .color(TEXT_DIM),
                            );
                        }
                        None => {
                            ui.label(
                                egui::RichText::new(
                                    "Library box art: measuring…",
                                )
                                .color(TEXT_DIM),
                            );
                        }
                    }
                    ui.label(
                        egui::RichText::new(format!(
                            "Cache lives in {}",
                            cache_dir().to_string_lossy()
                        ))
                        .color(TEXT_DIM),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("🗑 Clear Cache").clicked() {
                            clear_clicked = true;
                        }
                        if ui.button("↻ Refresh sizes").clicked() {
                            measure_clicked = true;
                        }
                    });
                    ui.add_space(8.0);
                    ui.separator();
                    ui.heading("Cover memory");
                    ui.label(
                        "How much RAM decoded covers may use. Lower is kinder to small machines; the covers themselves are never changed.",
                    );
                    ui.horizontal(|ui| {
                        for (val, label) in [
                            ("low", "Low (128 MB)"),
                            ("standard", "Standard (384 MB)"),
                            ("high", "High (1 GB)"),
                        ] {
                            if ui
                                .selectable_label(
                                    self.settings.cover_memory == val,
                                    label,
                                )
                                .clicked()
                            {
                                new_cover = Some(val.to_string());
                            }
                        }
                    });
                    if self.profile.low_spec {
                        ui.label(
                            egui::RichText::new(
                                "Low-spec mode is on: cover memory is held at 48 MB whatever this says (the choice applies when low-spec is off).",
                            )
                            .color(GOLD),
                        );
                    }
                    ui.add_space(8.0);
                    ui.separator();
                    ui.heading("Old / small machines");
                    let mut low = self.settings.low_spec;
                    if ui
                        .checkbox(
                            &mut low,
                            "Low-spec machine mode (48 MB cover memory, smaller renditions, one art worker, no prefetch, paced art, 100% UI scale)",
                        )
                        .changed()
                    {
                        new_low = Some(low);
                    }
                    ui.label(
                        egui::RichText::new(
                            "Budgets, prefetch and pacing change immediately; rendition size, worker count, window size and UI scale follow at the next launch.",
                        )
                        .color(TEXT_DIM),
                    );
                    ui.add_space(8.0);
                    ui.separator();
                    ui.heading("Discord presence");
                    let mut dp = self.settings.discord_presence;
                    if ui
                        .checkbox(
                            &mut dp,
                            "Show the game I'm playing in my Discord status (Now Playing)",
                        )
                        .changed()
                    {
                        new_discord = Some(dp);
                    }
                    ui.label(
                        egui::RichText::new(
                            "Discord application ID — from the Discord developer portal (an identifier, not a password; presence runs under it):",
                        )
                        .color(TEXT_DIM),
                    );
                    let mut id_text = self.discord_id_edit.clone();
                    let id_resp = ui.add(
                        egui::TextEdit::singleline(&mut id_text)
                            .desired_width(340.0)
                            .hint_text("Paste the application ID"),
                    );
                    if id_resp.changed() {
                        self.discord_id_edit = id_text.clone();
                    }
                    if id_resp.lost_focus() {
                        new_discord_id = Some(id_text.trim().to_string());
                    }
                    ui.label(
                        egui::RichText::new(
                            if presence_app_id(&self.settings).is_some() {
                                "Presence is configured: while a game runs, EmuFront tells your Discord app over a local pipe — no account sign-in, no tokens, no network."
                            } else if self.settings.discord_presence {
                                "No application ID yet: presence is off (and costs nothing) until one is pasted in above."
                            } else {
                                "Presence is off."
                            },
                        )
                        .color(TEXT_DIM),
                    );
                    if !self.settings.detected_disk.is_empty()
                        || self.settings.detected_ram_gb.is_some()
                    {
                        let ram = self
                            .settings
                            .detected_ram_gb
                            .map(|r| format!("{r:.0} GB RAM"))
                            .unwrap_or_else(|| "RAM unknown".into());
                        let disk = match self
                            .settings
                            .detected_disk
                            .as_str()
                        {
                            "ssd" => "SSD",
                            "hdd" => "hard drive (HDD)",
                            _ => "drive type unknown",
                        };
                        ui.label(
                            egui::RichText::new(format!(
                                "Detected on first run: {ram} · {disk}."
                            ))
                            .color(TEXT_DIM),
                        );
                    }
                });
            if clear_clicked {
                self.art_pool.reset_renditions();
                let tx = self.work_tx.clone();
                std::thread::spawn(move || {
                    clear_cache_tree();
                    let _ = tx.send(WorkMsg::CacheSizes(
                        dir_size(&cache_dir()),
                        dir_size(&art_dir()),
                    ));
                    let _ = tx.send(WorkMsg::Status(
                        "Cache cleared — the next launch rebuilds it (one full library parse).".into(),
                    ));
                });
            }
            if measure_clicked {
                self.spawn_cache_measure();
            }
            if let Some(v) = new_cover {
                self.settings.cover_memory = v;
                save_settings(&self.settings);
                self.apply_profile_budgets();
            }
            if let Some(v) = new_low {
                self.settings.low_spec = v;
                self.settings.low_spec_user_set = true;
                save_settings(&self.settings);
                self.apply_profile_budgets();
                self.status = if v {
                    "Low-spec mode on.".into()
                } else {
                    "Low-spec mode off.".into()
                };
            }
            if let Some(v) = new_discord {
                self.settings.discord_presence = v;
                save_settings(&self.settings);
                self.reset_presence_worker();
                self.status = if v {
                    "Discord presence on.".into()
                } else {
                    "Discord presence off.".into()
                };
            }
            if let Some(v) = new_discord_id {
                self.settings.discord_app_id = v;
                save_settings(&self.settings);
                self.reset_presence_worker();
                self.status = "Discord application ID saved.".into();
            }
            self.show_cache = open;
        }
        // ---- Manage Emulators
        if self.show_emulators {
            let mut open = true;
            let mut save_clicked = false;
            let mut add_clicked = false;
            let mut del_clicked = false;
            let mut emu_check_clicked = false;
            egui::Window::new("🎮 Manage Emulators")
                .open(&mut open)
                .default_size([860.0, 520.0])
                .show(ctx, |ui| {
                    // -- emulator update checks (the user, Oct 20) --
                    ui.horizontal(|ui| {
                        if self.emu_checking {
                            ui.label(
                                egui::RichText::new(
                                    "🔄 Checking emulator updates…",
                                )
                                .color(TEXT_DIM),
                            );
                        } else if ui
                            .button("🔄 Check emulator updates")
                            .clicked()
                        {
                            emu_check_clicked = true;
                        }
                        if self.settings.emu_updates_at > 0 {
                            let ago = now_epoch()
                                .saturating_sub(self.settings.emu_updates_at);
                            let when = if ago < 3600 {
                                "just now".to_string()
                            } else if ago < 24 * 3600 {
                                format!("{} h ago", ago / 3600)
                            } else {
                                format!("{} d ago", ago / 86400)
                            };
                            ui.label(
                                egui::RichText::new(format!(
                                    "last checked {when}"
                                ))
                                .color(TEXT_DIM),
                            );
                        }
                    });
                    if !self.settings.emu_updates.is_empty() {
                        egui::ScrollArea::vertical()
                            .id_salt("emuupd")
                            .max_height(132.0)
                            .show(ui, |ui| {
                                for (name, u) in
                                    &self.settings.emu_updates
                                {
                                    ui.horizontal(|ui| {
                                        let (icon, col) = match u
                                            .status
                                            .as_str()
                                        {
                                            "update" => {
                                                ("⬆", GOLD)
                                            }
                                            "current" => (
                                                "✔",
                                                Color32::from_rgb(
                                                    0x9e, 0xd9, 0x57,
                                                ),
                                            ),
                                            "manual" => ("🌐", TEXT_DIM),
                                            "info" => ("ℹ", TEXT_DIM),
                                            _ => ("?", TEXT_DIM),
                                        };
                                        let line = match u.status.as_str() {
                                            "update" => format!(
                                                "{name}: installed {} → latest {} — UPDATE AVAILABLE",
                                                u.installed, u.latest
                                            ),
                                            "current" => format!(
                                                "{name}: {} (up to date)",
                                                u.installed
                                            ),
                                            "manual" => format!(
                                                "{name}: {} — no automatic check",
                                                u.installed
                                            ),
                                            "info" => format!(
                                                "{name}: {} — {}",
                                                u.installed, u.latest
                                            ),
                                            _ => format!(
                                                "{name}: installed {} — couldn't compare",
                                                u.installed
                                            ),
                                        };
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{icon} {line}"
                                            ))
                                            .color(col),
                                        );
                                        if u.status == "update"
                                            || u.status == "manual"
                                            || u.status == "unknown"
                                        {
                                            ui.hyperlink_to(
                                                "Release page ↗",
                                                u.url.as_str(),
                                            );
                                        }
                                    });
                                }
                            });
                        ui.separator();
                    }
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_width(300.0);
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} profiles",
                                    self.emu_list.len()
                                ))
                                .color(TEXT_DIM),
                            );
                            let names: Vec<String> = self
                                .emu_list
                                .iter()
                                .map(|v| {
                                    v.get("name")
                                        .and_then(|x| x.as_str())
                                        .unwrap_or("(unnamed)")
                                        .to_string()
                                })
                                .collect();
                            let mut clicked: Option<usize> = None;
                            egui::ScrollArea::vertical()
                                .id_salt("emulist")
                                .max_height(380.0)
                                .show(ui, |ui| {
                                    for (i, name) in names.iter().enumerate() {
                                        if ui
                                            .selectable_label(
                                                self.emu_sel == Some(i),
                                                name,
                                            )
                                            .clicked()
                                            && self.emu_sel != Some(i)
                                        {
                                            clicked = Some(i);
                                        }
                                    }
                                });
                            if let Some(i) = clicked {
                                self.flush_emu_edit();
                                self.emu_sel = Some(i);
                                self.load_emu_edit(i);
                            }
                        });
                        ui.separator();
                        ui.vertical(|ui| {
                            ui.set_width(500.0);
                            if self.emu_sel.is_some() {
                                let e = &mut self.emu_edit;
                                egui::Grid::new("emued")
                                    .num_columns(2)
                                    .show(ui, |ui| {
                                        ui.label("Name:");
                                        ui.text_edit_singleline(&mut e.name);
                                        ui.end_row();
                                        ui.label("Program (exe):");
                                        ui.text_edit_singleline(&mut e.path);
                                        ui.end_row();
                                        ui.label("System:");
                                        ui.text_edit_singleline(
                                            &mut e.system,
                                        );
                                        ui.end_row();
                                        ui.label("RetroArch core:");
                                        ui.text_edit_singleline(&mut e.core);
                                        ui.end_row();
                                        ui.label("Extra args:");
                                        ui.text_edit_singleline(&mut e.args);
                                        ui.end_row();
                                        ui.label("Extensions:");
                                        ui.text_edit_singleline(
                                            &mut e.extensions,
                                        );
                                        ui.end_row();
                                        ui.label("Folder:");
                                        ui.text_edit_singleline(
                                            &mut e.folder,
                                        );
                                        ui.end_row();
                                    });
                                ui.label(
                                    egui::RichText::new(
                                        "RetroArch core: full path to the core .dll (blank for standalone emulators).",
                                    )
                                    .size(11.5)
                                    .color(TEXT_DIM),
                                );
                            } else {
                                ui.label("Pick a profile on the left, or add a new one.");
                            }
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui.button("＋ Add emulator").clicked() {
                                    add_clicked = true;
                                }
                                if ui
                                    .add_enabled(
                                        self.emu_sel.is_some(),
                                        egui::Button::new("🗑 Delete selected"),
                                    )
                                    .clicked()
                                {
                                    del_clicked = true;
                                }
                                if ui.button("💾 Save all").clicked() {
                                    save_clicked = true;
                                }
                            });
                            if !self.emu_msg.is_empty() {
                                ui.label(
                                    egui::RichText::new(&self.emu_msg)
                                        .color(GOLD),
                                );
                            }
                        });
                    });
                });
            if add_clicked {
                self.flush_emu_edit();
                self.emu_list.push(serde_json::json!({
                    "name": "New Emulator", "path": "", "system": "",
                    "core": "", "args": "", "extensions": "", "folder": "",
                }));
                let i = self.emu_list.len() - 1;
                self.emu_sel = Some(i);
                self.load_emu_edit(i);
            }
            if del_clicked {
                if let Some(i) = self.emu_sel.take() {
                    if i < self.emu_list.len() {
                        self.emu_list.remove(i);
                    }
                    self.emu_msg = "Profile deleted (Save to apply).".into();
                }
            }
            if save_clicked {
                self.flush_emu_edit();
                // Queued (§2.5): the reload + messages
                // land in drain_cfg when the write does —
                // the window never freezes on the save.
                self.cfg_enqueue(
                    CfgMutation::ReplaceEmulators {
                        profiles: serde_json::Value::Array(
                            self.emu_list.clone(),
                        ),
                    },
                    CfgTag::EmulatorProfiles,
                );
                self.emu_msg = "Saving…".into();
            }
            if emu_check_clicked {
                self.start_emu_check(ctx);
            }
            self.show_emulators = open;
        }
        // ---- Library Folders
        if self.show_folders {
            let mut open = true;
            let mut remove_idx: Option<usize> = None;
            let mut save_clicked = false;
            let mut rescan_clicked = false;
            egui::Window::new("📂 Library Folders")
                .open(&mut open)
                .default_size([640.0, 420.0])
                .show(ctx, |ui| {
                    ui.label(
                        "EmuFront scans these folders for games (system = first subfolder name).",
                    );
                    ui.add_space(6.0);
                    for (i, f) in self.folder_list.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.button("✕").clicked() {
                                remove_idx = Some(i);
                            }
                            ui.label(f);
                        });
                    }
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_folder);
                        if ui.button("Browse…").clicked() {
                            if let Some(p) = pick_folder_dialog() {
                                self.new_folder = p;
                            }
                        }
                        if ui.button("Add").clicked()
                            && !self.new_folder.trim().is_empty()
                        {
                            self.folder_list
                                .push(self.new_folder.trim().to_string());
                            self.new_folder.clear();
                        }
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("💾 Save folders").clicked() {
                            save_clicked = true;
                        }
                        if ui.button("🔄 Save + Rescan Library").clicked() {
                            save_clicked = true;
                            rescan_clicked = true;
                        }
                    });
                    if !self.folder_msg.is_empty() {
                        ui.label(
                            egui::RichText::new(&self.folder_msg)
                                .color(GOLD),
                        );
                    }
                });
            if let Some(i) = remove_idx {
                if i < self.folder_list.len() {
                    self.folder_list.remove(i);
                }
            }
            if save_clicked {
                // Queued (§2.5): drain_cfg reports the
                // save and starts the rescan on success.
                self.cfg_enqueue(
                    CfgMutation::ReplaceLibraryDirs {
                        dirs: serde_json::Value::Array(
                            self.folder_list
                                .iter()
                                .map(|f| {
                                    serde_json::Value::String(f.clone())
                                })
                                .collect(),
                        ),
                    },
                    CfgTag::LibraryDirs {
                        rescan: rescan_clicked,
                    },
                );
                self.folder_msg = "Saving…".into();
            }
            self.show_folders = open;
        }
        // ---- Hacks & patches
        if let Some(mut st) = self.hacks.take() {
            let mut open = true;
            let mut build: Option<HackEntry> = None;
            let title = format!("🔍 Hacks & patches — {}", st.game.name);
            egui::Window::new(title)
                .open(&mut open)
                .default_size([680.0, 460.0])
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new(&st.status).color(GOLD),
                    );
                    ui.add_space(6.0);
                    if let Some(entries) = &st.entries {
                        egui::ScrollArea::vertical()
                            .id_salt("hacks")
                            .show(ui, |ui| {
                                for e in entries {
                                    ui.horizontal(|ui| {
                                        if ui
                                            .add_enabled(
                                                !st.building,
                                                egui::Button::new(
                                                    "⬇ Download & Build",
                                                ),
                                            )
                                            .clicked()
                                        {
                                            build = Some(e.clone());
                                        }
                                        ui.label(&e.title);
                                    });
                                }
                            });
                    }
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(
                            "Patches build a NEW file in Hacked ROMs — your original ROM is never touched. Some Plaza entries open in your browser; download the file there and EmuFront picks it up automatically.",
                        )
                        .size(11.5)
                        .color(TEXT_DIM),
                    );
                });
            if let Some(e) = build {
                st.building = true;
                st.status = format!(
                    "Working on '{}' — downloading… (browser entries: click Download on the page that opens)",
                    e.title
                );
                let tx = self.work_tx.clone();
                let ctx2 = ctx.clone();
                let g2 = st.game.clone();
                let msu_mode = st.msu;
                std::thread::spawn(move || {
                    let res = if msu_mode {
                        download_and_build_msu(&g2, &e, false)
                    } else {
                        download_and_build(&g2, &e, false)
                    };
                    match res {
                        Ok(m) => {
                            let _ = tx.send(WorkMsg::Reload);
                            let _ = tx.send(WorkMsg::Status(m));
                        }
                        Err(m) => {
                            let _ = tx.send(WorkMsg::Status(format!("⚠ {m}")));
                        }
                    }
                    ctx2.request_repaint();
                });
            }
            if open {
                self.hacks = Some(st);
            }
        }
        // ---- Enhanced Games catalog. ONE system at a
        // time, by design: the two labeled buttons switch
        // between the Super Nintendo (MSU-1) catalog and
        // the Sega Genesis (MD+) catalog — the lists are
        // fetched, matched and rendered separately and
        // can never mix.
        if let Some(mut st) = self.enhanced_catalog.take() {
            let mut open = true;
            let mut build: Option<(Game, CatalogEntry)> = None;
            let mut launch_g: Option<Game> = None;
            let mut switch: Option<String> = None;
            let mut show_shelf = false;
            let mut refresh = false;
            egui::Window::new("🎵 Enhanced Games")
                .open(&mut open)
                .default_size([780.0, 540.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        for (sys, label) in [
                            ("Super Nintendo", "🕹 Super Nintendo Enhanced (MSU-1)"),
                            ("Sega Genesis", "🕹 Sega Genesis Enhanced (MD+)"),
                        ] {
                            if st.system == sys {
                                let _ = ui.add_enabled(
                                    false,
                                    egui::Button::new(
                                        egui::RichText::new(label)
                                            .strong()
                                            .color(GOLD),
                                    ),
                                );
                            } else if ui.button(label).clicked() {
                                switch = Some(sys.to_string());
                            }
                        }
                    });
                    ui.label(
                        egui::RichText::new(
                            "Two separate catalogs — only the picked system's enhanced games are listed. 📦 packs install as-is; 🧩 patches build a new file in Hacked ROMs. Your original ROMs are never touched.",
                        )
                        .size(11.5)
                        .color(TEXT_DIM),
                    );
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(&st.status).color(GOLD));
                    if !st.meta.is_empty() {
                        ui.label(
                            egui::RichText::new(&st.meta)
                                .size(11.5)
                                .color(TEXT_DIM),
                        );
                    }
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Filter:");
                        ui.text_edit_singleline(&mut st.filter);
                        ui.checkbox(&mut st.owned_only, "Only games I own");
                        // v1.06: the saved copy's escape
                        // hatch — re-ask the sources now
                        // instead of waiting out the TTL.
                        if ui
                            .add_enabled(
                                !st.refreshing
                                    && !st.building
                                    && st.rows.is_some(),
                                egui::Button::new("↻ Refresh catalog"),
                            )
                            .clicked()
                        {
                            refresh = true;
                        }
                        if st.refreshing {
                            ui.label(
                                egui::RichText::new("Updating…")
                                    .color(TEXT_DIM),
                            );
                        }
                    });
                    ui.add_space(4.0);
                    if let Some(rows) = &st.rows {
                        let filt = st.filter.to_lowercase();
                        let visible = |r: &EnhRow| {
                            (!st.owned_only || r.game.is_some())
                                && (filt.is_empty()
                                    || r.entry.title.to_lowercase().contains(&filt))
                        };
                        egui::ScrollArea::vertical()
                            .id_salt("enhcatalog")
                            .show(ui, |ui| {
                                for (patch, header) in [
                                    (false, "📦 FULL PACKS"),
                                    (true, "🧩 PATCHES"),
                                ] {
                                    let section: Vec<&EnhRow> = rows
                                        .iter()
                                        .filter(|r| {
                                            entry_is_patch(&r.entry) == patch
                                                && visible(r)
                                        })
                                        .collect();
                                    if section.is_empty() {
                                        continue;
                                    }
                                    ui.label(
                                        egui::RichText::new(header)
                                            .strong()
                                            .color(GOLD),
                                    );
                                    ui.add_space(2.0);
                                    for r in section {
                                        ui.horizontal(|ui| {
                                            if let Some(inst) = &r.installed {
                                                if ui
                                                    .button("▶ Launch")
                                                    .clicked()
                                                {
                                                    launch_g =
                                                        Some(inst.clone());
                                                }
                                            } else if let Some(g) = &r.game {
                                                if ui
                                                    .add_enabled(
                                                        !st.building,
                                                        egui::Button::new(
                                                            "⬇ Download & Build",
                                                        ),
                                                    )
                                                    .clicked()
                                                {
                                                    build = Some((
                                                        g.clone(),
                                                        r.entry.clone(),
                                                    ));
                                                }
                                            }
                                            ui.label(&r.entry.title);
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{} · {}",
                                                    r.entry.source, r.entry.kind
                                                ))
                                                .color(TEXT_DIM),
                                            );
                                            if let Some(inst) = &r.installed {
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "✔ Installed: {}",
                                                        inst.name
                                                    ))
                                                    .color(GOLD),
                                                );
                                            } else if let Some(g) = &r.game {
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "✓ {}",
                                                        g.name
                                                    ))
                                                    .color(TEXT_DIM),
                                                );
                                            } else {
                                                ui.label(
                                                    egui::RichText::new("—")
                                                        .color(TEXT_DIM),
                                                );
                                            }
                                        });
                                        if r.also_other {
                                            ui.label(
                                                egui::RichText::new(
                                                    "💡 also has a pack/patch in the other section",
                                                )
                                                .size(11.5)
                                                .color(TEXT_DIM),
                                            );
                                        }
                                    }
                                    ui.add_space(6.0);
                                }
                            });
                    }
                    ui.add_space(4.0);
                    if ui.button("⭐ Show installed shelf").clicked() {
                        show_shelf = true;
                    }
                });
            if let Some((game, entry)) = build {
                st.building = true;
                st.status = format!(
                    "Working on '{}' — downloading…",
                    entry.title
                );
                let tx = self.work_tx.clone();
                let ctx2 = ctx.clone();
                let sys = st.system.clone();
                let entries = st.entries.clone().unwrap_or_default();
                std::thread::spawn(move || {
                    let res = match catalog_build_entry(&entry) {
                        Ok(he) => download_and_build_msu(&game, &he, false),
                        Err(m) => Err(m),
                    };
                    match res {
                        Ok(m) => {
                            let _ = tx.send(WorkMsg::Reload);
                            let _ = tx.send(WorkMsg::CatalogBuilt(m));
                            let (games, _, _) = load_library();
                            let manuals = manual_game_refs();
                            let rows = build_catalog_rows(
                                &sys, &entries, &games, &manuals, data_dir_str(),
                            );
                            let _ = tx.send(WorkMsg::Catalog(
                                sys, entries, rows, None,
                            ));
                        }
                        Err(m) => {
                            let _ = tx
                                .send(WorkMsg::CatalogBuilt(format!("⚠ {m}")));
                        }
                    }
                    ctx2.request_repaint();
                });
            }
            if let Some(game) = launch_g {
                // The effective pick, same as the detail
                // panel: MSU-1 builds route to bsnes, MD+
                // builds to Genesis Plus GX.
                match launch_started(&game, &self.profiles) {
                    Ok((m, child)) => {
                        st.status = m.clone();
                        self.status = m;
                        self.begin_play(&game, child);
                    }
                    Err(m) => {
                        st.status = m.clone();
                        self.status = m;
                    }
                }
            }
            if show_shelf {
                self.open_enhanced();
            } else if open {
                self.enhanced_catalog = Some(st);
            }
            if refresh {
                self.refresh_enhanced_catalog(ctx);
            }
            if let Some(sys) = switch {
                // st was restored above when the window is
                // open, so the opener picks the filter and
                // owned-only toggle up from it, then starts
                // the other system's own load.
                self.open_enhanced_catalog(ctx, &sys);
            }
        }
        // ---- Emulators tab: standalone emulator starts.
        // Each entry starts the emulator PROGRAM itself (no
        // game loaded — except AmigaVision, whose entry
        // starts Amiberry with the AmigaVision config,
        // which is the point of that entry): the path is
        // the user's saved pick when he made one, else the
        // standalone profile's; when neither yields a file
        // that exists, the entry offers "Select
        // executable…" instead of a dead Run button
        // (the user's fallback rule, Oct 24).
        if self.show_emu_tab {
            let mut open = true;
            let mut run: Option<(String, String, Vec<String>)> = None;
            let mut select_for: Option<&EmuTabSpec> = None;
            let exists = |p: &str| Path::new(p).is_file();
            egui::Window::new("🕹 Emulators")
                .open(&mut open)
                .default_size([560.0, 360.0])
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new(
                            "Start an emulator on its own — no game loaded. Paths come from your emulator profiles; if one can't be found, pick its .exe yourself and EmuFront will remember it.",
                        )
                        .size(11.5)
                        .color(TEXT_DIM),
                    );
                    ui.add_space(6.0);
                    for spec in EMU_TAB_SPECS.iter() {
                        let saved = self
                            .settings
                            .emu_overrides
                            .get(spec.key)
                            .map(|s| s.as_str());
                        let (state, row_args) = resolve_emu_tab_row(
                            spec,
                            &self.profiles,
                            &self.games,
                            saved,
                            &exists,
                        );
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(spec.label)
                                        .size(17.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                );
                                ui.label(
                                    egui::RichText::new(spec.blurb)
                                        .color(TEXT_DIM),
                                );
                            });
                            match &state {
                                EmuTabState::Ready { path, manual } => {
                                    ui.label(
                                        egui::RichText::new(path.clone())
                                            .size(11.5)
                                            .color(TEXT_DIM),
                                    );
                                    ui.horizontal(|ui| {
                                        if ui
                                            .button(
                                                egui::RichText::new("▶ Run")
                                                    .size(14.0),
                                            )
                                            .clicked()
                                        {
                                            run = Some((
                                                spec.label.to_string(),
                                                path.clone(),
                                                row_args.clone(),
                                            ));
                                        }
                                        if *manual {
                                            ui.label(
                                                egui::RichText::new(
                                                    "(your pick)",
                                                )
                                                .size(11.5)
                                                .color(GOLD),
                                            );
                                            if ui.button("Change…").clicked()
                                            {
                                                select_for = Some(spec);
                                            }
                                        }
                                    });
                                }
                                EmuTabState::Missing { stale } => {
                                    if let Some(s) = stale {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "⚠ Not found: {s} — the file is gone or was moved."
                                            ))
                                            .size(11.5)
                                            .color(GOLD),
                                        );
                                    } else {
                                        ui.label(
                                            egui::RichText::new(
                                                "No standalone profile for this emulator was found.",
                                            )
                                            .size(11.5)
                                            .color(TEXT_DIM),
                                        );
                                    }
                                    if ui
                                        .button("Select executable…")
                                        .clicked()
                                    {
                                        select_for = Some(spec);
                                    }
                                }
                                EmuTabState::Blocked { reason } => {
                                    ui.label(
                                        egui::RichText::new(reason.clone())
                                            .size(11.5)
                                            .color(GOLD),
                                    );
                                }
                            }
                        });
                        ui.add_space(4.0);
                    }
                    if !self.emu_tab_status.is_empty() {
                        ui.label(
                            egui::RichText::new(&self.emu_tab_status)
                                .color(GOLD),
                        );
                    }
                });
            if let Some((label, path, args)) = run {
                let msg = match run_emulator_exe(&label, &path, &args) {
                    Ok(m) | Err(m) => m,
                };
                self.emu_tab_status = msg.clone();
                self.status = msg;
            }
            if let Some(spec) = select_for {
                if let Some(picked) = pick_exe_dialog() {
                    if emu_pick_ok(&picked, &exists) {
                        self.settings
                            .emu_overrides
                            .insert(spec.key.to_string(), picked.clone());
                        save_settings(&self.settings);
                        let msg = format!(
                            "✔ {} will now start from your pick: {picked}",
                            spec.label
                        );
                        self.emu_tab_status = msg.clone();
                        self.status = msg;
                    } else {
                        self.emu_tab_status = format!(
                            "⚠ That file isn't a runnable .exe — {} still has no executable.",
                            spec.label
                        );
                    }
                }
            }
            self.show_emu_tab = open;
        }
        // ---- Cheats
        if let Some(mut st) = self.cheats.take() {
            let mut open = true;
            let mut save_clicked = false;
            let mut fetch_clicked = false;
            let title = format!("🎮 Cheats — {}", st.name);
            egui::Window::new(title)
                .open(&mut open)
                .default_size([560.0, 480.0])
                .show(ctx, |ui| {
                    ui.label(
                        "One code per line. Optional description:  Description | CODE",
                    );
                    ui.add(
                        egui::TextEdit::multiline(&mut st.text)
                            .desired_rows(14)
                            .desired_width(f32::INFINITY),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("⬇ Fetch cheats online").clicked() {
                            fetch_clicked = true;
                        }
                        if ui.button("💾 Save cheats").clicked() {
                            save_clicked = true;
                        }
                    });
                    if !st.status.is_empty() {
                        ui.label(
                            egui::RichText::new(&st.status).color(GOLD),
                        );
                    }
                    ui.label(
                        egui::RichText::new(
                            "Saved cheats are written to a RetroArch .cht file and applied automatically on RetroArch launches.",
                        )
                        .size(11.5)
                        .color(TEXT_DIM),
                    );
                });
            if fetch_clicked {
                st.status = "Fetching from libretro-database…".into();
                if let Some(g) = self
                    .games
                    .iter()
                    .find(|g| g.path == st.game_path)
                    .cloned()
                {
                    let tx = self.work_tx.clone();
                    let ctx2 = ctx.clone();
                    std::thread::spawn(move || {
                        let pairs = fetch_libretro_cheats(&g);
                        let _ = tx.send(WorkMsg::CheatsFetched(pairs));
                        ctx2.request_repaint();
                    });
                }
            }
            if save_clicked {
                let txt = st.text.clone();
                // Both halves go through the queue, in
                // order — one worker, one file discipline.
                self.cfg_enqueue(
                    CfgMutation::SetGameField {
                        path: st.game_path.clone(),
                        key: "cheats_text".to_string(),
                        val: serde_json::Value::String(txt.clone()),
                    },
                    CfgTag::Quiet,
                );
                let enabled = !parse_cheat_lines(&txt).is_empty();
                self.cfg_enqueue(
                    CfgMutation::SetGameField {
                        path: st.game_path.clone(),
                        key: "cheats_enabled".to_string(),
                        val: serde_json::Value::Bool(enabled),
                    },
                    CfgTag::Quiet,
                );
                if let Some(g) = self
                    .games
                    .iter_mut()
                    .find(|g| g.path == st.game_path)
                {
                    g.cheats_text = txt;
                    g.cheats_enabled = enabled;
                    let gc = g.clone();
                    let _ = write_cht(&gc);
                }
                st.status = "Cheats saved.".into();
                self.status = "Cheats saved.".into();
            }
            if open {
                self.cheats = Some(st);
            }
        }
        // ---- Game info
        if let Some(st) = self.info.clone() {
            let mut open = true;
            let title = format!("ℹ {}", st.name);
            let body = if st.loading {
                "Looking it up on Wikipedia…".to_string()
            } else if let Some(info) = &st.info {
                format!("{}\n\n{}", info.title, info.extract)
            } else {
                "No Wikipedia description found for this game.".to_string()
            };
            egui::Window::new(title)
                .open(&mut open)
                .default_size([560.0, 400.0])
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("info")
                        .show(ui, |ui| {
                            ui.label(&body);
                        });
                    ui.label(
                        egui::RichText::new("Description from Wikipedia (CC BY-SA).")
                            .size(11.0)
                            .color(TEXT_DIM),
                    );
                });
            if !open {
                self.info = None;
            }
        }
        // ---- Texture packs
        if let Some(mut st) = self.textures_w.take() {
            let mut open = true;
            let title = format!("🧩 Texture packs — {}", st.name);
            let mut open_folder = false;
            let mut open_search = false;
            let mut open_page: Option<String> = None;
            let mut install: Option<String> = None;
            egui::Window::new(title)
                .open(&mut open)
                .default_size([620.0, 430.0])
                .show(ctx, |ui| {
                    ui.label(&st.status);
                    if let Some(f) = &st.folder {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(f.to_string_lossy())
                                .color(TEXT_DIM),
                        );
                        ui.add_space(6.0);
                        if ui.button("📂 Open pack folder").clicked() {
                            open_folder = true;
                        }
                    }
                    if !st.note.is_empty() {
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(&st.note).color(GOLD),
                        );
                    }
                    if !st.packs.is_empty() {
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(
                                "Verified packs for this game:",
                            )
                            .color(GOLD),
                        );
                        for p in &st.packs {
                            ui.horizontal(|ui| {
                                if ui
                                    .add_enabled(
                                        !st.busy,
                                        egui::Button::new(
                                            "⬇ Download & Install",
                                        ),
                                    )
                                    .clicked()
                                {
                                    install = Some(p.url.to_string());
                                }
                                ui.label(format!(
                                    "{}  ({})",
                                    p.title, p.size
                                ));
                                if ui
                                    .small_button("🌐 Pack page")
                                    .clicked()
                                {
                                    open_page = Some(p.page.to_string());
                                }
                            });
                        }
                    }
                    if st.folder.is_some() {
                        ui.add_space(10.0);
                        ui.label("Or install any pack from a direct link:");
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut st.url_text,
                                )
                                .desired_width(380.0)
                                .hint_text("https://…/pack.zip"),
                            );
                            if ui
                                .add_enabled(
                                    !st.busy
                                        && st.url_text
                                            .trim()
                                            .starts_with("http"),
                                    egui::Button::new("⬇ Install from URL"),
                                )
                                .clicked()
                            {
                                install =
                                    Some(st.url_text.trim().to_string());
                            }
                        });
                    }
                    if st.busy {
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(
                                "Working… big packs can take a few minutes to download and unpack.",
                            )
                            .color(GOLD),
                        );
                    }
                    ui.add_space(8.0);
                    if st.search.is_some()
                        && ui.button("🌐 Search packs online").clicked()
                    {
                        open_search = true;
                    }
                    ui.add_space(6.0);
                    let hint = match st.kind {
                        TexKind::Dolphin => "In Dolphin, enable Graphics \u{2192} Advanced \u{2192} Load Custom Textures. Packs install into the game's own folder, named by its game ID.",
                        TexKind::Pcsx2 => "PCSX2 already has texture replacements switched on for this PC. Packs install into the textures folder for the game's serial, in its replacements subfolder.",
                        TexKind::DuckStation => "DuckStation reads packs from the textures folder for the game's serial, in its replacements subfolder. Texture replacements are enabled in DuckStation's settings, so installed packs load automatically when the game starts.",
                        TexKind::GlideN64 => "Installing a pack also points GLideN64 at this PC's texture folders and switches hi-res textures on (its settings still named your old drive). Cache packs (.hts) go to the Plugin GFX cache folder; loose PNG packs go to the folder above.",
                        TexKind::Mesen => "Mesen loads the HD pack automatically when the game starts. The pack folder must match the ROM file name, which is handled for you.",
                        TexKind::Unsupported => "Verified one-click packs cover GameCube/Wii, PlayStation 2, PlayStation, Nintendo 64 and NES. For other systems, use the web search to find packs manually.",
                    };
                    ui.label(
                        egui::RichText::new(hint)
                            .size(11.5)
                            .color(TEXT_DIM),
                    );
                });
            if open_folder {
                if let Some(f) = &st.folder {
                    let _ = std::fs::create_dir_all(f);
                    let _ = Command::new("explorer")
                        .arg(f.to_string_lossy().as_ref())
                        .spawn();
                }
            }
            if open_search {
                if let Some(u) = &st.search {
                    let _ = Command::new("cmd")
                        .args(["/C", "start", "", u])
                        .creation_flags(0x0800_0000)
                        .spawn();
                }
            }
            if let Some(u) = open_page {
                let _ = Command::new("cmd")
                    .args(["/C", "start", "", &u])
                    .creation_flags(0x0800_0000)
                    .spawn();
            }
            if let Some(url) = install {
                st.busy = true;
                st.note = "Downloading…".into();
                let target = TexTarget {
                    status: String::new(),
                    folder: st.folder.clone(),
                    search: None,
                    kind: st.kind,
                    key: st.key.clone(),
                    gfx_dir: st.gfx_dir.clone(),
                    dolphin_user: st.dolphin_user.clone(),
                };
                let tx = self.work_tx.clone();
                let ctx2 = ctx.clone();
                std::thread::spawn(move || {
                    let msg = match install_texture_pack(&url, &target) {
                        Ok(m) => m,
                        Err(m) => format!("⚠ {m}"),
                    };
                    let _ = tx.send(WorkMsg::TexDone(msg));
                    ctx2.request_repaint();
                });
            }
            if open {
                self.textures_w = Some(st);
            }
        }
    }

    fn flush_emu_edit(&mut self) {
        if let Some(i) = self.emu_sel {
            if i < self.emu_list.len() {
                let e = self.emu_edit.clone();
                let v = &mut self.emu_list[i];
                v["name"] = serde_json::Value::String(e.name);
                v["path"] = serde_json::Value::String(e.path);
                v["system"] = serde_json::Value::String(e.system);
                v["core"] = serde_json::Value::String(e.core);
                v["args"] = serde_json::Value::String(e.args);
                v["extensions"] = serde_json::Value::String(e.extensions);
                v["folder"] = serde_json::Value::String(e.folder);
            }
        }
    }

    fn load_emu_edit(&mut self, i: usize) {
        if let Some(v) = self.emu_list.get(i) {
            let g = |k: &str| {
                v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string()
            };
            self.emu_edit = EmuEdit {
                name: g("name"),
                path: g("path"),
                system: g("system"),
                core: g("core"),
                args: g("args"),
                extensions: g("extensions"),
                folder: g("folder"),
            };
        }
    }
}

impl Drop for Proto {
    /// Last-chance bookkeeping: if the app goes away
    /// with a session live (closed between frames, a
    /// killed viewport, …), fold it and flush the
    /// sidecar synchronously. Both are idempotent, so
    /// the orderly close path (update's close_requested
    /// hook) having already run is fine.
    fn drop(&mut self) {
        self.end_play();
        self.flush_data_now();
    }
}

impl eframe::App for Proto {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.art_frame_count = 0;
        // Startup loading state (design Slice 2): until
        // the loader thread delivers the library, paint
        // only this. On a snapshot hit it lasts a frame
        // or two; on a cold parse on a slow machine it's
        // the difference between "alive" and "frozen
        // blank window".
        let mut arrived: Option<StartupLib> = None;
        let mut loader_died = false;
        if let Some(rx) = &self.loading {
            match rx.try_recv() {
                Ok(data) => arrived = Some(data),
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    loader_died = true;
                }
            }
        }
        if let Some(data) = arrived {
            self.loading = None;
            self.apply_loaded(data);
        } else if loader_died {
            // The loader thread panicked (it shouldn't):
            // open on an empty library rather than
            // loading forever.
            self.loading = None;
            self.status =
                "Library load failed — showing an empty library."
                    .into();
        } else if self.loading.is_some() {
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(BG))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(220.0);
                        ui.heading(
                            egui::RichText::new("🎮 EmuFront Rust")
                                .size(28.0)
                                .color(Color32::WHITE),
                        );
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(
                                "Loading your library…",
                            )
                            .color(TEXT_DIM),
                        );
                        ui.add_space(10.0);
                        ui.spinner();
                    });
                });
            ctx.request_repaint_after(Duration::from_millis(120));
            return;
        }
        self.drain_art(ctx);
        self.drain_ra();
        self.drain_work();
        self.drain_cfg(ctx);
        self.drain_data();
        self.poll_play(ctx);
        self.flush_data();
        if ctx.input(|i| i.viewport().close_requested()) {
            // The window is closing: fold the live
            // session and flush the store NOW, not at
            // the next debounce (Drop is the backstop).
            self.end_play();
            self.flush_data_now();
        }
        if self.profile.scale_lock {
            // Low-spec (study §3): fill cost scales with
            // pixels-per-point squared on weak iGPUs; the
            // lock is set every frame so it can't drift.
            ctx.set_pixels_per_point(1.0);
        }
        if !self.ra_started && !matches!(self.view, View::Picker) {
            self.ra_started = true;
            self.start_ra_check(ctx);
        }
        // Emulator update check: once per run, non-blocking, and only
        // when the cached results are stale (> 24 h) — the manual
        // button in Manage Emulators re-checks any time.
        if !self.emu_check_started && !matches!(self.view, View::Picker) {
            self.emu_check_started = true;
            if now_epoch().saturating_sub(self.settings.emu_updates_at)
                > 24 * 3600
            {
                self.start_emu_check(ctx);
            }
        }
        if matches!(self.view, View::Picker) {
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(BG))
                .show(ctx, |ui| self.picker(ui, ctx));
        } else {
            egui::TopBottomPanel::top("topbar")
                .frame(egui::Frame::none().fill(PANEL))
                .show(ctx, |ui| self.topbar(ui, ctx));
            if matches!(self.view, View::Wall(_)) {
                egui::SidePanel::right("gamepanel")
                    .exact_width(292.0)
                    .frame(egui::Frame::none().fill(BG))
                    .show(ctx, |ui| self.panel(ui, ctx));
            }
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(BG))
                .show(ctx, |ui| match self.view {
                    View::Home => self.home(ui, ctx),
                    View::Wall(_) => self.wall_central(ui, ctx),
                    View::Picker => {}
                });
            self.windows(ctx);
        }
        egui::TopBottomPanel::bottom("status")
            .frame(egui::Frame::none().fill(PANEL))
            .show(ctx, |ui| {
                // The queued-write feedback (§2.5): saves
                // happen on the worker now, so the bar
                // says so while any are in flight — the
                // status text itself is left untouched.
                let saving = if self.cfg_pending > 0 {
                    " · 💾 Saving…"
                } else {
                    ""
                };
                ui.label(
                    egui::RichText::new(format!(
                        "{}{}",
                        self.status, saving
                    ))
                    .color(TEXT_DIM),
                );
            });
    }
}

// ---------------------------------------------------------------------
// MUSIC PLAYER (design: MUSIC-FEATURE-PREP.md sec. 4-6)
// Per-game soundtrack player in the detail panel. A game's set is
// the playable files under data_dir()/Music/<System>/<Game>/ (the
// hacked_dir() shape, design sec. 4.2). Engines are the ones the
// --music-probe build gate proved on this toolchain: libgme via
// the game-music-emu crate for SPC/NSF/VGM, rustysynth + a
// SoundFont for MIDI, rodio (cpal/WASAPI) for output.
// HARD RULE inherited from the probe: never route a file through
// the binding's identify_header / from_file / from_data -- its
// header sniffer panics on unrecognized data. Everything routes
// by extension + load_data, so a bad file is a clean error.
// Playback lives on ONE audio thread that owns the rodio device
// handle + Player (below); the UI sends commands and reads
// position from shared atomics. No autoplay, ever: the user presses
// play.
const MUSIC_RATE: u32 = 44100;
// Chiptune tracks with no usable tag duration play this long
// (the community "about two loops" convention, design sec. 4.4),
// then fade out.
const MUSIC_DEFAULT_SECS: f64 = 180.0;
const MUSIC_DEFAULT_FADE_SECS: f64 = 10.0;
// The lounge loop asset (assets/lounge_loop.mid, an original
// composition for EmuFront -- authored by
// session-20261023/make_lounge.py) is 16 bars of ii-V-I-vi in F
// at 72 BPM.
const LOUNGE_SECS: f64 = 53.33;
const LOUNGE_TITLE: &str = "EmuFront Lounge Loop";
const LOUNGE_BYTES: &[u8] = include_bytes!("../assets/lounge_loop.mid");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MusicKind {
    Spc,
    Nsf,
    Vgm,
    Midi,
}

impl MusicKind {
    fn label(self) -> &'static str {
        match self {
            MusicKind::Spc => "SPC",
            MusicKind::Nsf => "NSF",
            MusicKind::Vgm => "VGM",
            MusicKind::Midi => "MIDI",
        }
    }
}

/// Route a file to an engine by extension ONLY (see the hard
/// rule above). None = not playable.
fn music_kind_for_ext(ext: &str) -> Option<MusicKind> {
    match ext.to_ascii_lowercase().as_str() {
        "spc" => Some(MusicKind::Spc),
        "nsf" | "nsfe" => Some(MusicKind::Nsf),
        "vgm" | "vgz" => Some(MusicKind::Vgm),
        "mid" | "midi" => Some(MusicKind::Midi),
        _ => None,
    }
}

fn music_kind_for_path(p: &Path) -> Option<MusicKind> {
    p.extension()
        .and_then(|e| e.to_str())
        .and_then(music_kind_for_ext)
}

/// libgme emulator for a kind + extension (NSFE gets its own
/// type; .vgz bytes are inflated before this is consulted).
/// None for MIDI (rustysynth's job).
fn music_emu_type(kind: MusicKind, ext: &str) -> Option<game_music_emu::EmuType> {
    use game_music_emu::EmuType;
    match kind {
        MusicKind::Spc => Some(EmuType::Spc),
        MusicKind::Nsf => {
            if ext.eq_ignore_ascii_case("nsfe") {
                Some(EmuType::Nsfe)
            } else {
                Some(EmuType::Nsf)
            }
        }
        MusicKind::Vgm => Some(EmuType::Vgm),
        MusicKind::Midi => None,
    }
}

/// Music/<System>/<Game> relative layout (both levels through
/// safe_folder, exactly like hacked_dir()).
fn music_subpath(system: &str, game: &str) -> PathBuf {
    let sys = safe_folder(system);
    let g = safe_folder(game);
    let g = if g.is_empty() { "Game".to_string() } else { g };
    if sys.is_empty() {
        PathBuf::from("Music").join(g)
    } else {
        PathBuf::from("Music").join(sys).join(g)
    }
}

fn music_root() -> PathBuf {
    data_dir().join("Music")
}

fn music_dir_for(game: &Game) -> PathBuf {
    data_dir().join(music_subpath(&game.system, &game.name))
}

/// SoundFont lookup, in order: the settings override, the
/// provisioned default (GeneralUser GS in Music/_soundfonts,
/// its LICENSE.txt kept beside it), then any other .sf2 in
/// that folder. None = MIDI and the lounge loop degrade
/// honestly; chiptune playback is unaffected.
fn soundfont_file(settings: &Settings) -> Option<PathBuf> {
    if !settings.soundfont_path.is_empty() {
        let p = PathBuf::from(&settings.soundfont_path);
        if p.is_file() {
            return Some(p);
        }
    }
    let dir = music_root().join("_soundfonts");
    let def = dir.join("GeneralUser GS.sf2");
    if def.is_file() {
        return Some(def);
    }
    if let Ok(rd) = std::fs::read_dir(&dir) {
        let mut v: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("sf2"))
                    .unwrap_or(false)
            })
            .collect();
        v.sort();
        if let Some(f) = v.into_iter().next() {
            return Some(f);
        }
    }
    None
}

fn fmt_mmss(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "0:00".into();
    }
    let s = secs.round() as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

// ---------------- music metadata parsers -----------------------
// Small, read-only, written against the format layouts (design
// sec. 4.4). All tolerate truncated/garbage input by returning
// empty metadata -- a bad tag must never break the track list.

fn tag_str(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_string()
}

fn ascii_num(bytes: &[u8]) -> Option<f64> {
    let s = tag_str(bytes);
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    s.parse::<f64>().ok()
}

#[derive(Clone, Default)]
struct SpcMeta {
    title: String,
    game: String,
    length_secs: Option<f64>,
    fade_secs: Option<f64>,
}

/// ID666 text tag (song @0x2E, game @0x4E, length seconds
/// @0xA9, fade ms @0xAC) + the XID6 song/game override.
fn parse_spc(data: &[u8]) -> SpcMeta {
    let mut m = SpcMeta::default();
    if data.len() < 0xB1 || &data[0..27] != b"SNES-SPC700 Sound File Data" {
        return m;
    }
    m.title = tag_str(&data[0x2E..0x4E]);
    m.game = tag_str(&data[0x4E..0x6E]);
    m.length_secs = ascii_num(&data[0xA9..0xAC]);
    m.fade_secs = ascii_num(&data[0xAC..0xB1]).map(|ms| ms / 1000.0);
    // XID6 extended chunk overrides the plain tag when present.
    if data.len() > 0x10208 && &data[0x10200..0x10204] == b"xid6" {
        let size = u32::from_le_bytes([
            data[0x10204],
            data[0x10205],
            data[0x10206],
            data[0x10207],
        ]) as usize;
        let end = (0x10208 + size).min(data.len());
        let mut i = 0x10208usize;
        while i + 4 <= end {
            let id = data[i];
            let ty = data[i + 1];
            let len = u16::from_le_bytes([data[i + 2], data[i + 3]]) as usize;
            let val = i + 4;
            if val + len > end {
                break;
            }
            if ty == 0 && (id == 0x01 || id == 0x02) {
                let s = tag_str(&data[val..val + len]);
                if !s.is_empty() {
                    if id == 0x01 {
                        m.title = s;
                    } else {
                        m.game = s;
                    }
                }
            }
            // sub-chunks are padded to a 4-byte boundary
            i = val + ((len + 3) & !3);
        }
    }
    m
}

#[derive(Clone, Default)]
struct NsfMeta {
    songs: usize,
    game: String,
    names: Vec<String>,
    times: Vec<Option<f64>>,
}

/// NSF header (song count @0x06, game string @0x0E) and the
/// NSFE chunk walk (INFO count, auth game, tlbl per-track
/// names, time per-track lengths in ms; negative = unset).
/// Plain NSF carries no per-track names or times.
fn parse_nsf(data: &[u8]) -> NsfMeta {
    let mut m = NsfMeta::default();
    if data.len() >= 0x80 && &data[0..4] == b"NESM" {
        m.songs = data[6] as usize;
        m.game = tag_str(&data[0x0E..0x2E]);
        return m;
    }
    if data.len() >= 8 && &data[0..4] == b"NSFE" {
        let mut i = 4usize;
        while i + 8 <= data.len() {
            let len = u32::from_le_bytes([
                data[i],
                data[i + 1],
                data[i + 2],
                data[i + 3],
            ]) as usize;
            let ty = &data[i + 4..i + 8];
            let body = i + 8;
            if body + len > data.len() {
                break;
            }
            let chunk = &data[body..body + len];
            if ty == b"NEND" {
                break;
            } else if ty == b"INFO" && len >= 10 {
                m.songs = chunk[8] as usize;
            } else if ty == b"auth" {
                if let Some(g) = chunk.split(|&b| b == 0).next() {
                    let g = tag_str(g);
                    if !g.is_empty() {
                        m.game = g;
                    }
                }
            } else if ty == b"tlbl" {
                m.names = chunk.split(|&b| b == 0).map(tag_str).collect();
                while m.names.last().map(|s| s.is_empty()).unwrap_or(false) {
                    m.names.pop();
                }
            } else if ty == b"time" {
                m.times = chunk
                    .chunks_exact(4)
                    .map(|c| {
                        let ms = i32::from_le_bytes([c[0], c[1], c[2], c[3]]);
                        if ms >= 0 {
                            Some(ms as f64 / 1000.0)
                        } else {
                            None
                        }
                    })
                    .collect();
            }
            i = body + len;
        }
    }
    m
}

#[derive(Clone, Default)]
struct VgmMeta {
    title: String,
    game: String,
    total_secs: Option<f64>,
}

/// VGM header (total samples @0x18, at 44.1 kHz) + GD3 tag
/// (pointer @0x14; track/game names as UTF-16LE strings in
/// EN, JP order).
fn parse_vgm(data: &[u8]) -> VgmMeta {
    let mut m = VgmMeta::default();
    if data.len() < 0x40 || &data[0..4] != b"Vgm " {
        return m;
    }
    let total = u32::from_le_bytes([data[0x18], data[0x19], data[0x1A], data[0x1B]]);
    if total > 0 {
        m.total_secs = Some(total as f64 / 44100.0);
    }
    let gd3_rel =
        u32::from_le_bytes([data[0x14], data[0x15], data[0x16], data[0x17]]) as usize;
    if gd3_rel == 0 {
        return m;
    }
    let at = 0x14 + gd3_rel;
    if at + 12 > data.len() || &data[at..at + 4] != b"Gd3 " {
        return m;
    }
    let gd3_len = u32::from_le_bytes([
        data[at + 8],
        data[at + 9],
        data[at + 10],
        data[at + 11],
    ]) as usize;
    let end = (at + 12 + gd3_len).min(data.len());
    let mut strings: Vec<String> = Vec::new();
    let mut i = at + 12;
    while i + 1 < end && strings.len() < 4 {
        let start = i;
        while i + 1 < end && !(data[i] == 0 && data[i + 1] == 0) {
            i += 2;
        }
        let units: Vec<u16> = data[start..i]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        strings.push(String::from_utf16_lossy(&units).trim().to_string());
        i += 2;
    }
    // GD3 order: track EN, track JP, game EN, game JP, ...
    if let Some(t) = strings.first() {
        if !t.is_empty() {
            m.title = t.clone();
        }
    }
    if m.title.is_empty() {
        if let Some(t) = strings.get(1) {
            m.title = t.clone();
        }
    }
    if let Some(g) = strings.get(2) {
        if !g.is_empty() {
            m.game = g.clone();
        }
    }
    if m.game.is_empty() {
        if let Some(g) = strings.get(3) {
            m.game = g.clone();
        }
    }
    m
}

/// VGMRips playlist.m3u entries: bare filenames; "#" lines are
/// comments; the NSFE "file::NSF,n" form keeps the file part.
fn parse_m3u(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut f = line.split("::").next().unwrap_or("").trim();
        if let Some((head, _)) = f.split_once(',') {
            if music_kind_for_path(Path::new(head)).is_some() {
                f = head.trim();
            }
        }
        if !f.is_empty() {
            out.push(f.to_string());
        }
    }
    out
}

/// Human filename order: digit runs compare numerically
/// ("track 2" < "track 10"), letters case-insensitively.
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let (mut i, mut j) = (0usize, 0usize);
    while i < a.len() && j < b.len() {
        let (ca, cb) = (a[i], b[j]);
        if ca.is_ascii_digit() && cb.is_ascii_digit() {
            let mut ea = i;
            while ea < a.len() && a[ea].is_ascii_digit() {
                ea += 1;
            }
            let mut eb = j;
            while eb < b.len() && b[eb].is_ascii_digit() {
                eb += 1;
            }
            let na = std::str::from_utf8(&a[i..ea])
                .unwrap_or("")
                .trim_start_matches('0');
            let nb = std::str::from_utf8(&b[j..eb])
                .unwrap_or("")
                .trim_start_matches('0');
            let ord = na.len().cmp(&nb.len()).then_with(|| na.cmp(nb));
            if ord != Equal {
                return ord;
            }
            i = ea;
            j = eb;
        } else {
            let ord = ca.to_ascii_lowercase().cmp(&cb.to_ascii_lowercase());
            if ord != Equal {
                return ord;
            }
            i += 1;
            j += 1;
        }
    }
    (a.len() - i).cmp(&(b.len() - j))
}

// ---------------- track building -------------------------------

#[derive(Clone)]
enum MusicMeta {
    Spc(SpcMeta),
    Nsf(NsfMeta),
    Vgm(VgmMeta),
    Midi(Option<f64>),
}

#[derive(Clone)]
struct ScannedMusic {
    file_name: String,
    path: PathBuf,
    kind: MusicKind,
    meta: MusicMeta,
}

#[derive(Clone)]
struct MusicTrack {
    title: String,
    file: Option<PathBuf>, // None = the embedded lounge asset
    bank: Option<usize>,   // NSF/NSFE internal track index
    kind: MusicKind,
    play_secs: f64,        // content length (before the fade)
    fade_secs: f64,        // linear fade-out window at the end
    embedded: bool,
}

impl MusicTrack {
    fn total_secs(&self) -> f64 {
        self.play_secs + self.fade_secs
    }
}

/// Scanned files -> display tracks. SPC/VGM/MIDI files are one
/// track each; an NSF/NSFE bank expands to one row per song.
/// playlist.m3u order wins where it names files; everything
/// else falls back to natural filename order.
fn build_tracks(files: &[ScannedMusic], playlist: &[String]) -> Vec<MusicTrack> {
    let order = |f: &ScannedMusic| -> (usize, usize) {
        let base = Path::new(&f.file_name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&f.file_name)
            .to_lowercase();
        match playlist.iter().position(|p| p.to_lowercase() == base) {
            Some(i) => (0, i),
            None => (1, 0),
        }
    };
    let mut sorted: Vec<&ScannedMusic> = files.iter().collect();
    sorted.sort_by(|a, b| {
        order(a)
            .cmp(&order(b))
            .then_with(|| natural_cmp(&a.file_name, &b.file_name))
    });
    let mut out = Vec::new();
    for f in sorted {
        let stem = Path::new(&f.file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("track")
            .to_string();
        match (&f.kind, &f.meta) {
            (MusicKind::Spc, MusicMeta::Spc(m)) => {
                let play = m.length_secs.unwrap_or(MUSIC_DEFAULT_SECS);
                let fade = if m.length_secs.is_some() {
                    m.fade_secs.unwrap_or(MUSIC_DEFAULT_FADE_SECS)
                } else {
                    MUSIC_DEFAULT_FADE_SECS
                };
                out.push(MusicTrack {
                    title: if m.title.is_empty() {
                        stem.clone()
                    } else {
                        m.title.clone()
                    },
                    file: Some(f.path.clone()),
                    bank: None,
                    kind: f.kind,
                    play_secs: play,
                    fade_secs: fade,
                    embedded: false,
                });
            }
            (MusicKind::Nsf, MusicMeta::Nsf(m)) => {
                let n = if m.songs == 0 { 1 } else { m.songs };
                for i in 0..n {
                    let t = m.names.get(i).cloned().unwrap_or_default();
                    let play = m
                        .times
                        .get(i)
                        .copied()
                        .flatten()
                        .unwrap_or(MUSIC_DEFAULT_SECS);
                    out.push(MusicTrack {
                        title: if t.is_empty() {
                            format!("Track {}", i + 1)
                        } else {
                            t
                        },
                        file: Some(f.path.clone()),
                        bank: Some(i),
                        kind: f.kind,
                        play_secs: play,
                        fade_secs: MUSIC_DEFAULT_FADE_SECS,
                        embedded: false,
                    });
                }
            }
            (MusicKind::Vgm, MusicMeta::Vgm(m)) => {
                out.push(MusicTrack {
                    title: if m.title.is_empty() {
                        stem.clone()
                    } else {
                        m.title.clone()
                    },
                    file: Some(f.path.clone()),
                    bank: None,
                    kind: f.kind,
                    play_secs: m.total_secs.unwrap_or(MUSIC_DEFAULT_SECS),
                    fade_secs: MUSIC_DEFAULT_FADE_SECS,
                    embedded: false,
                });
            }
            (MusicKind::Midi, MusicMeta::Midi(len)) => {
                out.push(MusicTrack {
                    title: stem.clone(),
                    file: Some(f.path.clone()),
                    bank: None,
                    kind: f.kind,
                    play_secs: len.unwrap_or(MUSIC_DEFAULT_SECS),
                    fade_secs: 0.0,
                    embedded: false,
                });
            }
            // kind/meta mismatches cannot come out of the
            // scanner; skip defensively rather than panic
            _ => {}
        }
    }
    out
}

fn read_capped(path: &Path, cap: usize) -> Vec<u8> {
    let Ok(mut f) = std::fs::File::open(path) else {
        return Vec::new();
    };
    let mut buf = Vec::new();
    let _ = IoRead::read_to_end(&mut IoRead::take(&mut f, cap as u64), &mut buf);
    buf
}

/// One music file's metadata. .vgz inflates through flate2
/// (pure Rust) exactly like the probe; MIDI length comes from
/// rustysynth's own parser (no SoundFont needed for that).
fn read_music_meta(path: &Path, kind: MusicKind) -> MusicMeta {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();
    match kind {
        MusicKind::Spc => MusicMeta::Spc(parse_spc(&read_capped(path, 0x40000))),
        MusicKind::Nsf => MusicMeta::Nsf(parse_nsf(&read_capped(path, 0x100000))),
        MusicKind::Vgm => {
            if ext.eq_ignore_ascii_case("vgz") {
                let mut meta = MusicMeta::Vgm(VgmMeta::default());
                if let Ok(raw) = std::fs::read(path) {
                    let mut data = Vec::new();
                    let mut dec = flate2::read::GzDecoder::new(raw.as_slice());
                    if IoRead::read_to_end(&mut dec, &mut data).is_ok() {
                        meta = MusicMeta::Vgm(parse_vgm(&data));
                    }
                }
                meta
            } else {
                MusicMeta::Vgm(parse_vgm(&read_capped(path, 0x100000)))
            }
        }
        MusicKind::Midi => {
            let len = std::fs::File::open(path)
                .ok()
                .and_then(|mut f| rustysynth::MidiFile::new(&mut f).ok())
                .map(|m| m.get_length());
            MusicMeta::Midi(len)
        }
    }
}

/// Every playable file under a game's music folder (recursive,
/// depth-bounded: sets extracted from archives can nest a
/// folder deep) plus its playlist.m3u if one exists.
fn scan_music_dir(dir: &Path) -> Vec<MusicTrack> {
    let mut files: Vec<ScannedMusic> = Vec::new();
    let mut playlist: Vec<String> = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = vec![(dir.to_path_buf(), 0)];
    while let Some((d, depth)) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if depth < 2 {
                    stack.push((p, depth + 1));
                }
                continue;
            }
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if name.eq_ignore_ascii_case("playlist.m3u") && playlist.is_empty() {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    playlist = parse_m3u(&text);
                }
                continue;
            }
            if let Some(kind) = music_kind_for_path(&p) {
                let meta = read_music_meta(&p, kind);
                files.push(ScannedMusic {
                    file_name: name,
                    path: p,
                    kind,
                    meta,
                });
            }
        }
    }
    build_tracks(&files, &playlist)
}

/// Programmatic lounge loop (design sec. 6 belt-and-braces):
/// if the embedded lounge_loop.mid asset ever fails to parse,
/// the same progression is generated in memory -- a minimal
/// SMF writer plus the chord table. 8 bars of ii-V-I-vi in F
/// at 72 BPM.
fn lounge_smf_bytes() -> Vec<u8> {
    fn varlen(out: &mut Vec<u8>, mut n: u32) {
        let mut stack = vec![(n & 0x7F) as u8];
        n >>= 7;
        while n > 0 {
            stack.push(((n & 0x7F) as u8) | 0x80);
            n >>= 7;
        }
        out.extend(stack.iter().rev());
    }
    const PPQ: u32 = 480;
    // (bass root, bass fifth, chord tones): Gm7, C7, Fmaj7, Dm7
    const PROG: [(u8, u8, [u8; 4]); 4] = [
        (43, 50, [55, 58, 62, 65]),
        (36, 43, [55, 58, 64, 62]),
        (41, 48, [53, 57, 60, 64]),
        (38, 45, [50, 53, 57, 60]),
    ];
    let tempo = 833_333u32; // 72 BPM
    let mut events: Vec<(u32, Vec<u8>)> = Vec::new();
    events.push((
        0,
        vec![
            0xFF,
            0x51,
            0x03,
            (tempo >> 16) as u8,
            (tempo >> 8) as u8,
            tempo as u8,
        ],
    ));
    events.push((0, vec![0xC0, 4])); // ch0 electric piano
    events.push((0, vec![0xC1, 32])); // ch1 acoustic bass
    for cycle in 0..2u32 {
        for (bar, (root, fifth, chord)) in PROG.iter().enumerate() {
            let t0 = (cycle * 4 + bar as u32) * PPQ * 4;
            for (i, p) in chord.iter().enumerate() {
                let on = t0 + i as u32 * 10;
                events.push((on, vec![0x90, *p, 54]));
                events.push((on + PPQ * 3 + PPQ / 2, vec![0x80, *p, 0]));
            }
            events.push((t0, vec![0x91, *root, 58]));
            events.push((t0 + PPQ * 2, vec![0x81, *root, 0]));
            events.push((t0 + PPQ * 2, vec![0x91, *fifth, 50]));
            events.push((t0 + PPQ * 3 + PPQ / 2, vec![0x81, *fifth, 0]));
        }
    }
    let total = 8 * PPQ * 4;
    events.push((total, vec![0xFF, 0x2F, 0x00]));
    events.sort_by_key(|e| e.0);
    let mut track: Vec<u8> = Vec::new();
    let mut last = 0u32;
    for (tick, data) in &events {
        varlen(&mut track, tick - last);
        track.extend_from_slice(data);
        last = *tick;
    }
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(b"MThd");
    out.extend_from_slice(&6u32.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&(PPQ as u16).to_be_bytes());
    out.extend_from_slice(b"MTrk");
    out.extend_from_slice(&(track.len() as u32).to_be_bytes());
    out.extend_from_slice(&track);
    out
}

// ---------------- playback -------------------------------------

/// Position/progress shared between a playing Source (audio
/// thread) and the panel (UI thread). Frames are stereo frames
/// at MUSIC_RATE.
struct MusicShared {
    pos_frames: AtomicU64,
    total_frames: AtomicU64,
    ended: AtomicBool,
}

impl MusicShared {
    fn new(total_frames: u64) -> Arc<Self> {
        Arc::new(Self {
            pos_frames: AtomicU64::new(0),
            total_frames: AtomicU64::new(total_frames),
            ended: AtomicBool::new(false),
        })
    }
}

/// libgme rendered as an f32 rodio Source, one track of one
/// file. The play limit and a linear fade-out are applied HERE
/// because the binding exposes no gme_set_fade (verified gap,
/// design sec. 3); the parsers above supply length + fade.
struct GmeSource {
    emu: game_music_emu::GameMusicEmu,
    chunk: Vec<i16>,
    cursor: usize,
    chunk_start: u64,
    filled: u64,
    total_frames: u64,
    fade_frames: u64,
    done: bool,
    shared: Arc<MusicShared>,
}

// The emu handle is created, used and dropped under a single
// owner (built on the UI thread, played on the audio thread,
// never shared concurrently). libgme objects are self-contained
// C++ state, so the move is sound; the binding just does not
// mark its raw pointer Send.
unsafe impl Send for GmeSource {}

impl GmeSource {
    fn new(
        emu: game_music_emu::GameMusicEmu,
        play_secs: f64,
        fade_secs: f64,
        shared: Arc<MusicShared>,
    ) -> Self {
        let total = (((play_secs + fade_secs) * MUSIC_RATE as f64) as u64).max(1);
        Self {
            emu,
            chunk: Vec::new(),
            cursor: 0,
            chunk_start: 0,
            filled: 0,
            total_frames: total,
            fade_frames: (fade_secs * MUSIC_RATE as f64) as u64,
            done: false,
            shared,
        }
    }
}

impl Iterator for GmeSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.done {
            return None;
        }
        if self.cursor >= self.chunk.len() {
            if self.filled >= self.total_frames || self.emu.track_ended() {
                self.done = true;
                self.shared.ended.store(true, Ordering::SeqCst);
                return None;
            }
            const CHUNK_FRAMES: u64 = 4096;
            let want = CHUNK_FRAMES.min(self.total_frames - self.filled) as usize;
            self.chunk = vec![0i16; want * 2];
            if self.emu.play(want * 2, &mut self.chunk).is_err() {
                self.done = true;
                self.shared.ended.store(true, Ordering::SeqCst);
                return None;
            }
            self.chunk_start = self.filled;
            self.filled += want as u64;
            self.cursor = 0;
        }
        let frame = self.chunk_start + (self.cursor / 2) as u64;
        self.shared.pos_frames.store(frame + 1, Ordering::SeqCst);
        let mut s = self.chunk[self.cursor] as f32 / 32768.0;
        if self.fade_frames > 0 {
            let remaining = self.total_frames.saturating_sub(frame);
            if remaining <= self.fade_frames {
                s *= remaining as f32 / self.fade_frames as f32;
            }
        }
        self.cursor += 1;
        Some(s)
    }
}

/// rustysynth rendered as an f32 rodio Source (interleaved
/// stereo). The lounge track loops inside the sequencer; its
/// displayed position wraps at the loop length.
struct SynthSource {
    seq: rustysynth::MidiFileSequencer,
    left: Vec<f32>,
    right: Vec<f32>,
    cursor: usize,
    pending: Option<f32>,
    frames_done: u64,
    total_frames: u64,
    loop_it: bool,
    done: bool,
    shared: Arc<MusicShared>,
}

impl SynthSource {
    fn new(
        seq: rustysynth::MidiFileSequencer,
        total_secs: f64,
        loop_it: bool,
        shared: Arc<MusicShared>,
    ) -> Self {
        Self {
            seq,
            left: Vec::new(),
            right: Vec::new(),
            cursor: 0,
            pending: None,
            frames_done: 0,
            total_frames: ((total_secs * MUSIC_RATE as f64) as u64).max(1),
            loop_it,
            done: false,
            shared,
        }
    }
}

impl Iterator for SynthSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if let Some(r) = self.pending.take() {
            return Some(r);
        }
        if self.done {
            return None;
        }
        if self.cursor >= self.left.len() {
            if !self.loop_it && self.frames_done >= self.total_frames {
                self.done = true;
                self.shared.ended.store(true, Ordering::SeqCst);
                return None;
            }
            const CHUNK: usize = 2048;
            self.left = vec![0f32; CHUNK];
            self.right = vec![0f32; CHUNK];
            self.seq.render(&mut self.left, &mut self.right);
            self.cursor = 0;
        }
        let l = self.left[self.cursor];
        let r = self.right[self.cursor];
        self.cursor += 1;
        self.frames_done += 1;
        let pos = if self.loop_it && self.total_frames > 0 {
            self.frames_done % self.total_frames
        } else {
            self.frames_done
        };
        self.shared.pos_frames.store(pos, Ordering::SeqCst);
        self.pending = Some(r);
        Some(l)
    }
}

/// The two engines behind one rodio Source (rodio 0.23's
/// Sample is f32 throughout, so both render f32 here).
enum AnySource {
    Gme(GmeSource),
    Synth(SynthSource),
}

impl Iterator for AnySource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        match self {
            AnySource::Gme(s) => s.next(),
            AnySource::Synth(s) => s.next(),
        }
    }
}

impl rodio::Source for AnySource {
    fn current_span_len(&self) -> Option<usize> {
        None // one span: rate/channels never change mid-track
    }
    fn channels(&self) -> std::num::NonZeroU16 {
        std::num::NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> std::num::NonZeroU32 {
        std::num::NonZero::new(MUSIC_RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        match self {
            AnySource::Gme(s) => Some(Duration::from_secs_f64(
                s.total_frames as f64 / MUSIC_RATE as f64,
            )),
            AnySource::Synth(s) => {
                if s.loop_it {
                    None // loops forever
                } else {
                    Some(Duration::from_secs_f64(
                        s.total_frames as f64 / MUSIC_RATE as f64,
                    ))
                }
            }
        }
    }
}

enum MusicCmd {
    Play(AnySource),
    Pause,
    Resume,
    Stop,
    SetVolume(f32),
}

/// The one music-player owner: a thread holding the rodio
/// device handle + Player. Created lazily on the first Play.
/// Dropping Proto drops the last command sender; the thread
/// then exits and dropping the handle ends all sound
/// (stop-on-drop -- no thread leaks).
struct MusicPlayer {
    tx: Sender<MusicCmd>,
}

impl MusicPlayer {
    fn spawn(status_tx: Sender<WorkMsg>, ctx: &egui::Context, volume: f32) -> Self {
        let (tx, rx) = channel::<MusicCmd>();
        let ctx2 = ctx.clone();
        std::thread::spawn(move || {
            let handle = match rodio::DeviceSinkBuilder::open_default_sink() {
                Ok(h) => h,
                Err(e) => {
                    let _ = status_tx.send(WorkMsg::MusicStatus(format!(
                        "Music: couldn't open the audio output ({e:?})."
                    )));
                    ctx2.request_repaint();
                    return;
                }
            };
            let player = rodio::Player::connect_new(handle.mixer());
            player.set_volume(volume);
            while let Ok(cmd) = rx.recv() {
                match cmd {
                    MusicCmd::Play(src) => {
                        player.stop();
                        player.append(src);
                        player.play();
                    }
                    MusicCmd::Pause => player.pause(),
                    MusicCmd::Resume => player.play(),
                    MusicCmd::Stop => player.stop(),
                    MusicCmd::SetVolume(v) => player.set_volume(v),
                }
            }
            // channel closed: handle + player drop here and
            // any playing sound ends with them
        });
        Self { tx }
    }

    fn send(&self, cmd: MusicCmd) {
        let _ = self.tx.send(cmd);
    }
}

/// Detail-panel music state for one game.
struct MusicUiState {
    game_path: String,
    tracks: Vec<MusicTrack>,
    lounge: bool,
    sf_path: Option<PathBuf>,
    sf_cache: Option<Arc<rustysynth::SoundFont>>,
    index: Option<usize>,
    playing: bool,
    paused: bool,
    shared: Option<Arc<MusicShared>>,
}

fn lounge_track() -> MusicTrack {
    MusicTrack {
        title: LOUNGE_TITLE.to_string(),
        file: None,
        bank: None,
        kind: MusicKind::Midi,
        play_secs: LOUNGE_SECS,
        fade_secs: 0.0,
        embedded: true,
    }
}

/// Panel state for one game: its scanned set, or the single
/// lounge row when the set is empty (design sec. 6).
fn build_music_state(game: &Game, settings: &Settings) -> MusicUiState {
    let dir = music_dir_for(game);
    let mut tracks = scan_music_dir(&dir);
    let lounge = tracks.is_empty();
    if lounge {
        tracks = vec![lounge_track()];
    }
    MusicUiState {
        game_path: game.path.clone(),
        tracks,
        lounge,
        sf_path: soundfont_file(settings),
        sf_cache: None,
        index: None,
        playing: false,
        paused: false,
        shared: None,
    }
}

/// Build the rodio Source for one track. Chiptune bytes route
/// by extension into libgme (never its identifying
/// constructors); .vgz inflates through flate2 first. MIDI
/// needs the SoundFont (cached after the first parse); the
/// lounge track plays the embedded asset, with the generated
/// SMF as fallback if the asset ever fails to parse.
fn build_music_source(
    st: &mut MusicUiState,
    track: &MusicTrack,
) -> Result<(AnySource, Arc<MusicShared>), String> {
    let shared = MusicShared::new(
        ((track.total_secs() * MUSIC_RATE as f64) as u64).max(1),
    );
    match track.kind {
        MusicKind::Midi => {
            let sf_path = st.sf_path.clone().ok_or_else(|| {
                "no SoundFont found (Music\\_soundfonts is empty) -- MIDI and the lounge loop need one.".to_string()
            })?;
            if st.sf_cache.is_none() {
                let mut f = std::fs::File::open(&sf_path)
                    .map_err(|e| format!("SoundFont open failed: {e}"))?;
                let sf = rustysynth::SoundFont::new(&mut f)
                    .map_err(|e| format!("SoundFont parse failed: {e:?}"))?;
                st.sf_cache = Some(Arc::new(sf));
            }
            let sf = st.sf_cache.clone().unwrap();
            let bytes: Vec<u8> = if track.embedded {
                LOUNGE_BYTES.to_vec()
            } else {
                let path = track
                    .file
                    .as_ref()
                    .ok_or_else(|| "Track has no file.".to_string())?;
                std::fs::read(path)
                    .map_err(|e| format!("Couldn't read the MIDI file: {e}"))?
            };
            let mut read = bytes.as_slice();
            let midi = match rustysynth::MidiFile::new(&mut read) {
                Ok(m) => m,
                Err(_) if track.embedded => {
                    let gen = lounge_smf_bytes();
                    let mut r2 = gen.as_slice();
                    rustysynth::MidiFile::new(&mut r2)
                        .map_err(|e| format!("Lounge loop failed to load: {e:?}"))?
                }
                Err(e) => return Err(format!("MIDI parse failed: {e:?}")),
            };
            let len = midi.get_length();
            let midi = Arc::new(midi);
            let synth = rustysynth::Synthesizer::new(
                &sf,
                &rustysynth::SynthesizerSettings::new(MUSIC_RATE as i32),
            )
            .map_err(|e| format!("Synthesizer failed: {e:?}"))?;
            let mut seq = rustysynth::MidiFileSequencer::new(synth);
            let loop_it = track.embedded;
            seq.play(&midi, loop_it);
            // the sequencer's own length is the truth for total
            let tsec = if len > 0.0 { len } else { track.play_secs };
            shared
                .total_frames
                .store(((tsec * MUSIC_RATE as f64) as u64).max(1), Ordering::SeqCst);
            Ok((
                AnySource::Synth(SynthSource::new(seq, tsec, loop_it, shared.clone())),
                shared,
            ))
        }
        kind => {
            let path = track
                .file
                .as_ref()
                .ok_or_else(|| "Track has no file.".to_string())?;
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            let raw = std::fs::read(path)
                .map_err(|e| format!("Couldn't read the track file: {e}"))?;
            let data: Vec<u8> = if ext.eq_ignore_ascii_case("vgz") {
                let mut out = Vec::new();
                let mut dec = flate2::read::GzDecoder::new(raw.as_slice());
                IoRead::read_to_end(&mut dec, &mut out)
                    .map_err(|e| format!("Couldn't inflate the .vgz: {e}"))?;
                out
            } else {
                raw
            };
            let emu_type = music_emu_type(kind, &ext)
                .ok_or_else(|| "No emulator for this format.".to_string())?;
            let emu = game_music_emu::GameMusicEmu::new(emu_type, MUSIC_RATE);
            emu.load_data(&data)
                .map_err(|e| format!("libgme refused the file: {e}"))?;
            emu.start_track(track.bank.unwrap_or(0))
                .map_err(|e| format!("libgme couldn't start the track: {e}"))?;
            Ok((
                AnySource::Gme(GmeSource::new(
                    emu,
                    track.play_secs,
                    track.fade_secs,
                    shared.clone(),
                )),
                shared,
            ))
        }
    }
}

// ---------------- soundtrack import ----------------------------

fn pick_music_file_dialog() -> Option<String> {
    ps_dialog(
        "Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.OpenFileDialog; $d.Filter = 'Soundtracks & archives (*.rsn;*.zip;*.rar;*.7z;*.spc;*.nsf;*.nsfe;*.vgm;*.vgz;*.mid)|*.rsn;*.zip;*.rar;*.7z;*.spc;*.nsf;*.nsfe;*.vgm;*.vgz;*.mid|All files (*.*)|*.*'; if ($d.ShowDialog() -eq 'OK') { $d.FileName }",
    )
}

/// "Find this soundtrack" targets (design sec. 4.5): only the
/// archives the design survey actually verified -- SNESmusic
/// for SNES, Zophar's music index for everything else.
fn soundtrack_url(system: &str) -> &'static str {
    if system.contains("Super Nintendo") || system.contains("SNES") {
        "https://snesmusic.org/v2/"
    } else {
        "https://www.zophar.net/music/"
    }
}

/// File a picked set into a game's Music folder: a folder's
/// playable files (+ playlist) are copied, a single music file
/// is copied, anything else is treated as an archive and
/// extracted through the app's existing 7-Zip helper
/// (SNESmusic .rsn included). Runs on a worker thread.
fn music_import_worker(picked: Option<String>, dest: &Path) -> String {
    let Some(picked) = picked else {
        return "Soundtrack import cancelled.".into();
    };
    let src = PathBuf::from(&picked);
    let _ = std::fs::create_dir_all(dest);
    if src.is_dir() {
        let mut copied = 0usize;
        let mut stack = vec![src.clone()];
        while let Some(d) = stack.pop() {
            if let Ok(rd) = std::fs::read_dir(&d) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        stack.push(p);
                    } else if music_kind_for_path(&p).is_some()
                        || p.file_name()
                            .and_then(|s| s.to_str())
                            .map(|n| n.eq_ignore_ascii_case("playlist.m3u"))
                            .unwrap_or(false)
                    {
                        if let Some(name) = p.file_name() {
                            if std::fs::copy(&p, dest.join(name)).is_ok() {
                                copied += 1;
                            }
                        }
                    }
                }
            }
        }
        return format!("Soundtrack import: copied {copied} file(s).");
    }
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if music_kind_for_ext(&ext).is_some() {
        if let Some(name) = src.file_name() {
            if std::fs::copy(&src, dest.join(name)).is_ok() {
                return "Soundtrack import: 1 file added.".into();
            }
        }
        return "Soundtrack import failed (couldn't copy the file).".into();
    }
    match extract_archive(&src, dest) {
        Ok(files) => {
            let n = files
                .iter()
                .filter(|f| music_kind_for_path(f).is_some())
                .count();
            format!("Soundtrack import: extracted, {n} playable file(s).")
        }
        Err(e) => format!("Soundtrack import failed: {e}"),
    }
}

// ---------------------------------------------------------------------
// MUSIC BUILD GATE (probe only -- no player UI exists yet)
// --music-probe <dir> renders ~2 s of PCM per format through the exact
// engine paths the future detail-panel player will use, and RMS-checks
// the output: SPC / NSF / VGM through game-music-emu (libgme), .vgz
// inflated through flate2 first, MIDI through rustysynth + a staged
// SoundFont, and the rendered MIDI samples re-wrapped in a rodio
// SamplesBuffer (the Source shape the player will feed rodio with).
// It proves the engines build, link and actually render BEFORE any
// player code is written against them (MUSIC-FEATURE-PREP.md sec. 7).
// Staged set in <dir>: probe.spc, probe.nsf, probe.vgz (or probe.vgm),
// probe.mid, soundfont.sf2. A missing file prints an explicit SKIP
// line -- never a silent pass. Exit 0 iff every staged leg passed and
// at least one leg was staged, else exit 1. Headless: samples render
// into memory buffers only; no window, no audio device is opened.
const MUSIC_PROBE_RATE: u32 = 44100;
const MUSIC_PROBE_SECS: usize = 2;
// i16 RMS floor for "non-silent" (full scale is 32767); real chiptune
// renders sit far above this, digital silence sits at 0.
const MUSIC_PROBE_MIN_RMS_I16: f64 = 50.0;
// f32 RMS floor for the rustysynth render (full scale is 1.0).
const MUSIC_PROBE_MIN_RMS_F32: f64 = 0.001;

fn music_probe_rms_i16(samples: &[i16]) -> (f64, i32) {
    let mut sum: f64 = 0.0;
    let mut peak: i32 = 0;
    for s in samples {
        let v = *s as f64;
        sum += v * v;
        let a = (*s as i32).abs();
        if a > peak {
            peak = a;
        }
    }
    let rms = if samples.is_empty() {
        0.0
    } else {
        (sum / samples.len() as f64).sqrt()
    };
    (rms, peak)
}

fn music_probe_rms_f32(samples: &[f32]) -> (f64, f32) {
    let mut sum: f64 = 0.0;
    let mut peak: f32 = 0.0;
    for s in samples {
        let v = *s as f64;
        sum += v * v;
        let a = s.abs();
        if a > peak {
            peak = a;
        }
    }
    let rms = if samples.is_empty() {
        0.0
    } else {
        (sum / samples.len() as f64).sqrt()
    };
    (rms, peak)
}

// One libgme leg. `data` is the (already inflated, for .vgz) file
// bytes; `emu_type` selects the emulator explicitly. Deliberately NOT
// using the binding's from_file / from_data / identify_header: its
// identify_header routes through EmuType::from_extension, which
// panics on an unrecognized header. Extension routing here plus
// libgme's own load error keeps every failure a clean FAIL line
// instead of a panic. Prints its own result line; returns pass/fail.
fn music_probe_gme(fmt: &str, emu_type: game_music_emu::EmuType, data: &[u8]) -> bool {
    let emu = game_music_emu::GameMusicEmu::new(emu_type, MUSIC_PROBE_RATE);
    if let Err(e) = emu.load_data(data) {
        println!("MUSIC-PROBE {fmt} result=FAIL stage=load err={e}");
        return false;
    }
    let tracks = emu.track_count();
    if let Err(e) = emu.start_track(0) {
        println!("MUSIC-PROBE {fmt} result=FAIL stage=start err={e}");
        return false;
    }
    // stereo interleaved i16: 2 samples per frame
    let want = MUSIC_PROBE_RATE as usize * MUSIC_PROBE_SECS * 2;
    let mut buf = vec![0i16; want];
    if let Err(e) = emu.play(want, &mut buf) {
        println!("MUSIC-PROBE {fmt} result=FAIL stage=play err={e}");
        return false;
    }
    let (rms, peak) = music_probe_rms_i16(&buf);
    let pass = rms >= MUSIC_PROBE_MIN_RMS_I16;
    println!(
        "MUSIC-PROBE {fmt} result={} tracks={} samples={} rms={:.1} peak={} tell_ms={} ended={}",
        if pass { "PASS" } else { "FAIL" },
        tracks,
        want,
        rms,
        peak,
        emu.tell(),
        emu.track_ended()
    );
    pass
}

// The .vgz leg: gzip-inflate through flate2 (pure-Rust miniz_oxide
// backend; the probe inflates .vgz itself rather than relying on
// any zlib inside libgme), then the plain VGM path.
fn music_probe_vgz(path: &Path) -> bool {
    let raw = match std::fs::read(path) {
        Ok(r) => r,
        Err(e) => {
            println!("MUSIC-PROBE VGM result=FAIL stage=read err={e}");
            return false;
        }
    };
    let mut dec = flate2::read::GzDecoder::new(raw.as_slice());
    let mut data: Vec<u8> = Vec::new();
    if let Err(e) = dec.read_to_end(&mut data) {
        println!("MUSIC-PROBE VGM result=FAIL stage=inflate err={e}");
        return false;
    }
    music_probe_gme("VGM", game_music_emu::EmuType::Vgm, &data)
}

// The MIDI leg: rustysynth SoundFont + MidiFileSequencer rendering
// into f32 buffers. On PASS returns the interleaved stereo samples so
// the caller can run the rodio leg on real rendered audio.
fn music_probe_midi(mid_path: &Path, sf_path: &Path) -> Option<Vec<f32>> {
    use rustysynth::{
        MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings,
    };
    let mut sf_file = match std::fs::File::open(sf_path) {
        Ok(f) => f,
        Err(e) => {
            println!("MUSIC-PROBE MIDI result=FAIL stage=sf-open err={e}");
            return None;
        }
    };
    let soundfont = match SoundFont::new(&mut sf_file) {
        Ok(s) => s,
        Err(e) => {
            println!("MUSIC-PROBE MIDI result=FAIL stage=sf-parse err={e}");
            return None;
        }
    };
    let presets = soundfont.get_presets().len();
    let soundfont = Arc::new(soundfont);
    let settings = SynthesizerSettings::new(MUSIC_PROBE_RATE as i32);
    let synth = match Synthesizer::new(&soundfont, &settings) {
        Ok(s) => s,
        Err(e) => {
            println!("MUSIC-PROBE MIDI result=FAIL stage=synth err={e}");
            return None;
        }
    };
    let mut mid_file = match std::fs::File::open(mid_path) {
        Ok(f) => f,
        Err(e) => {
            println!("MUSIC-PROBE MIDI result=FAIL stage=mid-open err={e}");
            return None;
        }
    };
    let midi = match MidiFile::new(&mut mid_file) {
        Ok(m) => m,
        Err(e) => {
            println!("MUSIC-PROBE MIDI result=FAIL stage=mid-parse err={e}");
            return None;
        }
    };
    let len_s = midi.get_length();
    let midi = Arc::new(midi);
    let mut seq = MidiFileSequencer::new(synth);
    seq.play(&midi, false);
    let frames = MUSIC_PROBE_RATE as usize * MUSIC_PROBE_SECS;
    let mut left = vec![0f32; frames];
    let mut right = vec![0f32; frames];
    seq.render(&mut left, &mut right);
    let mut inter: Vec<f32> = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        inter.push(left[i]);
        inter.push(right[i]);
    }
    let (rms, peak) = music_probe_rms_f32(&inter);
    let pass = rms >= MUSIC_PROBE_MIN_RMS_F32;
    println!(
        "MUSIC-PROBE MIDI result={} sf_presets={} mid_len_s={:.2} samples={} rms={:.5} peak={:.3}",
        if pass { "PASS" } else { "FAIL" },
        presets,
        len_s,
        inter.len(),
        rms,
        peak
    );
    if pass {
        Some(inter)
    } else {
        None
    }
}

// The rodio leg: wrap rendered samples in a SamplesBuffer and read
// its Source metadata back. Proves rodio (+ cpal under it) builds and
// links under the MinGW GNU toolchain and that the buffer-source
// shape the player will use behaves. Opens no audio device.
fn music_probe_rodio(interleaved: &[f32]) -> bool {
    use rodio::Source;
    let mut src = rodio::buffer::SamplesBuffer::new(
        std::num::NonZero::new(2).unwrap(),
        std::num::NonZero::new(MUSIC_PROBE_RATE).unwrap(),
        interleaved.to_vec(),
    );
    let ch = src.channels().get();
    let rate = src.sample_rate().get();
    let dur_ms = src
        .total_duration()
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let first = src.next();
    let pass = ch == 2 && rate == MUSIC_PROBE_RATE && first.is_some();
    println!(
        "MUSIC-PROBE RODIO result={} channels={} rate={} dur_ms={}",
        if pass { "PASS" } else { "FAIL" },
        ch,
        rate,
        dur_ms
    );
    pass
}

fn run_music_probe(dir: &Path) -> i32 {
    println!("MUSIC-PROBE dir={}", dir.display());
    let types: Vec<String> = game_music_emu::type_list()
        .iter()
        .map(|t| t.to_extension().to_string())
        .collect();
    println!("MUSIC-PROBE gme_types={}", types.join(","));
    let mut staged = 0usize;
    let mut passed = 0usize;
    // SPC leg
    let spc = dir.join("probe.spc");
    if spc.exists() {
        staged += 1;
        match std::fs::read(&spc) {
            Ok(data) => {
                if music_probe_gme("SPC", game_music_emu::EmuType::Spc, &data) {
                    passed += 1;
                }
            }
            Err(e) => println!("MUSIC-PROBE SPC result=FAIL stage=read err={e}"),
        }
    } else {
        println!("MUSIC-PROBE SPC result=SKIP file=probe.spc missing");
    }
    // NSF leg
    let nsf = dir.join("probe.nsf");
    if nsf.exists() {
        staged += 1;
        match std::fs::read(&nsf) {
            Ok(data) => {
                if music_probe_gme("NSF", game_music_emu::EmuType::Nsf, &data) {
                    passed += 1;
                }
            }
            Err(e) => println!("MUSIC-PROBE NSF result=FAIL stage=read err={e}"),
        }
    } else {
        println!("MUSIC-PROBE NSF result=SKIP file=probe.nsf missing");
    }
    // VGM leg: probe.vgz (flate2 path) preferred, else probe.vgm
    let vgz = dir.join("probe.vgz");
    let vgm = dir.join("probe.vgm");
    if vgz.exists() {
        staged += 1;
        if music_probe_vgz(&vgz) {
            passed += 1;
        }
    } else if vgm.exists() {
        staged += 1;
        match std::fs::read(&vgm) {
            Ok(data) => {
                if music_probe_gme("VGM", game_music_emu::EmuType::Vgm, &data) {
                    passed += 1;
                }
            }
            Err(e) => println!("MUSIC-PROBE VGM result=FAIL stage=read err={e}"),
        }
    } else {
        println!("MUSIC-PROBE VGM result=SKIP file=probe.vgz/probe.vgm missing");
    }
    // MIDI leg (+ the rodio leg on its rendered samples)
    let mid = dir.join("probe.mid");
    let sf = dir.join("soundfont.sf2");
    if !mid.exists() {
        println!("MUSIC-PROBE MIDI result=SKIP file=probe.mid missing");
        println!("MUSIC-PROBE RODIO result=SKIP reason=no-midi-render");
    } else if !sf.exists() {
        println!("MUSIC-PROBE MIDI result=SKIP reason=soundfont.sf2 missing");
        println!("MUSIC-PROBE RODIO result=SKIP reason=no-midi-render");
    } else {
        staged += 1;
        match music_probe_midi(&mid, &sf) {
            Some(samples) => {
                passed += 1;
                staged += 1; // rodio leg is staged once samples exist
                if music_probe_rodio(&samples) {
                    passed += 1;
                }
            }
            None => {
                println!("MUSIC-PROBE RODIO result=SKIP reason=midi-render-failed");
            }
        }
    }
    println!("MUSIC-PROBE SUMMARY staged={staged} passed={passed}");
    if staged > 0 && passed == staged {
        println!("MUSIC-PROBE VERDICT PASS");
        0
    } else {
        println!("MUSIC-PROBE VERDICT FAIL");
        1
    }
}

fn main() {
    if let Some(pos) = std::env::args().position(|a| a == "--plaza-probe") {
        // With a query argument, the probe classifies the
        // search's Plaza entries by what the v1.06 link
        // finder makes of each entry page: FILE hop 1
        // (the shipped finder could do this too), FILE
        // hop 2 (new — the file hid behind a download
        // page), PAGE (a page winner, no file — the
        // browser fallback's clientele) or NONE. Run
        // against the same queries before and after a
        // finder change, it is the link-less rate meter.
        if let Some(q) = std::env::args().nth(pos + 1) {
            let hits = search_plaza_query(&q);
            println!("query '{q}' -> {} plaza entries", hits.len());
            let (mut f1, mut f2, mut pg, mut no) = (0, 0, 0, 0);
            for h in hits.iter().take(12) {
                match resolve_download(&h.url, fetch_page_html) {
                    Ok((Some(u), pages)) => {
                        if is_file_url(&u) {
                            if pages <= 1 {
                                f1 += 1;
                                println!("  FILE hop1  {} -> {}", h.title, u);
                            } else {
                                f2 += 1;
                                println!("  FILE hop2  {} -> {}", h.title, u);
                            }
                        } else {
                            pg += 1;
                            println!("  PAGE       {} -> {}", h.title, u);
                        }
                    }
                    Ok((None, _)) => {
                        no += 1;
                        println!("  NONE       {}", h.title);
                    }
                    Err(e) => {
                        no += 1;
                        println!("  ERR        {} ({})", h.title, e);
                    }
                }
            }
            println!(
                "summary: file-hop1={f1} file-hop2={f2} page={pg} none={no}"
            );
            return;
        }
        let url = "https://romhackplaza.org/database?s=Donkey+Kong+Country";
        let agent = http_agent(25);
        match agent
            .get(url)
            .set(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36",
            )
            .call()
        {
            Ok(resp) => {
                println!("status={}", resp.status());
                let body = resp.into_string().unwrap_or_default();
                println!("len={}", body.len());
                println!(
                    "romhacks links={}",
                    body.matches("romhackplaza.org/romhacks/").count()
                );
                println!("head: {}", &body[..body.len().min(300)]);
            }
            Err(e) => println!("ERR: {e}"),
        }
        return;
    }
    if std::env::args().any(|a| a == "--selftest") {
        // CRC32 known vector ("123456789" -> 0xCBF43926)
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
        // IPS: rom 8 zero bytes; set [0..2]=AA BB; EOF truncate to 4
        let mut ips = b"PATCH".to_vec();
        ips.extend([0, 0, 0, 0, 2, 0xAA, 0xBB]);
        ips.extend(b"EOF");
        ips.extend([0, 0, 4]);
        let out = apply_ips(&[0u8; 8], &ips).expect("ips");
        assert_eq!(out, vec![0xAA, 0xBB, 0, 0]);
        // IPS RLE: 3 x 0x77 at offset 1
        let mut ips2 = b"PATCH".to_vec();
        ips2.extend([0, 0, 1, 0, 0, 0, 3, 0x77]);
        ips2.extend(b"EOF");
        let out2 = apply_ips(&[0u8; 4], &ips2).expect("ips rle");
        assert_eq!(out2, vec![0, 0x77, 0x77, 0x77]);
        // UPS: identity on 3-byte rom (skip 3, no XOR blocks)
        let rom3 = [1u8, 2, 3];
        let mut ups = b"UPS1".to_vec();
        ups.push(0x83); // src_size 3 (UPS varint)
        ups.push(0x83); // dst_size 3
        ups.push(0x83); // skip 3
        ups.extend(crc32(&rom3).to_le_bytes());
        ups.extend(crc32(&rom3).to_le_bytes());
        ups.extend(crc32(&ups.clone()).to_le_bytes()); // patch crc (unused by applier)
        let out3 = apply_ups(&rom3, &ups).expect("ups");
        assert_eq!(out3, rom3.to_vec());
        // naming
        assert_eq!(strip_catalog_prefix_full("004 Street Fighter"), "Street Fighter");
        assert_eq!(strip_catalog_prefix_full("007 - GoldenEye"), "007 - GoldenEye");
        assert_eq!(clean_game_name("01. Contra (USA).zip"), "Contra");
        // letter-range buckets: catalog prefix is stripped
        // first ("004 Street Fighter" files under S), digit-led
        // and accented names land in 0-9, case does not matter
        assert_eq!(bucket_of("004 Street Fighter"), 3);
        assert_eq!(bucket_of("007 - GoldenEye"), 0);
        assert_eq!(bucket_of("asteroids"), 1);
        assert_eq!(bucket_of("Zelda"), 4);
        assert_eq!(bucket_of("École"), 0);
        assert_eq!(bucket_of(""), 0);
        // back navigation: one step back from a wall -- inside
        // a bucket of a large system -> the chooser; at the
        // chooser, in a small system, or on either special
        // shelf -> the home systems grid
        assert_eq!(
            back_target("Super Nintendo", Some(2), 16098),
            BackTarget::Chooser
        );
        assert_eq!(
            back_target("Super Nintendo", None, 16098),
            BackTarget::Home
        );
        assert_eq!(back_target("GameCube", None, 155), BackTarget::Home);
        assert_eq!(back_target("GameCube", Some(1), 155), BackTarget::Home);
        assert_eq!(
            back_target("⭐ Enhanced & Hacked", None, 0),
            BackTarget::Home
        );
        assert_eq!(back_target("PC Ports", None, 9), BackTarget::Home);
        assert_eq!(back_target("PC Ports", Some(0), 9), BackTarget::Home);
        // systems selection order (the user, Oct 24): strict A–Z,
        // case-insensitive, natural numbers, nothing pinned —
        // the front reads Arcade / Atari 2600 / Atari 7800 with
        // Sega Genesis and Super Nintendo in their places
        let mut names = vec![
            "Super Nintendo",
            "Sega Genesis",
            "Atari 7800",
            "Arcade",
            "Atari 2600",
            "3DO",
            "MSX",
            "Microsoft Xbox 360",
            "sega saturn",
            "Sega CD",
            "System 10",
            "System 2",
        ];
        names.sort_by(|a, b| system_name_cmp(a, b));
        assert_eq!(
            names,
            vec![
                "3DO",
                "Arcade",
                "Atari 2600",
                "Atari 7800",
                "Microsoft Xbox 360",
                "MSX",
                "Sega CD",
                "Sega Genesis",
                "sega saturn",
                "Super Nintendo",
                "System 2",
                "System 10",
            ]
        );
        // folded-equal names still get a total order (the byte
        // tiebreak), never hash-map order
        assert_eq!(
            system_name_cmp("MSX", "MSX"),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            system_name_cmp("MSX", "Msx"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            system_name_cmp("Msx", "MSX"),
            std::cmp::Ordering::Greater
        );
        // sorted_systems: counts ride along, no system lost or
        // duplicated; "PC Ports" takes its plain alphabetical
        // place like any system (the ⭐ shelf is not a system
        // and never appears in this list)
        let mk_sys_game = |system: &str| Game {
            name: String::new(),
            path: String::new(),
            system: system.to_string(),
            emulator: String::new(),
            emulator_locked: false,
            patch: String::new(),
            cheats_text: String::new(),
            cheats_enabled: false,
        };
        let sys_games = vec![
            mk_sys_game("Super Nintendo"),
            mk_sys_game("Arcade"),
            mk_sys_game("PC Ports"),
            mk_sys_game("Arcade"),
            mk_sys_game("Sega Genesis"),
            mk_sys_game("Arcade"),
        ];
        assert_eq!(
            sorted_systems(&sys_games),
            vec![
                ("Arcade".to_string(), 3usize),
                ("PC Ports".to_string(), 1usize),
                ("Sega Genesis".to_string(), 1usize),
                ("Super Nintendo".to_string(), 1usize),
            ]
        );
        // texture packs: PS2 serial normalization
        assert_eq!(ps2_serial_normalize("SLUS_203.12").as_deref(), Some("SLUS-20312"));
        assert_eq!(ps2_serial_normalize("slus-20312").as_deref(), Some("SLUS-20312"));
        assert_eq!(ps2_serial_normalize("SLUS20312").as_deref(), Some("SLUS-20312"));
        assert_eq!(ps2_serial_normalize("20312").as_deref(), None);
        assert_eq!(ps2_serial_normalize("SLUS-2031").as_deref(), None);
        // "NTSC-Unk" must NOT pass as NTSC-U (substring trap)
        assert!(is_ntsc_u("NTSC-U"));
        assert!(!is_ntsc_u("NTSC-Unk"));
        assert!(!is_ntsc_u("NTSC-J"));
        assert!(!is_ntsc_u("PAL-E"));
        // texture packs: N64 header name across byte orders
        let mut hdr = [0u8; 20];
        hdr[..12].copy_from_slice(b"SUPER MARIO ");
        assert_eq!(n64_name_from_bytes(&hdr, "z64"), "SUPER MARIO");
        let mut swapped = hdr;
        for c in swapped.chunks_exact_mut(2) { c.swap(0, 1); }
        assert_eq!(n64_name_from_bytes(&swapped, "n64"), "SUPER MARIO");
        // texture packs: PS1 serials (SYSTEM.CNF BOOT style) and
        // serials printed in names/folder tags
        assert_eq!(ps2_serial_normalize("SLUS_005.94").as_deref(), Some("SLUS-00594"));
        assert_eq!(serial_token_in("[SLPS-01868][T+Eng1.02_GSHI]").as_deref(), Some("SLPS-01868"));
        assert_eq!(serial_token_in("[NTSC-U] [SCUS-94221]").as_deref(), Some("SCUS-94221"));
        assert_eq!(serial_token_in("Chrono Cross (Disc 1)"), None);
        // artwork: recorded misses stay fresh for the TTL window
        assert!(miss_is_fresh(1_000_000, 1_000_000 + 3600));
        assert!(!miss_is_fresh(1_000_000, 1_000_000 + ART_MISS_TTL_SECS + 1));
        // emulator update checks: version tuples + compare
        assert_eq!(ver_tuple("v2.1.8093"), vec![2, 1, 8093]);
        assert_eq!(ver_tuple("0.1-12307"), vec![0, 1, 12307]);
        assert!(ver_is_newer(&ver_tuple("2.2.0"), &ver_tuple("2.1.0")));
        assert!(!ver_is_newer(&ver_tuple("2.2.0"), &ver_tuple("2.2.0")));
        assert!(!ver_is_newer(&ver_tuple("2509"), &ver_tuple("2609.104")));
        assert!(ver_is_newer(&ver_tuple("1.63"), &ver_tuple("1.62.1")));
        assert!(!EMU_SOURCES.is_empty());
        // texture packs: title normalization for GameIndex lookups
        assert_eq!(normalize_title("Final Fantasy X"), "finalfantasyx");
        assert_eq!(normalize_title("Mortal Kombat - Deception"), "mortalkombatdeception");
        // music: extension routing (never the binding's
        // identifying constructors -- they can panic)
        assert_eq!(music_kind_for_ext("spc"), Some(MusicKind::Spc));
        assert_eq!(music_kind_for_ext("NSFE"), Some(MusicKind::Nsf));
        assert_eq!(music_kind_for_ext("vgz"), Some(MusicKind::Vgm));
        assert_eq!(music_kind_for_ext("mid"), Some(MusicKind::Midi));
        assert_eq!(music_kind_for_ext("exe"), None);
        assert_eq!(music_kind_for_ext(""), None);
        // music: soundtrack folder layout (safe_folder on both
        // levels, the hacked_dir shape)
        assert_eq!(
            music_subpath("Super Nintendo", "Chrono Trigger"),
            PathBuf::from("Music")
                .join("Super Nintendo")
                .join("Chrono Trigger")
        );
        assert_eq!(
            music_subpath("A:B", "X/Y"),
            PathBuf::from("Music").join("AB").join("XY")
        );
        // music: SPC ID666 text tag
        let mut spc = vec![0u8; 0x10200];
        spc[0..27].copy_from_slice(b"SNES-SPC700 Sound File Data");
        spc[0x2E..0x37].copy_from_slice(b"Test Song");
        spc[0x4E..0x57].copy_from_slice(b"Test Game");
        spc[0xA9..0xAC].copy_from_slice(b"123");
        spc[0xAC..0xB1].copy_from_slice(b"05000");
        let sm = parse_spc(&spc);
        assert_eq!(sm.title, "Test Song");
        assert_eq!(sm.game, "Test Game");
        assert_eq!(sm.length_secs, Some(123.0));
        assert_eq!(sm.fade_secs, Some(5.0));
        // music: NSF header
        let mut nsf = vec![0u8; 0x80];
        nsf[0..4].copy_from_slice(b"NESM");
        nsf[6] = 9;
        nsf[0x0E..0x16].copy_from_slice(b"Mega Man");
        let nm = parse_nsf(&nsf);
        assert_eq!(nm.songs, 9);
        assert_eq!(nm.game, "Mega Man");
        // music: NSFE chunks (INFO count, auth game, tlbl names)
        let mut nsfe: Vec<u8> = b"NSFE".to_vec();
        let mut add_chunk = |ty: &[u8; 4], body: &[u8]| {
            nsfe.extend_from_slice(&(body.len() as u32).to_le_bytes());
            nsfe.extend_from_slice(ty);
            nsfe.extend_from_slice(body);
        };
        add_chunk(b"INFO", &[0, 0, 0, 0, 0, 0, 0, 0, 7, 0]);
        add_chunk(b"auth", b"Test Game\0Someone\0\0");
        add_chunk(b"tlbl", b"First\0Second\0");
        add_chunk(b"NEND", &[]);
        let em = parse_nsf(&nsfe);
        assert_eq!(em.songs, 7);
        assert_eq!(em.game, "Test Game");
        assert_eq!(
            em.names,
            vec!["First".to_string(), "Second".to_string()]
        );
        // music: VGM header + GD3
        let mut vgm = vec![0u8; 0x100];
        vgm[0..4].copy_from_slice(b"Vgm ");
        vgm[0x18..0x1C].copy_from_slice(&(44100u32 * 95).to_le_bytes());
        let gd3_at = 0x40usize;
        let rel = (gd3_at - 0x14) as u32;
        vgm[0x14..0x18].copy_from_slice(&rel.to_le_bytes());
        vgm[gd3_at..gd3_at + 4].copy_from_slice(b"Gd3 ");
        let gstrs = ["Song A", "", "Game B", ""];
        let mut gbody: Vec<u8> = Vec::new();
        for s in gstrs {
            for u in s.encode_utf16() {
                gbody.extend_from_slice(&u.to_le_bytes());
            }
            gbody.extend_from_slice(&[0, 0]);
        }
        vgm[gd3_at + 8..gd3_at + 12]
            .copy_from_slice(&(gbody.len() as u32).to_le_bytes());
        vgm[gd3_at + 12..gd3_at + 12 + gbody.len()]
            .copy_from_slice(&gbody);
        let vm = parse_vgm(&vgm);
        assert_eq!(vm.title, "Song A");
        assert_eq!(vm.game, "Game B");
        assert_eq!(vm.total_secs, Some(95.0));
        // music: playlist parsing + natural order
        assert_eq!(
            parse_m3u("#EXTM3U\n01 - A.vgz\n02 - B.vgz\n"),
            vec!["01 - A.vgz".to_string(), "02 - B.vgz".to_string()]
        );
        assert!(matches!(
            natural_cmp("track 2", "track 10"),
            std::cmp::Ordering::Less
        ));
        assert!(matches!(
            natural_cmp("same", "same"),
            std::cmp::Ordering::Equal
        ));
        // music: track building from fake scanned files -- SPC
        // titles from tags in natural order; an NSF bank
        // expands to one row per song; playlist order wins
        let fake = |name: &str, kind: MusicKind, meta: MusicMeta| ScannedMusic {
            file_name: name.to_string(),
            path: PathBuf::from(name),
            kind,
            meta,
        };
        let mut meta_a = SpcMeta::default();
        meta_a.title = "Alpha".into();
        meta_a.length_secs = Some(100.0);
        meta_a.fade_secs = Some(5.0);
        let mut meta_b = SpcMeta::default();
        meta_b.title = "Beta".into();
        let files = vec![
            fake("02 - B.spc", MusicKind::Spc, MusicMeta::Spc(meta_b)),
            fake("01 - A.spc", MusicKind::Spc, MusicMeta::Spc(meta_a)),
            fake(
                "Mega.nsf",
                MusicKind::Nsf,
                MusicMeta::Nsf(NsfMeta {
                    songs: 3,
                    game: "Mega".into(),
                    names: vec![],
                    times: vec![],
                }),
            ),
        ];
        let tracks = build_tracks(&files, &[]);
        assert_eq!(tracks.len(), 5);
        assert_eq!(tracks[0].title, "Alpha");
        assert_eq!(tracks[0].play_secs, 100.0);
        assert_eq!(tracks[0].fade_secs, 5.0);
        assert_eq!(tracks[0].total_secs(), 105.0);
        assert_eq!(tracks[1].title, "Beta");
        // untagged SPC length falls back to the default policy
        assert_eq!(tracks[1].play_secs, MUSIC_DEFAULT_SECS);
        assert_eq!(tracks[2].title, "Track 1");
        assert_eq!(tracks[2].bank, Some(0));
        assert_eq!(tracks[4].bank, Some(2));
        let ordered = build_tracks(&files, &["Mega.nsf".to_string()]);
        assert_eq!(ordered[0].title, "Track 1");
        // music: time formatting + lounge SMF sanity (the
        // generated fallback and the embedded asset both parse
        // as SMF at the magic level)
        assert_eq!(fmt_mmss(0.0), "0:00");
        assert_eq!(fmt_mmss(65.4), "1:05");
        assert_eq!(fmt_mmss(600.0), "10:00");
        let lounge_gen = lounge_smf_bytes();
        assert!(lounge_gen.starts_with(b"MThd"));
        assert!(lounge_gen.windows(4).any(|w| w == &b"MTrk"[..]));
        assert!(lounge_gen.len() > 200);
        assert!(LOUNGE_BYTES.starts_with(b"MThd"));
        // MSU-1 / MD+ (v1.02): terminology, title filter,
        // search variants, staging names, cue detection,
        // archive-doc filtering, link resolution, page scoring.
        let mk_game = |name: &str, path: &str, system: &str| Game {
            name: name.to_string(),
            path: path.to_string(),
            system: system.to_string(),
            emulator: String::new(),
            emulator_locked: false,
            patch: String::new(),
            cheats_text: String::new(),
            cheats_enabled: false,
        };
        let g_gen = mk_game("Sonic", "C:\\roms\\Sonic.bin", "Sega Genesis");
        let g_snes = mk_game("Chrono", "C:\\roms\\Chrono.sfc", "Super Nintendo");
        assert_eq!(msu_term(&g_gen), "MD+");
        assert_eq!(msu_term(&g_snes), "MSU-1");
        assert_eq!(msu_term(&mk_game("X", "x", "")), "MSU-1");
        // Python's own _msu_title_ok cases
        assert!(msu_title_ok("Sonic MD+ Music Pack", true));
        assert!(!msu_title_ok("Sonic MD+ Music Pack", false));
        assert!(msu_title_ok("Streets of Rage 2 (MSU MD)", true));
        assert!(msu_title_ok("Final Fantasy IV MSU-1", false));
        assert!(!msu_title_ok("Some Plain Hack", true));
        assert!(msu_title_ok("Game md plus edition", true));
        assert!(msu_title_ok("Game MSU-MD edition", false));
        // name variants + aliases
        let ff2 = mk_game(
            "Final Fantasy II",
            "C:\\roms\\Final Fantasy II (USA).sfc",
            "Super Nintendo",
        );
        let vs = name_variants(&ff2);
        assert_eq!(vs[0], "Final Fantasy II");
        assert!(vs.len() <= 3);
        assert_eq!(search_alias("Final Fantasy II"), Some("Final Fantasy IV"));
        assert_eq!(search_alias("Final Fantasy III"), Some("Final Fantasy VI"));
        assert_eq!(search_alias("Other"), None);
        let sonic_v = name_variants(&mk_game(
            "Sonic the Hedgehog",
            "C:\\roms\\0042 - Sonic_The_Hedgehog_[!] v2.bin",
            "Sega Genesis",
        ));
        assert!(sonic_v.iter().all(|v| !v.contains('_')));
        // pcm staging names (Python's trailing-number rule)
        assert_eq!(pcm_staged_name("game-1.pcm", "stem"), "stem-1.pcm");
        assert_eq!(pcm_staged_name("track_12.pcm", "stem"), "stem-12.pcm");
        assert_eq!(pcm_staged_name("song3.pcm", "stem"), "stem-3.pcm");
        assert_eq!(pcm_staged_name("track.pcm", "stem"), "track.pcm");
        // ROM guard names
        assert!(msu_nameish(Path::new("game_msu1.sfc")));
        assert!(msu_nameish(Path::new("Game MD+.bin")));
        assert!(!msu_nameish(Path::new("random.sfc")));
        // cue trigger: exact stem match only, case-insensitive
        let listing = vec![
            "shadow dancer.bin".to_string(),
            "shadow dancer.cue".to_string(),
        ];
        assert!(msu_cue_matches("shadow dancer.bin", &listing));
        assert!(!msu_cue_matches(
            "shadow dancer.bin",
            &["other.cue".to_string()]
        ));
        assert!(!msu_cue_matches(
            "shadow dancer.bin",
            &["shadow dancer (track 1).cue".to_string()]
        ));
        // detection against a scratch data dir: pcm parity +
        // the Genesis cue trigger end to end
        let scratch =
            std::env::temp_dir().join(format!("efr-selftest-{}", std::process::id()));
        let gdir = scratch.join("Hacked ROMs").join("Sega Genesis").join("SD");
        std::fs::create_dir_all(&gdir).unwrap();
        std::fs::write(gdir.join("Shadow Dancer.bin"), b"rom").unwrap();
        let sg = mk_game(
            "Shadow Dancer",
            &gdir.join("Shadow Dancer.bin").to_string_lossy(),
            "Sega Genesis",
        );
        std::env::set_var("EMUFRONT_DIR", &scratch);
        assert!(!game_has_msu(&sg));
        std::fs::write(gdir.join("Shadow Dancer.cue"), b"FILE").unwrap();
        assert!(game_has_msu(&sg));
        std::fs::remove_file(gdir.join("Shadow Dancer.cue")).unwrap();
        std::fs::write(gdir.join("track-1.pcm"), b"pcm").unwrap();
        assert!(game_has_msu(&sg));
        // a non-Genesis game with a matching cue but no audio
        // files does NOT trigger (cue rule is Genesis-only)
        let ng = mk_game(
            "Shadow Dancer",
            &gdir.join("Shadow Dancer.bin").to_string_lossy(),
            "Super Nintendo",
        );
        std::fs::remove_file(gdir.join("track-1.pcm")).unwrap();
        std::fs::write(gdir.join("Shadow Dancer.cue"), b"FILE").unwrap();
        assert!(!game_has_msu(&ng));
        std::env::remove_var("EMUFRONT_DIR");
        let _ = std::fs::remove_dir_all(&scratch);
        // archive.org doc filtering (Python's fake-docs shape):
        // 3 docs in -> Genesis keeps 2, SNES keeps 1
        let docs = vec![
            serde_json::json!({"title": "Shadow Dancer MSU-MD patch and music pack", "identifier": "sd1"}),
            serde_json::json!({"title": "Shadow Dancer MD+ edition", "identifier": "sd2"}),
            serde_json::json!({"title": "Shadow Dancer (World)", "identifier": "sd3"}),
        ];
        let kept_g = filter_archive_docs(&docs, true);
        assert_eq!(kept_g.len(), 2);
        assert_eq!(kept_g[0].url, "https://archive.org/details/sd1");
        assert_eq!(filter_archive_docs(&docs, false).len(), 1);
        assert!(archive_ident("https://archive.org/details/sd1").is_some());
        assert!(archive_ident("https://example.com/x").is_none());
        // archive.org item file selection -- the loose-file
        // fallback's decisions -- on a Shadow Dancer-shaped
        // metadata listing (the real track16_202603 shape):
        // one bundled zip, a .bps patch, a .cue and 19 WAVs,
        // bookkeeping originals, generated derivatives.
        let mut sd_files = vec![
            serde_json::json!({"name": "Shadow Dancer - The Secret of Shinobi (World) MSU-MD patch and arranged music pack.zip", "source": "original"}),
            serde_json::json!({"name": "Shadow Dancer - The Secret of Shinobi (World).bps", "source": "original"}),
            serde_json::json!({"name": "Shadow Dancer - The Secret of Shinobi (World).cue", "source": "original"}),
            serde_json::json!({"name": "track16_202603_files.xml", "source": "original"}),
            serde_json::json!({"name": "track16_202603_meta.xml", "source": "original"}),
            serde_json::json!({"name": "Shadow Dancer - The Secret of Shinobi (World) 01.mp3", "source": "derivative"}),
            serde_json::json!({"name": "Shadow Dancer - The Secret of Shinobi (World) 01.flac", "source": "derivative"}),
            serde_json::json!({"name": "__ia_thumb.jpg", "source": "derivative"}),
        ];
        for t in 1..=19 {
            sd_files.push(serde_json::json!({
                "name": format!("Shadow Dancer - The Secret of Shinobi (World) {t:02}.wav"),
                "source": "original"
            }));
        }
        let sd_names = original_file_names(&sd_files);
        // derivatives never count, even when their names look
        // like pack files; bookkeeping originals do list
        assert_eq!(sd_names.len(), 24);
        assert!(sd_names
            .iter()
            .all(|n| !n.ends_with(".mp3") && !n.ends_with(".flac") && !n.ends_with(".jpg")));
        assert_eq!(
            bundled_archive_name(&sd_names).map(|s| s.ends_with(".zip")),
            Some(true)
        );
        let sd_loose = loose_file_names(&sd_names);
        // exactly the patch + cue + the 19 WAVs: not the
        // archive, not the bookkeeping XML files
        assert_eq!(sd_loose.len(), 21);
        assert_eq!(sd_loose.iter().filter(|n| n.ends_with(".wav")).count(), 19);
        assert!(sd_loose.iter().any(|n| n.ends_with(".bps")));
        assert!(sd_loose.iter().any(|n| n.ends_with(".cue")));
        assert!(sd_loose
            .iter()
            .all(|n| !n.ends_with(".zip") && !n.ends_with(".xml")));
        // an item with no bundled archive: loose route only,
        // and a 0-byte-.msu-style music entry still qualifies
        let bare: Vec<String> = vec!["patch.ips".into(), "game.msu".into()];
        assert_eq!(bundled_archive_name(&bare), None);
        assert_eq!(loose_file_names(&bare).len(), 2);
        // the corrupt-archive trigger: an Unpacked with
        // nothing the chain can use is not a pack
        assert!(!unpack_has_content(&Unpacked::default()));
        let mut only_music = Unpacked::default();
        only_music.music.push(std::path::PathBuf::from("x.pcm"));
        assert!(unpack_has_content(&only_music));
        // URL resolution (urljoin slice)
        assert_eq!(
            resolve_url("https://archive.org/details/x", "/download/x/f.zip"),
            "https://archive.org/download/x/f.zip"
        );
        assert_eq!(
            resolve_url("https://example.com/a/b", "https://cdn.com/f.zip"),
            "https://cdn.com/f.zip"
        );
        assert_eq!(
            resolve_url("https://example.com/a/b", "f.zip"),
            "https://example.com/a/f.zip"
        );
        // Zeldix thread link extraction: MSU links preferred
        let page = "see https://files.example.com/plain.zip and \
                    https://files.example.com/chrono_msu1.zip thanks";
        assert_eq!(
            parse_zeldix_thread(page),
            vec!["https://files.example.com/chrono_msu1.zip".to_string()]
        );
        assert_eq!(
            parse_zeldix_thread("only https://files.example.com/patch.ips here"),
            vec!["https://files.example.com/patch.ips".to_string()]
        );
        // GitHub hunt source (v1.03): the search filter on a
        // fixture shaped like the real API response -- the
        // Phantasy Star II repo is the one the archive.org /
        // Plaza hunt missed (existence + shape verified live
        // Oct 24 2026 against api.github.com).
        let gh_items = vec![
            serde_json::json!({
                "name": "Phantasy-Star-II-MD-Plus",
                "full_name": "Infinest/Phantasy-Star-II-MD-Plus",
                "description": "MD+ Patch for Phantasy Star II",
                "html_url": "https://github.com/Infinest/Phantasy-Star-II-MD-Plus",
                "topics": ["68000", "cd-audio", "genesis", "md-plus", "megadrive", "romhacking", "sega"]
            }),
            serde_json::json!({
                "name": "Gunstar-Heroes-MD-Plus",
                "full_name": "ArcadeFan/Gunstar-Heroes-MD-Plus",
                "description": "MD+ patch for Gunstar Heroes",
                "html_url": "https://github.com/ArcadeFan/Gunstar-Heroes-MD-Plus",
                "topics": ["md-plus"]
            }),
            serde_json::json!({
                "name": "Phantasy-Star-II-Translation",
                "full_name": "SomeUser/Phantasy-Star-II-Translation",
                "description": "English translation project for Phantasy Star II",
                "html_url": "https://github.com/SomeUser/Phantasy-Star-II-Translation",
                "topics": ["romhacking"]
            }),
        ];
        // right game + marker wins; wrong game and no-marker lose
        let gh = filter_github_repos(&gh_items, "Phantasy Star II", true);
        assert_eq!(gh.len(), 1);
        assert_eq!(
            gh[0].url,
            "https://github.com/Infinest/Phantasy-Star-II-MD-Plus"
        );
        assert!(gh[0].title.contains("MD+") && gh[0].title.contains("GitHub"));
        // the same items under the SNES (msu-only) reading:
        // the MD+ repo is not an MSU-1 result
        assert!(filter_github_repos(&gh_items, "Phantasy Star II", false).is_empty());
        // SNES shape: an MSU-1 repo answers for its game
        let gh_snes = vec![serde_json::json!({
            "name": "Chrono-Trigger-MSU-1",
            "full_name": "SomeAuthor/Chrono-Trigger-MSU-1",
            "description": "MSU-1 patch for Chrono Trigger",
            "html_url": "https://github.com/SomeAuthor/Chrono-Trigger-MSU-1",
            "topics": ["msu-1", "snes"]
        })];
        let gh2 = filter_github_repos(&gh_snes, "Chrono Trigger", false);
        assert_eq!(gh2.len(), 1);
        assert!(gh2[0].title.contains("MSU-1"));
        // repo URL parsing
        assert_eq!(
            github_repo_of("https://github.com/Infinest/Phantasy-Star-II-MD-Plus"),
            Some((
                "Infinest".to_string(),
                "Phantasy-Star-II-MD-Plus".to_string()
            ))
        );
        assert_eq!(
            github_repo_of(
                "https://github.com/Infinest/Phantasy-Star-II-MD-Plus/releases"
            ),
            Some((
                "Infinest".to_string(),
                "Phantasy-Star-II-MD-Plus".to_string()
            ))
        );
        assert!(github_repo_of("https://github.com/onlyowner").is_none());
        assert!(github_repo_of("https://archive.org/details/sd1").is_none());
        // asset picking: the real Release 1.0 shape (a single
        // patch.ips asset) and a mixed pack-shaped release
        let rel1 = serde_json::json!({
            "tag_name": "1.0",
            "zipball_url": "https://api.github.com/repos/Infinest/Phantasy-Star-II-MD-Plus/zipball/1.0",
            "assets": [{
                "name": "patch.ips",
                "browser_download_url": "https://github.com/Infinest/Phantasy-Star-II-MD-Plus/releases/download/1.0/patch.ips"
            }]
        });
        assert_eq!(
            github_pick_assets(&rel1),
            vec![(
                "patch.ips".to_string(),
                "https://github.com/Infinest/Phantasy-Star-II-MD-Plus/releases/download/1.0/patch.ips"
                    .to_string()
            )]
        );
        let rel2 = serde_json::json!({
            "tag_name": "2.0",
            "zipball_url": "https://api.github.com/repos/X/Y/zipball/2.0",
            "assets": [
                {"name": "game.bps", "browser_download_url": "https://example.com/game.bps"},
                {"name": "soundtrack.zip", "browser_download_url": "https://example.com/soundtrack.zip"},
                {"name": "track03.ogg", "browser_download_url": "https://example.com/track03.ogg"},
                {"name": "notes.txt", "browser_download_url": "https://example.com/notes.txt"},
                {"name": "cover.png", "browser_download_url": "https://example.com/cover.png"}
            ]
        });
        let picked2: Vec<String> = github_pick_assets(&rel2)
            .iter()
            .map(|(n, _)| n.clone())
            .collect();
        assert_eq!(
            picked2,
            vec![
                "game.bps".to_string(),
                "soundtrack.zip".to_string(),
                "track03.ogg".to_string()
            ]
        );
        assert_eq!(
            github_release_zipball(&rel2).as_deref(),
            Some("https://api.github.com/repos/X/Y/zipball/2.0")
        );
        // a release with no usable asset: the fetch route's
        // repo-archive fallback is fed by these two facts
        let rel3 = serde_json::json!({
            "tag_name": "0.1",
            "zipball_url": "https://api.github.com/repos/X/Z/zipball/0.1",
            "assets": [{"name": "readme.txt", "browser_download_url": "https://example.com/readme.txt"}]
        });
        assert!(github_pick_assets(&rel3).is_empty());
        assert!(github_asset_ok("TRACK01.WAV") && github_asset_ok("patch.BPS"));
        assert!(!github_asset_ok("notes.txt") && !github_asset_ok("shot.png"));
        // Minerva hunt source (v1.03): the listing parser
        // and the pack matcher on rows shaped exactly like
        // the real collection listing (captured live from
        // minerva-archive.org Oct 24 2026; the .dat and
        // thumbnail rows are real non-pack rows of the
        // same listing and must be filtered out).
        let mv_html = r#"<div class="entry" data-name="md+ phantasy star ii (ps2 arranged).7z"><a href="/rom?id=2250802" draggable="false">MD+ Phantasy Star II (PS2 Arranged).7z</a> <span>116.87 MB</span></div>
<div class="entry" data-name="md+ outrun (arcade).7z"><a href="/rom?id=2250825" draggable="false">MD+ OutRun (Arcade).7z</a> <span>129.52 MB</span></div>
<div class="entry" data-name="md+ golden axe (arcade).7z"><a href="/rom?id=2250794" draggable="false">MD+ Golden Axe (Arcade).7z</a></div>
<div class="entry" data-name="md+ golden axe ii (arranged).7z"><a href="/rom?id=2250803" draggable="false">MD+ Golden Axe II (Arranged).7z</a></div>
<div class="entry" data-name="_complete sega md+ collection 2024.07.10.dat"><a href="/rom?id=2250793" draggable="false">_Complete Sega MD+ Collection 2024.07.10.dat</a></div>
<div class="entry" data-name="__ia_thumb.jpg"><a href="/rom?id=2250795" draggable="false">__ia_thumb.jpg</a></div>"#;
        let mv_rows = parse_minerva_listing(mv_html);
        assert_eq!(mv_rows.len(), 4);
        assert_eq!(
            mv_rows[0],
            (
                "MD+ Phantasy Star II (PS2 Arranged).7z".to_string(),
                "https://minerva-archive.org/rom?id=2250802".to_string()
            )
        );
        let mv_ps = filter_minerva_index(&mv_rows, "Phantasy Star II");
        assert_eq!(mv_ps.len(), 1);
        assert_eq!(
            mv_ps[0].title,
            "MD+ Phantasy Star II (PS2 Arranged).7z · Minerva"
        );
        assert_eq!(
            mv_ps[0].url,
            "https://minerva-archive.org/rom?id=2250802"
        );
        // the sequel guard: "Golden Axe" must not claim
        // the Golden Axe II pack, and the II hunt must
        // not claim the plain one
        let mv_ga = filter_minerva_index(&mv_rows, "Golden Axe");
        assert_eq!(mv_ga.len(), 1);
        assert!(mv_ga[0].title.contains("Golden Axe (Arcade)"));
        let mv_ga2 = filter_minerva_index(&mv_rows, "Golden Axe II");
        assert_eq!(mv_ga2.len(), 1);
        assert!(mv_ga2[0].title.contains("Golden Axe II"));
        assert_eq!(filter_minerva_index(&mv_rows, "OutRun").len(), 1);
        assert!(filter_minerva_index(&mv_rows, "Phantasy Star III").is_empty());
        assert_eq!(
            minerva_rom_of("https://minerva-archive.org/rom?id=2250802"),
            Some("2250802".to_string())
        );
        assert!(
            minerva_rom_of("https://github.com/Infinest/Phantasy-Star-II-MD-Plus").is_none()
        );
        assert!(minerva_rom_of("https://minerva-archive.org/browse/").is_none());
        // ---- Enhanced Games catalog (v1.03): matching
        // (Python's pinned cases), the sequel guard, the
        // per-system separation invariant, installed
        // detection, merge, GitHub slice, other-section
        // hint.
        let mk_entry = |title: &str, source: &str, kind: &str| CatalogEntry {
            title: title.to_string(),
            url: "https://example.invalid/x".to_string(),
            source: source.to_string(),
            kind: kind.to_string(),
        };
        // Python's own pinned match_catalog_entry cases
        let ff2 = mk_game(
            "Final Fantasy II",
            "C:\\roms\\Final Fantasy II (USA).sfc",
            "Super Nintendo",
        );
        let ff2_prep = catalog_game_prep(&ff2);
        assert!(catalog_entry_matches(
            "Final Fantasy IV MSU-1 - Pixel Remaster PCM Pack",
            &ff2_prep
        ));
        let act = mk_game("ActRaiser", "C:\\roms\\ActRaiser.sfc", "Super Nintendo");
        assert!(catalog_entry_matches("ActRaiser", &catalog_game_prep(&act)));
        let sd = mk_game(
            "Shadow Dancer - The Secret of Shinobi",
            "D:\\roms\\Shadow Dancer.md",
            "Sega Genesis",
        );
        assert!(catalog_entry_matches(
            "Shadow Dancer - The Secret of Shinobi (World) MSU-MD patch and arranged music pack",
            &catalog_game_prep(&sd)
        ));
        assert!(!catalog_entry_matches("No Such Game MSU-1", &ff2_prep));
        // sequel guard: cores nest ("finalfantasyiii"
        // contains "finalfantasyii") — numbers must agree
        assert!(!catalog_entry_matches(
            "Final Fantasy III MSU-1 pack",
            &ff2_prep
        ));
        let ga = mk_game("Golden Axe", "D:\\roms\\Golden Axe.bin", "Sega Genesis");
        let ga2 = mk_game(
            "Golden Axe II",
            "D:\\roms\\Golden Axe II.bin",
            "Sega Genesis",
        );
        assert!(catalog_entry_matches(
            "MD+ Golden Axe (Arcade)",
            &catalog_game_prep(&ga)
        ));
        assert!(!catalog_entry_matches(
            "MD+ Golden Axe (Arcade)",
            &catalog_game_prep(&ga2)
        ));
        assert!(catalog_entry_matches(
            "MD+ Golden Axe II",
            &catalog_game_prep(&ga2)
        ));
        assert!(!catalog_entry_matches(
            "MD+ Golden Axe II",
            &catalog_game_prep(&ga)
        ));
        // merge: first occurrence of a title wins, sorted
        let merged = merge_catalog_entries(vec![
            vec![mk_entry("Beta MSU-1 pack", "archive.org", "MSU-1 pack")],
            vec![
                mk_entry("beta msu-1 pack", "GitHub", "MSU-1 patch"),
                mk_entry("Alpha MSU-1 pack", "archive.org", "MSU-1 pack"),
            ],
        ]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].title, "Alpha MSU-1 pack");
        assert_eq!(merged[1].source, "archive.org");
        // separation invariant: build each system's rows
        // with BOTH systems' games and entries on the
        // table — nothing crosses over
        let mixed_games = vec![ff2.clone(), sd.clone()];
        let mixed_entries = vec![
            mk_entry("Final Fantasy IV MSU-1 pack", "archive.org", "MSU-1 pack"),
            mk_entry(
                "Shadow Dancer - The Secret of Shinobi (World) MSU-MD patch and arranged music pack",
                "archive.org",
                "MD+ pack",
            ),
        ];
        let snes_rows = build_catalog_rows(
            "Super Nintendo",
            &mixed_entries,
            &mixed_games,
            &[],
            "C:\\stage",
        );
        assert!(snes_rows[0].game.as_ref().map(|g| &g.name)
            == Some(&ff2.name));
        assert!(snes_rows[1].game.is_none());
        let gen_rows = build_catalog_rows(
            "Sega Genesis",
            &mixed_entries,
            &mixed_games,
            &[],
            "C:\\stage",
        );
        // matched-first ordering: the Genesis catalog's
        // own matched row leads, the Nintendo row trails
        // unmatched — and vice versa above
        assert!(gen_rows[0].game.as_ref().map(|g| &g.name)
            == Some(&sd.name));
        assert!(gen_rows[1].game.is_none());
        assert!(snes_rows[0].game.is_some() && gen_rows[0].game.is_some());
        // installed, rule A: Python's "<game> [<tag>]"
        // manual name (the Shadow Dancer build's shape)
        let sd_manual = ManualRef {
            name: "Shadow Dancer - The Secret of Shinobi [Shadow Dancer - The Secret of Shinobi (World) MSU-MD patch and arranged music pack]".to_string(),
            system: "Sega Genesis".to_string(),
            path: "C:\\stage\\EmuFront\\Hacked ROMs\\Sega Genesis\\Shadow Dancer - The Secret of Shinobi\\Shadow Dancer - The Secret of Shinobi.md".to_string(),
        };
        let sd_rows = build_catalog_rows(
            "Sega Genesis",
            &mixed_entries[1..],
            &mixed_games,
            &[sd_manual],
            "C:\\stage\\EmuFront",
        );
        assert_eq!(sd_rows.len(), 1);
        assert!(sd_rows[0].installed.is_some());
        assert!(sd_rows[0]
            .installed
            .as_ref()
            .map(|g| g.name.contains("["))
            .unwrap_or(false));
        // other-section hint: one game, a pack AND a patch
        let ff_entries = vec![
            mk_entry("Final Fantasy IV MSU-1 pack", "archive.org", "MSU-1 pack"),
            mk_entry("Final Fantasy IV", "Zeldix", "MSU-1 patch"),
        ];
        let ff_rows = build_catalog_rows(
            "Super Nintendo",
            &ff_entries,
            &[ff2.clone()],
            &[],
            "C:\\stage",
        );
        assert_eq!(ff_rows.len(), 2);
        assert!(ff_rows.iter().all(|r| r.also_other));
        // GitHub catalog slice: marker rule per system
        let gh_items = vec![
            serde_json::json!({
                "name": "Phantasy-Star-II-MD-Plus",
                "full_name": "Infinest/Phantasy-Star-II-MD-Plus",
                "description": "Phantasy Star II MD+ patch",
                "html_url": "https://github.com/Infinest/Phantasy-Star-II-MD-Plus",
                "topics": ["md-plus"]
            }),
            serde_json::json!({
                "name": "some-translation",
                "full_name": "x/some-translation",
                "description": "English translation patch",
                "html_url": "https://github.com/x/some-translation",
                "topics": []
            }),
            serde_json::json!({
                "name": "chrono-msu1",
                "full_name": "y/chrono-msu1",
                "description": "MSU-1 patch",
                "html_url": "https://github.com/y/chrono-msu1",
                "topics": ["msu-1"]
            }),
        ];
        let gh_gen = filter_github_catalog(&gh_items, true);
        assert_eq!(gh_gen.len(), 1);
        assert_eq!(gh_gen[0].title, "Infinest/Phantasy-Star-II-MD-Plus");
        assert_eq!(gh_gen[0].kind, "MD+ patch");
        let gh_snes = filter_github_catalog(&gh_items, false);
        assert_eq!(gh_snes.len(), 1);
        assert_eq!(gh_snes[0].kind, "MSU-1 patch");
        // installed, rule B: a pack install registered
        // under the ROM's own name, proven by the staged
        // audio in its folder (sandboxed data dir)
        {
            let scratch = std::env::temp_dir()
                .join(format!("efr-selftest-cat-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&scratch);
            let folder = scratch
                .join("Hacked ROMs")
                .join("Sega Genesis")
                .join("Golden Axe");
            std::fs::create_dir_all(&folder).unwrap();
            std::fs::write(folder.join("Golden Axe.md"), b"rom").unwrap();
            std::fs::write(folder.join("Golden Axe.cue"), b"cue").unwrap();
            std::env::set_var("EMUFRONT_DIR", &scratch);
            let prefix = scratch.to_string_lossy().to_string();
            let ga_manual = ManualRef {
                name: "Golden Axe".to_string(),
                system: "Sega Genesis".to_string(),
                path: folder
                    .join("Golden Axe.md")
                    .to_string_lossy()
                    .to_string(),
            };
            let ga_rows = build_catalog_rows(
                "Sega Genesis",
                &[mk_entry("MD+ Golden Axe", "Minerva", "MD+ pack")],
                &[ga.clone()],
                &[ga_manual],
                &prefix,
            );
            assert_eq!(ga_rows.len(), 1);
            assert!(ga_rows[0].installed.is_some());
            std::env::remove_var("EMUFRONT_DIR");
            let _ = std::fs::remove_dir_all(&scratch);
        }
        // ---- Emulators tab (v1.03, fourth row v1.04):
        // the spec list is the user's three emulators plus
        // AmigaVision; profile resolution is
        // standalone-only (a core profile's path is
        // RetroArch, not the emulator), preferred system
        // first; the precedence table is manual pick >
        // profile, with stale paths (file gone) falling
        // through to the next source and finally to the
        // Select state; picks must be existing .exe
        // files; the settings shape round-trips.
        let mk_profile = |name: &str, path: &str, core: &str, system: &str| Profile {
            name: name.to_string(),
            path: path.to_string(),
            core: core.to_string(),
            system: system.to_string(),
            args: String::new(),
        };
        assert_eq!(EMU_TAB_SPECS.len(), 4);
        assert_eq!(
            EMU_TAB_SPECS.iter().map(|s| s.key).collect::<Vec<_>>(),
            vec!["PCSX2", "Dolphin", "DuckStation", "AmigaVision"]
        );
        // only the AmigaVision row carries a launch
        // argument; the original three stay arg-free
        assert!(EMU_TAB_SPECS[..3].iter().all(|s| s.arg_game.is_none()));
        assert_eq!(
            EMU_TAB_SPECS[3].arg_game,
            Some("AmigaVision (AGS)")
        );
        let tab_profiles = vec![
            // a RetroArch core profile must NOT be taken
            // for the standalone emulator itself
            mk_profile(
                "RetroArch (PCSX2 core)",
                "C:\\emu\\retroarch.exe",
                "C:\\cores\\pcsx2_libretro.dll",
                "Sony PlayStation 2",
            ),
            mk_profile("PCSX2", "C:\\emu\\pcsx2\\pcsx2.exe", "", "Sony PlayStation 2"),
            mk_profile(
                "Dolphin",
                "C:\\emu\\dolphin\\Dolphin.exe",
                "",
                "Nintendo GameCube",
            ),
            mk_profile(
                "Dolphin",
                "C:\\emu\\dolphin-wii\\Dolphin.exe",
                "",
                "Nintendo Wii",
            ),
            mk_profile(
                "DuckStation",
                "C:\\emu\\ps1duck\\duckstation.exe",
                "",
                "Sony PlayStation",
            ),
        ];
        let all_exist = |_: &str| true;
        let none_exist = |_: &str| false;
        let only = |keep: &'static str| move |p: &str| p == keep;
        // profile resolution: standalone only; Dolphin's
        // paths come GameCube first, Wii second
        assert_eq!(
            emu_tab_profile_paths(&tab_profiles, &EMU_TAB_SPECS[0]),
            vec!["C:\\emu\\pcsx2\\pcsx2.exe".to_string()]
        );
        assert_eq!(
            emu_tab_profile_paths(&tab_profiles, &EMU_TAB_SPECS[1]),
            vec![
                "C:\\emu\\dolphin\\Dolphin.exe".to_string(),
                "C:\\emu\\dolphin-wii\\Dolphin.exe".to_string()
            ]
        );
        assert_eq!(
            emu_tab_profile_paths(&tab_profiles, &EMU_TAB_SPECS[2]),
            vec!["C:\\emu\\ps1duck\\duckstation.exe".to_string()]
        );
        assert!(emu_tab_profile_paths(&[], &EMU_TAB_SPECS[0]).is_empty());
        // precedence: a live manual pick wins over profiles
        assert_eq!(
            resolve_emu_tab(
                &EMU_TAB_SPECS[0],
                &tab_profiles,
                Some("D:\\mine\\pcsx2.exe"),
                &all_exist
            ),
            EmuTabState::Ready {
                path: "D:\\mine\\pcsx2.exe".to_string(),
                manual: true
            }
        );
        // a blank pick counts as no pick at all
        assert_eq!(
            resolve_emu_tab(&EMU_TAB_SPECS[0], &tab_profiles, Some("  "), &all_exist),
            EmuTabState::Ready {
                path: "C:\\emu\\pcsx2\\pcsx2.exe".to_string(),
                manual: false
            }
        );
        // no pick -> the profile's path
        assert_eq!(
            resolve_emu_tab(&EMU_TAB_SPECS[1], &tab_profiles, None, &all_exist),
            EmuTabState::Ready {
                path: "C:\\emu\\dolphin\\Dolphin.exe".to_string(),
                manual: false
            }
        );
        // a deleted GameCube Dolphin exe falls through to
        // the Wii profile's path
        assert_eq!(
            resolve_emu_tab(
                &EMU_TAB_SPECS[1],
                &tab_profiles,
                None,
                &only("C:\\emu\\dolphin-wii\\Dolphin.exe")
            ),
            EmuTabState::Ready {
                path: "C:\\emu\\dolphin-wii\\Dolphin.exe".to_string(),
                manual: false
            }
        );
        // a stale manual pick falls back to a live profile
        assert_eq!(
            resolve_emu_tab(
                &EMU_TAB_SPECS[0],
                &tab_profiles,
                Some("D:\\gone\\pcsx2.exe"),
                &only("C:\\emu\\pcsx2\\pcsx2.exe")
            ),
            EmuTabState::Ready {
                path: "C:\\emu\\pcsx2\\pcsx2.exe".to_string(),
                manual: false
            }
        );
        // everything gone -> Missing, naming his stale pick
        assert_eq!(
            resolve_emu_tab(
                &EMU_TAB_SPECS[2],
                &tab_profiles,
                Some("D:\\gone\\duck.exe"),
                &none_exist
            ),
            EmuTabState::Missing {
                stale: Some("D:\\gone\\duck.exe".to_string())
            }
        );
        // a profile whose file was deleted -> Missing
        // naming the profile's path
        assert_eq!(
            resolve_emu_tab(&EMU_TAB_SPECS[1], &tab_profiles, None, &none_exist),
            EmuTabState::Missing {
                stale: Some("C:\\emu\\dolphin\\Dolphin.exe".to_string())
            }
        );
        // no profiles and no pick -> Missing, nothing stale
        assert_eq!(
            resolve_emu_tab(&EMU_TAB_SPECS[0], &[], None, &all_exist),
            EmuTabState::Missing { stale: None }
        );
        // pick validation: existing .exe only (any case)
        assert!(emu_pick_ok("C:\\emu\\Dolphin.EXE", &all_exist));
        assert!(!emu_pick_ok("C:\\emu\\dolphin.txt", &all_exist));
        assert!(!emu_pick_ok("C:\\emu\\Dolphin.exe", &none_exist));
        assert!(!emu_pick_ok("  ", &all_exist));
        // persistence shape: the overrides serialize as a
        // plain {key: path} object and parse back; blanks
        // are dropped on both sides of the round-trip
        let mut ov = BTreeMap::new();
        ov.insert("PCSX2".to_string(), "D:\\mine\\pcsx2.exe".to_string());
        ov.insert("Dolphin".to_string(), "  ".to_string());
        let ov_doc = serde_json::json!({ "emu_overrides": emu_overrides_json(&ov) });
        let ov_back = parse_emu_overrides(&ov_doc);
        assert_eq!(ov_back.len(), 1);
        assert_eq!(
            ov_back.get("PCSX2").map(|s| s.as_str()),
            Some("D:\\mine\\pcsx2.exe")
        );
        assert!(parse_emu_overrides(&serde_json::json!({})).is_empty());
        // ---- v1.04 (perf Slice 1 + AmigaVision row).
        // LRU ledger: eviction is least-recently-used
        // first, a read counts as use, the byte
        // accounting is exact. Search index: the
        // precomputed filter returns exactly the old
        // per-keystroke lowercase scan's set, and the
        // precomputed keys order identically. Art queue:
        // visible work before prefetch, a key queues
        // once, stale generations drop. Session cache:
        // keyed lookups; only non-empty results stored.
        // Cover budgets pin the design's three presets.
        // AmigaVision: resolution, override precedence,
        // template expansion (incl. the empty-template
        // bare path), Blocked when the entry is gone.
        {
            let mut led = TexLedger::new(1000);
            assert!(led.insert("a", 400).is_empty());
            assert!(led.insert("b", 400).is_empty());
            assert_eq!(led.insert("c", 400), vec!["a".to_string()]);
            assert_eq!(led.bytes, 800);
            led.touch("b");
            assert_eq!(led.insert("d", 400), vec!["c".to_string()]);
            assert!(led.contains("b") && led.contains("d"));
            // a single oversized cover is never evicted
            // by its own insert
            let mut big = TexLedger::new(100);
            assert!(big.insert("huge", 5000).is_empty());
            assert!(big.contains("huge"));
            assert_eq!(big.bytes, 5000);
            // re-insert replaces the recorded size
            let mut re = TexLedger::new(1000);
            re.insert("x", 300);
            re.insert("x", 100);
            assert_eq!(re.bytes, 100);
            // a budget cut evicts LRU-first down to size
            let mut cut = TexLedger::new(1000);
            cut.insert("p", 400);
            cut.insert("q", 400);
            cut.budget = 500;
            assert_eq!(cut.evict_over_budget(), vec!["p".to_string()]);
            assert_eq!(cut.bytes, 400);
            // the worker-count clamp (design: 2–4)
            assert_eq!(art_worker_count(1), 2);
            assert_eq!(art_worker_count(2), 2);
            assert_eq!(art_worker_count(3), 3);
            assert_eq!(art_worker_count(16), 4);
            // cover budgets (design §2.3 presets)
            assert_eq!(cover_budget_bytes("low"), 128 * 1024 * 1024);
            assert_eq!(cover_budget_bytes("standard"), 384 * 1024 * 1024);
            assert_eq!(cover_budget_bytes("high"), 1024 * 1024 * 1024);
            assert_eq!(
                cover_budget_bytes("anything-else"),
                384 * 1024 * 1024
            );
            // decode round-trip: a real PNG in, wall-
            // sized pixels out; garbage is an error
            let mut png = Vec::new();
            {
                let img = image::RgbImage::from_pixel(
                    440,
                    600,
                    image::Rgb([200, 40, 40]),
                );
                image::DynamicImage::ImageRgb8(img)
                    .write_to(
                        &mut std::io::Cursor::new(&mut png),
                        image::ImageFormat::Png,
                    )
                    .expect("encode fixture png");
            }
            let decoded = decode_cover(&png).expect("fixture png decodes");
            assert_eq!(decoded.size, [220, 300]);
            assert!(decode_cover(b"not an image").is_err());
        }
        {
            let job = |key: &str, gen: u64| ArtJob {
                key: key.to_string(),
                game: mk_game(key, "C:\\x", "Sys"),
                gen,
            };
            let mut q = ArtQueue::default();
            assert!(q.push(job("low1", 1), false));
            assert!(q.push(job("hi1", 1), true));
            // a key already queued is refused
            assert!(!q.push(job("hi1", 1), true));
            // high priority pops before low
            assert_eq!(q.pop(1).expect("hi first").key, "hi1");
            assert_eq!(q.pop(1).expect("then low").key, "low1");
            assert!(q.pop(1).is_none());
            // stale-generation jobs are dropped on pop
            let mut q2 = ArtQueue::default();
            q2.push(job("old", 1), true);
            q2.push(job("new", 2), false);
            assert_eq!(q2.pop(2).expect("stale skipped").key, "new");
            assert!(q2.pop(2).is_none());
            // purge keeps only the current generation
            let mut q3 = ArtQueue::default();
            q3.push(job("a", 1), true);
            q3.push(job("b", 2), true);
            q3.purge(2);
            assert_eq!(q3.pop(2).expect("b survives purge").key, "b");
            assert!(q3.pop(2).is_none());
        }
        {
            let names = [
                "Zelda II",
                "zelda",
                "METROID",
                "Super Metroid (USA)",
                "F-Zero",
                "1942",
                "Éclair Quest",
            ];
            let fixtures: Vec<Game> = names
                .iter()
                .map(|n| mk_game(n, "C:\\x", "Sys"))
                .collect();
            let (lower, keys) = build_search_index(&fixtures);
            for q_raw in ["", "zelda", "METROID", "super", "194", "éclair", "zzz"] {
                let q = q_raw.to_lowercase();
                let via_index: Vec<&str> = fixtures
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| wall_game_matches(&lower[*i], &q))
                    .map(|(_, g)| g.name.as_str())
                    .collect();
                let naive: Vec<&str> = fixtures
                    .iter()
                    .filter(|g| {
                        q.is_empty() || g.name.to_lowercase().contains(&q)
                    })
                    .map(|g| g.name.as_str())
                    .collect();
                assert_eq!(via_index, naive, "query {q_raw:?}");
            }
            // sort parity: precomputed keys order exactly
            // like the old per-comparison sort_key calls
            let mut by_index: Vec<usize> = (0..fixtures.len()).collect();
            by_index.sort_by(|a, b| {
                keys[*a]
                    .cmp(&keys[*b])
                    .then(fixtures[*a].name.cmp(&fixtures[*b].name))
            });
            let mut by_naive: Vec<usize> = (0..fixtures.len()).collect();
            by_naive.sort_by(|a, b| {
                sort_key(&fixtures[*a].name)
                    .cmp(&sort_key(&fixtures[*b].name))
                    .then(fixtures[*a].name.cmp(&fixtures[*b].name))
            });
            assert_eq!(by_index, by_naive);
        }
        {
            let cache: std::sync::OnceLock<
                std::sync::Mutex<HashMap<String, Vec<String>>>,
            > = std::sync::OnceLock::new();
            assert!(session_cache_lookup(&cache, "k").is_none());
            // an empty result is NOT stored — it may be
            // a transient failure; the next ask retries
            session_cache_store(&cache, "k".to_string(), &[]);
            assert!(session_cache_lookup(&cache, "k").is_none());
            session_cache_store(&cache, "k".to_string(), &["hit".to_string()]);
            assert_eq!(
                session_cache_lookup(&cache, "k"),
                Some(vec!["hit".to_string()])
            );
            assert!(session_cache_lookup(&cache, "other").is_none());
        }
        {
            let amiga_spec = &EMU_TAB_SPECS[3];
            let amiberry = |args: &str| Profile {
                name: "Amiberry (AmigaVision)".to_string(),
                path: "C:\\emu\\AmigaVision\\amiberry.exe".to_string(),
                core: String::new(),
                system: "Commodore Amiga".to_string(),
                args: args.to_string(),
            };
            let ags = mk_game(
                "AmigaVision (AGS)",
                "C:\\emu\\AmigaVision\\AmigaVision.uae",
                "Commodore Amiga",
            );
            let games = vec![ags];
            // resolves through the profile; the empty
            // template means the bare .uae path is the arg
            let (st, args) = resolve_emu_tab_row(
                amiga_spec,
                &[amiberry("")],
                &games,
                None,
                &all_exist,
            );
            assert_eq!(
                st,
                EmuTabState::Ready {
                    path: "C:\\emu\\AmigaVision\\amiberry.exe".to_string(),
                    manual: false
                }
            );
            assert_eq!(
                args,
                vec!["C:\\emu\\AmigaVision\\AmigaVision.uae".to_string()]
            );
            // a {rom} template expands in place
            let (_, args) = resolve_emu_tab_row(
                amiga_spec,
                &[amiberry("-f {rom}")],
                &games,
                None,
                &all_exist,
            );
            assert_eq!(
                args,
                vec![
                    "-f".to_string(),
                    "C:\\emu\\AmigaVision\\AmigaVision.uae".to_string()
                ]
            );
            // a template that never names the file gets
            // the path appended bare
            let (_, args) = resolve_emu_tab_row(
                amiga_spec,
                &[amiberry("--fullscreen")],
                &games,
                None,
                &all_exist,
            );
            assert_eq!(
                args,
                vec![
                    "--fullscreen".to_string(),
                    "C:\\emu\\AmigaVision\\AmigaVision.uae".to_string()
                ]
            );
            // a manual pick wins for the exe; with no
            // profile at that path the argument is the
            // bare path (the install-proven form)
            let (st, args) = resolve_emu_tab_row(
                amiga_spec,
                &[amiberry("-f {rom}")],
                &games,
                Some("D:\\mine\\amiberry.exe"),
                &all_exist,
            );
            assert_eq!(
                st,
                EmuTabState::Ready {
                    path: "D:\\mine\\amiberry.exe".to_string(),
                    manual: true
                }
            );
            assert_eq!(
                args,
                vec!["C:\\emu\\AmigaVision\\AmigaVision.uae".to_string()]
            );
            // a manual pick AT the profile's own path
            // keeps that profile's template
            let (_, args) = resolve_emu_tab_row(
                amiga_spec,
                &[amiberry("-f {rom}")],
                &games,
                Some("C:\\emu\\AmigaVision\\amiberry.exe"),
                &all_exist,
            );
            assert_eq!(
                args,
                vec![
                    "-f".to_string(),
                    "C:\\emu\\AmigaVision\\AmigaVision.uae".to_string()
                ]
            );
            // the library entry is gone -> Blocked, the
            // reason names the entry, and there is no Run
            let (st, args) = resolve_emu_tab_row(
                amiga_spec,
                &[amiberry("")],
                &[],
                None,
                &all_exist,
            );
            match st {
                EmuTabState::Blocked { reason } => {
                    assert!(reason.contains("AmigaVision (AGS)"))
                }
                other => panic!("expected Blocked, got {other:?}"),
            }
            assert!(args.is_empty());
            // the exe missing entirely -> the Missing /
            // Select flow stands even though the entry
            // exists
            let (st, _) = resolve_emu_tab_row(
                amiga_spec,
                &[amiberry("")],
                &games,
                None,
                &none_exist,
            );
            assert!(matches!(st, EmuTabState::Missing { .. }));
            // the original rows stay arg-free through the
            // row resolver too
            let (st, args) = resolve_emu_tab_row(
                &EMU_TAB_SPECS[0],
                &tab_profiles,
                &games,
                None,
                &all_exist,
            );
            assert!(matches!(st, EmuTabState::Ready { .. }));
            assert!(args.is_empty());
        }
        // ============ v1.05 ============
        // -- typed config parse vs the legacy DOM walk:
        //    the same fixtures through both parsers must
        //    produce identical libraries (the old lenient
        //    semantics, pinned field by field).
        {
            let same = |a: &(Vec<Game>, Vec<Profile>),
                        b: &(Vec<Game>, Vec<Profile>)| {
                assert_eq!(a.0.len(), b.0.len(), "game count");
                for (x, y) in a.0.iter().zip(b.0.iter()) {
                    assert_eq!(x.name, y.name);
                    assert_eq!(x.path, y.path);
                    assert_eq!(x.system, y.system);
                    assert_eq!(x.emulator, y.emulator);
                    assert_eq!(x.emulator_locked, y.emulator_locked);
                    assert_eq!(x.patch, y.patch);
                    assert_eq!(x.cheats_text, y.cheats_text);
                    assert_eq!(x.cheats_enabled, y.cheats_enabled);
                }
                assert_eq!(a.1.len(), b.1.len(), "profile count");
                for (x, y) in a.1.iter().zip(b.1.iter()) {
                    assert_eq!(x.name, y.name);
                    assert_eq!(x.path, y.path);
                    assert_eq!(x.core, y.core);
                    assert_eq!(x.system, y.system);
                    assert_eq!(x.args, y.args);
                }
            };
            let fixtures: Vec<&str> = vec![
                // a well-formed mini config
                r#"{"games":[{"name":"Chrono Trigger","path":"C:\\roms\\ct.sfc","system":"Super Nintendo","emulator":"RetroArch (snes9x)","emulator_locked":true,"patch":"","cheats_text":"abc","cheats_enabled":true}],"emulators":[{"name":"RetroArch (snes9x)","path":"C:\\emu\\RetroArch\\retroarch.exe","core":"snes9x","system":"Super Nintendo","args":"-L {core} {rom}"}],"library_dirs":["C:\\roms"]}"#,
                // missing fields default; unknown fields ignored
                r#"{"games":[{"name":"Bare"}],"emulators":[{}],"extra_top":{"x":1}}"#,
                // wrong types read as "" / false
                r#"{"games":[{"name":42,"path":true,"system":null,"emulator":["x"],"emulator_locked":"yes","patch":{"a":1},"cheats_text":3.5,"cheats_enabled":1}],"emulators":[{"name":7}]}"#,
                // sections that aren't arrays read as empty
                r#"{"games":{"name":"x"},"emulators":"nope"}"#,
                // a non-object entry is a default entry
                r#"{"games":["just a string",{"name":"Real"}],"emulators":[42]}"#,
                // duplicate keys: the last one wins
                r#"{"games":[{"name":"First","name":"Second"}]}"#,
                // no library sections at all
                r#"{"library_dirs":[]}"#,
                // not JSON at all
                "this is not json",
                // JSON that isn't an object at the top
                r#"[{"name":"in an array"}]"#,
                "null",
                "",
            ];
            for fx in &fixtures {
                let typed = parse_config_bytes(fx.as_bytes());
                let legacy = legacy_parse_str(fx);
                same(&typed, &legacy);
            }
            // spot-pin the interesting readings
            let (g, _) = parse_config_bytes(fixtures[2].as_bytes());
            assert_eq!(g.len(), 1);
            assert_eq!(g[0].name, "");
            assert!(!g[0].emulator_locked);
            assert!(!g[0].cheats_enabled);
            let (g, _) = parse_config_bytes(fixtures[4].as_bytes());
            assert_eq!(g.len(), 2);
            assert_eq!(g[0].name, "");
            assert_eq!(g[1].name, "Real");
            let (g, _) = parse_config_bytes(fixtures[5].as_bytes());
            assert_eq!(g[0].name, "Second");
        }
        // -- snapshot: round-trip, tamper rejection, the
        //    fingerprint tier table, and enter_wall's
        //    ordering parity against the derived lists
        {
            let games = vec![
                mk_game("Zelda", "C:\\roms\\zelda.sfc", "Super Nintendo"),
                mk_game("asteroids", "C:\\roms\\ast.bin", "Atari 2600"),
                mk_game("Sonic", "C:\\roms\\sonic.bin", "Sega Genesis"),
                mk_game(
                    "Hacked",
                    // under the resolved data dir, wherever
                    // this machine keeps it — the invariant
                    // the star flag actually tests
                    &format!("{}\\Hacked ROMs\\h.sfc", data_dir_str()),
                    "Super Nintendo",
                ),
            ];
            let profiles = vec![Profile {
                name: "P".into(),
                path: "p".into(),
                core: "c".into(),
                system: "Sega Genesis".into(),
                args: "a".into(),
            }];
            let derived = build_derived(&games);
            let fp = CfgFingerprint {
                size: 123,
                mtime_secs: 456,
                mtime_nanos: 7,
                sha1: "abc".into(),
            };
            let snap = build_snapshot(&games, &profiles, &derived, &fp);
            assert!(snap_valid(&snap));
            let text = serde_json::to_string(&snap).expect("snap ser");
            let back: SnapFile =
                serde_json::from_str(&text).expect("snap de");
            assert!(snap_valid(&back));
            let (g2, p2, d2) = snap_into_parts(back);
            assert_eq!(g2.len(), games.len());
            for (x, y) in games.iter().zip(g2.iter()) {
                assert_eq!(x.name, y.name);
                assert_eq!(x.path, y.path);
                assert_eq!(x.system, y.system);
            }
            assert_eq!(p2.len(), 1);
            assert_eq!(d2.lower_names, derived.lower_names);
            assert_eq!(d2.sort_keys, derived.sort_keys);
            assert_eq!(d2.art_keys, derived.art_keys);
            assert_eq!(d2.star, derived.star);
            assert_eq!(d2.systems, derived.systems);
            assert_eq!(d2.system_games, derived.system_games);
            assert_eq!(d2.system_buckets, derived.system_buckets);
            // the data-dir-prefixed game is the shelf member
            assert!(d2.star[3]);
            assert!(!d2.star[0]);
            // bucket membership agrees with bucket_counts
            assert_eq!(
                derived.bucket_counts("Atari 2600"),
                Some(bucket_counts(&games, "Atari 2600"))
            );
            // ordering parity: the derived per-system list
            // is enter_wall's comparator applied once
            let mut naive: Vec<usize> = games
                .iter()
                .enumerate()
                .filter(|(_, g)| g.system == "Super Nintendo")
                .map(|(i, _)| i)
                .collect();
            naive.sort_by(|a, b| {
                derived.sort_keys[*a]
                    .cmp(&derived.sort_keys[*b])
                    .then(games[*a].name.cmp(&games[*b].name))
            });
            assert_eq!(derived.system_list("Super Nintendo"), Some(&naive));
            // corruption: a truncated file never parses
            let truncated = &text[..text.len() / 2];
            assert!(serde_json::from_str::<SnapFile>(truncated).is_err());
            // tampered shapes fail the sanity gate
            let mut bad: SnapFile =
                serde_json::from_str(&text).unwrap();
            bad.magic = "nope".into();
            assert!(!snap_valid(&bad));
            let mut bad2: SnapFile =
                serde_json::from_str(&text).unwrap();
            bad2.lower.pop();
            assert!(!snap_valid(&bad2));
            let mut bad3: SnapFile =
                serde_json::from_str(&text).unwrap();
            bad3.sysgames[0].1.push(9999);
            assert!(!snap_valid(&bad3));
            // the fingerprint tier table
            assert_eq!(snap_tier(true, false), SnapTier::Use);
            assert_eq!(snap_tier(true, true), SnapTier::Use);
            assert_eq!(snap_tier(false, true), SnapTier::UseRefreshStat);
            assert_eq!(snap_tier(false, false), SnapTier::Rebuild);
        }
        // -- config write model: apply semantics per
        //    mutation shape + the guard's decision table
        {
            let mut cfg = serde_json::json!({
                "games": [
                    {"name": "A", "path": "p1", "emulator": "", "emulator_locked": false},
                    {"name": "B", "path": "p2", "emulator": "", "emulator_locked": false}
                ],
                "manual_games": [
                    {"name": "A", "path": "p1", "emulator": "", "emulator_locked": false}
                ]
            });
            CfgMutation::SetGameEmulator {
                path: "p1".into(),
                name: "Dolphin".into(),
            }
            .apply(&mut cfg)
            .expect("emu apply");
            assert_eq!(cfg["games"][0]["emulator"], "Dolphin");
            assert_eq!(cfg["games"][0]["emulator_locked"], true);
            assert_eq!(cfg["manual_games"][0]["emulator"], "Dolphin");
            assert_eq!(cfg["games"][1]["emulator"], "");
            CfgMutation::SetGameEmulator {
                path: "p1".into(),
                name: "".into(),
            }
            .apply(&mut cfg)
            .unwrap();
            assert_eq!(cfg["games"][0]["emulator_locked"], false);
            CfgMutation::SetGameField {
                path: "p2".into(),
                key: "cheats_text".into(),
                val: serde_json::json!("xyz"),
            }
            .apply(&mut cfg)
            .unwrap();
            assert_eq!(cfg["games"][1]["cheats_text"], "xyz");
            let entry = serde_json::json!({
                "name": "M", "path": "pm", "system": "PC Ports"
            });
            CfgMutation::RegisterManual {
                entry: entry.clone(),
            }
            .apply(&mut cfg)
            .unwrap();
            assert_eq!(cfg["games"].as_array().unwrap().len(), 3);
            assert_eq!(cfg["manual_games"].as_array().unwrap().len(), 2);
            // the same path again: a no-op (dedupe by path)
            CfgMutation::RegisterManual { entry }
                .apply(&mut cfg)
                .unwrap();
            assert_eq!(cfg["games"].as_array().unwrap().len(), 3);
            // a config without the lists errors, with the
            // old helper's exact message
            let mut bare = serde_json::json!({});
            assert_eq!(
                CfgMutation::RegisterManual {
                    entry: serde_json::json!({"path": "q"}),
                }
                .apply(&mut bare),
                Err("config missing games list".to_string())
            );
            CfgMutation::ReplaceLibraryDirs {
                dirs: serde_json::json!(["C:\\roms"]),
            }
            .apply(&mut cfg)
            .unwrap();
            assert_eq!(cfg["library_dirs"][0], "C:\\roms");
            assert_eq!(write_decide(0, false), WriteDecision::Write);
            assert_eq!(write_decide(0, true), WriteDecision::Retry);
            assert_eq!(write_decide(1, true), WriteDecision::GiveUp);
            assert_eq!(write_decide(1, false), WriteDecision::Write);
        }
        // -- renditions: name round-trip, eviction plan,
        //    per-tier decode sizing
        {
            assert_eq!(
                rendition_name("0123456789abcdef", 220, 300, 12345, 678),
                "0123456789abcdef_220x300_12345_678.png"
            );
            assert_eq!(
                parse_rendition_name(
                    "0123456789abcdef_220x300_12345_678.png"
                ),
                Some((220, 300, 12345, 678))
            );
            assert_eq!(parse_rendition_name("something else.png"), None);
            assert_eq!(
                parse_rendition_name(
                    "0123456789abcdef_220x300_12345_678.tmp"
                ),
                None
            );
            let entries = vec![
                ("a".to_string(), 100u64, 600u64),
                ("b".to_string(), 200, 600),
                ("c".to_string(), 300, 600),
            ];
            // 1800 over a 1000 cap: the two oldest go
            assert_eq!(
                eviction_plan(&entries, 1800, 1000),
                vec!["a".to_string(), "b".to_string()]
            );
            assert!(eviction_plan(&entries, 900, 1000).is_empty());
            let img = image::RgbImage::from_pixel(
                440,
                600,
                image::Rgb([10, 20, 30]),
            );
            let mut png = Vec::new();
            image::DynamicImage::ImageRgb8(img)
                .write_to(
                    &mut std::io::Cursor::new(&mut png),
                    image::ImageFormat::Png,
                )
                .expect("encode fixture png");
            assert_eq!(
                decode_cover_to(&png, 220, 300)
                    .expect("standard tier")
                    .size,
                [220, 300]
            );
            assert_eq!(
                decode_cover_to(&png, 176, 240)
                    .expect("low-spec tier")
                    .size,
                [176, 240]
            );
            let rend = encode_rendition(
                &decode_cover_to(&png, 220, 300).unwrap(),
            )
            .expect("rendition encode");
            assert_eq!(
                decode_rendition(&rend).expect("rendition decode").size,
                [220, 300]
            );
        }
        // -- machine profile resolution, detection rules,
        //    probe parsing (the whole study §3 table)
        {
            let mut s = settings_blank();
            let p = resolve_profile(&s, 8);
            assert!(!p.low_spec);
            assert_eq!(p.cover_budget, 384 * 1024 * 1024);
            assert_eq!(p.rendition, (220, 300));
            assert_eq!(p.art_workers, 4);
            assert!(p.prefetch);
            assert_eq!(p.frame_art_cap, usize::MAX);
            assert!(!p.scale_lock);
            assert_eq!(p.window, [1560.0, 980.0]);
            // cover-memory presets flow through
            s.cover_memory = "high".into();
            assert_eq!(
                resolve_profile(&s, 8).cover_budget,
                1024 * 1024 * 1024
            );
            s.cover_memory = "low".into();
            assert_eq!(
                resolve_profile(&s, 8).cover_budget,
                128 * 1024 * 1024
            );
            s.cover_memory = "standard".into();
            // a recorded HDD shallows the pool and stops
            // prefetch; the mode itself is untouched
            s.detected_disk = "hdd".into();
            let p = resolve_profile(&s, 8);
            assert!(!p.low_spec);
            assert_eq!(p.art_workers, 2);
            assert!(!p.prefetch);
            s.detected_disk = String::new();
            // two cores still pool at the clamp's floor
            assert_eq!(resolve_profile(&s, 2).art_workers, 2);
            // manual low-spec takes over completely
            s.low_spec = true;
            let p = resolve_profile(&s, 8);
            assert!(p.low_spec);
            assert_eq!(p.cover_budget, LOW_SPEC_BUDGET);
            assert_eq!(p.rendition, (176, 240));
            assert_eq!(p.art_workers, 1);
            assert!(!p.prefetch);
            assert_eq!(p.frame_art_cap, 8);
            assert!(p.scale_lock);
            assert_eq!(p.window, [1280.0, 800.0]);
            // the never-auto rule: detection values living
            // in an existing settings file never flip the
            // mode by themselves — low_spec is the only
            // switch resolve_profile reads for mode
            s.low_spec = false;
            s.detected_ram_gb = Some(2.0);
            s.detected_disk = "hdd".into();
            assert!(!resolve_profile(&s, 1).low_spec);
            // detection default rule
            assert!(detection_low_spec(2, None));
            assert!(detection_low_spec(1, Some(16.0)));
            assert!(detection_low_spec(8, Some(4.0)));
            assert!(detection_low_spec(8, Some(3.9)));
            assert!(!detection_low_spec(8, Some(4.1)));
            assert!(!detection_low_spec(4, Some(8.0)));
            assert!(!detection_low_spec(3, None));
            // probe output parsing: CRLF, decimal commas,
            // Unspecified media, missing lines, junk
            assert_eq!(
                parse_probe_output("RAMGB=3.9\r\nDISK=HDD\r\n"),
                (Some(3.9), DiskKind::Hdd)
            );
            assert_eq!(
                parse_probe_output("RAMGB=15,9\nDISK=SSD\n"),
                (Some(15.9), DiskKind::Ssd)
            );
            assert_eq!(
                parse_probe_output("DISK=Unspecified\n"),
                (None, DiskKind::Unknown)
            );
            assert_eq!(
                parse_probe_output("garbage"),
                (None, DiskKind::Unknown)
            );
            assert_eq!(
                parse_probe_output("RAMGB=\nDISK=\n"),
                (None, DiskKind::Unknown)
            );
        }
        // -- settings round-trip including the v1.05
        //    keys; a pre-v1.05 file reads as defaults
        {
            let mut s = settings_blank();
            s.cover_memory = "high".into();
            s.low_spec = true;
            s.low_spec_user_set = true;
            s.detected_ram_gb = Some(3.9);
            s.detected_disk = "hdd".into();
            let back = settings_from_json(&settings_to_json(&s));
            assert_eq!(back.cover_memory, "high");
            assert!(back.low_spec);
            assert!(back.low_spec_user_set);
            assert_eq!(back.detected_ram_gb, Some(3.9));
            assert_eq!(back.detected_disk, "hdd");
            let old: serde_json::Value = serde_json::from_str(
                r#"{"ra_channel":"stable","ra_ask":false,"music_volume":0.5,"soundfont_path":"","emu_updates":{},"emu_updates_at":0,"emu_overrides":{},"cover_memory":"low"}"#,
            )
            .unwrap();
            let back = settings_from_json(&old);
            assert_eq!(back.cover_memory, "low");
            assert!(!back.low_spec);
            assert!(!back.low_spec_user_set);
            assert_eq!(back.detected_ram_gb, None);
            assert_eq!(back.detected_disk, "");
        }
        // ---- v1.06 Slice 3: catalog disk cache — TTL
        //    boundaries, age labels, the file round-trip
        //    and corruption cases, the plan table. All
        //    pure: no disk, no network.
        {
            let t0 = 1_700_000_000i64;
            assert_eq!(CATALOG_TTL_SECS, 24 * 3600);
            assert!(!catalog_is_stale(t0, t0));
            assert!(!catalog_is_stale(t0, t0 + CATALOG_TTL_SECS - 1));
            assert!(catalog_is_stale(t0, t0 + CATALOG_TTL_SECS));
            assert!(catalog_is_stale(t0, t0 + CATALOG_TTL_SECS * 3));
            assert!(catalog_is_stale(0, t0));
            // a stamp in the future (clock stepped back)
            // is trusted, not punished with a refetch loop
            assert!(!catalog_is_stale(t0 + 500, t0));
            assert_eq!(catalog_age_label(30), "just now");
            assert_eq!(catalog_age_label(89), "just now");
            assert_eq!(catalog_age_label(600), "10 min ago");
            assert_eq!(catalog_age_label(3 * 3600), "3 h ago");
            assert_eq!(catalog_age_label(23 * 3600), "23 h ago");
            assert_eq!(catalog_age_label(2 * 86_400), "2 d ago");
            let entries = vec![
                CatalogEntry {
                    title: "Alpha MSU-1 pack".into(),
                    url: "https://example.invalid/a".into(),
                    source: "archive.org".into(),
                    kind: "MSU-1 pack".into(),
                },
                CatalogEntry {
                    title: "Beta MD+ pack".into(),
                    url: "https://example.invalid/b".into(),
                    source: "Minerva".into(),
                    kind: "MD+ pack".into(),
                },
            ];
            let disk_file = CatalogDiskFile {
                magic: CATALOG_DISK_MAGIC.to_string(),
                schema: CATALOG_DISK_SCHEMA,
                system: "Super Nintendo".to_string(),
                fetched_at: t0,
                entries: entries.clone(),
            };
            let text = serde_json::to_string(&disk_file).unwrap();
            let (sys, at, back) =
                parse_catalog_disk(&text).expect("disk round-trip");
            assert_eq!(sys, "Super Nintendo");
            assert_eq!(at, t0);
            assert_eq!(
                serde_json::to_string(&back).unwrap(),
                serde_json::to_string(&entries).unwrap()
            );
            // corruption and foreign files read as absent
            assert!(parse_catalog_disk("not json at all").is_none());
            assert!(parse_catalog_disk("{}").is_none());
            assert!(parse_catalog_disk(
                &text.replace(CATALOG_DISK_MAGIC, "something-else")
            )
            .is_none());
            assert!(parse_catalog_disk(
                &text.replace("\"schema\":1", "\"schema\":99")
            )
            .is_none());
            // the state machine: absent / fresh / stale
            assert!(matches!(
                catalog_disk_decide(None, t0),
                CatalogDisk::Absent
            ));
            let d = catalog_disk_decide(Some((t0, entries.clone())), t0 + 60);
            assert!(matches!(d, CatalogDisk::Fresh(_, t) if t == t0));
            let d = catalog_disk_decide(
                Some((t0, entries.clone())),
                t0 + CATALOG_TTL_SECS,
            );
            assert!(matches!(d, CatalogDisk::Stale(_, t) if t == t0));
            assert!(CatalogDisk::Absent.entries().is_none());
            assert_eq!(
                CatalogDisk::Fresh(entries.clone(), t0)
                    .entries()
                    .map(|v| v.len()),
                Some(2)
            );
            // the plan table (force = the Refresh button)
            let (show, live) = catalog_plan(&CatalogDisk::Absent, false);
            assert!(show.is_none() && live);
            let fresh = CatalogDisk::Fresh(entries.clone(), t0);
            let (show, live) = catalog_plan(&fresh, false);
            assert_eq!(show.map(|v| v.len()), Some(2));
            assert!(!live);
            let stale = CatalogDisk::Stale(entries.clone(), t0);
            let (show, live) = catalog_plan(&stale, false);
            assert!(show.is_some() && live);
            let (show, live) = catalog_plan(&fresh, true);
            assert!(show.is_none() && live);
            let (show, live) = catalog_plan(&CatalogDisk::Absent, true);
            assert!(show.is_none() && live);
        }
        // ---- v1.06 Slice 3: the widened link finder.
        //    Fixture pages through resolve_download with
        //    an injected fetcher: quote styles, the
        //    second hop, the hop cap, form downloads,
        //    anchor text, and the non-link negatives.
        {
            use std::cell::Cell;
            let mut pages: std::collections::HashMap<&str, &str> =
                std::collections::HashMap::new();
            pages.insert(
                "https://x.test/entry-dq",
                r#"<a href="https://x.test/files/pack.zip">Get it</a>"#,
            );
            pages.insert(
                "https://x.test/entry-sq",
                "<a href='https://x.test/files/patch.ips'>Get</a>",
            );
            pages.insert(
                "https://x.test/entry-uq",
                "<a href=https://x.test/files/patch.bps>Get</a>",
            );
            pages.insert(
                "https://x.test/entry-page",
                r#"<a href="/download/patch">Download page</a>"#,
            );
            pages.insert(
                "https://x.test/download/patch",
                r#"<a href="https://x.test/files/real.7z">File</a>"#,
            );
            pages.insert(
                "https://x.test/entry-dead",
                r#"<a href="/download/dead">Download</a>"#,
            );
            pages.insert(
                "https://x.test/download/dead",
                r#"<p>Nothing here</p><a href="/about">About</a>"#,
            );
            pages.insert(
                "https://x.test/entry-loop",
                r#"<a href="/download/loop">Download</a>"#,
            );
            pages.insert(
                "https://x.test/download/loop",
                r#"<a href="https://x.test/entry-loop">Download</a>"#,
            );
            pages.insert(
                "https://x.test/entry-form",
                r#"<form method="post" action="https://x.test/files/form.zip"><input type="submit" value="Download"></form>"#,
            );
            pages.insert(
                "https://x.test/entry-formpage",
                r#"<form action="/download"><button type="submit">Download now</button></form>"#,
            );
            pages.insert(
                "https://x.test/download",
                r#"<a href="https://x.test/files/via-form.rar">f</a>"#,
            );
            pages.insert(
                "https://x.test/entry-none",
                r##"<a href="javascript:void(0)">Download</a><a href="mailto:a@b.c">Download</a><a href="#top">Download</a>"##,
            );
            let fetches = Cell::new(0usize);
            let fetch = |u: &str| -> Result<String, String> {
                fetches.set(fetches.get() + 1);
                pages
                    .get(u)
                    .map(|s| s.to_string())
                    .ok_or_else(|| format!("no fixture page: {u}"))
            };
            // the shipped shapes still work (hop 1)
            let (got, n) = resolve_download("https://x.test/entry-dq", fetch)
                .unwrap();
            assert_eq!(got.as_deref(), Some("https://x.test/files/pack.zip"));
            assert_eq!(n, 1);
            let (got, n) = resolve_download("https://x.test/entry-sq", fetch)
                .unwrap();
            assert_eq!(got.as_deref(), Some("https://x.test/files/patch.ips"));
            assert_eq!(n, 1);
            let (got, n) = resolve_download("https://x.test/entry-uq", fetch)
                .unwrap();
            assert_eq!(got.as_deref(), Some("https://x.test/files/patch.bps"));
            assert_eq!(n, 1);
            // a form action that IS the file (hop 1)
            let (got, n) = resolve_download("https://x.test/entry-form", fetch)
                .unwrap();
            assert_eq!(got.as_deref(), Some("https://x.test/files/form.zip"));
            assert_eq!(n, 1);
            // the second hop: file behind a download page
            let (got, n) = resolve_download("https://x.test/entry-page", fetch)
                .unwrap();
            assert_eq!(got.as_deref(), Some("https://x.test/files/real.7z"));
            assert_eq!(n, 2);
            // ... and behind a form's download page
            let (got, n) =
                resolve_download("https://x.test/entry-formpage", fetch)
                    .unwrap();
            assert_eq!(
                got.as_deref(),
                Some("https://x.test/files/via-form.rar")
            );
            assert_eq!(n, 2);
            // a dead-end second page: the first winner
            // stands (what the old finder returned)
            let (got, n) = resolve_download("https://x.test/entry-dead", fetch)
                .unwrap();
            assert_eq!(
                got.as_deref(),
                Some("https://x.test/download/dead")
            );
            assert_eq!(n, 2);
            // the hop cap: page 2 points back at page 1 —
            // no third fetch, the first winner stands
            let before = fetches.get();
            let (got, n) = resolve_download("https://x.test/entry-loop", fetch)
                .unwrap();
            assert_eq!(
                got.as_deref(),
                Some("https://x.test/download/loop")
            );
            assert_eq!(n, 2);
            assert_eq!(fetches.get() - before, 2);
            // only non-links on the page: no winner
            let (got, n) = resolve_download("https://x.test/entry-none", fetch)
                .unwrap();
            assert_eq!(got, None);
            assert_eq!(n, 1);
            // a direct file entry never touches the network
            let before = fetches.get();
            let (got, n) =
                resolve_download("https://x.test/files/direct.zip", fetch)
                    .unwrap();
            assert_eq!(
                got.as_deref(),
                Some("https://x.test/files/direct.zip")
            );
            assert_eq!(n, 0);
            assert_eq!(fetches.get(), before);
            // anchor text arbitrates between page links
            let pick = pick_download_candidate(
                "https://x.test/p",
                r#"<a href="/get?id=1">here</a> <a href="/get?id=2">Download the patch</a>"#,
            );
            assert_eq!(pick.as_deref(), Some("https://x.test/get?id=2"));
            // ... but a real file beats a download page,
            // whatever the labels say (the shipped order)
            let pick = pick_download_candidate(
                "https://x.test/p",
                r#"<a href="/download/page">Download</a> <a href="https://x.test/files/mirror.zip">mirror</a>"#,
            );
            assert_eq!(
                pick.as_deref(),
                Some("https://x.test/files/mirror.zip")
            );
            // scoring pins
            assert!(score_page_link("https://x.test/f.zip", "") == 5);
            assert!(score_page_link("https://x.test/download", "") == 4);
            assert!(score_page_link("https://x.test/get", "Download") == 3);
            assert!(score_page_link("https://x.test/about", "About") == 0);
            assert!(is_file_url("https://x.test/f.ZIP?x=1"));
            assert!(!is_file_url("https://x.test/download"));
        }
        // ---- v1.06 Slice 3: prefetch window math at
        //    the profile tiers (disabled = low-spec and
        //    HDD profiles; enabled = the standard tier).
        {
            assert_eq!(scroll_dir(None, 5), ScrollDir::Still);
            assert_eq!(scroll_dir(Some(5), 5), ScrollDir::Still);
            assert_eq!(scroll_dir(Some(5), 7), ScrollDir::Down);
            assert_eq!(scroll_dir(Some(7), 5), ScrollDir::Up);
            let (a, b) = prefetch_ranges(10, 14, 5, 500, ScrollDir::Down, false);
            assert!(a.is_empty() && b.is_empty());
            // Still reproduces the v1.04 ranges exactly:
            // one viewport each way
            let (pri, sec) =
                prefetch_ranges(10, 14, 5, 500, ScrollDir::Still, true);
            assert_eq!(pri, 70..90);
            assert_eq!(sec, 30..50);
            // Down: 1.5 viewports ahead, 0.5 behind
            let (pri, sec) =
                prefetch_ranges(10, 14, 5, 500, ScrollDir::Down, true);
            assert_eq!(pri, 70..100);
            assert_eq!(sec, 40..50);
            // Up: mirrored, and the above range leads
            let (pri, sec) =
                prefetch_ranges(10, 14, 5, 500, ScrollDir::Up, true);
            assert_eq!(pri, 20..50);
            assert_eq!(sec, 70..80);
            // clamps at the wall's ends
            let (pri, sec) =
                prefetch_ranges(0, 4, 5, 500, ScrollDir::Down, true);
            assert_eq!(pri, 20..50);
            assert!(sec.is_empty());
            let (pri, sec) =
                prefetch_ranges(96, 100, 5, 500, ScrollDir::Down, true);
            assert!(pri.is_empty());
            assert_eq!(sec, 470..480);
            // an empty wall has no window at any tier
            let (a, b) = prefetch_ranges(0, 0, 5, 0, ScrollDir::Down, true);
            assert!(a.is_empty() && b.is_empty());
        }
        // ---- v1.07: sidecar store, play folding,
        // first-seen baseline, presence config, shelf
        // order, formatters.
        let vg = |name: &str, path: &str, system: &str| Game {
            name: name.to_string(),
            path: path.to_string(),
            system: system.to_string(),
            emulator: String::new(),
            emulator_locked: false,
            patch: String::new(),
            cheats_text: String::new(),
            cheats_enabled: false,
        };
        // settings: a pre-v1.07 file (no discord keys)
        // reads as toggle-on + empty ID; the round-trip
        // keeps both fields; presence needs the toggle
        // AND a non-empty ID.
        let pre = serde_json::json!({ "ra_channel": "stable" });
        let s = settings_from_json(&pre);
        assert!(s.discord_presence);
        assert_eq!(s.discord_app_id, "");
        assert_eq!(presence_app_id(&s), None);
        let mut s2 = settings_blank();
        s2.discord_app_id = " 123456 ".to_string();
        let rt = settings_from_json(&settings_to_json(&s2));
        assert_eq!(rt.discord_app_id, "123456");
        assert_eq!(presence_app_id(&rt), Some("123456"));
        let mut s3 = settings_blank();
        s3.discord_presence = false;
        s3.discord_app_id = "123456".to_string();
        assert_eq!(presence_app_id(&s3), None);
        // sidecar JSON round-trip + the frozen guard +
        // lenient reads.
        let mut d = RustData::default();
        d.play.insert(
            "p1".to_string(),
            PlayRec { last: 100, secs: 200, sessions: 3 },
        );
        d.first_seen.insert("p1".to_string(), 50);
        d.first_seen_baseline = 40;
        let back = rust_data_from_json(&rust_data_to_json(&d));
        assert_eq!(
            back.play["p1"],
            PlayRec { last: 100, secs: 200, sessions: 3 }
        );
        assert_eq!(back.first_seen["p1"], 50);
        assert_eq!(back.first_seen_baseline, 40);
        assert!(!back.frozen);
        let future = serde_json::json!({ "version": 99 });
        assert!(rust_data_from_json(&future).frozen);
        let junk = serde_json::json!({ "play": 42, "first_seen": "x" });
        let jd = rust_data_from_json(&junk);
        assert!(jd.play.is_empty() && jd.first_seen.is_empty());
        // session folding: a stub-length session moves
        // last-played only; real sessions accrue time
        // and count; last never moves backwards.
        let mut d = RustData::default();
        fold_session(&mut d, "g", 1000, 3);
        assert_eq!(
            d.play["g"],
            PlayRec { last: 1000, secs: 0, sessions: 0 }
        );
        fold_session(&mut d, "g", 2000, 125);
        assert_eq!(
            d.play["g"],
            PlayRec { last: 2000, secs: 125, sessions: 1 }
        );
        fold_session(&mut d, "g", 1500, 65);
        assert_eq!(d.play["g"].last, 2000);
        assert_eq!(d.play["g"].secs, 190);
        assert_eq!(d.play["g"].sessions, 2);
        assert!(d.dirty);
        // first-seen: day one baselines the whole
        // library; an empty library never baselines;
        // later arrivals stamp at their own moment and
        // old stamps never move.
        let mut d = RustData::default();
        assert!(d.observe_first_seen(["a", "b"].iter().copied(), 500));
        assert_eq!(d.first_seen_baseline, 500);
        assert_eq!(d.first_seen["a"], 500);
        assert!(!d.observe_first_seen(["a", "b"].iter().copied(), 600));
        assert!(d.observe_first_seen(["a", "c"].iter().copied(), 700));
        assert_eq!(d.first_seen["c"], 700);
        assert_eq!(d.first_seen["a"], 500);
        let mut e = RustData::default();
        assert!(!e.observe_first_seen(std::iter::empty(), 500));
        assert_eq!(e.first_seen_baseline, 0);
        // new-this-week edges: the baseline stamp itself
        // is never new; exactly 7 days is still in; a
        // future stamp (clock stepped back) counts.
        assert!(!is_new_this_week(1000, 1000, 1000));
        assert!(is_new_this_week(1001, 1000, 1001));
        assert!(is_new_this_week(1001, 1000, 1001 + NEW_WEEK_SECS));
        assert!(!is_new_this_week(1001, 1000, 1002 + NEW_WEEK_SECS));
        assert!(is_new_this_week(5000, 1000, 4000));
        assert!(!is_new_this_week(1001, 0, 1001));
        // shelf order: Continue = last-played desc, the
        // tie breaks on total secs; unplayed games are
        // absent; the search filter applies. New =
        // first-seen desc with the baseline excluded.
        let games = vec![
            vg("Alpha", "pa", "SNES"),
            vg("Beta", "pb", "SNES"),
            vg("Gamma", "pg", "Genesis"),
            vg("Delta", "pd", "Genesis"),
        ];
        let lowers: Vec<String> =
            games.iter().map(|g| g.name.to_lowercase()).collect();
        let mut play = BTreeMap::new();
        play.insert(
            "pb".to_string(),
            PlayRec { last: 300, secs: 50, sessions: 1 },
        );
        play.insert(
            "pa".to_string(),
            PlayRec { last: 300, secs: 90, sessions: 2 },
        );
        play.insert(
            "pg".to_string(),
            PlayRec { last: 900, secs: 10, sessions: 1 },
        );
        assert_eq!(
            continue_indices(&games, &play, &lowers, ""),
            vec![2, 0, 1]
        );
        assert_eq!(continue_indices(&games, &play, &lowers, "amm"), vec![2]);
        let mut d = RustData::default();
        d.first_seen_baseline = 100;
        d.first_seen.insert("pa".to_string(), 100);
        d.first_seen.insert("pb".to_string(), 500);
        d.first_seen.insert("pg".to_string(), 900);
        assert_eq!(new_indices(&games, &d, &lowers, "", 1000), vec![2, 1]);
        assert_eq!(
            new_indices(&games, &d, &lowers, "", 900 + NEW_WEEK_SECS + 1),
            Vec::<usize>::new()
        );
        // formatters.
        assert_eq!(fmt_playtime(0), "0s");
        assert_eq!(fmt_playtime(59), "59s");
        assert_eq!(fmt_playtime(60), "1m");
        assert_eq!(fmt_playtime(3599), "59m");
        assert_eq!(fmt_playtime(3600), "1h 0m");
        assert_eq!(fmt_playtime(45210), "12h 33m");
        assert_eq!(fmt_day(0), "Jan 1");
        assert_eq!(fmt_day(1_792_929_600), "Oct 25");
        assert_eq!(fmt_day(1_709_208_000), "Feb 29");
        assert_eq!(fmt_day(1_767_225_600), "Jan 1");
        // shelf titles: Back always leaves them for Home.
        assert!(is_shelf_title("⭐ Enhanced & Hacked"));
        assert!(is_shelf_title("PC Ports"));
        assert!(is_shelf_title(CONTINUE_TITLE));
        assert!(is_shelf_title(NEW_TITLE));
        assert!(!is_shelf_title("Super Nintendo"));
        assert_eq!(back_target(CONTINUE_TITLE, None, 0), BackTarget::Home);
        assert_eq!(back_target(NEW_TITLE, Some(2), 99999), BackTarget::Home);
        println!("SELFTEST PASS");
        return;
    }
    if let Some(pos) = std::env::args().position(|a| a == "--search-test") {
        let name = std::env::args()
            .nth(pos + 1)
            .unwrap_or_else(|| "Super Metroid".to_string());
        let (games, _p, _ms) = load_library();
        let mut cands: Vec<&Game> = games
            .iter()
            .filter(|g| {
                g.system == "Super Nintendo"
                    && g.name.to_lowercase().contains(&name.to_lowercase())
            })
            .collect();
        cands.sort_by_key(|g| g.name.len());
        if let Some(g) = cands.first() {
            let hits = search_plaza(g);
            println!("search '{}' -> {} entries", g.name, hits.len());
            for h in hits.iter().take(5) {
                println!("  - {} | {}", h.title, h.url);
            }
        } else {
            println!("game not found: {name}");
        }
        return;
    }
    // headless self-check: --check prints library stats and exits
    if std::env::args().any(|a| a == "--check") {
        let (games, profiles, ms) = load_library();
        let mut counts: HashMap<String, usize> = HashMap::new();
        for g in &games {
            *counts.entry(g.system.clone()).or_insert(0) += 1;
        }
        println!("games={} profiles={} load_ms={:.0}",
                 games.len(), profiles.len(), ms);
        println!("snes={}", counts.get("Super Nintendo").unwrap_or(&0));
        if let Some(ct) = games
            .iter()
            .find(|g| g.name.starts_with("Chrono Trigger ["))
        {
            println!("ct_msu={} engine={}",
                     game_has_msu(ct),
                     engine_label(ct, &profiles));
        }
        return;
    }
    // letter-range proof: --buckets loads the library (same
    // loader as --check) and, for every system over BUCKET_MIN,
    // prints its five bucket counts. Asserts each system's
    // bucket counts sum to its game count, so every game maps
    // to exactly one bucket; exits non-zero on any mismatch.
    // Headless; needs no display.
    if std::env::args().any(|a| a == "--buckets") {
        let (games, _profiles, _ms) = load_library();
        let mut totals: HashMap<String, usize> = HashMap::new();
        for g in &games {
            *totals.entry(g.system.clone()).or_insert(0) += 1;
        }
        let mut systems: Vec<&String> = totals.keys().collect();
        systems.sort();
        let mut bad = 0usize;
        let mut checked = 0usize;
        for sys in systems {
            let total = totals[sys];
            let counts = bucket_counts(&games, sys);
            let sum: usize = counts.iter().sum();
            if sum != total {
                bad += 1;
            }
            if total > BUCKET_MIN {
                checked += 1;
                println!(
                    "system={} total={} 0-9={} A-G={} H-M={} N-T={} U-Z={} sum={} {}",
                    sys,
                    total,
                    counts[0],
                    counts[1],
                    counts[2],
                    counts[3],
                    counts[4],
                    sum,
                    if sum == total { "OK" } else { "MISMATCH" },
                );
            }
        }
        println!(
            "buckets: {} systems over threshold ({}), {} mismatches",
            checked, BUCKET_MIN, bad
        );
        if bad > 0 {
            std::process::exit(1);
        }
        return;
    }
    // defaults sweep: --sweep runs the real pick_profile over
    // every game and reports, per system, the effective default
    // profile plus any cross-system picks. Read-only; PC Ports
    // are skipped (they launch directly, no profile involved).
    if std::env::args().any(|a| a == "--sweep") {
        let (games, profiles, _ms) = load_library();
        let mut tally: std::collections::BTreeMap<(String, String), usize> =
            std::collections::BTreeMap::new();
        let mut nones: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        let mut mism: Vec<String> = Vec::new();
        let mut locks = 0usize;
        let mut judged = 0usize;
        for g in &games {
            if is_port(g) {
                continue;
            }
            match pick_profile(g, &profiles) {
                None => {
                    *nones.entry(g.system.clone()).or_insert(0) += 1;
                }
                Some(p) => {
                    *tally
                        .entry((g.system.clone(), p.name.clone()))
                        .or_insert(0) += 1;
                    if g.emulator_locked && g.emulator == p.name {
                        locks += 1;
                        continue;
                    }
                    judged += 1;
                    let same = p.system == g.system;
                    let exc = ((g.system == "Nintendo GameCube"
                        || g.system == "Nintendo Wii")
                        && p.name.contains("Dolphin"))
                        || (g.system == "Sony PlayStation 2"
                            && p.name.contains("PCSX2"))
                        || (GENESIS_FAMILY.contains(&g.system.as_str())
                            && p.core.to_lowercase().contains("genesis_plus"));
                    let frag_ok = best_pick_frags(&g.system)
                        .iter()
                        .any(|f| p.name.contains(f) || p.core.contains(f));
                    if !same && !exc && !frag_ok {
                        mism.push(format!(
                            "{} [{}] -> {} [{}]",
                            g.name, g.system, p.name, p.system
                        ));
                    }
                }
            }
        }
        println!(
            "sweep games={} judged={} locks_fired={} mismatches={}",
            games.len(),
            judged,
            locks,
            mism.len()
        );
        let mut cur = String::new();
        for ((sys, prof), n) in &tally {
            if *sys != cur {
                println!("SYSTEM {sys}");
                cur = sys.clone();
            }
            println!("  {n:7} -> {prof}");
        }
        for (sys, n) in &nones {
            println!("NOPROFILE {sys}: {n}");
        }
        for m in mism.iter().take(50) {
            println!("MISMATCH {m}");
        }
        return;
    }
    // music build gate: --music-probe <dir> renders ~2 s per
    // format through the music engines and RMS-checks the output
    // (see run_music_probe above). Headless; exit 0 = every staged
    // format rendered non-silent audio.
    if let Some(pos) = std::env::args().position(|a| a == "--music-probe") {
        let dir = std::env::args()
            .nth(pos + 1)
            .unwrap_or_else(|| "music-probe".to_string());
        std::process::exit(run_music_probe(Path::new(&dir)));
    }
    // register a PC Ports entry (appended to games + manual_games,
    // deduped by path): --register-port "<name>" "<exe path>"
    if let Some(pos) = std::env::args().position(|a| a == "--register-port") {
        let name = std::env::args().nth(pos + 1).unwrap_or_default();
        let path = std::env::args().nth(pos + 2).unwrap_or_default();
        if name.is_empty() || path.is_empty() {
            println!("usage: --register-port \"<name>\" \"<exe path>\"");
            return;
        }
        let entry = serde_json::json!({
            "name": name,
            "path": path,
            "system": "PC Ports",
            "emulator": "",
            "patch": "",
            "cheats_text": "",
            "cheats_enabled": false,
        });
        match register_manual_game(entry) {
            Ok(()) => println!("REGISTERED: {name}"),
            Err(e) => println!("REGISTER-ERR: {e}"),
        }
        return;
    }
    // launch a PC Ports entry directly (the LAUNCH button's path):
    // --launch-port "<name fragment>"
    if let Some(pos) = std::env::args().position(|a| a == "--launch-port") {
        let frag = std::env::args()
            .nth(pos + 1)
            .unwrap_or_default()
            .to_lowercase();
        let (games, _p, _ms) = load_library();
        match games
            .iter()
            .find(|g| is_port(g) && g.name.to_lowercase().contains(&frag))
        {
            Some(g) => match launch_direct(g) {
                Ok(m) => println!("LAUNCH-OK: {m}"),
                Err(m) => println!("LAUNCH-ERR: {m}"),
            },
            None => println!("LAUNCH-ERR: no PC Ports entry matches '{frag}'"),
        }
        return;
    }
    // texture pack target probe: --tex-info "<name fragment>"
    if let Some(pos) = std::env::args().position(|a| a == "--tex-info") {
        let frag = std::env::args().nth(pos + 1).unwrap_or_default();
        let (games, profiles, _ms) = load_library();
        let hits: Vec<&Game> = games
            .iter()
            .filter(|g| {
                g.name.to_lowercase().contains(&frag.to_lowercase())
            })
            .take(6)
            .collect();
        if hits.is_empty() {
            println!("no game matches '{frag}'");
        }
        for g in hits {
            let t = texture_target(g, &profiles);
            println!("game={} system={}", g.name, g.system);
            println!(
                "  kind={:?} key={} folder={}",
                t.kind,
                t.key,
                t.folder
                    .as_ref()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "-".into())
            );
            println!(
                "  packs={} status={}",
                packs_for(&t).len(),
                t.status
            );
        }
        return;
    }
    // texture pack install test: --tex-install "<name fragment>" "<url>"
    if let Some(pos) = std::env::args().position(|a| a == "--tex-install") {
        let frag = std::env::args().nth(pos + 1).unwrap_or_default();
        let url = std::env::args().nth(pos + 2).unwrap_or_default();
        let (games, profiles, _ms) = load_library();
        // candidate order: exact name + supported target, exact name,
        // contains + supported, any contains. (Exact matters: "Contra"
        // must pick the NES game, not a pirate cart containing the
        // word, and not the Arcade game.)
        let fl = frag.to_lowercase();
        let sys_fl = std::env::args()
            .nth(pos + 3)
            .unwrap_or_default()
            .to_lowercase();
        let supported = |g: &Game| {
            texture_target(g, &profiles).kind != TexKind::Unsupported
        };
        let pick = |pred: &dyn Fn(&&Game) -> bool| {
            games
                .iter()
                .filter(|g| {
                    g.name.to_lowercase().contains(&fl)
                        && (sys_fl.is_empty()
                            || g.system.to_lowercase().contains(&sys_fl))
                })
                .find(|g| pred(g))
        };
        let Some(g) = pick(&|g| g.name.to_lowercase() == fl && supported(g))
            .or_else(|| pick(&|g| g.name.to_lowercase() == fl))
            .or_else(|| pick(&|g| supported(g)))
            .or_else(|| pick(&|_| true))
        else {
            println!("TEX-ERR: game not found: {frag}");
            return;
        };
        let t = texture_target(g, &profiles);
        let count_now = |t: &TexTarget| -> usize {
            let mut n = t
                .folder
                .as_ref()
                .map(|f| count_files_recursive(f))
                .unwrap_or(0);
            // GLideN64 cache packs land in the cache folder instead
            if let Some(gfx) = &t.gfx_dir {
                if let Ok(rd) = std::fs::read_dir(gfx.join("cache")) {
                    n += rd
                        .flatten()
                        .filter(|e| {
                            e.file_name()
                                .to_string_lossy()
                                .to_uppercase()
                                .starts_with(&t.key.to_uppercase())
                        })
                        .count();
                }
            }
            n
        };
        let before = count_now(&t);
        println!(
            "game={} kind={:?} key={} before={}",
            g.name, t.kind, t.key, before
        );
        match install_texture_pack(&url, &t) {
            Ok(m) => {
                println!("TEX-OK: {m}");
                println!("after={}", count_now(&t));
            }
            Err(m) => println!("TEX-ERR: {m}"),
        }
        return;
    }
    // emulator update check: --emu-updates prints installed vs latest
    // for every emulator the profiles use, and caches the result in
    // the Rust settings (the Manage Emulators window reads it too).
    if std::env::args().any(|a| a == "--emu-updates") {
        let (_games, profiles, _ms) = load_library();
        let map = check_emu_updates(&profiles);
        let mut n_upd = 0;
        for (name, u) in &map {
            if u.status == "update" {
                n_upd += 1;
            }
            println!(
                "{} | installed={} | latest={} | status={} | {}",
                name, u.installed, u.latest, u.status, u.url
            );
        }
        println!("checked={} updates_available={}", map.len(), n_upd);
        let mut st = load_settings();
        st.emu_updates = map;
        st.emu_updates_at = now_epoch();
        save_settings(&st);
        return;
    }
    if std::env::args().any(|a| a == "--ra-check") {
        let st = load_settings();
        let (ch, label) = ra_stamp();
        println!("settings channel={} ask={}", st.ra_channel, st.ra_ask);
        println!("stamp channel={} label={}", ch, label);
        match ra_remote(&st.ra_channel) {
            Some((url, l)) => println!("remote label={} url={}", l, url),
            None => println!("remote unreachable"),
        }
        println!("ra_running={}", ra_running());
        return;
    }
    // launch test: run the exact launch() the LAUNCH button uses
    if std::env::args().any(|a| a == "--launch-test") {
        let (games, profiles, _ms) = load_library();
        if let Some(ct) = games
            .iter()
            .find(|g| g.name.starts_with("Chrono Trigger ["))
        {
            println!("game={} msu={}", ct.name, game_has_msu(ct));
            match launch(ct, &profiles) {
                Ok(m) => println!("LAUNCH-OK: {m}"),
                Err(m) => println!("LAUNCH-ERR: {m}"),
            }
        } else {
            println!("LAUNCH-ERR: chrono trigger msu entry not found");
        }
        return;
    }
    // launch-one probe: --launch-one "<name fragment>" "<profile
    // fragment>" runs launch_with() — the exact code path the
    // panel's LAUNCH button uses — against a named profile.
    if let Some(pos) = std::env::args().position(|a| a == "--launch-one") {
        let frag = std::env::args().nth(pos + 1).unwrap_or_default();
        let pfrag = std::env::args()
            .nth(pos + 2)
            .unwrap_or_default()
            .to_lowercase();
        let (games, profiles, _ms) = load_library();
        let fl = frag.to_lowercase();
        let g = games
            .iter()
            .find(|g| g.name.to_lowercase() == fl)
            .or_else(|| {
                games
                    .iter()
                    .find(|g| g.name.to_lowercase().contains(&fl))
            });
        let p = profiles
            .iter()
            .find(|p| p.name.to_lowercase().contains(&pfrag));
        match (g, p) {
            (Some(g), Some(p)) => {
                println!("game={} profile={}", g.name, p.name);
                match launch_with(g, p) {
                    Ok(m) => println!("LAUNCH-OK: {m}"),
                    Err(m) => println!("LAUNCH-ERR: {m}"),
                }
            }
            _ => println!("LAUNCH-ERR: game or profile not found"),
        }
        return;
    }
    // First-run detection happens here and nowhere
    // else: when no settings file exists yet this is a
    // fresh install — probe the machine, record it, set
    // the low-spec default if called for, and only then
    // size the window. Every headless mode above has
    // already returned, and every existing install takes
    // the pure-read path (first_run_profile).
    let profile = first_run_profile();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("EmuFront Rust")
            .with_inner_size(profile.window),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "EmuFront Rust",
        options,
        Box::new(|cc| Ok(Box::new(Proto::new(cc)))),
    );
}

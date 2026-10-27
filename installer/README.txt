EmuFront Rust - Installer
==========================
EmuFront Rust 1.01 - developed by Huntergokuz & Maya

WHAT THIS IS
------------
EmuFrontRust-Setup.exe installs EmuFront Rust, a front end for your
game library: it gathers your ROM folders into one library and launches
each game with the right emulator. Version 1.01 adds the PC Ports
shelf: native Windows ports of classic games (Ship of Harkinian,
Starship, SpaghettiKart and more, installed with Quiver Launcher)
launch directly from EmuFront, alongside DuckStation (PlayStation 1)
support and downloadable texture packs.

TO INSTALL
----------
Double-click EmuFrontRust-Setup.exe and follow the wizard:

1. Welcome - a quick hello.
2. Install location - where EmuFront Rust goes.
   The default is V:\EmuFrontRust; change it if you like.
3. ROM folders - tell Setup where your game ROMs live. You can add
   as many folders as you like (type a path and click Add, or use
   Browse...), and remove ones you don't want. If EmuFront is already
   set up, the list starts with your current library folders.
   Setup only updates the folder list in your EmuFront settings -
   your games and emulator profiles are never touched, and Setup
   makes a timestamped backup of the settings file first.
4. BIOS files (optional) - point Setup at a folder and click Scan:
   it searches that folder AND every folder inside it for known BIOS
   files and shows you what it found. When the install finishes, the
   found files are copied into the RetroArch system folder, where the
   emulators look for them. Files that are already there are never
   overwritten. Leave the folder blank to skip this step.
5. Additional tasks - choose whether to create a desktop shortcut.
6. Ready / Installing / Finished - Setup reports what it did,
   including how many BIOS files were copied.

WHAT ELSE IT DOES
-----------------
- 7-Zip: EmuFront Rust uses 7-Zip to open archive files. If 7-Zip
  isn't installed, Setup installs it for you automatically
  (the official 7-Zip 26.04 installer is bundled). If you already
  have 7-Zip, nothing happens.
  (7-Zip is free software under the GNU LGPL - see 7-zip.org.)
- Creates a Start Menu entry and, if you chose it, a desktop
  shortcut, both with the EmuFront Rust icon (emufront-rust.ico,
  included in this folder and also installed next to the app).
- Your personal EmuFront Rust settings (emufront_rust.json) are
  created by the app itself and are kept if you reinstall.

TO UNINSTALL
------------
Settings > Apps > EmuFront Rust, or use the Uninstall entry in the
Start Menu. Uninstalling never deletes your ROMs, your library
settings, or any BIOS files.

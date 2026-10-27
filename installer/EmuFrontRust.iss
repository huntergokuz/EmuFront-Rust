; ============================================================
;  EmuFront Rust - Setup script (Inno Setup 6)
;  App:      EmuFront Rust (native egui build), packaged exe is
;            staged next to this script under pkg\.
;  Design requirements baked in:
;   1. Custom icon (emufront-rust.ico) for Setup + shortcuts.
;   2. Installer asks where to install (default V:\EmuFrontRust).
;   3. Bundles what the app needs: detects 7-Zip (the app shells
;      out to 7z.exe for archives) and silently installs the
;      official 7-Zip installer only when 7-Zip is missing.
;   4. ROM folders page: add MANY library folders during install;
;      the list is merged into the shared EmuFront config's
;      library_dirs key ONLY (nothing else in the config is
;      touched; a timestamped backup is made first).
;   5. BIOS page: pick a folder, recursive scan of ALL
;      subdirectories for known BIOS filenames (bios_data.inc,
;      generated from the RetroArch core .info files on this PC),
;      found files are copied into the RetroArch system folder;
;      files already present are never overwritten.
;  The app's own settings file (emufront_rust.json, beside the
;  exe) is never installed, overwritten, or deleted by Setup.
; ============================================================

#define MyAppName "EmuFront Rust"
#define MyAppVersion "1.01"
#define MyAppPublisher "Huntergokuz"
#define MyAppExeName "EmuFrontRust.exe"

[Setup]
AppId={{7C4A9E12-3B6D-4F05-8A9C-2E5D7B1F04AA}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppComments=Emulator front end - developed by Huntergokuz & Maya
DefaultDirName={localappdata}\Programs\EmuFrontRust
UsePreviousAppDir=no
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=.
OutputBaseFilename=EmuFrontRust-Setup
SetupIconFile=emufront-rust.ico
UninstallDisplayName={#MyAppName} {#MyAppVersion}
UninstallDisplayIcon={app}\emufront-rust.ico
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
VersionInfoProductName={#MyAppName}
VersionInfoVersion=1.1.0.0

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; \
    GroupDescription: "Shortcuts:"

[Files]
; The app itself (single native exe, no runtime dependencies).
Source: "pkg\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
; The icon, installed beside the exe so shortcuts and the
; uninstall entry keep using it.
Source: "emufront-rust.ico"; DestDir: "{app}"; Flags: ignoreversion
; 7-Zip (official installer, unmodified) - only extracted when
; 7-Zip is not already installed; run silently from [Run].
; 7-Zip is free software under the GNU LGPL; see 7-zip.org.
Source: "pkg\7z-setup.exe"; DestDir: "{tmp}"; \
    Flags: deleteafterinstall; Check: not Is7ZipPresent

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; \
    IconFilename: "{app}\emufront-rust.ico"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{userdesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; \
    IconFilename: "{app}\emufront-rust.ico"; Tasks: desktopicon

[Run]
; Install 7-Zip first if it was missing (its installer may ask
; for admin rights via UAC; that is 7-Zip's own prompt).
Filename: "{tmp}\7z-setup.exe"; Parameters: "/S"; \
    StatusMsg: "Installing 7-Zip (needed for archive support)..."; \
    Flags: runhidden waituntilterminated; Check: not Is7ZipPresent
Filename: "{app}\{#MyAppExeName}"; \
    Description: "Launch {#MyAppName}"; \
    Flags: nowait postinstall skipifsilent

[Code]
#include "bios_data.inc"

var
  RomPage: TWizardPage;
  RomList: TNewListBox;
  RomEdit: TNewEdit;
  BiosPage: TWizardPage;
  BiosRootEdit: TNewEdit;
  BiosDestEdit: TNewEdit;
  BiosListBox: TNewListBox;
  BiosStatus: TNewStaticText;
  BiosFoundSrc: TStringList;
  BiosFoundDest: TStringList;
  OrigRomDirs: String;
  InstallSummary: String;
  BiosNames: TStringList;
  BiosDests: TStringList;
  BiosStrict: TStringList;

{ ---------- small helpers ---------- }

function ConfigPath: String;
begin
  Result := GetEnv('USERPROFILE') + '\maya\EmuFront\emufront_config.json';
end;

function JoinDirs(List: TStrings): String;
var
  I: Integer;
begin
  Result := '';
  for I := 0 to List.Count - 1 do
  begin
    if I > 0 then Result := Result + '|';
    Result := Result + List[I];
  end;
end;

{ ---------- shared-config JSON: library_dirs span surgery ----------
  The config is a ~59 MB JSON written by Python json.dump
  (pure ASCII, keys: library_dirs, emulators, games,
  manual_games). We locate the "library_dirs" array by bracket
  matching that respects quoted strings and backslash escapes,
  and replace ONLY that span. Everything else stays identical.
  Inno's file functions use AnsiString; the file is verified
  pure ASCII, so the round-trip is byte-exact. }

function FindArraySpan(const S, Key: AnsiString;
  var StartPos, EndPos: Integer): Boolean;
var
  KeyPos, I, Depth: Integer;
  InStr, Esc: Boolean;
  C1: AnsiString;
begin
  Result := False;
  KeyPos := Pos('"' + Key + '"', S);
  if KeyPos = 0 then Exit;
  I := Pos('[', Copy(S, KeyPos, Length(S)));
  if I = 0 then Exit;
  StartPos := KeyPos + I - 1;
  Depth := 0;
  InStr := False;
  Esc := False;
  I := StartPos;
  while I <= Length(S) do
  begin
    C1 := Copy(S, I, 1);
    if InStr then
    begin
      if Esc then Esc := False
      else if C1 = '\' then Esc := True
      else if C1 = '"' then InStr := False;
    end
    else
    begin
      if C1 = '"' then InStr := True
      else if C1 = '[' then Depth := Depth + 1
      else if C1 = ']' then
      begin
        Depth := Depth - 1;
        if Depth = 0 then
        begin
          EndPos := I;
          Result := True;
          Exit;
        end;
      end;
    end;
    I := I + 1;
  end;
end;

procedure ParseStringArray(const S: AnsiString;
  StartPos, EndPos: Integer; OutList: TStringList);
var
  I: Integer;
  InStr, Esc: Boolean;
  C1: AnsiString;
  Cur: AnsiString;
begin
  OutList.Clear;
  InStr := False;
  Esc := False;
  Cur := '';
  I := StartPos;
  while I <= EndPos do
  begin
    C1 := Copy(S, I, 1);
    if InStr then
    begin
      if Esc then
      begin
        Esc := False;
        if C1 = 'n' then Cur := Cur + #10
        else if C1 = 't' then Cur := Cur + #9
        else if C1 = 'r' then Cur := Cur + #13
        else if C1 = 'u' then
        begin
          { \uXXXX - ROM paths are plain ASCII drive paths; skip
            the 4 hex digits (practically unreachable). }
          I := I + 4;
        end
        else Cur := Cur + C1;
      end
      else if C1 = '\' then Esc := True
      else if C1 = '"' then
      begin
        InStr := False;
        OutList.Add(String(Cur));
        Cur := '';
      end
      else Cur := Cur + C1;
    end
    else if C1 = '"' then InStr := True;
    I := I + 1;
  end;
end;

function JsonEscapeA(const S: AnsiString): AnsiString;
var
  I: Integer;
  C1: AnsiString;
begin
  Result := '';
  for I := 1 to Length(S) do
  begin
    C1 := Copy(S, I, 1);
    if (C1 = '\') or (C1 = '"') then Result := Result + '\';
    Result := Result + C1;
  end;
end;

function BuildArrayText(List: TStrings): AnsiString;
var
  I: Integer;
begin
  Result := '[';
  for I := 0 to List.Count - 1 do
  begin
    if I > 0 then Result := Result + ',';
    Result := Result + #13#10 + '  "' +
      JsonEscapeA(AnsiString(List[I])) + '"';
  end;
  if List.Count > 0 then Result := Result + #13#10 + ' ';
  Result := Result + ']';
end;

procedure ReadExistingRomDirs(OutList: TStringList);
var
  S: AnsiString;
  SP, EP: Integer;
begin
  OutList.Clear;
  OrigRomDirs := '';
  if not FileExists(ConfigPath) then Exit;
  if not LoadStringFromFile(ConfigPath, S) then Exit;
  if FindArraySpan(S, 'library_dirs', SP, EP) then
  begin
    ParseStringArray(S, SP, EP, OutList);
    OrigRomDirs := JoinDirs(OutList);
  end;
end;

procedure MergeRomDirsIntoConfig;
var
  S, NewS: AnsiString;
  Cfg, Bak, Tmp: String;
  SP, EP: Integer;
  Dirs: TStringList;
  I: Integer;
begin
  { Only write when the user actually changed the list. }
  if JoinDirs(RomList.Items) = OrigRomDirs then
  begin
    InstallSummary := InstallSummary +
      'ROM folders: unchanged (already in the EmuFront config).' + #13#10;
    Exit;
  end;
  Cfg := ConfigPath;
  Dirs := TStringList.Create;
  try
    for I := 0 to RomList.Items.Count - 1 do
      Dirs.Add(RomList.Items[I]);
    if FileExists(Cfg) then
    begin
      if not LoadStringFromFile(Cfg, S) then
      begin
        InstallSummary := InstallSummary +
          'ROM folders: could not read the existing EmuFront config; left unchanged.' + #13#10;
        Exit;
      end;
      if not FindArraySpan(S, 'library_dirs', SP, EP) then
      begin
        InstallSummary := InstallSummary +
          'ROM folders: library_dirs not found in the EmuFront config; left unchanged.' + #13#10;
        Exit;
      end;
      { Timestamped backup before the one-key edit. }
      Bak := Cfg + '.bak-installer-' +
        GetDateTimeString('yyyymmdd-hhnnss', '-', '-');
      if not CopyFile(Cfg, Bak, False) then
      begin
        InstallSummary := InstallSummary +
          'ROM folders: could not back up the EmuFront config; left unchanged.' + #13#10;
        Exit;
      end;
      NewS := Copy(S, 1, SP - 1) + BuildArrayText(Dirs) +
        Copy(S, EP + 1, Length(S));
      { Write via temp file + rename, same pattern as the apps.
        The original is only deleted after the temp write is
        known-good; if the rename then fails we roll back from
        the backup. }
      Tmp := Cfg + '.tmp-installer';
      if not SaveStringToFile(Tmp, NewS, False) then
      begin
        InstallSummary := InstallSummary +
          'ROM folders: temp write failed; config left unchanged.' + #13#10;
        Exit;
      end;
      DeleteFile(Cfg);
      if not RenameFile(Tmp, Cfg) then
      begin
        { Roll back from the backup if the rename failed. }
        CopyFile(Bak, Cfg, False);
        InstallSummary := InstallSummary +
          'ROM folders: write failed; config restored from backup.' + #13#10;
        Exit;
      end;
      InstallSummary := InstallSummary +
        'ROM folders: ' + IntToStr(Dirs.Count) +
        ' folder(s) saved to the EmuFront config (backup: ' +
        ExtractFileName(Bak) + ').' + #13#10;
    end
    else
    begin
      { Fresh machine: create a minimal config the app can load. }
      ForceDirectories(ExtractFilePath(Cfg));
      S := '{' + #13#10 + ' "library_dirs": ' +
        BuildArrayText(Dirs) + ',' + #13#10 +
        ' "emulators": [],' + #13#10 +
        ' "games": [],' + #13#10 +
        ' "manual_games": []' + #13#10 + '}';
      SaveStringToFile(Cfg, S, False);
      InstallSummary := InstallSummary +
        'ROM folders: new EmuFront config created with ' +
        IntToStr(Dirs.Count) + ' folder(s).' + #13#10;
    end;
  finally
    Dirs.Free;
  end;
end;

{ ---------- 7-Zip detection ---------- }

function Is7ZipPresent: Boolean;
begin
  Result :=
    FileExists(ExpandConstant('{commonpf}\7-Zip\7z.exe')) or
    FileExists(ExpandConstant('{commonpf64}\7-Zip\7z.exe'));
end;

{ ---------- BIOS matching + recursive scan ---------- }

procedure LoadBiosData;
var
  Data, Token, Name, Dest: String;
  I, Eq: Integer;
begin
  if BiosNames <> nil then Exit;
  BiosNames := TStringList.Create;
  BiosDests := TStringList.Create;
  BiosStrict := TStringList.Create;
  { Manual '|' split of name=dest pairs (no DelimitedText quirks). }
  Data := BiosDataString + '|';
  Token := '';
  for I := 1 to Length(Data) do
  begin
    if Data[I] = '|' then
    begin
      if Token <> '' then
      begin
        Eq := Pos('=', Token);
        if Eq > 0 then
        begin
          Name := Lowercase(Copy(Token, 1, Eq - 1));
          Dest := Copy(Token, Eq + 1, Length(Token));
        end
        else
        begin
          Name := Lowercase(Token);
          Dest := '';
        end;
        BiosNames.Add(Name);
        BiosDests.Add(Dest);
      end;
      Token := '';
    end
    else
      Token := Token + Data[I];
  end;
  Data := BiosStrictString + '|';
  Token := '';
  for I := 1 to Length(Data) do
  begin
    if Data[I] = '|' then
    begin
      if Token <> '' then BiosStrict.Add(Lowercase(Token));
      Token := '';
    end
    else
      Token := Token + Data[I];
  end;
end;

function BiosDestFor(const FileName, ParentName: String;
  var DestFolder: String): Boolean;
var
  N: String;
  Idx: Integer;
begin
  Result := False;
  LoadBiosData;
  N := Lowercase(FileName);
  Idx := BiosNames.IndexOf(N);
  if Idx >= 0 then
  begin
    DestFolder := BiosDests[Idx];
    if BiosStrict.IndexOf(N) >= 0 then
    begin
      { Generic basename: the immediate parent folder must be
        the folder the BIOS belongs in (e.g. SGB1.sfc). }
      Result := CompareText(ParentName,
        ExtractFileName(DestFolder)) = 0;
    end
    else
      Result := True;
    Exit;
  end;
  { Pattern rule: PlayStation 1/2 BIOS dumps (scph*.bin,
    including scph-*.bin variants). }
  if (Pos('scph', N) = 1) and (ExtractFileExt(N) = '.bin') then
  begin
    DestFolder := '';
    Result := True;
  end;
end;

procedure ScanDirRecursive(const Dir: String);
var
  FindRec: TFindRec;
  Full, DestFolder: String;
begin
  if not FindFirst(Dir + '\*', FindRec) then Exit;
  try
    repeat
      if (FindRec.Name <> '.') and (FindRec.Name <> '..') then
      begin
        Full := Dir + '\' + FindRec.Name;
        if (FindRec.Attributes and FILE_ATTRIBUTE_DIRECTORY) <> 0 then
          ScanDirRecursive(Full)
        else if BiosDestFor(FindRec.Name,
            ExtractFileName(Dir), DestFolder) then
        begin
          BiosFoundSrc.Add(Full);
          BiosFoundDest.Add(DestFolder);
          BiosListBox.Items.Add(FindRec.Name + '  (' +
            IntToStr(FindRec.SizeLow) + ' bytes)  -  ' + Full);
        end;
      end;
    until not FindNext(FindRec);
  finally
    FindClose(FindRec);
  end;
end;

procedure BiosScanClick(Sender: TObject);
var
  Root: String;
begin
  Root := Trim(BiosRootEdit.Text);
  BiosFoundSrc.Clear;
  BiosFoundDest.Clear;
  BiosListBox.Items.Clear;
  if (Root = '') or (not DirExists(Root)) then
  begin
    BiosStatus.Caption :=
      'Pick an existing folder first, then press Scan.';
    Exit;
  end;
  BiosStatus.Caption := 'Scanning...';
  ScanDirRecursive(Root);
  if BiosFoundSrc.Count = 0 then
    BiosStatus.Caption :=
      'No known BIOS files found under that folder.'
  else
    BiosStatus.Caption := IntToStr(BiosFoundSrc.Count) +
      ' BIOS file(s) found. They will be copied when setup ' +
      'finishes (files already in the system folder are kept).';
end;

procedure CopyFoundBios;
var
  I, Copied, Skipped, Failed: Integer;
  DestDir, DestFile: String;
begin
  Copied := 0; Skipped := 0; Failed := 0;
  if BiosFoundSrc.Count = 0 then
  begin
    InstallSummary := InstallSummary +
      'BIOS: no files selected for import.' + #13#10;
    Exit;
  end;
  for I := 0 to BiosFoundSrc.Count - 1 do
  begin
    DestDir := Trim(BiosDestEdit.Text);
    if BiosFoundDest[I] <> '' then
      DestDir := DestDir + '\' + BiosFoundDest[I];
    DestFile := DestDir + '\' + ExtractFileName(BiosFoundSrc[I]);
    if FileExists(DestFile) then
    begin
      Skipped := Skipped + 1;
    end
    else
    begin
      ForceDirectories(DestDir);
      if CopyFile(BiosFoundSrc[I], DestFile, False) then
        Copied := Copied + 1
      else
        Failed := Failed + 1;
    end;
  end;
  InstallSummary := InstallSummary + 'BIOS import: ' +
    IntToStr(Copied) + ' copied, ' + IntToStr(Skipped) +
    ' already present (kept), ' + IntToStr(Failed) +
    ' failed.' + #13#10;
end;

{ ---------- wizard pages ---------- }

procedure RomAddClick(Sender: TObject);
var
  D: String;
begin
  D := Trim(RomEdit.Text);
  if D <> '' then
  begin
    if RomList.Items.IndexOf(D) < 0 then RomList.Items.Add(D);
    RomEdit.Text := '';
  end;
end;

procedure RomBrowseClick(Sender: TObject);
var
  D: String;
begin
  D := Trim(RomEdit.Text);
  if BrowseForFolder('Pick a ROM folder:', D, True) then
  begin
    if RomList.Items.IndexOf(D) < 0 then RomList.Items.Add(D);
  end;
end;

procedure RomRemoveClick(Sender: TObject);
begin
  if RomList.ItemIndex >= 0 then
    RomList.Items.Delete(RomList.ItemIndex);
end;

procedure BiosBrowseClick(Sender: TObject);
var
  D: String;
begin
  D := Trim(BiosRootEdit.Text);
  if BrowseForFolder('Pick the folder to scan for BIOS files:', D, False) then
    BiosRootEdit.Text := D;
end;

procedure InitializeWizard;
var
  Lbl: TNewStaticText;
  Btn: TNewButton;
  Existing: TStringList;
  I: Integer;
begin
  BiosFoundSrc := TStringList.Create;
  BiosFoundDest := TStringList.Create;

  { ---- ROM folders page (multi-add) ---- }
  RomPage := CreateCustomPage(wpSelectDir,
    'ROM folders',
    'Where are your game ROMs? Add as many folders as you like - ' +
    'EmuFront Rust will scan them into one library.');
  Lbl := TNewStaticText.Create(RomPage);
  Lbl.Parent := RomPage.Surface;
  Lbl.Left := 0; Lbl.Top := 0; Lbl.Width := RomPage.SurfaceWidth;
  Lbl.Caption := 'Library folders:';
  RomList := TNewListBox.Create(RomPage);
  RomList.Parent := RomPage.Surface;
  RomList.Left := 0; RomList.Top := 20;
  RomList.Width := RomPage.SurfaceWidth;
  RomList.Height := 190;
  RomEdit := TNewEdit.Create(RomPage);
  RomEdit.Parent := RomPage.Surface;
  RomEdit.Left := 0; RomEdit.Top := 220;
  RomEdit.Width := RomPage.SurfaceWidth - 190;
  Btn := TNewButton.Create(RomPage);
  Btn.Parent := RomPage.Surface;
  Btn.Left := RomPage.SurfaceWidth - 180; Btn.Top := 218;
  Btn.Width := 85; Btn.Height := 25;
  Btn.Caption := '&Add';
  Btn.OnClick := @RomAddClick;
  Btn := TNewButton.Create(RomPage);
  Btn.Parent := RomPage.Surface;
  Btn.Left := RomPage.SurfaceWidth - 90; Btn.Top := 218;
  Btn.Width := 90; Btn.Height := 25;
  Btn.Caption := '&Browse...';
  Btn.OnClick := @RomBrowseClick;
  Btn := TNewButton.Create(RomPage);
  Btn.Parent := RomPage.Surface;
  Btn.Left := 0; Btn.Top := 252;
  Btn.Width := 110; Btn.Height := 25;
  Btn.Caption := '&Remove selected';
  Btn.OnClick := @RomRemoveClick;

  Existing := TStringList.Create;
  try
    ReadExistingRomDirs(Existing);
    for I := 0 to Existing.Count - 1 do
      RomList.Items.Add(Existing[I]);
  finally
    Existing.Free;
  end;

  { ---- BIOS scan page ---- }
  BiosPage := CreateCustomPage(RomPage.ID,
    'BIOS files',
    'Optional: point setup at a folder and it will scan every ' +
    'subfolder for known BIOS files and install the ones your ' +
    'emulators need.');
  Lbl := TNewStaticText.Create(BiosPage);
  Lbl.Parent := BiosPage.Surface;
  Lbl.Left := 0; Lbl.Top := 0; Lbl.Width := BiosPage.SurfaceWidth;
  Lbl.Caption := 'Folder to scan (all subfolders are searched):';
  BiosRootEdit := TNewEdit.Create(BiosPage);
  BiosRootEdit.Parent := BiosPage.Surface;
  BiosRootEdit.Left := 0; BiosRootEdit.Top := 20;
  BiosRootEdit.Width := BiosPage.SurfaceWidth - 190;
  Btn := TNewButton.Create(BiosPage);
  Btn.Parent := BiosPage.Surface;
  Btn.Left := BiosPage.SurfaceWidth - 180; Btn.Top := 18;
  Btn.Width := 85; Btn.Height := 25;
  Btn.Caption := '&Browse...';
  Btn.OnClick := @BiosBrowseClick;
  Btn := TNewButton.Create(BiosPage);
  Btn.Parent := BiosPage.Surface;
  Btn.Left := BiosPage.SurfaceWidth - 90; Btn.Top := 18;
  Btn.Width := 90; Btn.Height := 25;
  Btn.Caption := '&Scan';
  Btn.OnClick := @BiosScanClick;
  BiosListBox := TNewListBox.Create(BiosPage);
  BiosListBox.Parent := BiosPage.Surface;
  BiosListBox.Left := 0; BiosListBox.Top := 55;
  BiosListBox.Width := BiosPage.SurfaceWidth;
  BiosListBox.Height := 130;
  BiosStatus := TNewStaticText.Create(BiosPage);
  BiosStatus.Parent := BiosPage.Surface;
  BiosStatus.Left := 0; BiosStatus.Top := 192;
  BiosStatus.Width := BiosPage.SurfaceWidth;
  BiosStatus.Height := 30;
  BiosStatus.Caption :=
    'Nothing scanned yet. Leave blank to skip BIOS import.';
  Lbl := TNewStaticText.Create(BiosPage);
  Lbl.Parent := BiosPage.Surface;
  Lbl.Left := 0; Lbl.Top := 228; Lbl.Width := BiosPage.SurfaceWidth;
  Lbl.Caption :=
    'BIOS files go to this RetroArch system folder:';
  BiosDestEdit := TNewEdit.Create(BiosPage);
  BiosDestEdit.Parent := BiosPage.Surface;
  BiosDestEdit.Left := 0; BiosDestEdit.Top := 248;
  BiosDestEdit.Width := BiosPage.SurfaceWidth;
  BiosDestEdit.Text := 'C:\RetroArch\system';
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    MergeRomDirsIntoConfig;
    CopyFoundBios;
  end;
end;

procedure CurPageChanged(CurPageID: Integer);
begin
  if (CurPageID = wpFinished) and (InstallSummary <> '') then
  begin
    WizardForm.FinishedLabel.Caption :=
      WizardForm.FinishedLabel.Caption + #13#10 + #13#10 +
      InstallSummary;
  end;
end;

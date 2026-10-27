{ BIOS match data for EmuFront Rust Setup. }
{ Generated from the firmware paths declared by the RetroArch }
{ core .info files installed on this PC (same source the Python }
{ EmuFront BIOS Manager reads), plus a few standalone classics. }
{ Format: basename=destfolder pairs joined by '|'; dest '' = the }
{ RetroArch system root. Strict names also need parent-folder match. }
function BiosDataString: String;
begin
  Result :=
    '000-lo.lo=neocd|128p-0.rom=fuse|128p-1.rom=fuse|256s-0.rom=fuse|256s-1.rom=fuse|256s-2.rom=fuse|256s-3.rom=fuse|3do_arcade_saot.bin=|5200.rom=|64dd_ipl.bin=|7800 bios (u).rom=' +
    '8.bin=bbkemu\a4980|aes.zip=|airlbios.zip=dc|areplay.bin=|ataribas.rom=|atariosa.rom=|atariosb.rom=|atarixl.rom=|awbios.zip=dc|b11m_bos.rom=bk|b11m_ext.rom=bk|bas11m_0.rom=bk' +
    'bas11m_1.rom=bk|basic10.rom=bk|basic20.rom=ep128emu\roms|basic21.rom=ep128emu\roms|bb01r4_os.rom=|bios.cof=nuance|bios.col=|bios.gg=|bios.min=|bios.rom=np2kai|bios.sms=|bios7.bin=' +
    'bios9.bin=|bios_cd_e.bin=|bios_cd_j.bin=|bios_cd_u.bin=|bios_e.sms=|bios_j.sms=|bios_md.bin=|bios_u.sms=|bioscv.rom=|boot.bin=dc|bootloader-dbvz.rom=|brd.rom=ep128emu\roms' +
    'bs-x.bin=|bubsys.zip=fbneo|c52.bin=|cchip.zip=fbneo|cdibios.zip=same_cdi\bios|cdimono1.zip=|cdimono2.zip=same_cdi\bios|cgb_bios.bin=|cgb_boot.bin=|channelf.zip=fbneo' +
    'chrgen.bin=galaksija|cm32l_control.rom=|cm32l_pcm.rom=|codehandler.bin=dolphin-emu\sys|coleco.rom=|coleco.zip=fbneo|colecovision.rom=|cpc464.rom=ep128emu\roms' +
    'cpc6128.rom=ep128emu\roms|cpc664.rom=ep128emu\roms|cpc_amsdos.rom=ep128emu\roms|cx4.data.rom=|dc_boot.bin=dc|dc_flash.bin=dc|decocass.zip=fbneo|disk.rom=|disk_327.rom=bk' +
    'disksys.rom=|dmg_boot.bin=|dsi_bios7.bin=|dsi_bios9.bin=|dsi_firmware.bin=|dsi_nand.bin=|dsi_sd_card.bin=|dsp1.data.rom=|dsp1.program.rom=|dsp1b.data.rom=|dsp1b.program.rom=' +
    'dsp2.data.rom=|dsp2.program.rom=|dsp3.data.rom=|dsp3.program.rom=|dsp4.data.rom=|dsp4.program.rom=|e.bin=bbkemu\a4980|eos.rom=|epd19hft.rom=ep128emu\roms' +
    'epfileio.rom=ep128emu\roms|erom.bin=|exdos13.rom=ep128emu\roms|exec.bin=|exos20.rom=ep128emu\roms|exos21.rom=ep128emu\roms|exos24uk.rom=ep128emu\roms|f355bios.zip=dc' +
    'f355dlx.zip=dc|fdsbios.zip=fbneo|firmware.bin=|flash.bin=dc|fmpac.rom=|fmt_dic.rom=geartowns|fmt_dos.rom=geartowns|fmt_f20.rom=geartowns|fmt_fnt.rom=geartowns' +
    'fmt_sys.rom=geartowns|focal10.rom=bk|font.bmp=np2kai|font.rom=np2kai|front-sp1.bin=neocd|g7400.bin=|gamegenie.nes=|gameindex.yaml=pcsx2\resources|gb_bios.bin=|gba_bios.bin=' +
    'gbc_bios.bin=|gexpress.pce=|ggenie.bin=|gluck.rom=fuse|goldstar.bin=|grim-patch.lab=scummvm\extra|grom.bin=|hod2bios.zip=dc|hun.rom=ep128emu\roms|ipl.bin=dolphin-emu\sys\gc\eur' +
    'ipl.n64=mupen64plus|iplrom.x1=xmil|iplrom.x1t=xmil|irrmaze.zip=|isgsm.zip=fbneo|itf.rom=np2kai|jiffydos_1541-ii.bin=vice|jiffydos_1571_repl310654.bin=vice|jiffydos_1581.bin=vice' +
    'jiffydos_c128.bin=vice|jiffydos_c64.bin=vice|jopac.bin=|kanji.rom=|kick33180.a500=|kick34005.a500=|kick34005.cdtv=|kick37175.a500=|kick37350.a600=|kick39106.a1200=' +
    'kick39106.a4000=|kick40060.cd32=|kick40060.cd32.ext=|kick40063.a600=|kick40068.a1200=|kick40068.a4000=|lynxboot.img=|macii.rom=|midssio.zip=fbneo|minibios.cof=nuance' +
    'minibiosx.cof=nuance|monit10.rom=bk|monkey4-patch.m4b=scummvm\extra|mpr-17933.bin=|mpr-18811-mx.ic1=|mpr-19367-mx.ic1=|msx.rom=|msx.zip=fbneo|msx2.rom=|msx2ext.rom=|msx2p.rom=' +
    'msx2pext.rom=|msxdos2.rom=|mt32_control.rom=|mt32_pcm.rom=|n88.rom=quasi88|n88_0.rom=quasi88|n88_1.rom=quasi88|n88_2.rom=quasi88|n88_3.rom=quasi88|n88jisho.rom=quasi88' +
    'n88knj1.rom=quasi88|n88knj2.rom=quasi88|n88n.rom=quasi88|namcoc69.zip=fbneo|namcoc70.zip=fbneo|namcoc75.zip=fbneo|naomi.zip=dc|naomi2.zip=dc|nds_sd_card.bin=|neocd.bin=neocd' +
    'neocd.zip=|neocd_f.rom=neocd|neocd_sf.rom=neocd|neocd_st.rom=neocd|neocd_sz.rom=neocd|neocd_t.rom=neocd|neocd_z.rom=neocd|neocdz.zip=|neogeo.zip=|nes.pal=|ng-lo.rom=neocd' +
    'ngp.zip=fbneo|nmk004.zip=fbneo|o2rom.bin=|pac-n1.bin=|pac-n10.bin=|painter.rom=|palmos40-en-m500.rom=|palmos41-en-m515.rom=|palmos52-en-t3.rom=|palmos60-en-t3.rom=' +
    'panafz1-kanji.bin=|panafz1.bin=|panafz10-norsa.bin=|panafz10-patched.bin=|panafz10.bin=|panafz10e-anvil-norsa.bin=|panafz10e-anvil-patched.bin=|panafz10e-anvil.bin=' +
    'panafz10ja-anvil-kanji.bin=|panafz1j-kanji.bin=|panafz1j-norsa.bin=|panafz1j.bin=|pce-lp1.bin=|pcfx.rom=|pgm.zip=fbneo|program.rom=sgb1.sfc|ps1_rom.bin=|psxonpsp660.bin=' +
    'queen.tbl=scummvm\extra|residualvm.zip=scummvm\theme|roland_sc-55.sf2=scummvm\extra|rom.db=|rom1.bin=galaksija|rom2.bin=galaksija|sanyotry.bin=|saturn_bios.bin=|scph5500.bin=' +
    'scph5501.bin=|scph5502.bin=|scummclassic.zip=scummvm\theme|scummmodern.zip=scummvm\theme|scummremastered.zip=scummvm\theme|sega_101.bin=|sgb.boot.rom=|sgb1.boot.rom=sgb1.sfc' +
    'sgb1.sfc=|sgb2.boot.rom=sgb2.sfc|sgb2.sfc=|sgb2_boot.bin=|sgb_bios.bin=|sgb_boot.bin=|sk.bin=|sk2chip.bin=|skns.zip=fbneo|sky.cpt=scummvm\extra|sl31253.bin=|sl31254.bin=' +
    'sl90025.bin=|sound.rom=np2kai|spec128.zip=fbneo|spec1282a.zip=fbneo|spectrum.zip=fbneo|squirreljme.sqc=|st010.data.rom=|st010.program.rom=|st011.data.rom=|st011.program.rom=' +
    'st018.data.rom=|st018.program.rom=|stbios.bin=|stvbios.zip=kronos|syscard1.pce=|syscard2.pce=|syscard3.pce=|ti83.rom=|ti83plus.rom=|ti83se.rom=|top-sp1.bin=neocd|tos.img=' +
    'trdos.rom=fuse|tvc22_ext.rom=ep128emu\roms|tvc22_sys.rom=ep128emu\roms|tvc_dos12d.rom=ep128emu\roms|tvcfileio.rom=ep128emu\roms|uni-bioscd.rom=neocd|upd7801g.s01=' +
    'vircon32bios.v32=|vkeybd_default.zip=scummvm\extra|wintermute.zip=scummvm\extra|writer.rom=|xegame.rom=|ym2608.zip=fbneo|zt19hfnt.rom=ep128emu\roms|zt19uk.rom=ep128emu\roms' +
    'zx128.rom=ep128emu\roms|zx48.rom=ep128emu\roms';
end;

function BiosStrictString: String;
begin
  Result :=
    '8.bin|boot.bin|e.bin|flash.bin|program.rom|rom1.bin';
end;

const
  BIOS_ENTRY_COUNT = 298;

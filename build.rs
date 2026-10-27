// Embed the EmuFront Rust icon into the Windows executable
// (a fancy icon that shows in Explorer, on the
// Desktop shortcut, and in the installer). If the .ico is missing
// the build just proceeds without it.
fn main() {
    #[cfg(windows)]
    {
        let ico = std::path::Path::new("icon/emufront-rust.ico");
        if ico.exists() {
            let mut res = winresource::WindowsResource::new();
            res.set_icon("icon/emufront-rust.ico");
            if let Err(e) = res.compile() {
                println!("cargo:warning=icon embed failed: {e}");
            }
        }
    }
}

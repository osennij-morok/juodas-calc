fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("CARGO_CFG_TARGET_FAMILY")? == "windows" {
        let mut res = winres::WindowsResource::new();
        match std::env::var("CARGO_CFG_TARGET_ENV")?.as_str() {
            "gnu" => {
                res.set_ar_path("x86_64-w64-mingw32-ar")
                   .set_windres_path("x86_64-w64-mingw32-windres");
            }
            "msvc" => {}
            _ => panic!("unsupported env"),
        };
        res.set_icon("calculator.ico");
        res.compile()?;
    }
    Ok(())
}
// https://github.com/emilk/egui/discussions/2026

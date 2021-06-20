fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut res = winres::WindowsResource::new();
    // placeholder icon
    // https://iconarchive.com/show/gentle-edges-icons-by-pixelkit/Game-Controller-icon.html
    res.set_icon_with_id("assets/icon.ico", "icon");
    res.compile()?;

    Ok(())
}

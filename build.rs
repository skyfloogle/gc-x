fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut res = winres::WindowsResource::new();
    res.set_icon_with_id("assets/icon.ico", "icon");
    res.compile()?;

    Ok(())
}

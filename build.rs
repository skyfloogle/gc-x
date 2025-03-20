use std::process::Command;
use time::OffsetDateTime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build date
    let date = OffsetDateTime::now_utc();
    println!("cargo:rustc-env=BUILD_DATE={}-{:02}-{:02}", date.year(), date.month() as u8, date.day());

    // git hash
    let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output()?;
    let git_hash = String::from_utf8(output.stdout)?;
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // icon
    let mut res = tauri_winres::WindowsResource::new();
    res.set_icon_with_id("assets/icon.ico", "icon")
        .set("FileDescription", "GC-X")
        .set("ProductName", "GC-X")
        .set("CompanyName", "Floogle");
    res.compile()?;

    Ok(())
}

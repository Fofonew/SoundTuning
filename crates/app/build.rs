const EXECUTABLE_ICON: &str = "assets/icon/SoundTuning.ico";
#[cfg(windows)]
const PRODUCT_NAME: &str = "SoundTuning";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={EXECUTABLE_ICON}");
    let targets_windows = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "windows");
    if targets_windows {
        embed_windows_resources()?;
    }
    Ok(())
}

#[cfg(windows)]
fn embed_windows_resources() -> Result<(), Box<dyn std::error::Error>> {
    winresource::WindowsResource::new()
        .set_icon(EXECUTABLE_ICON)
        .set("ProductName", PRODUCT_NAME)
        .set("FileDescription", PRODUCT_NAME)
        .set("InternalName", PRODUCT_NAME)
        .compile()?;
    Ok(())
}

#[cfg(not(windows))]
fn embed_windows_resources() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

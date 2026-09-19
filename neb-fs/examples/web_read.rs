use neb_fs::{VirtualFile, WEB_FILE_SYSTEM};

const LINK: &'static str = "https://images.rawpixel.com/image_png_800/cHJpdmF0ZS9sci9pbWFnZXMvd2Vic2l0ZS8yMDIzLTA0L3B4NjgyMDU0LWltYWdlLWpvYjE4MTlfMS5wbmc.png";
const OUTPUT: &'static str = "test-web-read.png";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = VirtualFile::open(WEB_FILE_SYSTEM, LINK);
    let bytes = file.read_bytes().await?;
    std::fs::write(OUTPUT, bytes)?;
    Ok(())
}

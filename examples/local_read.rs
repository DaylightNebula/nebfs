use anyhow::bail;
use neb_fs::{LOCAL_FILE_SYSTEM, File};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let true_content = std::fs::read("examples/local_read.rs")?;

    let nebfs_file = File::open(LOCAL_FILE_SYSTEM, "examples/local_read.rs");
    let nebfs_content = nebfs_file.read_bytes().await?;
    if true_content != nebfs_content { bail!("True and NebFS contents do not match") }

    println!("File contents match");

    Ok(())
}
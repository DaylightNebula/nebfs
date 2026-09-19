use neb_fs::{VirtualFile, LOCAL_FILE_SYSTEM};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = VirtualFile::open(LOCAL_FILE_SYSTEM, "test.txt");
    file.write_text("HELLO WORLD\nTHIS IS SOME test text!").await?;
    Ok(())
}
use neb_fs::{VirtualFile, local_file_system};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = VirtualFile::open(local_file_system(), "test.txt");
    file.write_text("HELLO WORLD\nTHIS IS SOME test text!").await?;
    Ok(())
}
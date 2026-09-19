use neb_fs::{VirtualFile, local_file_system};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = VirtualFile::open(local_file_system(), "test_stream.txt");
    let mut stream = file.as_write_stream().await?;
    stream.append_bytes("test".into()).await?;
    stream.append_bytes("hello".into()).await?;
    stream.append_bytes("world".into()).await?;
    stream.append_bytes("!!".into()).await?;
    println!("Stream complete!");
    Ok(())
}

use neb_fs::{File, LOCAL_FILE_SYSTEM};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = File::open(LOCAL_FILE_SYSTEM, "test_stream.txt");
    let mut stream = file.as_write_stream().await?;
    stream.append_bytes("test".into()).await?;
    stream.append_bytes("hello".into()).await?;
    stream.append_bytes("world".into()).await?;
    stream.append_bytes("!!".into()).await?;
    println!("Stream complete!");
    Ok(())
}

use neb_fs::{VirtualFile, local_file_system};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = VirtualFile::open(local_file_system(), "examples/local_read_stream.rs");
    let mut stream = file.as_read_stream().await?;
    while let Ok(bytes) = stream.stream_bytes(2).await {
        let str = String::from_utf8(bytes)?;
        if str.len() <= 0 { break }
        println!("Streamed two bytes {str:?}");
    }
    println!("Stream complete!");
    Ok(())
}

use neb_fs::{File, LOCAL_FILE_SYSTEM};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = File::open(LOCAL_FILE_SYSTEM, "examples/local_read_stream.rs");
    let mut stream = file.as_read_stream().await?;
    while let Ok(bytes) = stream.stream_bytes(2).await {
        let str = String::from_utf8(bytes)?;
        if str.len() <= 0 { break }
        println!("Streamed two bytes {str:?}");
    }
    println!("Stream complete!");
    Ok(())
}

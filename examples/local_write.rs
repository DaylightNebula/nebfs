use neb_fs::{File, LOCAL_FILE_SYSTEM};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = File::open(LOCAL_FILE_SYSTEM, "test.txt");
    file.write_text("HELLO WORLD\nTHIS IS SOME test text!").await?;
    Ok(())
}
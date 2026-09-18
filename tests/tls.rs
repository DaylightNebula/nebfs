#[tokio::test]
async fn https_works_without_aws_lc() {
    use neb_fs::{FileSystem, WEB_FILE_SYSTEM};
    let t = WEB_FILE_SYSTEM.read_text("https://example.com").await.unwrap();
    assert!(t.contains("Example Domain"), "got: {}", &t[..t.len().min(200)]);
}

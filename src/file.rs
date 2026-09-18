use crate::FileSystem;

pub struct File<'a> {
    file_system: &'a dyn FileSystem,
    path: &'a str
}

impl <'a> File<'a> {
    pub fn open(file_system: &'a dyn FileSystem, path: impl Into<&'a str>) -> Self {
        Self { file_system, path: path.into() }
    }

    pub async fn read_bytes(&self) -> anyhow::Result<Vec<u8>> {
        self.file_system.read_bytes(self.path).await
    }

    pub async fn read_text(&self) -> anyhow::Result<String> {
        self.file_system.read_text(self.path).await
    }

    pub async fn write_bytes(&self, bytes: Vec<u8>) -> anyhow::Result<()> {
        self.file_system.write_bytes(self.path, bytes).await
    }

    pub async fn write_text(&self, text: impl Into<String>) -> anyhow::Result<()> {
        self.file_system.write_text(self.path, text.into()).await
    }
}

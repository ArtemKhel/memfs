use std::{collections::HashMap, sync::Arc};

use error::FileSystemError;
use file::File;
use tokio::sync::RwLock;

pub mod error;
mod file;

pub type Result<T> = std::result::Result<T, FileSystemError>;

/// An in-memory file system that stores files as byte arrays.
///
/// This file system only supports files (no directories) and provides
/// basic operations for creating, reading, and writing files.
///
/// # Examples
///
/// ```rust
/// use memfs::FileSystem;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let fs = FileSystem::new();
///
///     fs.touch("/log.txt").await?;
///
///     fs.write("/log.txt", 0, b"hello").await?;
///     fs.write("/log.txt", 5, b" world").await?;
///
///     let content = fs.read("/log.txt", 0, 11).await?;
///     assert_eq!(content, b"hello world");
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct FileSystem {
    files: Arc<RwLock<HashMap<String, File>>>,
}

impl FileSystem {
    /// Creates a new empty file system.
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a file at the specified path if it doesn't exist.
    ///
    /// If the file already exists, this operation does nothing.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the file should be created
    ///
    /// # Errors
    ///
    /// Returns `FileSystemError::InvalidPath` if the path is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// let fs = memfs::FileSystem::new();
    /// fs.touch("/file.txt").await.unwrap();
    /// # });
    /// ```
    pub async fn touch(&self, path: &str) -> Result<()> {
        if path.is_empty() {
            return Err(FileSystemError::InvalidPath("Path cannot be empty".to_string()));
        }

        let mut files = self.files.write().await;
        files.entry(path.to_string()).or_insert_with(File::new);
        Ok(())
    }

    /// Writes data to a file at the specified offset.
    ///
    /// If the file doesn't exist, it will be created. If the offset is beyond
    /// the current file size, the file will be extended with zero bytes.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file
    /// * `offset` - The byte offset where to start writing
    /// * `data` - The data to write
    ///
    /// # Errors
    ///
    /// Returns `FileSystemError::InvalidPath` if the path is empty.
    /// Returns `FileSystemError::WriteError` if the operation would cause overflow.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// let fs = memfs::FileSystem::new();
    /// fs.write("/file.txt", 0, b"hello").await.unwrap();
    /// fs.write("/file.txt", 5, b" world").await.unwrap();
    /// # });
    /// ```
    pub async fn write(&self, path: &str, offset: usize, data: &[u8]) -> Result<()> {
        if path.is_empty() {
            return Err(FileSystemError::InvalidPath("Path cannot be empty".to_string()));
        }

        if offset.checked_add(data.len()).is_none() {
            return Err(FileSystemError::WriteError(
                "Write operation would cause overflow".to_string(),
            ));
        }

        let mut files = self.files.write().await;
        let file = files.entry(path.to_string()).or_insert_with(File::new);
        file.write(offset, data);
        Ok(())
    }

    /// Reads data from a file starting at the specified offset.
    ///
    /// If the file doesn't exist, returns an empty vector. If the offset is
    /// beyond the file size, returns an empty vector. If the requested length
    /// extends beyond the file, returns as much data as possible.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file
    /// * `offset` - The byte offset where to start reading
    /// * `len` - The number of bytes to read
    ///
    /// # Returns
    ///
    /// A vector containing the read data, which may be shorter than `len`
    /// if the file is smaller than requested.
    ///
    /// # Errors
    ///
    /// Returns `FileSystemError::InvalidPath` if the path is empty.
    /// Returns `FileSystemError::ReadError` if the operation would cause overflow.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # tokio_test::block_on(async {
    /// let fs = memfs::FileSystem::new();
    /// fs.write("/file.txt", 0, b"hello world").await.unwrap();
    ///
    /// let content = fs.read("/file.txt", 0, 5).await.unwrap();
    /// assert_eq!(content, b"hello");
    ///
    /// let content = fs.read("/file.txt", 6, 100).await.unwrap();
    /// assert_eq!(content, b"world");
    /// # });
    /// ```
    pub async fn read(&self, path: &str, offset: usize, len: usize) -> Result<Vec<u8>> {
        if path.is_empty() {
            return Err(FileSystemError::InvalidPath("Path cannot be empty".to_string()));
        }

        if len == 0 {
            return Ok(Vec::new());
        }

        if offset.checked_add(len).is_none() {
            return Err(FileSystemError::ReadError(
                "Read operation would cause overflow".to_string(),
            ));
        }

        let files = self.files.read().await;
        match files.get(path) {
            Some(file) => Ok(file.read(offset, len)),
            None => Ok(Vec::new()),
        }
    }
}

impl Default for FileSystem {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_operations() -> Result<()> {
        let fs = FileSystem::new();

        fs.touch("/log.txt").await?;

        fs.write("/log.txt", 0, b"hello").await?;
        fs.write("/log.txt", 5, b" world").await?;

        let content = fs.read("/log.txt", 0, 11).await?;
        assert_eq!(content, b"hello world");

        Ok(())
    }

    #[tokio::test]
    async fn test_read_beyond_file() -> Result<()> {
        let fs = FileSystem::new();

        fs.touch("/test.txt").await?;
        fs.write("/test.txt", 0, b"hello").await?;

        let content = fs.read("/test.txt", 3, 10).await?;
        assert_eq!(content, b"lo");

        let content = fs.read("/test.txt", 10, 5).await?;
        assert_eq!(content, b"");

        Ok(())
    }

    #[tokio::test]
    async fn test_write_with_gap() -> Result<()> {
        let fs = FileSystem::new();

        fs.touch("/gap.txt").await?;

        fs.write("/gap.txt", 5, b"world").await?;

        let content = fs.read("/gap.txt", 0, 10).await?;
        assert_eq!(content, b"\0\0\0\0\0world");

        Ok(())
    }

    #[tokio::test]
    async fn test_write_with_gap_2() -> Result<()> {
        let fs = FileSystem::new();

        fs.touch("/gap2.txt").await?;

        fs.write("/gap2.txt", 0, b"hello").await?;
        fs.write("/gap2.txt", 10, b"world").await?;

        let content = fs.read("/gap2.txt", 0, 15).await?;
        assert_eq!(content, b"hello\0\0\0\0\0world");

        Ok(())
    }

    #[tokio::test]
    async fn test_override() -> Result<()> {
        let fs = FileSystem::new();

        fs.touch("/override.txt").await?;

        fs.write("/override.txt", 0, b"hello dlrow").await?;

        fs.write("/override.txt", 6, b"world").await?;

        let content = fs.read("/override.txt", 0, 11).await?;
        assert_eq!(content, b"hello world");

        Ok(())
    }

    #[tokio::test]
    async fn test_nonexistent_file() -> Result<()> {
        let fs = FileSystem::new();

        let content = fs.read("/nonexistent.txt", 0, 10).await?;
        assert_eq!(content, b"");

        Ok(())
    }

    #[tokio::test]
    async fn test_write_without_touch() -> Result<()> {
        let fs = FileSystem::new();

        fs.write("/auto.txt", 0, b"created").await?;

        let content = fs.read("/auto.txt", 0, 7).await?;
        assert_eq!(content, b"created");

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_files() -> Result<()> {
        let fs = FileSystem::new();

        fs.touch("/file1.txt").await?;
        fs.touch("/file2.txt").await?;

        fs.write("/file1.txt", 0, b"first").await?;
        fs.write("/file2.txt", 0, b"second").await?;

        let content1 = fs.read("/file1.txt", 0, 5).await?;
        let content2 = fs.read("/file2.txt", 0, 6).await?;

        assert_eq!(content1, b"first");
        assert_eq!(content2, b"second");

        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling() -> Result<()> {
        let fs = FileSystem::new();

        let result = fs.touch("").await;
        assert!(matches!(result, Err(FileSystemError::InvalidPath(_))));

        let result = fs.write("", 0, b"test").await;
        assert!(matches!(result, Err(FileSystemError::InvalidPath(_))));

        let result = fs.read("", 0, 5).await;
        assert!(matches!(result, Err(FileSystemError::InvalidPath(_))));

        let result = fs.write("/test.txt", usize::MAX, b"data").await;
        assert!(matches!(result, Err(FileSystemError::WriteError(_))));

        let result = fs.read("/test.txt", usize::MAX, 10).await;
        assert!(matches!(result, Err(FileSystemError::ReadError(_))));

        Ok(())
    }

    #[tokio::test]
    async fn test_write_zero_length() -> Result<()> {
        let fs = FileSystem::new();

        fs.write("/empty.txt", 0, b"").await?;
        let content = fs.read("/empty.txt", 0, 5).await?;
        assert_eq!(content, b"");

        Ok(())
    }

    #[tokio::test]
    async fn test_write_zero_length_at_offset() -> Result<()> {
        let fs = FileSystem::new();

        fs.write("/empty.txt", 10, b"").await?;
        let content = fs.read("/empty.txt", 0, 5).await?;
        assert_eq!(content, b"");

        Ok(())
    }

    #[tokio::test]
    async fn test_read_zero_length() -> Result<()> {
        let fs = FileSystem::new();

        fs.write("/test.txt", 0, b"hello").await?;
        let content = fs.read("/test.txt", 0, 0).await?;
        assert_eq!(content, b"");

        Ok(())
    }
}

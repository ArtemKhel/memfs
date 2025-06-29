#[derive(Debug, Clone)]
pub struct File {
    data: Vec<u8>,
}

impl File {
    pub(crate) fn new() -> Self { Self { data: Vec::new() } }

    /// Writes data to the file at the specified offset.
    ///
    /// # Safety
    /// The caller must ensure that `offset + data.len()` does not overflow.
    pub(crate) fn write(&mut self, offset: usize, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        let end_pos = offset + data.len();

        if end_pos > self.data.len() {
            self.data.resize(end_pos, 0);
        }

        self.data[offset..end_pos].copy_from_slice(data);
    }

    /// Reads data from the file starting at the specified offset.
    pub(crate) fn read(&self, offset: usize, len: usize) -> Vec<u8> {
        if offset >= self.data.len() {
            return Vec::new();
        }

        let end_pos = std::cmp::min(offset + len, self.data.len());
        self.data[offset..end_pos].to_vec()
    }
}

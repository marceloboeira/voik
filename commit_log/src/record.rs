pub struct Record {
    /// The current offset within current segment.
    pub current_offset: usize,

    /// Index to current segment.
    pub segment_index: usize,
}

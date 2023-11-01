pub struct EntryAllocator<T> {
    entries: Vec<Option<T>>,
    free_indices: Vec<usize>
}

impl<T> EntryAllocator<T> {
    pub fn new(size: usize) -> Self {
        Self {
            entries: std::iter::repeat_with(|| None).take(size).collect(),
            free_indices: (0..size).collect()
        }
    }

    pub fn alloc(&mut self, entry: T) -> Option<(usize, &T)> {
        let idx = self.free_indices.pop()?;
        Some((idx, self.entries[idx].insert(entry)))
    }

    pub fn dealloc(&mut self, idx: usize) -> Option<T> {
        match self.entries[idx].take() {
            Some(conn) => {
                self.free_indices.push(idx);
                Some(conn)
            }
            None => None
        }
    }

    pub fn is_full(&self) -> bool {
        self.free_indices.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.free_indices.len() == self.entries.len()
    }

    pub fn size(&self) -> usize {
        self.entries.len() - self.free_indices.len()
    }
}
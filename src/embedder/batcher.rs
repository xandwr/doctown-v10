// batcher.rs - throughput optimizer

pub struct Batcher {
    pub batch_size: usize,
}

impl Batcher {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }

    pub fn split<'a>(&self, items: &'a [String]) -> Vec<&'a [String]> {
        items.chunks(self.batch_size).collect()
    }
}

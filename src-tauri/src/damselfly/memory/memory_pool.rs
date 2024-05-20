#[derive(Default, Clone, Eq, Hash, PartialEq)]
pub struct MemoryPool {
    start: usize,
    size: usize,
    name: String
}

impl MemoryPool {
    pub fn new(start: usize, size: usize, name: String) -> Self {
        Self {
            start,
            size,
            name
        }
    }
    
    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }
    
    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }
    
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
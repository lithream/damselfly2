use std::cmp::Ordering;

#[derive(Default, Clone, Hash)]
pub struct MemoryPool {
    start: usize,
    size: usize,
    name: String
}

impl PartialEq<Self> for MemoryPool {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.size == other.size && self.name == other.name
    }
}

impl PartialOrd<Self> for MemoryPool {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.start.cmp(&other.start))
    }
}

impl Eq for MemoryPool {
    
}

impl Ord for MemoryPool {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
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
    
    pub fn get_start(&self) -> usize {
        self.start
    }
    
    pub fn get_size(&self) -> usize {
        self.size
    }
    
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
use std::sync::Arc;

trait MemoryUpdate {
    fn get_absolute_address(&self) -> usize;
    fn get_absolute_size(&self) -> usize;
    fn get_callstack(&self) -> Arc<String>;
}

pub struct Allocation {
    address: usize,
    size: usize,
    callstack: Arc<String>,
}

pub struct Free {
    address: usize,
    size: usize,
    callstack: Arc<String>,
}

impl MemoryUpdate for Allocation {
    fn get_absolute_address(&self) -> usize {
        self.address
    }

    fn get_absolute_size(&self) -> usize {
        self.size
    }

    fn get_callstack(&self) -> Arc<String> {
        Arc::clone(&(self.callstack))
    }
}

impl MemoryUpdate for Free {
    fn get_absolute_address(&self) -> usize {
        self.address
    }

    fn get_absolute_size(&self) -> usize {
        self.size
    }

    fn get_callstack(&self) -> Arc<String> {
        Arc::clone(&(self.callstack))
    }
}
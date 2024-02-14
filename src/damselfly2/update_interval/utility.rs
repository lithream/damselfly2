use crate::damselfly2::memory::memory_update::{MemoryUpdate, MemoryUpdateType};

pub struct Utility {}

impl Utility {
    pub(crate) fn get_start_and_stop(memory_update: &MemoryUpdateType) -> (usize, usize) {
        let (start, stop) = match memory_update {
            MemoryUpdateType::Allocation(allocation) => {
                let alloc_address = allocation.get_absolute_address();
                (alloc_address, alloc_address + allocation.get_absolute_size())
            }
            MemoryUpdateType::Free(free) => {
                let free_address = free.get_absolute_address();
                (free_address, free_address + free.get_absolute_size())
            }
        };

        (start, stop)
    }
}
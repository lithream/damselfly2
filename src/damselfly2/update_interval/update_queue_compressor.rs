use crate::damselfly2::memory::memory_update::{MemoryUpdate, MemoryUpdateType};

pub struct UpdateQueueCompressor { }

impl UpdateQueueCompressor {
    pub fn compress_to_allocs_only(updates: &Vec<MemoryUpdateType>) -> Vec<MemoryUpdateType> {
        let mut compressed_updates = Vec::new();
        for update in updates {
            match update {
                MemoryUpdateType::Allocation(allocation) => compressed_updates.push(allocation.wrap_in_enum()),
                MemoryUpdateType::Free(free) => {
                    compressed_updates.remove(
                        compressed_updates
                            .iter()
                            .position(|update| {
                                match update {
                                    MemoryUpdateType::Allocation(allocation) =>
                                        allocation.get_absolute_address() == free.get_absolute_address(),
                                    MemoryUpdateType::Free(_) => panic!("[UpdateQueueCompressor::compress_to_allocs_only]: Free found in compressed_updates"),
                                }
                            })
                            .expect("[UpdateQueueCompressor::strip_frees_and_corresponding_allocs]: Cannot find alloc corresponding to free"));
                }
            };
        }
        compressed_updates
    }
}


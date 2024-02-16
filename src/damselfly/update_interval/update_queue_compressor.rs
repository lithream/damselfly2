use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};

pub struct UpdateQueueCompressor { }

impl UpdateQueueCompressor {
    pub fn compress_to_allocs(updates: &Vec<MemoryUpdateType>) -> Vec<MemoryUpdateType> {
        let mut compressed_updates = Vec::new();
        for update in updates {
            match update {
                MemoryUpdateType::Allocation(allocation) => compressed_updates.push(allocation.clone().wrap_in_enum()),
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

    pub fn compress_ref_to_allocs(updates: &Vec<&MemoryUpdateType>) -> Vec<MemoryUpdateType> {
        let mut compressed_updates = Vec::new();
        for update in updates {
            match update {
                MemoryUpdateType::Allocation(allocation) => compressed_updates.push(allocation.clone().wrap_in_enum()),
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

mod tests {
    use crate::damselfly::consts::{OVERLAP_FINDER_TEST_LOG, TEST_BINARY_PATH};
    use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
    use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
    use crate::damselfly::update_interval::interval_to_update_converter::IntervalToUpdateConverter;
    use crate::damselfly::update_interval::overlap_finder::OverlapFinder;
    use crate::damselfly::update_interval::update_interval_factory::UpdateIntervalFactory;
    use crate::damselfly::update_interval::update_queue_compressor::UpdateQueueCompressor;

    fn initialise_test_log() -> OverlapFinder {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log_directly(OVERLAP_FINDER_TEST_LOG, TEST_BINARY_PATH);
        let update_intervals = UpdateIntervalFactory::new(updates).construct_enum_vector();
        OverlapFinder::new(update_intervals)
    }

    #[test]
    fn compress_updates_test() {
        let overlap_finder = initialise_test_log();
        let overlaps = overlap_finder.find_overlaps(0, 400);
        let updates = IntervalToUpdateConverter::convert_intervals_to_updates(&overlaps);
        let compressed_updates = UpdateQueueCompressor::compress_ref_to_allocs(&updates);

        assert_eq!(compressed_updates.len(), 5);
        for update in &compressed_updates {
            assert!(matches!(*update, MemoryUpdateType::Allocation(_)));
        }

        if let MemoryUpdateType::Allocation(allocation) = &compressed_updates[0] {
            assert_eq!(allocation.get_absolute_address(), 0);
            assert_eq!(allocation.get_absolute_size(), 20);
        }

        if let MemoryUpdateType::Allocation(allocation) = &compressed_updates[1] {
            assert_eq!(allocation.get_absolute_address(), 32);
            assert_eq!(allocation.get_absolute_size(), 20);
        }

        if let MemoryUpdateType::Allocation(allocation) = &compressed_updates[2] {
            assert_eq!(allocation.get_absolute_address(), 64);
            assert_eq!(allocation.get_absolute_size(), 276);
        }

        if let MemoryUpdateType::Allocation(allocation) = &compressed_updates[3] {
            assert_eq!(allocation.get_absolute_address(), 344);
            assert_eq!(allocation.get_absolute_size(), 20);
        }

        if let MemoryUpdateType::Allocation(allocation) = &compressed_updates[4] {
            assert_eq!(allocation.get_absolute_address(), 364);
            assert_eq!(allocation.get_absolute_size(), 20);
        }
    }
}


use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use super::UpdateInterval;

pub struct UpdateIntervalSorter;

impl UpdateIntervalSorter {
    pub fn sort_by_timestamp(updates: &mut Vec<&UpdateInterval>) {
        updates.sort_unstable_by(|prev, next| {
            let get_timestamp = |update_interval: &&UpdateInterval| -> usize {
                match &update_interval.val {
                    MemoryUpdateType::Allocation(allocation) => allocation.get_timestamp(),
                    MemoryUpdateType::Free(free) => free.get_timestamp(),
                }
            };

            get_timestamp(prev).cmp(&get_timestamp(next))
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::damselfly::consts::{OVERLAP_FINDER_TEST_LOG, TEST_BINARY_PATH};
    use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
    use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
    use crate::damselfly::update_interval::overlap_finder::OverlapFinder;
    use crate::damselfly::update_interval::update_interval_factory::UpdateIntervalFactory;
    use crate::damselfly::update_interval::update_interval_sorter::UpdateIntervalSorter;

    fn initialise_test_log() -> OverlapFinder {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log_directly(OVERLAP_FINDER_TEST_LOG, TEST_BINARY_PATH).memory_updates;
        let update_intervals = UpdateIntervalFactory::new(updates).construct_enum_vector();
        OverlapFinder::new(update_intervals)
    }

    #[test]
    fn sort_zero_intervals() {
        let overlap_finder = initialise_test_log();
        let mut overlaps = overlap_finder.find_overlaps(500, 600);
        UpdateIntervalSorter::sort_by_timestamp(&mut overlaps);
        assert_eq!(overlaps.len(), 0);
    }

    #[test]
    fn sort_overlaps() {
        let overlap_finder = initialise_test_log();
        let mut overlaps = overlap_finder.find_overlaps(0, 400);
        UpdateIntervalSorter::sort_by_timestamp(&mut overlaps);
        assert_eq!(overlaps.len(), 7);
        assert!(matches!(overlaps[0].val, MemoryUpdateType::Allocation(_)));
        assert!(matches!(overlaps[1].val, MemoryUpdateType::Allocation(_)));
        assert!(matches!(overlaps[2].val, MemoryUpdateType::Allocation(_)));
        assert!(matches!(overlaps[3].val, MemoryUpdateType::Allocation(_)));
        assert!(matches!(overlaps[4].val, MemoryUpdateType::Allocation(_)));
        assert!(matches!(overlaps[5].val, MemoryUpdateType::Allocation(_)));
        assert!(matches!(overlaps[6].val, MemoryUpdateType::Free(_)));

        if let MemoryUpdateType::Allocation(allocation) = &overlaps[0].val {
            assert_eq!(allocation.get_absolute_address(), 0);
            assert_eq!(allocation.get_absolute_size(), 20);
        }

        if let MemoryUpdateType::Allocation(allocation) = &overlaps[1].val {
            assert_eq!(allocation.get_absolute_address(), 32);
            assert_eq!(allocation.get_absolute_size(), 20);
        }

        if let MemoryUpdateType::Allocation(allocation) = &overlaps[2].val {
            assert_eq!(allocation.get_absolute_address(), 64);
            assert_eq!(allocation.get_absolute_size(), 276);
        }

        if let MemoryUpdateType::Allocation(allocation) = &overlaps[3].val {
            assert_eq!(allocation.get_absolute_address(), 344);
            assert_eq!(allocation.get_absolute_size(), 20);
        }


        if let MemoryUpdateType::Allocation(allocation) = &overlaps[4].val {
            assert_eq!(allocation.get_absolute_address(), 364);
            assert_eq!(allocation.get_absolute_size(), 20);
        }

        if let MemoryUpdateType::Allocation(allocation) = &overlaps[5].val {
            assert_eq!(allocation.get_absolute_address(), 364);
            assert_eq!(allocation.get_absolute_size(), 20);
        }

        if let MemoryUpdateType::Free(free) = &overlaps[6].val {
            assert_eq!(free.get_absolute_address(), 364);
            assert_eq!(free.get_absolute_size(), 20);
        }
    }
}
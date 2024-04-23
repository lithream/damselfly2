use rust_lapper::{Interval, Lapper};
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::update_interval::UpdateInterval;

pub struct OverlapFinder {
    lapper: Lapper<usize, MemoryUpdateType>
}

impl OverlapFinder {
    pub fn default() -> OverlapFinder {
        OverlapFinder {
            lapper: Lapper::new(vec![])
        }
    }
    
    pub fn new(intervals: Vec<Interval<usize, MemoryUpdateType>>) -> OverlapFinder {
        OverlapFinder {
            lapper: Lapper::new(intervals)
        }
    }

    pub fn push_interval(&mut self, interval: Interval<usize, MemoryUpdateType>) {
        self.lapper.insert(interval);
    }
    
    // start..end, not start..=end
    pub fn find_overlaps(&self, start: usize, end: usize) -> Vec<&UpdateInterval> {
        self.lapper.find(start, end).collect::<Vec<&UpdateInterval>>()
    }
}

mod tests {
    use crate::damselfly::consts::{OVERLAP_FINDER_TEST_LOG, TEST_BINARY_PATH, TEST_LOG};
    use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
    use crate::damselfly::update_interval::overlap_finder::OverlapFinder;
    use crate::damselfly::update_interval::update_interval_factory::UpdateIntervalFactory;
    use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};

    fn initialise_test_log() -> OverlapFinder {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log_directly(OVERLAP_FINDER_TEST_LOG, TEST_BINARY_PATH).memory_updates;
        let intervals = UpdateIntervalFactory::new(updates).construct_enum_vector();
        OverlapFinder::new(intervals)
    }

    #[test]
    fn find_zero_overlaps_test() {
        let lapper = initialise_test_log();
        let overlaps = lapper.find_overlaps(400, 500);
        assert_eq!(overlaps.len(), 0);
    }

    #[test]
    fn find_one_overlap_test() {
        let lapper = initialise_test_log();
        let overlaps = lapper.find_overlaps(0, 10);
        assert_eq!(overlaps.len(), 1);
        assert!(matches!(&overlaps[0].val, MemoryUpdateType::Allocation(_)));
        if let MemoryUpdateType::Allocation(allocation) = &overlaps[0].val {
            assert_eq!(allocation.get_absolute_address(), 0);
            assert_eq!(allocation.get_absolute_size(), 20);
        }
    }

    #[test]
    fn find_two_overlaps_test() {
        let lapper = initialise_test_log();
        let overlaps = lapper.find_overlaps(0, 48);
        assert_eq!(overlaps.len(), 2);
        assert!(matches!(&overlaps[0].val, MemoryUpdateType::Allocation(_)));
        if let MemoryUpdateType::Allocation(allocation) = &overlaps[0].val {
            assert_eq!(allocation.get_absolute_address(), 0);
            assert_eq!(allocation.get_absolute_size(), 20);
        }

        if let MemoryUpdateType::Allocation(allocation) = &overlaps[1].val {
            assert_eq!(allocation.get_absolute_address(), 32);
            assert_eq!(allocation.get_absolute_size(), 20);
        }
    }

    #[test]
    fn find_all_overlaps_test() {
        let lapper = initialise_test_log();
        let overlaps = lapper.find_overlaps(0, 400);
        assert_eq!(overlaps.len(), 7);
        assert!(matches!(&overlaps[0].val, MemoryUpdateType::Allocation(_)));
        assert!(matches!(&overlaps[1].val, MemoryUpdateType::Allocation(_)));
        assert!(matches!(&overlaps[2].val, MemoryUpdateType::Allocation(_)));
        assert!(matches!(&overlaps[3].val, MemoryUpdateType::Allocation(_)));
        assert!(matches!(&overlaps[4].val, MemoryUpdateType::Allocation(_)));
        assert!(matches!(&overlaps[5].val, MemoryUpdateType::Allocation(_)));
        assert!(matches!(&overlaps[6].val, MemoryUpdateType::Free(_)));

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

        if let MemoryUpdateType::Free(allocation) = &overlaps[5].val {
            assert_eq!(allocation.get_absolute_address(), 364);
            assert_eq!(allocation.get_absolute_size(), 20);
        }


        if let MemoryUpdateType::Free(free) = &overlaps[6].val {
            assert_eq!(free.get_absolute_address(), 364);
            assert_eq!(free.get_absolute_size(), 20);
        }
    }
}
use std::collections::HashMap;
use crate::damselfly_viewer::consts::{DEFAULT_BLOCK_SIZE};
use crate::memory::{MemoryStatus, MemoryUpdate};

pub struct MapManipulator {
}

impl MapManipulator {
    pub fn allocate_memory(map: &mut HashMap<usize, MemoryStatus>, absolute_address: usize, absolute_size: usize, callstack: String) {
        let scaled_address = absolute_address / DEFAULT_BLOCK_SIZE;
        let scaled_size = absolute_size / DEFAULT_BLOCK_SIZE;
        for i in 0..scaled_size {
            map.insert(scaled_address + i, MemoryStatus::Allocated(scaled_address, callstack.clone()));
        }
    }

    pub fn free_memory(map: &mut HashMap<usize, MemoryStatus>, absolute_address: usize, callstack: String) {
        let scaled_address = absolute_address / DEFAULT_BLOCK_SIZE;
        let mut adjacent_address = scaled_address + 1;
        while map.get(&adjacent_address).is_some_and(|block_state| {
            match block_state {
                MemoryStatus::Allocated(parent_block, _) =>
                    *parent_block == scaled_address,
                MemoryStatus::PartiallyAllocated(parent_block, _) =>
                    *parent_block == scaled_address,
                MemoryStatus::Free(_) => false,
            }
        }) {
            map.insert(adjacent_address, MemoryStatus::Free(callstack.clone()));
            adjacent_address += 1;
        }
        map.insert(scaled_address, MemoryStatus::Free(callstack.clone()));
    }

    pub fn view_memory(map: &mut HashMap<usize, MemoryStatus>, absolute_address: usize) -> (Option<&MemoryStatus>, usize) {
        let scaled_address = absolute_address / DEFAULT_BLOCK_SIZE;
        (map.get(&scaled_address), scaled_address)
    }

    pub fn scale_address_down(absolute_address: usize) -> usize {
        absolute_address / DEFAULT_BLOCK_SIZE
    }

    pub fn scale_address_up(relative_address: usize) -> usize {
        relative_address * DEFAULT_BLOCK_SIZE
    }

    pub fn get_address_of_row(row_length: usize, relative_address: usize) -> usize {
        (relative_address / row_length) * row_length
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::map_manipulator::MapManipulator;
    use crate::memory::MemoryStatus;

    #[test]
    fn allocate_memory_test() {
        let mut map: HashMap<usize, MemoryStatus> = HashMap::new();
        MapManipulator::allocate_memory(&mut map, 0, 20, "callstack".to_string());
        for i in 0..5 {
            assert!(matches!(map.get(&i).unwrap(), MemoryStatus::Allocated(..)));
        }
        assert!(map.get(&5).is_none());
    }

    #[test]
    fn allocate_memory_test_multiple_test() {
        let mut map: HashMap<usize, MemoryStatus> = HashMap::new();
        MapManipulator::allocate_memory(&mut map, 0, 20, "callstack".to_string());
        MapManipulator::allocate_memory(&mut map, 24, 20, "callstack2".to_string());

        for i in 0..5 {
            assert!(matches!(map.get(&i).unwrap(), MemoryStatus::Allocated(..)));
        }
        assert!(map.get(&5).is_none());

        for i in 6..11 {
            assert!(matches!(map.get(&i).unwrap(), MemoryStatus::Allocated(..)));
        }
        assert!(map.get(&11).is_none());
    }

    #[test]
    fn free_memory_test() {
        let mut map: HashMap<usize, MemoryStatus> = HashMap::new();
        MapManipulator::allocate_memory(&mut map, 0, 20, "callstack".to_string());
        MapManipulator::free_memory(&mut map, 0, "callstack".to_string());
        for i in 0..5 {
            assert!(matches!(map.get(&i).unwrap(), MemoryStatus::Free(..)));
        }
        assert!(map.get(&5).is_none());
    }

    #[test]
    fn free_memory_multiple_test() {
        let mut map: HashMap<usize, MemoryStatus> = HashMap::new();
        MapManipulator::allocate_memory(&mut map, 0, 20, "callstack".to_string());
        MapManipulator::allocate_memory(&mut map, 20, 20, "callstack".to_string());
        MapManipulator::free_memory(&mut map, 0, "callstack".to_string());
        for i in 0..5 {
            assert!(matches!(map.get(&i).unwrap(), MemoryStatus::Free(..)));
        }
        assert!(matches!(map.get(&5).unwrap(), MemoryStatus::Allocated(..)));

        MapManipulator::free_memory(&mut map, 20, "callstack".to_string());
        for i in 0..5 {
            assert!(matches!(map.get(&i).unwrap(), MemoryStatus::Free(..)));
        }
        for i in 5..10 {
            assert!(matches!(map.get(&i).unwrap(), MemoryStatus::Free(..)));
        }
        assert!(map.get(&10).is_none());
    }
}
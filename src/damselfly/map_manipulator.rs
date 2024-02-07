use std::rc::Rc;
use crate::damselfly::consts::{DEFAULT_BLOCK_SIZE};
use crate::damselfly::memory_structs::{MemoryStatus};
use crate::damselfly::memory_structs::NoHashMap;

pub fn allocate_memory(map: &mut NoHashMap<usize, MemoryStatus>, absolute_address: usize, bytes: usize, callstack: Rc<String>) {
    let scaled_address = absolute_address / DEFAULT_BLOCK_SIZE;
    let scaled_size = bytes / DEFAULT_BLOCK_SIZE;
    for i in 0..scaled_size {
        map.insert(scaled_address + i, MemoryStatus::Allocated(scaled_address, bytes, Rc::clone(&callstack)));
    }
}

pub fn free_memory(map: &mut NoHashMap<usize, MemoryStatus>, absolute_address: usize, callstack: Rc<String>) -> usize {
    let mut freed_memory = 1;
    let scaled_address = absolute_address / DEFAULT_BLOCK_SIZE;
    let mut adjacent_address = scaled_address + 1;
    while map.get(&adjacent_address)
        .is_some_and(|block_state| {
            match block_state {
                MemoryStatus::Allocated(parent_block, _, _) =>
                    *parent_block == scaled_address,
                MemoryStatus::PartiallyAllocated(parent_block, _) =>
                    *parent_block == scaled_address,
                MemoryStatus::Free(_) => false,
            }
        })
    {
        map.insert(adjacent_address, MemoryStatus::Free(Rc::clone(&callstack)));
        adjacent_address += 1;
        freed_memory += 1;
    }
    map.insert(scaled_address, MemoryStatus::Free(Rc::clone(&callstack)));
    freed_memory
}

pub fn view_memory(map: &NoHashMap<usize, MemoryStatus>, absolute_address: usize) -> Option<&MemoryStatus> {
    let logical_address = absolute_address / DEFAULT_BLOCK_SIZE;
    map.get(&logical_address)
}

pub fn absolute_to_logical(absolute_address: usize) -> usize {
    absolute_address / DEFAULT_BLOCK_SIZE
}

pub fn logical_to_absolute(relative_address: usize) -> usize {
    relative_address * DEFAULT_BLOCK_SIZE
}

pub fn get_address_of_row(row_length: usize, relative_address: usize) -> usize {
    (relative_address / row_length) * row_length
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use crate::damselfly::NoHashMap;
    use crate::damselfly::map_manipulator;
    use crate::damselfly::memory_structs::MemoryStatus;

    #[test]
    fn allocate_memory_test() {
        let mut map: NoHashMap<usize, MemoryStatus> = NoHashMap::default();
        map_manipulator::allocate_memory(&mut map, 0, 20, Rc::new("callstack".to_string()));
        for i in 0..5 {
            assert!(matches!(map.get(&i).unwrap(), MemoryStatus::Allocated(..)));
        }
        assert!(map.get(&5).is_none());
    }

    #[test]
    fn allocate_memory_test_multiple_test() {
        let mut map: NoHashMap<usize, MemoryStatus> = NoHashMap::default();
        map_manipulator::allocate_memory(&mut map, 0, 20, Rc::new("callstack".to_string()));
        map_manipulator::allocate_memory(&mut map, 24, 20, Rc::new("callstack2".to_string()));

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
        let mut map: NoHashMap<usize, MemoryStatus> = NoHashMap::default();
        map_manipulator::allocate_memory(&mut map, 0, 20, Rc::new("callstack".to_string()));
        map_manipulator::free_memory(&mut map, 0, Rc::new("callstack".to_string()));
        for i in 0..5 {
            assert!(matches!(map.get(&i).unwrap(), MemoryStatus::Free(..)));
        }
        assert!(map.get(&5).is_none());
    }

    #[test]
    fn free_memory_multiple_test() {
        let mut map: NoHashMap<usize, MemoryStatus> = NoHashMap::default();
        map_manipulator::allocate_memory(&mut map, 0, 20, Rc::new("callstack".to_string()));
        map_manipulator::allocate_memory(&mut map, 20, 20, Rc::new("callstack".to_string()));
        map_manipulator::free_memory(&mut map, 0, Rc::new("callstack".to_string()));
        for i in 0..5 {
            assert!(matches!(map.get(&i).unwrap(), MemoryStatus::Free(..)));
        }
        assert!(matches!(map.get(&5).unwrap(), MemoryStatus::Allocated(..)));

        map_manipulator::free_memory(&mut map, 20, Rc::new("callstack".to_string()));
        for i in 0..5 {
            assert!(matches!(map.get(&i).unwrap(), MemoryStatus::Free(..)));
        }
        for i in 5..10 {
            assert!(matches!(map.get(&i).unwrap(), MemoryStatus::Free(..)));
        }
        assert!(map.get(&10).is_none());
    }
}
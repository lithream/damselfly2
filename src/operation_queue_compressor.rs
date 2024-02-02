use crate::memory::MemoryUpdate;

struct OperationQueueCompressor<'a> {
    instruction_queue: &'a mut Vec<MemoryUpdate>
}

impl<'a> OperationQueueCompressor<'a> {

}
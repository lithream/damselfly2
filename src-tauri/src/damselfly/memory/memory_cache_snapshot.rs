//! MemoryCacheSnapshot
//! 
//! A cached map. MemoryCache manages collections of snapshots, so you should not need to create
//! MemoryCacheSnapshots separately. Use MemoryCache instead to generate a cache and manage/query it.
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::update_interval::UpdateInterval;
use crate::damselfly::viewer::memory_canvas::MemoryCanvas;

pub struct MemoryCacheSnapshot {
    base: MemoryCanvas,
    temporary_updates: Vec<UpdateInterval>
}

impl MemoryCacheSnapshot {
    /// Constructor
    /// 
    /// # Arguments 
    /// 
    /// * `base`: A MemoryCanvas to use as a base. Useful when reusing a previous cache to generate this one.
    /// * `temporary_updates`: A Vec of updates to paint on top of the base.
    /// 
    /// returns: MemoryCacheSnapshot 
    pub fn new(base: MemoryCanvas, temporary_updates: Vec<UpdateInterval>) -> Self {
        Self {
            base,
            temporary_updates,
        }
    }
    /// Renders a map that takes the base and paints additional updates over it.
    /// This is slightly slower as it manually compares each update's timestamp to time.
    /// Useful when rendering updates split into pools, as updates may not be consecutive.
    /// 
    /// e.g. Pool 1 receives update 0, pool 2 receives update 1, pool 1 receives update 2.
    /// In this case rendering at t=1 on pool 1 using this method will render only update 0.
    /// To use time as an index into temporary updates (where t=1 renders updates 0 and 2 for pool 1),
    /// use render_this_many instead.
    /// 
    /// # Arguments 
    /// 
    /// * `time`: Operations up to and including this timestamp will be rendered. This must be the 
    /// absolute operation timestamp - not a timestamp relative to the snapshot's timestamp.
    /// e.g. if the snapshot's base is at t=2000, a time of 2345 will render 345 temporary updates on
    /// top of the base. 
    /// 
    /// returns: Vec<MemoryStatus, Global> 
    pub fn render_till_timestamp(&self, time: usize) -> Vec<MemoryStatus> {
        let mut updates_to_append = Vec::new();
        for update in &self.temporary_updates {
            if update.val.get_timestamp() > time {
                break;
            }
            updates_to_append.push(update.clone());
        }
        self.base.render_temporary(updates_to_append)
    }

    /// Renders a map that takes the base and paints additional updates over it.
    /// This method uses time as an index to slice temporary updates and paint those updates. 
    /// It does not check timestamps at all.
    /// 
    /// e.g. Pool 1 receives update 0, pool 2 receives update 1, pool 1 receives update 2.
    /// In this case rendering at t=1 on pool 1 using this method will render both updates 0 and 2,
    /// as it renders temporary_updates[0..=1];
    /// 
    /// To paint updates using an absolute operation time, use render_till_timestamp instead.
    /// 
    /// # Arguments 
    /// 
    /// * `time`: Number of temporary updates to paint over the cache base
    /// 
    /// returns: Vec<MemoryStatus, Global> 
    pub fn render_this_many(&self, time: usize) -> Vec<MemoryStatus> {
        let mut updates_to_append = Vec::new();
        for update in &self.temporary_updates[0..=time] {
            updates_to_append.push(update.clone());
        }

        self.base.render_temporary(updates_to_append)
    }
    
    pub fn get_base(&self) -> &MemoryCanvas {
        &self.base
    }
}
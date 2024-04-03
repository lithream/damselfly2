use crate::damselfly::memory::memory_usage_sample::MemoryUsageSample;
use crate::damselfly::memory::memory_usage::MemoryUsage;
use crate::damselfly::memory::sampled_memory_usages_factory::SampledMemoryUsagesFactory;

pub struct SampledMemoryUsages {
    samples: Vec<MemoryUsageSample>,
    memory_usages: Vec<MemoryUsage>,
    sample_interval: u64,
}

impl SampledMemoryUsages {
    pub fn new(sample_interval: u64, memory_usages: Vec<MemoryUsage>) -> Self {
        let buckets = SampledMemoryUsagesFactory::new(sample_interval, memory_usages.clone()).divide_usages_into_buckets();
        Self {
            samples: buckets,
            memory_usages,
            sample_interval,
        }
    }
    
    pub fn get_samples(&self) -> &Vec<MemoryUsageSample> {
        &self.samples
    }
    
    pub fn set_sample_interval(&mut self, new_sample_interval: u64) {
        let buckets = SampledMemoryUsagesFactory::new(new_sample_interval, self.memory_usages.clone()).divide_usages_into_buckets();
        self.samples = buckets;
        self.sample_interval = new_sample_interval;
    }
    
    pub fn set_memory_usages(&mut self, new_memory_usages: Vec<MemoryUsage>) {
        let buckets = SampledMemoryUsagesFactory::new(self.sample_interval, new_memory_usages.clone()).divide_usages_into_buckets();
        self.samples = buckets;
        self.memory_usages = new_memory_usages;
    }
}
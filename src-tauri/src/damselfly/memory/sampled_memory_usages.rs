use crate::damselfly::memory::memory_usage_sample::MemoryUsageSample;
use crate::damselfly::memory::memory_usage::MemoryUsage;
use crate::damselfly::memory::sampled_memory_usages_factory::SampledMemoryUsagesFactory;

pub struct SampledMemoryUsages {
    samples: Vec<MemoryUsageSample>,
    sample_interval: u64,
}

impl SampledMemoryUsages {
    pub fn new(sample_interval: u64, memory_usages: Vec<MemoryUsage>) {
        let buckets = SampledMemoryUsagesFactory::new(sample_interval, memory_usages).divide_usages_into_buckets();
        buckets
            .iter()
            .map(|bucket| MemoryUsageSample::)
        Self {
            samples: SampledMemoryUsagesFactory::new(sample_interval, memory_usages).divide_usages_into_buckets(),
            sample_interval,
        }
        
    }
    
    pub fn get_samples(&self) -> &Vec<MemoryUsageSample> {
        &self.samples
    }
}
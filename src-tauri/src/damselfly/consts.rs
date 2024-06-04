pub const DEFAULT_TIMESPAN: usize = 100;
pub const DEFAULT_MEMORYSPAN: usize = 2048;
pub const DEFAULT_MEMORY_SIZE: usize = 4294967295;
pub const DEFAULT_SAMPLE_INTERVAL: u64 = 50000;
pub const DEFAULT_ROW_LENGTH: usize = 64;
pub const MIN_ROW_LENGTH: usize = 4;
pub const DEFAULT_BLOCK_SIZE: usize = 512;
pub const DEFAULT_BLOCKS_TO_TRUNCATE: usize = 256;
pub const MAX_BLOCK_SIZE: usize = 16777216;
pub const MAX_MAP_SPAN: usize = 16777216;
pub const DEFAULT_OPERATION_LOG_SIZE: usize = 32;
pub const TEST_LOG_PATH: &str = "./test.log";
pub const DEFAULT_GADDR2LINE_PATH: &str = "/opt/ghs/arm2018.5.4a/gaddr2line";
pub const DEFAULT_BINARY_PATH: &str = "/work/hpdev/dune/build/output/threadx-cortexa7-debug/ares/dragonfly-lp1/debug/defaultProductGroup/threadxApp";
pub const DEFAULT_LOG_PATH: &str = "./trace.log";
pub const TEST_BINARY_PATH: &str = "/work/dev/hp/dune/build/output/threadx-cortexa7-debug/ares/dragonfly-lp1/debug/defaultProductGroup/threadxApp";
pub const TEST_GADDR2LINE_PATH: &str = "./gaddr2line";
pub const GRAPH_VERTICAL_SCALE_OFFSET: f64 = 1.2;
pub const DEFAULT_CACHE_INTERVAL: u64 = 1000;
pub const DEFAULT_TICK_RATE: u64 = 100;
pub const LARGE_FILE_TICK_RATE: u64 = 500;
pub const TEST_LOG: &str = "00000811: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> + 0 14
00000812: 039da1f3 |V|A|005|        0 us   0001.676 s    < DT:0xE14DEEBC> ^ 0 [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0001.677 s    < DT:0xE14DEEBC> + 20 14
00000831: 039da3f2 |V|A|005|        0 us   0001.877 s    < DT:0xE14DEEBC> ^ 20 [e045d83b]
00000850: 039da4e5 |V|A|005|        0 us   0001.977 s    < DT:0xE14DEEBC> + 40 114
00000851: 039da4e5 |V|A|005|        0 us   0002.477 s    < DT:0xE14DEEBC> ^ 40 [e045d83b]
00000811: 039da1f3 |V|A|005|        0 us   0002.478 s    < DT:0xE14DEEBC> + 158 14
00000812: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> ^ 80 [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 16c 14
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 20 [e045d83b]";
pub const OVERLAP_FINDER_TEST_LOG: &str = "00000811: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> + 0 14
00000812: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> ^ 0 [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 20 14
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 20 [e045d83b]
00000850: 039da4e5 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 40 114
00000851: 039da4e5 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 40 [e045d83b]
00000811: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> + 158 14
00000812: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> ^ 158 [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 16c 14
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 16c [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 16c 14
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 16c [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> - 16c
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 16c [e045d83b]
";

pub const UPDATE_INTERVAL_SORTER_TEST_LOG: &str = 
"00000811: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> + 0 14
00000812: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> ^ 0 [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 20 14
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 20 [e045d83b]
00000850: 039da4e5 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 40 114
00000851: 039da4e5 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 40 [e045d83b]
00000811: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> + 158 14
00000812: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> ^ 158 [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 16c 14
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 16c [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 16c 14
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 16c [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> - 16c
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 16c [e045d83b]
";
pub const UPDATE_QUEUE_COMPRESSOR_TEST_LOG: &str = 
"00000811: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> + 0 14
00000812: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> ^ 0 [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 20 14
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 20 [e045d83b]
00000850: 039da4e5 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 40 114
00000851: 039da4e5 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 40 [e045d83b]
00000811: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> + 158 14
00000812: 039da1f3 |V|A|005|        0 us   0003.676 s    < DT:0xE14DEEBC> ^ 158 [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 16c 14
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 16c [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + 16c 14
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 16c [e045d83b]
00000830: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> - 16c
00000831: 039da3f2 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ 16c [e045d83b]
";
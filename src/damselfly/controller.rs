use crate::damselfly::consts::{DEFAULT_MEMORY_SIZE, DEFAULT_MEMORYSPAN, DEFAULT_TIMESPAN};
use crate::damselfly::viewer::DamselflyViewer;

struct DamselflyController {
    viewer: DamselflyViewer,
    graph_highlight: usize,
    map_highlight: usize,
    timespan: (usize, usize),
    memory_span: (usize, usize),
    timespan_freelook: bool,
    memoryspan_freelook: bool,
}

impl DamselflyController {
    fn new() -> DamselflyController {
        DamselflyController {
            viewer: DamselflyViewer::new(),
            graph_highlight: 0,
            map_highlight: 0,
            timespan: (0, DEFAULT_TIMESPAN),
            memory_span: (0, DEFAULT_MEMORYSPAN),
            timespan_freelook: false,
            memoryspan_freelook: false,
        }
    }
}
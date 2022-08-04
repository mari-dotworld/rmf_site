use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Source {
    index: usize,
    rate: f64,
    lambda: f64
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Sink {
    index: usize,
    r: f64
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SourceSink {
    source: Source,
    sink: Sink,
    repeat: bool
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct CrowdSim {
    pub desire_paths: Vec<SourceSink>,
}

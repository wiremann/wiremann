pub struct WorkerConfig {
    pub metadata: usize,
    pub thumbnail: usize,
    pub cacher: usize,
}

pub fn calculate_worker_config() -> WorkerConfig {
    let logical = num_cpus::get().max(1);

    let usable_threads = (((logical as f32) * 0.80).floor() as usize).max(2);

    let scanner_total = usable_threads;

    let cacher = usable_threads;

    let metadata = (scanner_total / 2).max(1).clamp(1, 4);
    let thumbnail = (scanner_total - metadata).max(1);

    WorkerConfig {
        metadata,
        thumbnail,
        cacher,
    }
}

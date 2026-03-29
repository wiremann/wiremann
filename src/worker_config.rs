pub struct WorkerConfig {
    pub metadata: usize,
    pub thumbnail: usize,
    pub cacher: usize,
}

pub fn calculate_worker_config() -> WorkerConfig {
    let logical = num_cpus::get().max(1);

    let usable = ((logical as f32) * 0.9).floor() as usize;
    let usable = usable.max(2);

    let cacher = (usable / 4).max(1);
    let scanner_total = usable - cacher;

    let metadata = scanner_total.min(8);
    let thumbnail = scanner_total.min(8);

    WorkerConfig {
        metadata,
        thumbnail,
        cacher,
    }
}

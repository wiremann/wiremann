pub struct WorkerConfig {
    pub metadata_workers: usize,
    pub thumbnail_workers: usize,
    pub cacher_workers: usize,
}

pub fn calculate_worker_config() -> WorkerConfig {
    let logical = num_cpus::get().max(1);

    let usable_threads = (((logical as f32) * 0.80).floor() as usize).max(2);

    let scanner_total = usable_threads;

    let cacher_workers = usable_threads;

    let metadata_workers = (scanner_total / 2).max(1).clamp(1, 4);
    let thumbnail_workers = (scanner_total - metadata_workers).max(1);

    WorkerConfig {
        metadata_workers,
        thumbnail_workers,
        cacher_workers,
    }
}

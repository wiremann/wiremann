pub struct WorkerConfig {
    pub scanner_workers: usize,
    pub cacher_workers: usize,
}

pub fn calculate_worker_config() -> WorkerConfig {
    let physical = num_cpus::get().max(1);

    let target_usage = 0.64; // noice

    let max_active = ((physical as f32) * target_usage).floor() as usize;

    let reserved = 2.min(max_active);

    let mut usable = max_active.saturating_sub(reserved);

    if usable == 0 {
        usable = 1;
    }

    let cacher_workers = if usable > 1 { 1 } else { 1 };
    let scanner_workers = usable;

    WorkerConfig {
        scanner_workers,
        cacher_workers,
    }
}
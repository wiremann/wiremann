use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    thread,
};

use tracing::{error, info};

pub struct WorkerConfig {
    pub metadata: usize,
    pub thumbnail: usize,
    pub cacher: usize,
}

pub fn spawn_worker<F, E>(name: &'static str, f: F)
where
    F: FnOnce() -> Result<(), E> + Send + 'static,
    E: core::fmt::Debug,
{
    thread::Builder::new()
        .name(name.into())
        .spawn(move || {
            info!("Spawning worker thread for [{name}] engine...");

            let result = catch_unwind(AssertUnwindSafe(f));

            match result {
                Ok(Ok(())) => {
                    info!("worker exited cleanly");
                }
                Ok(Err(e)) => {
                    error!(error = ?e, "worker crashed");
                }
                Err(e) => {
                    error!(panic = ?e, "worker panicked");
                }
            }
        })
        .unwrap_or_else(|e| {
            panic!("failed to spawn worker thread '{name}': {e}");
        });
}

pub fn calculate_worker_config() -> WorkerConfig {
    let logical = num_cpus::get().max(1);

    info!("detected logical cpus = {}", logical);

    let usable = ((logical as f32) * 0.9).floor() as usize;
    let usable = usable.max(2);

    let cacher = usable.max(4);
    let metadata = usable.min(8);
    let thumbnail = usable.min(8);

    WorkerConfig {
        metadata,
        thumbnail,
        cacher,
    }
}

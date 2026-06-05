use std::{collections::VecDeque, path::PathBuf};

use crate::controller::state::PlaylistId;

#[derive(PartialEq, Default, Clone)]
pub struct ScanManager {
    pub next_job_id: u64,
    pub current_job: Option<u64>,
    pub queue: VecDeque<ScanJob>,
    pub state: State,
}

#[derive(PartialEq, Default, Clone)]
pub struct ScanJob {
    pub id: u64,
    pub path: PathBuf,
    pub playlist_id: Option<PlaylistId>,
}

#[derive(PartialEq, Default, Clone)]
pub enum State {
    #[default]
    Idle,
    Scanning,
}

impl ScanManager {
    pub fn next_job_id(&mut self) -> u64 {
        let id = self.next_job_id;
        self.next_job_id += 1;
        id
    }

    pub fn enqueue(&mut self, job: ScanJob) {
        self.queue.push_back(job);
    }

    pub fn dequeue(&mut self) -> Option<ScanJob> {
        self.queue.pop_front()
    }

    pub fn is_idle(&self) -> bool {
        self.state == State::Idle
    }

    pub fn start_job(&mut self, id: u64) {
        self.state = State::Scanning;
        self.current_job = Some(id);
    }

    pub fn finish_job(&mut self) {
        self.current_job = None;

        if self.queue.is_empty() {
            self.state = State::Idle;
        }
    }

    pub fn set_idle(&mut self) {
        self.state = State::Idle;
    }
}

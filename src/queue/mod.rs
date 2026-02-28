use crate::controller::state::QueueState;
use crate::library::TrackId;

impl QueueState {
    pub fn get_id(&self) -> Option<TrackId> {
        self.order
            .get(self.index)
            .and_then(|&i| self.tracks.get(i))
            .copied()
    }

    pub fn get_index(&self, id: TrackId) -> Option<usize> {
        let track_idx = self.tracks.iter().position(|&t| t == id)?;
        self.order.iter().position(|&o| o == track_idx)
    }
}
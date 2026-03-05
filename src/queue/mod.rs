use crate::controller::state::QueueState;
use crate::library::TrackId;

impl QueueState {
    #[must_use]
    pub fn get_id(&self, index: usize) -> Option<TrackId> {
        self.order
            .get(index)
            .and_then(|&i| self.tracks.get(i))
            .copied()
    }

    #[must_use]
    pub fn get_index(&self, id: TrackId) -> Option<usize> {
        let track_idx = self.tracks.iter().position(|&t| t == id)?;
        self.order.iter().position(|&o| o == track_idx)
    }
}

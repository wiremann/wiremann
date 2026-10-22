use std::time::Duration;

use wgpui_motion::prelude::{Transition, expo_in_out};

pub const PAGE_TRANSITION_DURATION: Duration = Duration::from_millis(300);

pub fn page_transition() -> Transition {
    Transition::new(PAGE_TRANSITION_DURATION).curve(expo_in_out())
}

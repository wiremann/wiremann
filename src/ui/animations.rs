use std::time::Duration;

use wgpui_motion::prelude::{Spring, Transition, expo_in_out};

pub const PAGE_TRANSITION_DURATION: Duration = Duration::from_millis(300);
pub const SIDEBAR_HOVER_SPRING: Spring = Spring {
    mass: 0.5,
    ..Spring::SNAPPY
};

pub fn page_transition() -> Transition {
    Transition::new(PAGE_TRANSITION_DURATION).curve(expo_in_out())
}

pub fn sidebar_transition() -> Transition {
    Transition::spring(SIDEBAR_HOVER_SPRING)
}

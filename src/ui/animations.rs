use std::time::Duration;

use wgpui_motion::prelude::{Transition, expo_in_out};

pub fn page_transition() -> Transition {
    Transition::new(Duration::from_millis(300)).curve(expo_in_out())
}

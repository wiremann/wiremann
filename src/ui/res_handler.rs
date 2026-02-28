use crate::controller::events::{AudioEvent, ScannerEvent};
use gpui::*;

pub enum Event {
    Audio(AudioEvent),
    Scanner(ScannerEvent),
}

#[derive(Clone, Copy)]
pub struct ResHandler {}

impl ResHandler {
    pub fn handle(&mut self, cx: &mut Context<Self>, event: Event) {
        cx.emit(event);
        cx.notify();
    }
}

impl EventEmitter<Event> for ResHandler {}

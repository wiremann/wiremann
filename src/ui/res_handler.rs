use crate::controller::events::{AudioEvent, CacherEvent, ImageProcessorEvent, ScannerEvent};
use gpui::{Context, EventEmitter};

pub enum Event {
    Audio(AudioEvent),
    Scanner(ScannerEvent),
    Cacher(CacherEvent),
    ImageProcessor(ImageProcessorEvent),
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

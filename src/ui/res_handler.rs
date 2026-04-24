use crate::controller::events::{
    AudioEvent, CacherEvent, ImageProcessorEvent, LyricsEvent, ScannerEvent, SystemIntegrationEvent,
};
use gpui::{Context, EventEmitter};

pub enum Event {
    Audio(AudioEvent),
    Scanner(ScannerEvent),
    Cacher(CacherEvent),
    ImageProcessor(ImageProcessorEvent),
    SystemIntegration(SystemIntegrationEvent),
    LyricsEvent(LyricsEvent),
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

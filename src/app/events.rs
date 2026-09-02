use crate::{
    controller::Controller,
    ui::wiremann::Wiremann,
};
use gpui::{App, Entity};
use std::time::{Duration, Instant};

pub fn spawn_event_loop(cx: &mut App, mut controller: Controller, view: Entity<Wiremann>) {
    let mut app = cx.clone();
    cx.spawn(async move {
        let mut last_pos_request = Instant::now();
        let mut last_track_ended_request = Instant::now();

        loop {
            while let Ok(event) = controller.audio_rx.try_recv() {
                handle_controller_event(&mut controller, &mut app, AudioEventRef::Audio(&event), &view);
            }
            while let Ok(event) = controller.scanner_rx.try_recv() {
                handle_controller_event(&mut controller, &mut app, AudioEventRef::Scanner(&event), &view);
            }
            while let Ok(event) = controller.cacher_rx.try_recv() {
                handle_controller_event(&mut controller, &mut app, AudioEventRef::Cacher(&event), &view);
            }
            while let Ok(event) = controller.image_processor_rx.try_recv() {
                handle_controller_event(&mut controller, &mut app, AudioEventRef::ImageProcessor(&event), &view);
            }
            while let Ok(event) = controller.system_integration_rx.try_recv() {
                handle_controller_event(&mut controller, &mut app, AudioEventRef::SystemIntegration(&event), &view);
            }
            while let Ok(event) = controller.lyrics_manager_rx.try_recv() {
                handle_controller_event(&mut controller, &mut app, AudioEventRef::Lyrics(&event), &view);
            }

            if last_pos_request.elapsed() >= Duration::from_millis(16) {
                controller.get_pos();
                last_pos_request = Instant::now();
            }
            if last_track_ended_request.elapsed() >= Duration::from_millis(512) {
                controller.check_track_ended();
                last_track_ended_request = Instant::now();
            }

            app.background_executor().timer(Duration::from_millis(16)).await;
        }
    }).detach();
}

enum AudioEventRef<'a> {
    Audio(&'a crate::controller::events::AudioEvent),
    Scanner(&'a crate::controller::events::ScannerEvent),
    Cacher(&'a crate::controller::events::CacherEvent),
    ImageProcessor(&'a crate::controller::events::ImageProcessorEvent),
    SystemIntegration(&'a crate::controller::events::SystemIntegrationEvent),
    Lyrics(&'a crate::controller::events::LyricsEvent),
}

fn handle_controller_event(
    controller: &mut Controller,
    cx: &mut App,
    event: AudioEventRef<'_>,
    view: &Entity<Wiremann>,
) {
    let result = match event {
        AudioEventRef::Audio(event) => controller.handle_audio_event(cx, event, view),
        AudioEventRef::Scanner(event) => controller.handle_scanner_event(cx, event, view),
        AudioEventRef::Cacher(event) => controller.handle_cacher_event(cx, event, view),
        AudioEventRef::ImageProcessor(event) => controller.handle_image_processor_event(cx, event, view),
        AudioEventRef::SystemIntegration(event) => controller.handle_system_integration_event(cx, event, view),
        AudioEventRef::Lyrics(event) => controller.handle_lyrics_event(cx, event, view),
    };
    if let Err(error) = result {
        tracing::error!(?error, "controller event handling failed");
    }
}

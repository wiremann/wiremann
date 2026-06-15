use crate::{
    controller::Controller,
    ui::{
        res_handler::{Event, ResHandler},
        wiremann::Wiremann,
    },
};
use gpui::{App, Entity};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub fn spawn_event_loop(cx: &mut App, controller: Controller, arc_res: Arc<Entity<ResHandler>>) {
    cx.spawn(async move |cx| {
        let mut last_pos_request = Instant::now();
        let mut last_track_ended_request = Instant::now();

        loop {
            while let Ok(e) = controller.audio_rx.try_recv() {
                arc_res
                    .update(cx, |res_handler, cx| {
                        res_handler.handle(cx, Event::Audio(e));
                    })
                    .ok();
            }

            while let Ok(e) = controller.scanner_rx.try_recv() {
                arc_res
                    .update(cx, |res_handler, cx| {
                        res_handler.handle(cx, Event::Scanner(e));
                    })
                    .ok();
            }

            while let Ok(e) = controller.cacher_rx.try_recv() {
                arc_res
                    .update(cx, |res_handler, cx| {
                        res_handler.handle(cx, Event::Cacher(e));
                    })
                    .ok();
            }

            while let Ok(e) = controller.image_processor_rx.try_recv() {
                arc_res
                    .update(cx, |res_handler, cx| {
                        res_handler.handle(cx, Event::ImageProcessor(e));
                    })
                    .ok();
            }

            while let Ok(e) = controller.system_integration_rx.try_recv() {
                arc_res
                    .update(cx, |res_handler, cx| {
                        res_handler.handle(cx, Event::SystemIntegration(e));
                    })
                    .ok();
            }

            while let Ok(e) = controller.lyrics_manager_rx.try_recv() {
                arc_res
                    .update(cx, |res_handler, cx| {
                        res_handler.handle(cx, Event::LyricsEvent(e));
                    })
                    .ok();
            }

            if last_pos_request.elapsed() >= Duration::from_millis(16) {
                controller.get_pos();

                last_pos_request = Instant::now();
            }

            if last_track_ended_request.elapsed() >= Duration::from_millis(512) {
                controller.check_track_ended();

                last_track_ended_request = Instant::now();
            }

            cx.background_executor()
                .timer(Duration::from_millis(16))
                .await;
        }
    })
    .detach();
}

pub fn subscribe_controller_events(
    cx: &mut App,
    res_handler: &Entity<ResHandler>,
    mut controller: Controller,
    view: Entity<Wiremann>,
) {
    cx.subscribe(res_handler, move |_, event, cx| {
        if let Err(e) = match event {
            Event::Audio(event) => controller.handle_audio_event(cx, event, &view),
            Event::Scanner(event) => controller.handle_scanner_event(cx, event, &view),
            Event::Cacher(event) => controller.handle_cacher_event(cx, event, &view),
            Event::ImageProcessor(event) => {
                controller.handle_image_processor_event(cx, event, &view)
            }
            Event::SystemIntegration(event) => {
                controller.handle_system_integration_event(cx, event, &view)
            }
            Event::LyricsEvent(event) => controller.handle_lyrics_event(cx, event, &view),
        } {
            tracing::error!(error = ?e, "Controller error occured");
        }
    })
    .detach();
}

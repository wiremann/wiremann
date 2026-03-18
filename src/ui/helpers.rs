use gpui::RenderImage;
use std::sync::Arc;

#[must_use]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn slider_to_secs(slider: f32, duration_secs: u64) -> u64 {
    ((slider.clamp(0.0, 100.0) / 100.0) * duration_secs as f32) as u64
}

#[must_use]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn secs_to_slider(pos: u64, duration: u64) -> f32 {
    if duration == 0 {
        0.0
    } else {
        (pos as f32 / duration as f32) * 100.0
    }
}

pub fn drop_image_from_app(cx: &mut gpui::App, image: Arc<RenderImage>) {
    cx.defer(move |cx| {
        for window in cx.windows() {
            let image = image.clone();

            window
                .update(cx, move |_, window, _| {
                    let _ = window.drop_image(image);
                })
                .ok();
        }
    });
}

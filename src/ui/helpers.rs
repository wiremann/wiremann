use crate::controller::state::PlaylistId;
use crate::controller::state::TrackId;
use gpui::RenderImage;
use std::sync::Arc;
use std::time::Duration;

#[must_use]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
pub fn slider_to_duration(slider: f32, duration: Duration) -> Duration {
    Duration::from_millis(((slider.clamp(0.0, 100.0) / 100.0) * duration.as_millis() as f32) as u64)
}

#[must_use]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
pub fn duration_to_slider(pos: Duration, duration: Duration) -> f32 {
    let duration_ms = duration.as_millis();

    if duration_ms == 0 {
        0.0
    } else {
        (pos.as_millis() as f32 / duration_ms as f32) * 100.0
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

pub fn fingerprint_tracks(ids: impl IntoIterator<Item = TrackId>) -> u128 {
    let mut acc = 0u128;

    for id in ids {
        acc ^= u128::from_le_bytes(id.0);
    }

    acc
}

pub fn fingerprint_playlists(ids: impl IntoIterator<Item = PlaylistId>) -> u128 {
    let mut acc = 0u128;

    for id in ids {
        acc ^= u128::from_le_bytes(*id.0.as_bytes());
    }

    acc
}

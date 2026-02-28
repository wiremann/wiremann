use gpui::{App, RenderImage};
use image::imageops::thumbnail;
use image::{Frame, ImageReader, RgbaImage};
use smallvec::smallvec;
use std::io::Cursor;
use std::sync::Arc;

#[allow(unused)]
pub fn drop_image_from_app(cx: &mut App, image: Arc<RenderImage>) {
    for window in cx.windows() {
        let image = image.clone();

        window
            .update(cx, move |_, window, _| {
                window.drop_image(image).expect("Could not drop image");
            })
            .expect("Couldn't get window");
    }
}

pub fn rgb_to_bgr(img: &mut RgbaImage) {
    for px in img.pixels_mut() {
        px.0.swap(0, 2);
    }
}

pub fn decode_thumbnail(data: Box<[u8]>, small: bool) -> anyhow::Result<Arc<RenderImage>> {
    let mut image = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?
        .into_rgba8();

    rgb_to_bgr(&mut image);

    let frame = if small {
        Frame::new(thumbnail(&image, 64, 64))
    } else {
        Frame::new(image)
    };

    Ok(Arc::new(RenderImage::new(smallvec![frame])))
}

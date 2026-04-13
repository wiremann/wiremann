pub fn ease_in_out_quart() -> impl Fn(f32) -> f32 {
    move |delta| {
        if delta < 0.5 {
            8.0 * delta.powi(4)
        } else {
            1.0 - (-2.0 * delta + 2.0).powi(4) / 2.0
        }
    }
}

pub fn ease_in_out_expo() -> impl Fn(f32) -> f32 {
    move |delta| {
        if delta == 0.0 {
            0.0
        } else if (delta - 1.0).abs() < 0.01 {
            1.0
        } else if delta < 0.5 {
            (2.0_f32.powf(20.0 * delta - 10.0)) / 2.0
        } else {
            (2.0 - 2.0_f32.powf(-20.0 * delta + 10.0)) / 2.0
        }
    }
}

use gpui::{
    App, Bounds, Size, WindowBackgroundAppearance, WindowBounds, WindowDecorations, WindowIcon,
    WindowKind, WindowOptions, px, size,
};

pub fn build_window_options(app_icon: Option<WindowIcon>, cx: &mut App) -> WindowOptions {
    let bounds = Bounds::centered(None, size(px(1280.0), px(760.0)), cx);

    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        app_id: Some(String::from("wiremann")),
        focus: true,
        titlebar: None,
        kind: WindowKind::Normal,
        is_resizable: true,
        window_decorations: Some(WindowDecorations::Client),
        window_min_size: Some(Size {
            width: px(800.0),
            height: px(740.0),
        }),
        app_icon,
        window_background: WindowBackgroundAppearance::Blurred,
        ..Default::default()
    }
}

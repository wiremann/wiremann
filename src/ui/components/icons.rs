use gpui::{
    AnyElement, App, AppContext, Context, Entity, Hsla, IntoElement, Radians, Render, RenderOnce,
    SharedString, StyleRefinement, Styled, TextColor, Transformation, Window,
    prelude::FluentBuilder as _, svg, white,
};

pub trait IconNamed {
    fn path(self) -> SharedString;
}

#[derive(IntoElement)]
pub struct Icon {
    path: SharedString,
    style: StyleRefinement,
    color: Option<Hsla>,
    transform: Option<Transformation>,
}

impl Default for Icon {
    fn default() -> Self {
        Self {
            path: "".into(),
            style: StyleRefinement::default(),
            color: None,
            transform: None,
        }
    }
}

/// Main constructor.
///
/// Usage:
/// ```rust
/// icon(Icons::Play)
///     .size_4()
/// ```
pub fn icon(icon: impl IconNamed) -> Icon {
    Icon {
        path: icon.path(),
        ..Default::default()
    }
}

impl Icon {
    pub fn view(self, cx: &mut App) -> Entity<Self> {
        cx.new(|_| self)
    }

    #[must_use]
    pub fn rotate(mut self, radians: impl Into<Radians>) -> Self {
        self.transform = Some(Transformation::rotate(radians));
        self
    }

    #[must_use]
    pub fn transform(mut self, transform: Transformation) -> Self {
        self.transform = Some(transform);
        self
    }
}

impl Styled for Icon {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }

    fn text_color(mut self, color: impl Into<TextColor>) -> Self {
        self.color = Some(color.into().to_hsla());
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let color = self
            .color
            .unwrap_or_else(|| window.text_style().color.to_hsla());

        let text_size = window.text_style().font_size.to_pixels(window.rem_size());

        let has_size = self.style.size.width.is_some() || self.style.size.height.is_some();

        let mut svg = svg().flex_none();

        *svg.style() = self.style;

        svg.flex_shrink_0()
            .text_color(color)
            .when(!has_size, |this| this.size(text_size))
            .path(self.path)
            .when_some(self.transform, |this, transform| {
                this.with_transformation(transform)
            })
    }
}

impl Render for Icon {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let color = self.color.unwrap_or_else(white);

        let text_size = window.text_style().font_size.to_pixels(window.rem_size());

        let has_size = self.style.size.width.is_some() || self.style.size.height.is_some();

        let mut svg = svg().flex_none();

        *svg.style() = self.style.clone();

        svg.flex_shrink_0()
            .text_color(color)
            .when(!has_size, |this| this.size(text_size))
            .path(self.path.clone())
            .when_some(self.transform, |this, transform| {
                this.with_transformation(transform)
            })
    }
}

impl From<Icon> for AnyElement {
    fn from(icon: Icon) -> Self {
        icon.into_any_element()
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Icons {
    Music,
    MusicList,
    WinClose,
    WinMax,
    WinRes,
    WinMin,
    Settings,
    Play,
    Pause,
    Next,
    Prev,
    Shuffle,
    Repeat,
    Volume0,
    Volume1,
    Volume2,
    VolumeMute,
    Menu,
    Search,
    ToastInfo,
    ToastSuccess,
    ToastError,
    Loader,
    Scan,
    PanelRight,
    Home,
    Disc,
    Playlist,
    Plugins,
    User,
    Heart,
}

impl Icons {
    #[must_use]
    pub fn icon(self) -> Icon {
        icon(self)
    }
}

impl IconNamed for Icons {
    fn path(self) -> SharedString {
        match self {
            Icons::Music => "icons/music.svg",
            Icons::MusicList => "icons/playlist.svg",
            Icons::WinClose => "icons/window-close.svg",
            Icons::WinMax => "icons/window-maximize.svg",
            Icons::WinRes => "icons/window-restore.svg",
            Icons::WinMin => "icons/window-minimize.svg",
            Icons::Settings => "icons/settings.svg",
            Icons::Play => "icons/play.svg",
            Icons::Pause => "icons/pause.svg",
            Icons::Next => "icons/next.svg",
            Icons::Prev => "icons/prev.svg",
            Icons::Shuffle => "icons/shuffle.svg",
            Icons::Repeat => "icons/repeat.svg",
            Icons::Volume0 => "icons/volume-0.svg",
            Icons::Volume1 => "icons/volume-1.svg",
            Icons::Volume2 => "icons/volume-2.svg",
            Icons::VolumeMute => "icons/volume-mute.svg",
            Icons::Menu => "icons/menu.svg",
            Icons::Search => "icons/search.svg",
            Icons::ToastInfo => "icons/toast_info.svg",
            Icons::ToastSuccess => "icons/toast_success.svg",
            Icons::ToastError => "icons/toast_error.svg",
            Icons::Loader => "icons/loader.svg",
            Icons::Scan => "icons/scan.svg",
            Icons::PanelRight => "icons/panel_right.svg",
            Icons::Home => "icons/home.svg",
            Icons::Disc => "icons/disc.svg",
            Icons::Playlist => "icons/playlist.svg",
            Icons::Plugins => "icons/plugins.svg",
            Icons::User => "icons/user.svg",
            Icons::Heart => "icons/heart.svg",
        }
        .into()
    }
}

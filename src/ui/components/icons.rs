use gpui::{
    AnyElement, App, AppContext, Entity, Hsla, IntoElement, Radians, SharedString, StyleRefinement,
    Styled, Svg, TextColor, Transformation, svg,
};

pub trait IconNamed {
    fn path(self) -> SharedString;
}

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

impl IntoElement for Icon {
    type Element = Svg;

    fn into_element(self) -> Self::Element {
        let mut svg = svg().flex_none();

        *svg.style() = self.style;

        svg = svg.path(self.path);

        if let Some(color) = self.color {
            svg = svg.text_color(color);
        }

        if let Some(transform) = self.transform {
            svg = svg.with_transformation(transform);
        }

        svg
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
}

impl IconNamed for Icons {
    fn path(self) -> SharedString {
        match self {
            Icons::Music => "icons/music.svg",
            Icons::MusicList => "icons/list-music.svg",
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
        }
        .into()
    }
}

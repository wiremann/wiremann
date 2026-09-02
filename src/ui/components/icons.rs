use gpui::{
    App, Context, Entity, EntityFactory, Hsla, IntoElement, Render, RenderOnce, SharedString, Transformation,
    Window, svg, white,
};

pub trait IconNamed {
    fn path(self) -> SharedString;
}

#[derive(IntoElement)]
pub struct Icon {
    path: SharedString,
    size: f32,
    color: Option<Hsla>,
    transform: Option<Transformation>,
}

impl Default for Icon {
    fn default() -> Self {
        Self {
            path: "".into(),
            size: 16.0,
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
    pub fn rotate(mut self, radians: f32) -> Self {
        self.transform = Some(Transformation::rotate(radians));
        self
    }

    #[must_use]
    pub fn transform(mut self, transform: Transformation) -> Self {
        self.transform = Some(transform);
        self
    }

    #[must_use]
    pub fn size_4(mut self) -> Self {
        self.size = 16.0;
        self
    }

    #[must_use]
    pub fn size_5(mut self) -> Self {
        self.size = 20.0;
        self
    }

    #[must_use]
    pub fn size_6(mut self) -> Self {
        self.size = 24.0;
        self
    }

    #[must_use]
    pub fn size_8(mut self) -> Self {
        self.size = 32.0;
        self
    }

    #[must_use]
    pub fn size_16(mut self) -> Self {
        self.size = 64.0;
        self
    }

    #[must_use]
    pub fn size_full(mut self) -> Self {
        self.size = 32.0;
        self
    }

    #[must_use]
    pub fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        svg()
            .size(self.size)
            .text_color(self.color.unwrap_or_else(white))
            .path(self.path)
            .when_some(self.transform, |this, transform| {
                this.with_transformation(transform)
            })
    }
}

impl Render for Icon {
    fn render(&mut self, _window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        svg()
            .size(self.size)
            .text_color(self.color.unwrap_or_else(white))
            .path(self.path.clone())
            .when_some(self.transform, |this, transform| {
                this.with_transformation(transform)
            })
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
    Ellipsis,
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
    FolderOpen,
    Trash,
    Chart,
    PopOut,
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
            Icons::Ellipsis => "icons/ellipsis.svg",
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
            Icons::FolderOpen => "icons/folder-open.svg",
            Icons::Trash => "icons/trash.svg",
            Icons::Chart => "icons/chart.svg",
            Icons::PopOut => "icons/pop-out.svg",
        }
        .into()
    }
}

use gpui::*;
use gpui_component::{Icon, IconNamed};

#[derive(IntoElement)]
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
}

impl IconNamed for Icons {
    fn path(self) -> gpui::SharedString {
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
        }
        .into()
    }
}

impl RenderOnce for Icons {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        Icon::empty().path(self.path())
    }
}

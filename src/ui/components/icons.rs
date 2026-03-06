// Ref: https://github.com/longbridge/gpui-component/blob/main/crates/ui/src/icon.rs
use gpui::{prelude::FluentBuilder as _, svg, white, AnyElement, App, AppContext, Context, Entity, Hsla, IntoElement, Radians, Render, RenderOnce, SharedString, StyleRefinement, Styled, Svg, Transformation, Window};

/// Types implementing this trait can automatically be converted to [`Icon`].
///
/// This allows you to implement a custom version of [`Icons`] that functions as a drop-in
/// replacement for other UI components.
pub trait IconNamed {
    /// Returns the embedded path of the icon.
    fn path(self) -> SharedString;
}

impl<T: IconNamed> From<T> for Icon {
    fn from(value: T) -> Self {
        Icon::build(value)
    }
}

impl Icons {
    /// Return the icon as a Entity<Icon>
    pub fn view(self, cx: &mut App) -> Entity<Icon> {
        Icon::build(self).view(cx)
    }
}

impl From<Icons> for AnyElement {
    fn from(val: Icons) -> Self {
        Icon::build(val).into_any_element()
    }
}

#[derive(IntoElement)]
pub struct Icon {
    base: Svg,
    style: StyleRefinement,
    path: SharedString,
    text_color: Option<Hsla>,
    rotation: Option<Radians>,
}

impl Default for Icon {
    fn default() -> Self {
        Self {
            base: svg().flex_none().size_4(),
            style: StyleRefinement::default(),
            path: "".into(),
            text_color: None,
            rotation: None,
        }
    }
}

impl Clone for Icon {
    fn clone(&self) -> Self {
        let mut this = Self::default().path(self.path.clone());
        this.style = self.style.clone();
        this.rotation = self.rotation;
        this.text_color = self.text_color;
        this
    }
}

impl Icon {
    pub fn new(icon: impl Into<Icon>) -> Self {
        icon.into()
    }

    fn build(name: impl IconNamed) -> Self {
        Self::default().path(name.path())
    }

    /// Set the icon path of the Assets bundle
    ///
    /// For example: `icons/foo.svg`
    pub fn path(mut self, path: impl Into<SharedString>) -> Self {
        self.path = path.into();
        self
    }

    /// Create a new view for the icon
    pub fn view(self, cx: &mut App) -> Entity<Icon> {
        cx.new(|_| self)
    }

    pub fn transform(mut self, transformation: Transformation) -> Self {
        self.base = self.base.with_transformation(transformation);
        self
    }

    pub fn empty() -> Self {
        Self::default()
    }

    /// Rotate the icon by the given angle
    pub fn rotate(mut self, radians: impl Into<Radians>) -> Self {
        self.base = self
            .base
            .with_transformation(Transformation::rotate(radians));
        self
    }
}

impl Styled for Icon {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }

    fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.text_color = Some(color.into());
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let text_color = self.text_color.unwrap_or_else(|| window.text_style().color);
        let text_size = window.text_style().font_size.to_pixels(window.rem_size());
        let has_base_size = self.style.size.width.is_some() || self.style.size.height.is_some();

        let mut base = self.base;
        *base.style() = self.style;

        base.flex_shrink_0()
            .text_color(text_color)
            .when(!has_base_size, |this| this.size(text_size))
            .path(self.path)
    }
}

impl From<Icon> for AnyElement {
    fn from(val: Icon) -> Self {
        val.into_any_element()
    }
}

impl Render for Icon {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let text_color = self.text_color.unwrap_or_else(|| white());
        let text_size = window.text_style().font_size.to_pixels(window.rem_size());
        let has_base_size = self.style.size.width.is_some() || self.style.size.height.is_some();

        let mut base = svg().flex_none();
        *base.style() = self.style.clone();

        base.flex_shrink_0()
            .text_color(text_color)
            .when(!has_base_size, |this| this.size(text_size))
            .path(self.path.clone())
            .when_some(self.rotation, |this, rotation| {
                this.with_transformation(Transformation::rotate(rotation))
            })
    }
}

#[allow(dead_code)]
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

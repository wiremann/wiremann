// Ref: https://github.com/longbridge/gpui-component/blob/main/crates/ui/src/element_ext.rs
use gpui::{canvas, App, Bounds, ParentElement, Pixels, Styled, Window};

pub trait ElementExt: ParentElement + Sized {
    /// Add a prepaint callback to the element.
    ///
    /// This is a helper method to get the bounds of the element after paint.
    ///
    /// The first argument is the bounds of the element in pixels.
    ///
    /// See also [`gpui::canvas`].
    fn on_prepaint<F>(self, f: F) -> Self
    where
        F: FnOnce(Bounds<Pixels>, &mut Window, &mut App) + 'static,
    {
        self.child(
            canvas(
                move |bounds, window, cx| f(bounds, window, cx),
                |_, _, _, _| {},
            )
                .absolute()
                .size_full(),
        )
    }
}

impl<T: ParentElement> ElementExt for T {}

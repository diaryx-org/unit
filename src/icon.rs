//! Icons: a path on a 24-unit grid, stroked in one of the theme's colours.
//!
//! An icon is stroked in `currentColor`, which Blitz's SVG and a browser's
//! both take from the CSS `color` round it: it follows the text it sits in
//! (a highlighted menu row's, a primary button's) and the appearance,
//! the system's included, as text does. An [`Ink`] other than the text's
//! sets the icon's own `color` from the theme: an accent, a danger.
//!
//! An icon is decoration beside a label, or the whole of a control's face
//! with the control's `label` naming it: either way it is hidden from
//! assistive tech.

use dioxus::prelude::*;

use crate::theme::{Appearance, Palette};
use crate::unit::Unit;

/// Which of the theme's colours an icon is stroked in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ink {
    /// The text's.
    #[default]
    Text,
    /// Secondary text's, and what cannot be chosen.
    Dim,
    /// The accent: what is on, or chosen.
    Accent,
    /// Text on the accent: a primary button's, a highlighted menu row's.
    AccentText,
    /// A destructive action's.
    Danger,
}

impl Ink {
    /// The class that sets the icon's colour; the text's needs none.
    fn class(self) -> &'static str {
        match self {
            Ink::Text => "unit-svg",
            Ink::Dim => "unit-svg unit-ink-dim",
            Ink::Accent => "unit-svg unit-ink-accent",
            Ink::AccentText => "unit-svg unit-ink-accent-text",
            Ink::Danger => "unit-svg unit-ink-danger",
        }
    }

    fn of(self, p: &Palette) -> &'static str {
        match self {
            Ink::Text => p.text,
            Ink::Dim => p.dim,
            Ink::Accent => p.accent,
            Ink::AccentText => p.accent_text,
            Ink::Danger => p.danger,
        }
    }
}

impl Unit {
    /// The colour `ink` is in the page's theme and appearance, for what a
    /// page draws in Rust rather than CSS. The system's appearance is read
    /// as light, as Rust cannot see `prefers-color-scheme`.
    pub fn ink(&self, ink: Ink) -> &'static str {
        let theme = self.theme.peek();
        let palette = match *self.appearance.read() {
            Appearance::Dark => &theme.dark,
            Appearance::Light | Appearance::System => &theme.light,
        };
        ink.of(palette)
    }
}

/// An icon: `d` is an SVG path's data on a 24 × 24 grid, stroked. `size` is
/// how large it is drawn, in CSS pixels.
#[component]
pub fn Icon(
    d: &'static str,
    #[props(default)] ink: Ink,
    #[props(default = 18.0)] size: f64,
) -> Element {
    rsx! {
        svg {
            class: ink.class(),
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d }
        }
    }
}

/// A few paths for what most pages need, on the grid [`Icon`] draws. A page
/// brings its own for the rest.
pub mod paths {
    pub const CHEVRON_RIGHT: &str = "M9 5l7 7-7 7";
    pub const CHEVRON_LEFT: &str = "M15 5l-7 7 7 7";
    pub const CHEVRON_DOWN: &str = "M5 9l7 7 7-7";
    pub const CHECK: &str = "M4.5 12.5l5 5 10-11";
    pub const CLOSE: &str = "M6 6l12 12M18 6L6 18";
    pub const PLUS: &str = "M12 5v14M5 12h14";
    pub const MORE: &str = "M5.5 12h.01M12 12h.01M18.5 12h.01";
    pub const SEARCH: &str = "M10.5 4a6.5 6.5 0 1 0 0 13a6.5 6.5 0 1 0 0-13zM15.4 15.4L21 21";
    pub const TRASH: &str = "M4 7h16M10 11v6M14 11v6M5.5 7l1 13h11l1-13M9 7V4h6v3";
    pub const SHARE: &str = "M12 3v12M7.5 7.5L12 3l4.5 4.5M5 12v8h14v-8";
}

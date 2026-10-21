//! Icons: a path on a 24-unit grid, stroked in one of the theme's colours.
//!
//! Blitz draws SVG through usvg, which takes `stroke` from the element's
//! attribute and not from CSS, so an icon under Blitz cannot inherit its
//! colour from the text round it. Its colour is chosen here instead, from
//! the page's [`Theme`](crate::Theme) and appearance, as the [`Ink`] its
//! place asks for: a menu's highlighted row draws its icon in the accent's
//! text colour, a destructive one in the danger colour. A browser does read
//! CSS, and there an icon is stroked in `currentColor`, so it follows the
//! text it sits in, `:hover` included.
//!
//! An icon is decoration beside a label, or the whole of a control's face
//! with the control's `label` naming it: either way it is hidden from
//! assistive tech.

use dioxus::prelude::*;

use crate::theme::{Appearance, Palette};
use crate::unit::{Unit, use_unit};

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
    /// The colour `ink` is in the page's theme and appearance. The system's
    /// appearance is read as light: Blitz does not report the system's
    /// scheme on every platform, so a host there passes
    /// [`Appearance::Light`] or [`Appearance::Dark`] itself.
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
    let unit = use_unit();
    let stroke = if cfg!(target_arch = "wasm32") {
        "currentColor"
    } else {
        unit.ink(ink)
    };
    rsx! {
        svg {
            class: "unit-svg",
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke,
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

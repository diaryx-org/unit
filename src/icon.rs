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
/// how large it is drawn, in CSS pixels. `filled` fills its shape in the
/// same ink as well: a bookmark that is set, a play triangle.
#[component]
pub fn Icon(
    d: &'static str,
    #[props(default)] ink: Ink,
    #[props(default = 18.0)] size: f64,
    #[props(default)] filled: bool,
) -> Element {
    rsx! {
        svg {
            class: ink.class(),
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: if filled { "currentColor" } else { "none" },
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
    pub const CHEVRON_UP: &str = "M5 15l7-7 7 7";
    /// Opens upward or downward: a band that grows, a picker that drops.
    pub const CHEVRON_UP_DOWN: &str = "M8 9.5l4-4 4 4M8 14.5l4 4 4-4";
    pub const ARROW_UP: &str = "M12 19V5M6 11l6-6 6 6";
    pub const ARROW_DOWN: &str = "M12 5v14M6 13l6 6 6-6";
    pub const ARROW_RIGHT: &str = "M5 12h14M13 6l6 6-6 6";
    pub const MINUS: &str = "M5 12h14";
    /// Drawn `filled` when it is set.
    pub const BOOKMARK: &str = "M6.5 3.5h11v17l-5.5-4.2-5.5 4.2z";
    pub const GEAR: &str = "M12 9a3 3 0 1 0 0 6a3 3 0 1 0 0-6zM12 2.8l1.6 2.3 2.7-.7.6 2.7 2.6 1-1 2.6 1.9 2-1.9 2 1 2.6-2.6 1-.6 2.7-2.7-.7L12 21.2l-1.6-2.3-2.7.7-.6-2.7-2.6-1 1-2.6L3.6 12l1.9-2-1-2.6 2.6-1 .6-2.7 2.7.7z";
    pub const PHOTO: &str =
        "M3.5 6.5h17v12h-17zM3.5 15l5-4.5 4 3.5 3-2.5 5 4M15.5 9.2a1 1 0 1 0 0.01 0";
    pub const DOCUMENT: &str = "M6 3h8l4 4v14H6zM14 3v4h4";
    pub const LINK: &str = "M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1.2 1.2M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1.2-1.2";
    pub const UNDO: &str = "M9 14L4 9l5-5M4 9h10a6 6 0 0 1 0 12h-3";
    pub const LIST: &str = "M9 6h11M9 12h11M9 18h11M4.5 6h.01M4.5 12h.01M4.5 18h.01";
    pub const BUBBLE: &str = "M4 5h16v11H9l-5 4z";
    pub const ENVELOPE: &str = "M3.5 6h17v12h-17zM3.5 6l8.5 7 8.5-7";
    pub const BOLT: &str = "M13 3L5 14h6l-1 7 8-11h-6z";
    pub const SPARKLE: &str = "M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z";
    pub const SHUFFLE: &str = "M4 7h3.5c4.5 0 4.5 10 9 10H20M4 17h3.5c1.6 0 2.6-1.3 3.4-3M13.1 10c.8-1.7 1.8-3 3.4-3H20M17 4l3 3-3 3M17 14l3 3-3 3";
    /// Not seen: hidden from someone.
    pub const EYE_OFF: &str = "M3 12s3.5-6 9-6 9 6 9 6-3.5 6-9 6-9-6-9-6zM12 9.5a2.5 2.5 0 1 0 0 5a2.5 2.5 0 1 0 0-5zM4 4l16 16";
    /// Puts the keyboard away.
    pub const KEYBOARD_DOWN: &str =
        "M3.5 3.5h17v10h-17zM7 7h.01M10.3 7h.01M13.7 7h.01M17 7h.01M8 10.5h8M9 17.5l3 3 3-3";
    /// Drawn `filled`.
    pub const PLAY: &str = "M7 4.5v15l12-7.5z";
    pub const PAUSE: &str = "M8 5v14M16 5v14";
    /// Drawn `filled`.
    pub const STOP: &str = "M6 6h12v12H6z";
    /// Back a little way: a recording's skip back.
    pub const REPLAY: &str = "M4 12a8 8 0 1 0 2.3-5.7M4 4v4h4";
    /// On a little way: a recording's skip forward.
    pub const FORWARD: &str = "M20 12a8 8 0 1 1-2.3-5.7M20 4v4h-4";

    /// Every path above, by its name: for a page that shows them all.
    pub const ALL: &[(&str, &str)] = &[
        ("CHEVRON_RIGHT", CHEVRON_RIGHT),
        ("CHEVRON_LEFT", CHEVRON_LEFT),
        ("CHEVRON_DOWN", CHEVRON_DOWN),
        ("CHECK", CHECK),
        ("CLOSE", CLOSE),
        ("PLUS", PLUS),
        ("MORE", MORE),
        ("SEARCH", SEARCH),
        ("TRASH", TRASH),
        ("SHARE", SHARE),
        ("CHEVRON_UP", CHEVRON_UP),
        ("CHEVRON_UP_DOWN", CHEVRON_UP_DOWN),
        ("ARROW_UP", ARROW_UP),
        ("ARROW_DOWN", ARROW_DOWN),
        ("ARROW_RIGHT", ARROW_RIGHT),
        ("MINUS", MINUS),
        ("BOOKMARK", BOOKMARK),
        ("GEAR", GEAR),
        ("PHOTO", PHOTO),
        ("DOCUMENT", DOCUMENT),
        ("LINK", LINK),
        ("UNDO", UNDO),
        ("LIST", LIST),
        ("BUBBLE", BUBBLE),
        ("ENVELOPE", ENVELOPE),
        ("BOLT", BOLT),
        ("SPARKLE", SPARKLE),
        ("SHUFFLE", SHUFFLE),
        ("EYE_OFF", EYE_OFF),
        ("KEYBOARD_DOWN", KEYBOARD_DOWN),
        ("PLAY", PLAY),
        ("PAUSE", PAUSE),
        ("STOP", STOP),
        ("REPLAY", REPLAY),
        ("FORWARD", FORWARD),
    ];
}

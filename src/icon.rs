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

/// The paths most pages need, on the grid [`Icon`] draws. A page brings its
/// own for the rest.
///
/// They are [Lucide](https://lucide.dev)'s icons (v1.61.0, ISC licence, in
/// `LICENSE-LUCIDE`), each written as one path: a circle, a rectangle or a
/// line of Lucide's is the same shape spelled as path data.
pub mod paths {
    /// Lucide's `chevron-right`.
    pub const CHEVRON_RIGHT: &str = "M9 18l6-6-6-6";
    /// Lucide's `chevron-left`.
    pub const CHEVRON_LEFT: &str = "M15 18l-6-6 6-6";
    /// Lucide's `chevron-down`.
    pub const CHEVRON_DOWN: &str = "M6 9l6 6 6-6";
    /// Lucide's `check`.
    pub const CHECK: &str = "M20 6 9 17l-5-5";
    /// Lucide's `x`.
    pub const CLOSE: &str = "M18 6 6 18M6 6l12 12";
    /// Lucide's `plus`.
    pub const PLUS: &str = "M5 12h14M12 5v14";
    /// Lucide's `ellipsis`.
    pub const MORE: &str = "M11 12a1 1 0 1 0 2 0a1 1 0 1 0 -2 0M18 12a1 1 0 1 0 2 0a1 1 0 1 0 -2 0M4 12a1 1 0 1 0 2 0a1 1 0 1 0 -2 0";
    /// Lucide's `search`.
    pub const SEARCH: &str = "M21 21l-4.34-4.34M3 11a8 8 0 1 0 16 0a8 8 0 1 0 -16 0";
    /// Lucide's `trash`.
    pub const TRASH: &str = "M10 11v6M14 11v6M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2";
    /// Lucide's `share`.
    pub const SHARE: &str = "M12 2v13M16 6l-4-4-4 4M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8";
    /// Lucide's `chevron-up`.
    pub const CHEVRON_UP: &str = "M18 15l-6-6-6 6";
    /// Lucide's `chevrons-up-down`. Opens upward or downward: a band that grows, a picker that drops.
    pub const CHEVRON_UP_DOWN: &str = "M7 15l5 5 5-5M7 9l5-5 5 5";
    /// Lucide's `arrow-up`.
    pub const ARROW_UP: &str = "M5 12l7-7 7 7M12 19V5";
    /// Lucide's `arrow-down`.
    pub const ARROW_DOWN: &str = "M12 5v14M19 12l-7 7-7-7";
    /// Lucide's `arrow-right`.
    pub const ARROW_RIGHT: &str = "M5 12h14M12 5l7 7-7 7";
    /// Lucide's `minus`.
    pub const MINUS: &str = "M5 12h14";
    /// Lucide's `bookmark`. Drawn `filled` when it is set.
    pub const BOOKMARK: &str = "M17 3a2 2 0 0 1 2 2v15a1 1 0 0 1-1.496.868l-4.512-2.578a2 2 0 0 0-1.984 0l-4.512 2.578A1 1 0 0 1 5 20V5a2 2 0 0 1 2-2z";
    /// Lucide's `settings`.
    pub const GEAR: &str = "M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915M9 12a3 3 0 1 0 6 0a3 3 0 1 0 -6 0";
    /// Lucide's `image`.
    pub const PHOTO: &str = "M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14a2 2 0 0 1 2 -2ZM7 9a2 2 0 1 0 4 0a2 2 0 1 0 -4 0M21 15l-3.086-3.086a2 2 0 0 0-2.828 0L6 21";
    /// Lucide's `file`.
    pub const DOCUMENT: &str = "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2zM14 2v5a1 1 0 0 0 1 1h5";
    /// Lucide's `link`.
    pub const LINK: &str = "M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71";
    /// Lucide's `undo-2`.
    pub const UNDO: &str = "M9 14 4 9l5-5M4 9h10.5a5.5 5.5 0 0 1 5.5 5.5a5.5 5.5 0 0 1-5.5 5.5H11";
    /// Lucide's `list`.
    pub const LIST: &str = "M3 5h.01M3 12h.01M3 19h.01M8 5h13M8 12h13M8 19h13";
    /// Lucide's `message-circle`.
    pub const BUBBLE: &str = "M2.992 16.342a2 2 0 0 1 .094 1.167l-1.065 3.29a1 1 0 0 0 1.236 1.168l3.413-.998a2 2 0 0 1 1.099.092 10 10 0 1 0-4.777-4.719";
    /// Lucide's `mail`.
    pub const ENVELOPE: &str = "M22 7l-8.991 5.727a2 2 0 0 1-2.009 0L2 7M4 4h16a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-16a2 2 0 0 1 -2 -2v-12a2 2 0 0 1 2 -2Z";
    /// Lucide's `zap`.
    pub const BOLT: &str = "M15.914 4a1.5 1.5 0 00-2.474-1.561l-9 9A1.5 1.5 0 005.5 14h4.002a.5.5 0 01.471.666L8.086 20a1.5 1.5 0 002.475 1.56l9-9A1.5 1.5 0 0018.5 10h-3.997a.5.5 0 01-.472-.667z";
    /// Lucide's `sparkle`.
    pub const SPARKLE: &str = "M11.017 2.814a1 1 0 0 1 1.966 0l1.051 5.558a2 2 0 0 0 1.594 1.594l5.558 1.051a1 1 0 0 1 0 1.966l-5.558 1.051a2 2 0 0 0-1.594 1.594l-1.051 5.558a1 1 0 0 1-1.966 0l-1.051-5.558a2 2 0 0 0-1.594-1.594l-5.558-1.051a1 1 0 0 1 0-1.966l5.558-1.051a2 2 0 0 0 1.594-1.594z";
    /// Lucide's `shuffle`.
    pub const SHUFFLE: &str = "M18 14l4 4-4 4M18 2l4 4-4 4M2 18h1.973a4 4 0 0 0 3.3-1.7l5.454-8.6a4 4 0 0 1 3.3-1.7H22M2 6h1.972a4 4 0 0 1 3.6 2.2M22 18h-6.041a4 4 0 0 1-3.3-1.8l-.359-.45";
    /// Lucide's `eye-off`. Not seen: hidden from someone.
    pub const EYE_OFF: &str = "M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49M14.084 14.158a3 3 0 0 1-4.242-4.242M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143M2 2l20 20";
    /// Lucide's `keyboard-off`. Puts the keyboard away.
    pub const KEYBOARD_DOWN: &str = "M 20 4 A2 2 0 0 1 22 6M 22 6 L 22 16.41M 7 16 L 16 16M 9.69 4 L 20 4M14 8h.01M18 8h.01M2 2l20 20M20 20H4a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2M6 8h.01M8 12h.01";
    /// Lucide's `play`. Drawn `filled`.
    pub const PLAY: &str =
        "M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z";
    /// Lucide's `pause`.
    pub const PAUSE: &str = "M15 3h3a1 1 0 0 1 1 1v16a1 1 0 0 1 -1 1h-3a1 1 0 0 1 -1 -1v-16a1 1 0 0 1 1 -1ZM6 3h3a1 1 0 0 1 1 1v16a1 1 0 0 1 -1 1h-3a1 1 0 0 1 -1 -1v-16a1 1 0 0 1 1 -1Z";
    /// Lucide's `square`. Drawn `filled`.
    pub const STOP: &str =
        "M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14a2 2 0 0 1 2 -2Z";
    /// Lucide's `rotate-ccw`. Back a little way: a recording's skip back.
    pub const REPLAY: &str = "M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8M3 3v5h5";
    /// Lucide's `rotate-cw`. On a little way: a recording's skip forward.
    pub const FORWARD: &str = "M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8M21 3v5h-5";
    /// Lucide's `library-big`. Books on a shelf.
    pub const LIBRARY: &str = "M4 3h6a1 1 0 0 1 1 1v16a1 1 0 0 1 -1 1h-6a1 1 0 0 1 -1 -1v-16a1 1 0 0 1 1 -1ZM7 3v18M20.4 18.9c.2.5-.1 1.1-.6 1.3l-1.9.7c-.5.2-1.1-.1-1.3-.6L11.1 5.1c-.2-.5.1-1.1.6-1.3l1.9-.7c.5-.2 1.1.1 1.3.6Z";
    /// Lucide's `brush`.
    pub const BRUSH: &str = "M11 10l3 3M6.5 21A3.5 3.5 0 1 0 3 17.5a2.62 2.62 0 0 1-.708 1.792A1 1 0 0 0 3 21zM9.969 17.031 21.378 5.624a1 1 0 0 0-3.002-3.002L6.967 14.031";
    /// Lucide's `highlighter`.
    pub const HIGHLIGHTER: &str =
        "M9 11l-6 6v3h9l3-3M22 12l-4.6 4.6a2 2 0 0 1-2.8 0l-5.2-5.2a2 2 0 0 1 0-2.8L14 4";

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
        ("LIBRARY", LIBRARY),
        ("BRUSH", BRUSH),
        ("HIGHLIGHTER", HIGHLIGHTER),
    ];
}

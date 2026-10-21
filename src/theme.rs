//! The theme: every colour, radius and size a component uses is a CSS custom
//! property, and a [`Theme`] is the values of those properties — a palette
//! for the light appearance, one for the dark, and the sizes. [`Theme::css`]
//! writes them out, under the root's `data-appearance` and `data-density`,
//! so a page forces an appearance or a touch density with an attribute and
//! nothing is written twice by hand.
//!
//! The names follow one scheme, `--unit-<what>`, and the component styles
//! (`style.rs`) use nothing else, so a host restyles everything from here.

use std::fmt::Write;

/// The colours of one appearance.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    /// The page behind everything.
    pub bg: &'static str,
    /// A raised surface: a menu, a dialog, a card.
    pub surface: &'static str,
    /// A control's own face: a button, a field, a track.
    pub control: &'static str,
    /// A control's face under the pointer.
    pub control_hover: &'static str,
    /// A groove a control sits in: a segmented control's, a switch's when
    /// off, a slider's rail, a progress bar's.
    pub track: &'static str,
    /// What stands up from a groove: the chosen segment.
    pub raised: &'static str,
    pub text: &'static str,
    /// Secondary text, and anything that cannot be chosen.
    pub dim: &'static str,
    /// A control's edge, and a raised surface's.
    pub border: &'static str,
    /// A rule between rows or sections, lighter than an edge.
    pub separator: &'static str,
    /// What is chosen, on, or the default: and its text.
    pub accent: &'static str,
    pub accent_text: &'static str,
    /// A destructive action.
    pub danger: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    /// The ring round the control the keys go to.
    pub focus: &'static str,
    /// Behind a modal dialog.
    pub scrim: &'static str,
    /// Selected text in a field.
    pub selection: &'static str,
    /// A shadow under what floats.
    pub shadow: &'static str,
    /// The faint shadow that lifts a control or a card off what it is on.
    pub shadow_soft: &'static str,
}

impl Palette {
    pub const LIGHT: Palette = Palette {
        bg: "#f5f5f7",
        surface: "#ffffff",
        control: "#ffffff",
        control_hover: "#f2f2f4",
        track: "#e6e6ea",
        raised: "#ffffff",
        text: "#1d1d1f",
        dim: "#86868b",
        border: "#d4d4d8",
        separator: "#e8e8ec",
        accent: "#2f6fde",
        accent_text: "#ffffff",
        danger: "#d93a30",
        success: "#1e8e3e",
        warning: "#c77700",
        focus: "#6d9cf0",
        scrim: "rgba(0, 0, 0, 0.25)",
        selection: "#cfe0fb",
        shadow: "rgba(0, 0, 0, 0.16)",
        shadow_soft: "rgba(0, 0, 0, 0.07)",
    };

    pub const DARK: Palette = Palette {
        bg: "#1c1c1e",
        surface: "#2a2a2c",
        control: "#3a3a3c",
        control_hover: "#444446",
        track: "#3a3a3c",
        raised: "#636366",
        text: "#ececee",
        dim: "#98989d",
        border: "#454548",
        separator: "#38383b",
        accent: "#3b7ff0",
        accent_text: "#ffffff",
        danger: "#ff5f57",
        success: "#30d158",
        warning: "#ffb340",
        focus: "#5b97ff",
        scrim: "rgba(0, 0, 0, 0.45)",
        selection: "#2b4a7a",
        shadow: "rgba(0, 0, 0, 0.5)",
        shadow_soft: "rgba(0, 0, 0, 0.3)",
    };

    fn entries(&self) -> [(&'static str, &'static str); 20] {
        [
            ("bg", self.bg),
            ("surface", self.surface),
            ("control", self.control),
            ("control-hover", self.control_hover),
            ("track", self.track),
            ("raised", self.raised),
            ("text", self.text),
            ("dim", self.dim),
            ("border", self.border),
            ("separator", self.separator),
            ("accent", self.accent),
            ("accent-text", self.accent_text),
            ("danger", self.danger),
            ("success", self.success),
            ("warning", self.warning),
            ("focus", self.focus),
            ("scrim", self.scrim),
            ("selection", self.selection),
            ("shadow", self.shadow),
            ("shadow-soft", self.shadow_soft),
        ]
    }
}

/// The sizes of one density.
#[derive(Clone, Debug, PartialEq)]
pub struct Metrics {
    /// A control's height: a button, a field, a menu's row.
    pub control: f64,
    /// The text's size.
    pub font_size: f64,
    /// The gap between controls in a row or a stack.
    pub gap: f64,
    /// A control's corners, and a raised surface's.
    pub radius: f64,
    pub radius_large: f64,
}

impl Metrics {
    /// For a pointer: the Mac's sizes.
    pub const REGULAR: Metrics = Metrics {
        control: 24.0,
        font_size: 13.0,
        gap: 8.0,
        radius: 6.0,
        radius_large: 10.0,
    };

    /// For a finger: 44-point targets, as Apple's guidelines ask.
    pub const TOUCH: Metrics = Metrics {
        control: 44.0,
        font_size: 17.0,
        gap: 12.0,
        radius: 10.0,
        radius_large: 14.0,
    };

    fn entries(&self) -> [(&'static str, f64); 5] {
        [
            ("control-height", self.control),
            ("font-size", self.font_size),
            ("gap", self.gap),
            ("radius", self.radius),
            ("radius-large", self.radius_large),
        ]
    }
}

/// Which appearance a page shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Appearance {
    /// The system's, through `prefers-color-scheme`.
    #[default]
    System,
    Light,
    Dark,
}

impl Appearance {
    /// The root's `data-appearance`.
    pub fn attr(self) -> &'static str {
        match self {
            Appearance::System => "system",
            Appearance::Light => "light",
            Appearance::Dark => "dark",
        }
    }
}

/// How large controls are drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    /// For a pointer.
    #[default]
    Regular,
    /// For a finger.
    Touch,
}

impl Density {
    /// The root's `data-density`.
    pub fn attr(self) -> &'static str {
        match self {
            Density::Regular => "regular",
            Density::Touch => "touch",
        }
    }
}

/// Everything a page's components are drawn with.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub light: Palette,
    pub dark: Palette,
    pub regular: Metrics,
    pub touch: Metrics,
    /// The font stack.
    pub font: &'static str,
    /// The headings' font stack: a heading, a card's and a dialog's title.
    /// The text's unless a page sets its own, a serif over a sans text.
    pub heading_font: &'static str,
}

/// The system's text face, on each platform.
pub const SYSTEM_FONT: &str = "system-ui, -apple-system, \"Segoe UI\", Roboto, sans-serif";

impl Default for Theme {
    fn default() -> Self {
        Self {
            light: Palette::LIGHT,
            dark: Palette::DARK,
            regular: Metrics::REGULAR,
            touch: Metrics::TOUCH,
            font: SYSTEM_FONT,
            heading_font: SYSTEM_FONT,
        }
    }
}

impl Theme {
    /// The custom properties, for the root (`.unit-root`): the light palette
    /// unless the system or the root asks for the dark, and the regular
    /// sizes unless the root asks for touch.
    pub fn css(&self) -> String {
        let mut s = String::new();
        let palette = |s: &mut String, p: &Palette| {
            for (name, value) in p.entries() {
                let _ = writeln!(s, "  --unit-{name}: {value};");
            }
        };
        let metrics = |s: &mut String, m: &Metrics| {
            for (name, value) in m.entries() {
                let _ = writeln!(s, "  --unit-{name}: {value}px;");
            }
        };
        s.push_str(".unit-root {\n");
        let _ = writeln!(s, "  --unit-font: {};", self.font);
        let _ = writeln!(s, "  --unit-heading-font: {};", self.heading_font);
        palette(&mut s, &self.light);
        metrics(&mut s, &self.regular);
        s.push_str("}\n");
        s.push_str(
            "@media (prefers-color-scheme: dark) {\n.unit-root[data-appearance=\"system\"] {\n",
        );
        palette(&mut s, &self.dark);
        s.push_str("}\n}\n");
        s.push_str(".unit-root[data-appearance=\"dark\"] {\n");
        palette(&mut s, &self.dark);
        s.push_str("}\n");
        s.push_str(".unit-root[data-density=\"touch\"] {\n");
        metrics(&mut s, &self.touch);
        s.push_str("}\n");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every property the component styles use is one the theme sets.
    #[test]
    fn every_property_the_styles_use_is_set() {
        let theme = Theme::default().css();
        for used in crate::style::CSS.split("var(").skip(1) {
            let name = &used[..used.find([')', ',']).unwrap()];
            assert!(theme.contains(&format!("{name}:")), "{name} is not set");
        }
    }

    /// A colour and a size never share a name: the root writes both in one
    /// block, so the later would replace the earlier, and every control
    /// would lose its face or its height.
    #[test]
    fn no_colour_and_size_share_a_name() {
        let palette = Palette::LIGHT.entries().map(|(name, _)| name);
        for (name, _) in Metrics::REGULAR.entries() {
            assert!(!palette.contains(&name), "--unit-{name} is both");
        }
    }

    #[test]
    fn dark_is_set_for_the_system_and_when_forced() {
        let css = Theme::default().css();
        assert_eq!(css.matches("--unit-accent: #3b7ff0").count(), 2, "{css}");
        assert!(css.contains("[data-density=\"touch\"]"));
    }

    #[test]
    fn headings_take_their_own_font() {
        let theme = Theme {
            heading_font: "Georgia, serif",
            ..Theme::default()
        };
        let css = theme.css();
        assert!(
            css.contains("--unit-heading-font: Georgia, serif;"),
            "{css}"
        );
        assert!(
            css.contains(&format!("--unit-font: {SYSTEM_FONT};")),
            "{css}"
        );
    }
}

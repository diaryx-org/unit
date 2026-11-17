//! The theme: every colour, radius, edge, shadow and size a component uses
//! is a CSS custom property, and a [`Theme`] is the values of those
//! properties — a palette for the light appearance, one for the dark, the
//! sizes of each density, and a [`Shape`]: how thick an edge is, how round a
//! pill, how deep a shadow. [`Theme::css`]
//! writes them out, under the root's `data-appearance` and `data-density`,
//! so a page forces an appearance or a touch density with an attribute and
//! nothing is written twice by hand.
//!
//! The names follow one scheme, `--unit-<what>`, and the component styles
//! (`style.rs`) use nothing else, so a host restyles everything from here.
//!
//! unit ships three: [`Theme::default`], its own; [`Theme::soft`], rounder
//! and quieter, its faces drawn by fill rather than edge, on Diaryx's warm
//! neutrals; and [`Theme::sharp`], square, ruled and flat. [`Theme::all`]
//! lists them by name, for a page that offers a choice.

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
    /// The face of what is typed in or ticked: a field, a code's cells, a
    /// checkbox, a radio's ring. A control's own face unless a theme sets
    /// buttons apart by fill and fields by an edge on a lighter face.
    pub field: &'static str,
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
        field: "#ffffff",
        track: "#e6e6ea",
        raised: "#ffffff",
        text: "#1d1d1f",
        dim: "#6e6e73",
        border: "#d4d4d8",
        separator: "#e8e8ec",
        accent: "#2f6fde",
        accent_text: "#ffffff",
        danger: "#d93a30",
        success: "#1b843b",
        warning: "#a86400",
        focus: "#3f7fe6",
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
        field: "#3a3a3c",
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

    /// [`Theme::soft`]'s light: Diaryx's warm paper and ink, its blue.
    pub const SOFT_LIGHT: Palette = Palette {
        bg: "#f4f3ef",
        surface: "#fcfcfb",
        control: "#e9e8e3",
        control_hover: "#e0dfd9",
        field: "#fcfcfb",
        track: "#e4e3dd",
        raised: "#fcfcfb",
        text: "#1a1a19",
        dim: "#63615b",
        // A field's and a checkbox's edge: 3:1 on the paper, as WCAG asks
        // of what shows where a control is. Rules are the separator's.
        border: "#8e8b84",
        separator: "#e6e4de",
        accent: "#2163c0",
        accent_text: "#ffffff",
        danger: "#c2301c",
        success: "#2a7444",
        warning: "#94600f",
        focus: "#2163c0",
        scrim: "rgba(26, 26, 25, 0.22)",
        selection: "#d3e3f7",
        shadow: "rgba(40, 36, 28, 0.16)",
        shadow_soft: "rgba(40, 36, 28, 0.06)",
    };

    /// [`Theme::soft`]'s dark: warm greys, never black, and a light blue
    /// with dark text on it, which reads both as a fill and as a link on
    /// the dark ground (no one blue does both with white text).
    pub const SOFT_DARK: Palette = Palette {
        bg: "#161615",
        surface: "#211f1e",
        control: "#2e2c2a",
        control_hover: "#383633",
        field: "#1b1a19",
        track: "#2e2c2a",
        raised: "#4a4744",
        text: "#efeee9",
        dim: "#a19f97",
        border: "#76736c",
        separator: "#2f2d2b",
        accent: "#6fa6ee",
        accent_text: "#0c1a2e",
        danger: "#f0604a",
        success: "#4cc475",
        warning: "#e7a83e",
        focus: "#6fa6ee",
        scrim: "rgba(0, 0, 0, 0.5)",
        selection: "#28456b",
        shadow: "rgba(0, 0, 0, 0.55)",
        shadow_soft: "rgba(0, 0, 0, 0.25)",
    };

    /// [`Theme::sharp`]'s light: white, black ink, and rules in between.
    pub const SHARP_LIGHT: Palette = Palette {
        bg: "#ffffff",
        surface: "#ffffff",
        control: "#ffffff",
        control_hover: "#f0f0f0",
        field: "#ffffff",
        track: "#e8e8e8",
        raised: "#ffffff",
        text: "#111111",
        dim: "#5f5f5f",
        border: "#111111",
        separator: "#d0d0d0",
        accent: "#111111",
        accent_text: "#ffffff",
        danger: "#c4161c",
        success: "#1d7a35",
        warning: "#9a5b00",
        focus: "#0050d0",
        scrim: "rgba(255, 255, 255, 0.6)",
        selection: "#dddddd",
        shadow: "rgba(0, 0, 0, 0)",
        shadow_soft: "rgba(0, 0, 0, 0)",
    };

    /// [`Theme::sharp`]'s dark: the light turned over.
    pub const SHARP_DARK: Palette = Palette {
        bg: "#0a0a0a",
        surface: "#0a0a0a",
        control: "#0a0a0a",
        control_hover: "#1e1e1e",
        field: "#0a0a0a",
        track: "#262626",
        raised: "#0a0a0a",
        text: "#f2f2f2",
        dim: "#a3a3a3",
        border: "#f2f2f2",
        separator: "#3a3a3a",
        accent: "#f2f2f2",
        accent_text: "#0a0a0a",
        danger: "#ff5a52",
        success: "#3ccf6a",
        warning: "#ffb340",
        focus: "#6ea2ff",
        scrim: "rgba(0, 0, 0, 0.6)",
        selection: "#3a3a3a",
        shadow: "rgba(0, 0, 0, 0)",
        shadow_soft: "rgba(0, 0, 0, 0)",
    };

    /// This palette with the system's accent, focus ring and selection.
    pub(crate) fn with(mut self, system: SystemColours) -> Palette {
        self.accent = system.accent;
        self.focus = system.focus;
        self.selection = system.selection;
        self
    }

    fn entries(&self) -> [(&'static str, &'static str); 21] {
        [
            ("bg", self.bg),
            ("surface", self.surface),
            ("control", self.control),
            ("control-hover", self.control_hover),
            ("field", self.field),
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

/// Where a page's accent comes from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Accent {
    /// The theme's.
    #[default]
    Theme,
    /// The system's, where unit can read one: on a Mac with the `apple`
    /// feature, the accent and highlight colours chosen in System Settings,
    /// followed when they change. The theme's anywhere else.
    System,
}

/// The colours a system chooses for one appearance, over the theme's: what
/// [`Accent::System`] puts in each palette.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SystemColours {
    pub accent: &'static str,
    pub focus: &'static str,
    pub selection: &'static str,
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
    /// A control's corners.
    pub radius: f64,
    /// A raised surface's corners: a card, a group of rows, a menu, a
    /// dialog.
    pub radius_large: f64,
    /// The corners of what sits inside a control or a surface: a menu's
    /// highlighted row, a segment's face, a checkbox.
    pub radius_small: f64,
}

impl Metrics {
    /// For a pointer: the Mac's sizes.
    pub const REGULAR: Metrics = Metrics {
        control: 24.0,
        font_size: 13.0,
        gap: 8.0,
        radius: 6.0,
        radius_large: 10.0,
        radius_small: 4.0,
    };

    /// For a finger: 44-point targets, as Apple's guidelines ask.
    pub const TOUCH: Metrics = Metrics {
        control: 44.0,
        font_size: 17.0,
        gap: 12.0,
        radius: 10.0,
        radius_large: 14.0,
        radius_small: 8.0,
    };

    /// For a terminal's cells ([`Density::Cell`]): a control a row high,
    /// text a monospace face's 16px, a gap of about one cell's width, and
    /// square corners.
    pub const CELL: Metrics = Metrics {
        control: 24.0,
        font_size: 16.0,
        gap: 10.0,
        radius: 0.0,
        radius_large: 0.0,
        radius_small: 0.0,
    };

    /// These sizes with other corners: what a theme of another shape
    /// changes and nothing else, as [`Theme::soft`] and [`Theme::sharp`] do.
    pub const fn rounded(mut self, radius: f64, large: f64, small: f64) -> Metrics {
        self.radius = radius;
        self.radius_large = large;
        self.radius_small = small;
        self
    }

    fn entries(&self) -> [(&'static str, f64); 6] {
        [
            ("control-height", self.control),
            ("font-size", self.font_size),
            ("gap", self.gap),
            ("radius", self.radius),
            ("radius-large", self.radius_large),
            ("radius-small", self.radius_small),
        ]
    }
}

/// What a theme's pieces are shaped like, at every density: the edges they
/// are drawn with, how round a pill is, how far what floats stands off the
/// page, and how a menu's rows sit in it. The corners are [`Metrics`]',
/// since they grow with a finger's sizes.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    /// A control's edge, in pixels: a button's, a chip's, a list's. 0
    /// draws a face by its fill alone.
    pub edge: f64,
    /// The edge of what is typed in or ticked: a field, a code's cells, a
    /// checkbox, a radio's ring. Its edge is what shows where it is, so a
    /// theme keeps it where it draws buttons by fill alone.
    pub field_edge: f64,
    /// A raised surface's edge: a card's, a group of rows', a table's.
    pub surface_edge: f64,
    /// The edge of what floats over the page: a menu, a dialog, a toast.
    pub float_edge: f64,
    /// The corners of a pill, in pixels: a chip, a badge, a switch, a
    /// radio's ring. Half its height or more is round.
    pub pill: f64,
    /// The shadow under a control, as CSS; `none` for none. It may name the
    /// palette's shadows: `var(--unit-shadow-soft)`.
    pub control_shadow: &'static str,
    /// The shadow under a card and a group of rows.
    pub surface_shadow: &'static str,
    /// The shadow under a menu, a dialog, a toast: what floats.
    pub float_shadow: &'static str,
    /// The shadow under the chosen segment and a switch's knob: what stands
    /// up from a groove.
    pub raised_shadow: &'static str,
    /// How far a menu's rows stand in from its sides, so a highlighted row
    /// is a rounded bar within it (5) or the menu's whole width (0).
    pub menu_inset: f64,
    /// The focus ring's width.
    pub focus_ring: f64,
    /// The weight of a heading, a card's title and a dialog's.
    pub heading_weight: u16,
}

impl Shape {
    /// unit's own: hairline edges on every control, round pills, and soft
    /// shadows under what floats.
    pub const DEFAULT: Shape = Shape {
        edge: 1.0,
        field_edge: 1.0,
        surface_edge: 1.0,
        float_edge: 1.0,
        pill: 999.0,
        control_shadow: "0 1px 2px var(--unit-shadow-soft)",
        surface_shadow: "0 1px 3px var(--unit-shadow-soft)",
        float_shadow: "0 8px 24px var(--unit-shadow)",
        raised_shadow: "0 1px 3px var(--unit-shadow)",
        menu_inset: 5.0,
        focus_ring: 2.0,
        heading_weight: 700,
    };

    /// [`Theme::soft`]'s: buttons and chips drawn by fill, not edge, but
    /// fields and checkboxes still edged; surfaces lifted by
    /// a shadow rather than ruled; a wide, soft shadow under what floats.
    pub const SOFT: Shape = Shape {
        edge: 0.0,
        field_edge: 1.0,
        surface_edge: 0.0,
        float_edge: 0.0,
        pill: 999.0,
        control_shadow: "none",
        surface_shadow: "0 1px 2px var(--unit-shadow-soft), 0 0 0 1px var(--unit-shadow-soft)",
        float_shadow: "0 0 0 1px var(--unit-shadow-soft), 0 12px 32px var(--unit-shadow)",
        raised_shadow: "0 1px 2px var(--unit-shadow), 0 0 0 1px var(--unit-shadow-soft)",
        menu_inset: 6.0,
        focus_ring: 3.0,
        heading_weight: 600,
    };

    /// [`Theme::sharp`]'s: square, ruled edges everywhere, and nothing that
    /// casts a shadow.
    pub const SHARP: Shape = Shape {
        edge: 1.0,
        field_edge: 1.0,
        surface_edge: 1.0,
        float_edge: 1.0,
        pill: 0.0,
        control_shadow: "none",
        surface_shadow: "none",
        float_shadow: "4px 4px 0 var(--unit-separator)",
        raised_shadow: "none",
        menu_inset: 0.0,
        focus_ring: 2.0,
        heading_weight: 800,
    };

    fn entries(&self) -> [(&'static str, String); 12] {
        let px = |v: f64| format!("{v}px");
        [
            ("edge", px(self.edge)),
            ("field-edge", px(self.field_edge)),
            ("surface-edge", px(self.surface_edge)),
            ("float-edge", px(self.float_edge)),
            ("pill", px(self.pill)),
            ("control-shadow", self.control_shadow.to_string()),
            ("surface-shadow", self.surface_shadow.to_string()),
            ("float-shadow", self.float_shadow.to_string()),
            ("raised-shadow", self.raised_shadow.to_string()),
            ("menu-inset", px(self.menu_inset)),
            ("focus-ring", px(self.focus_ring)),
            ("heading-weight", self.heading_weight.to_string()),
        ]
    }
}

impl Default for Shape {
    fn default() -> Self {
        Shape::DEFAULT
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
    /// For a terminal's character cells: a renderer that paints the page
    /// into a grid (leaf's terminal host) rather than pixels. Every row a
    /// menu or a dialog has is one cell high — [`Metrics::CELL`]'s control,
    /// 24px — with no rounded corners or shadows, which cells cannot draw.
    /// The host gives the theme its monospace [`Theme::font`], whose advance
    /// is its cell.
    Cell,
}

impl Density {
    /// The root's `data-density`.
    pub fn attr(self) -> &'static str {
        match self {
            Density::Regular => "regular",
            Density::Touch => "touch",
            Density::Cell => "cell",
        }
    }
}

/// Everything a page's components are drawn with.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// What a page that offers a choice of themes calls it.
    pub name: &'static str,
    pub light: Palette,
    pub dark: Palette,
    pub regular: Metrics,
    pub touch: Metrics,
    pub cell: Metrics,
    /// The font stack.
    pub font: &'static str,
    /// The headings' font stack: a heading, a card's and a dialog's title.
    /// The text's unless a page sets its own, a serif over a sans text.
    pub heading_font: &'static str,
    /// The edges, pills and shadows.
    pub shape: Shape,
}

/// The system's text face, on each platform.
pub const SYSTEM_FONT: &str = "system-ui, -apple-system, \"Segoe UI\", Roboto, sans-serif";

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "Unit",
            light: Palette::LIGHT,
            dark: Palette::DARK,
            regular: Metrics::REGULAR,
            touch: Metrics::TOUCH,
            cell: Metrics::CELL,
            font: SYSTEM_FONT,
            heading_font: SYSTEM_FONT,
            shape: Shape::DEFAULT,
        }
    }
}

impl Theme {
    /// Soft: what Diaryx is drawn in. Diaryx's warm paper and ink and its
    /// blue; rounder corners; faces told apart by fill rather than edge;
    /// groups of rows and cards lifted by a shadow rather than ruled, and
    /// what floats on a wide, soft one. A finger's sizes and a terminal's
    /// cells are unit's own, but rounder for a finger.
    pub fn soft() -> Theme {
        Theme {
            name: "Soft",
            light: Palette::SOFT_LIGHT,
            dark: Palette::SOFT_DARK,
            regular: Metrics::REGULAR.rounded(8.0, 12.0, 6.0),
            touch: Metrics::TOUCH.rounded(12.0, 18.0, 10.0),
            shape: Shape::SOFT,
            ..Theme::default()
        }
    }

    /// Sharp: square corners, every edge ruled in ink, and no shadow but a
    /// flat offset under what floats.
    pub fn sharp() -> Theme {
        Theme {
            name: "Sharp",
            light: Palette::SHARP_LIGHT,
            dark: Palette::SHARP_DARK,
            regular: Metrics::REGULAR.rounded(0.0, 0.0, 0.0),
            touch: Metrics::TOUCH.rounded(0.0, 0.0, 0.0),
            shape: Shape::SHARP,
            ..Theme::default()
        }
    }

    /// Every theme unit ships, its own first: for a page that offers them.
    pub fn all() -> Vec<Theme> {
        vec![Theme::default(), Theme::soft(), Theme::sharp()]
    }

    /// The sizes of `density`.
    pub fn metrics(&self, density: Density) -> &Metrics {
        match density {
            Density::Regular => &self.regular,
            Density::Touch => &self.touch,
            Density::Cell => &self.cell,
        }
    }

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
        for (name, value) in self.shape.entries() {
            let _ = writeln!(s, "  --unit-{name}: {value};");
        }
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
        s.push_str(".unit-root[data-density=\"cell\"] {\n");
        metrics(&mut s, &self.cell);
        // Cells draw no curve and no shadow, whatever the theme's shape.
        s.push_str("  --unit-pill: 0px;\n");
        for shadow in ["control", "surface", "float", "raised"] {
            let _ = writeln!(s, "  --unit-{shadow}-shadow: none;");
        }
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

    /// Every theme unit ships sets every property, and is told apart by
    /// its name.
    #[test]
    fn every_shipped_theme_is_whole_and_named() {
        let all = Theme::all();
        let names: Vec<_> = all.iter().map(|t| t.name).collect();
        assert_eq!(names, ["Unit", "Soft", "Sharp"]);
        let reference = Theme::default().css();
        let declared = |css: &str| -> Vec<String> {
            css.lines()
                .filter_map(|l| l.trim().strip_prefix("--unit-"))
                .map(|l| l[..l.find(':').unwrap()].to_string())
                .collect()
        };
        for theme in &all {
            assert_eq!(
                declared(&theme.css()),
                declared(&reference),
                "{}",
                theme.name
            );
        }
    }

    /// A theme's shape reaches the page: Sharp is square and casts no
    /// shadow, Soft draws its controls without an edge.
    #[test]
    fn a_theme_changes_shapes_as_well_as_colours() {
        let sharp = Theme::sharp().css();
        assert!(sharp.contains("--unit-pill: 0px;"), "{sharp}");
        assert!(sharp.contains("--unit-radius-large: 0px;"), "{sharp}");
        assert!(sharp.contains("--unit-control-shadow: none;"), "{sharp}");
        let soft = Theme::soft().css();
        assert!(soft.contains("--unit-edge: 0px;"), "{soft}");
        assert!(soft.contains("--unit-radius-large: 12px;"), "{soft}");
        assert!(soft.contains("--unit-accent: #2163c0;"), "{soft}");
        // Cells round nothing, in any theme.
        let cell = &soft[soft.find("[data-density=\"cell\"]").unwrap()..];
        assert!(cell.contains("--unit-pill: 0px;") && cell.contains("--unit-float-shadow: none;"));
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

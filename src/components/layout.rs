//! Layout, text and feedback: what places controls and says things, and
//! takes no keys.

use dioxus::prelude::*;

use crate::icon::paths;
use crate::layer::Tone;
use crate::{Button, Variant};

/// Which way a stack runs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Axis {
    #[default]
    Vertical,
    Horizontal,
}

/// Where a stack's children sit across it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Stretch,
    Start,
    Center,
    End,
}

impl Align {
    fn css(self) -> &'static str {
        match self {
            Align::Stretch => "stretch",
            Align::Start => "flex-start",
            Align::Center => "center",
            Align::End => "flex-end",
        }
    }
}

/// Children one after another, the theme's gap apart.
#[component]
pub fn Stack(
    #[props(default)] axis: Axis,
    #[props(default)] align: Align,
    /// The gap, in CSS pixels; the theme's unless given.
    gap: Option<f64>,
    /// Whether a row wraps onto further lines where it runs out of room.
    #[props(default)]
    wrap: bool,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let dir = match axis {
        Axis::Vertical => "column",
        Axis::Horizontal => "row",
    };
    let gap = gap.map_or("var(--unit-gap)".to_string(), |g| format!("{g}px"));
    let wrap = if wrap { "wrap" } else { "nowrap" };
    let align = align.css();
    rsx! {
        div {
            class: "unit-stack {class}",
            style: "flex-direction: {dir}; align-items: {align}; gap: {gap}; flex-wrap: {wrap}",
            {children}
        }
    }
}

/// Room that takes up whatever a stack has left, pushing what follows to
/// the far end.
#[component]
pub fn Spacer() -> Element {
    rsx! { div { class: "unit-spacer" } }
}

/// A raised surface round related content.
#[component]
pub fn Card(title: Option<String>, #[props(default)] class: String, children: Element) -> Element {
    rsx! {
        section { class: "unit-card {class}",
            if let Some(t) = title {
                h3 { class: "unit-card-title", "{t}" }
            }
            {children}
        }
    }
}

/// A rule between things.
#[component]
pub fn Divider() -> Element {
    rsx! { div { class: "unit-divider", role: "separator" } }
}

/// A row of a form: a label on the left, its control on the right, and a
/// note under both — a settings pane's row.
#[component]
pub fn Field(label: String, note: Option<String>, children: Element) -> Element {
    rsx! {
        div { class: "unit-form-row",
            div { class: "unit-form-label", "{label}" }
            div { class: "unit-form-control",
                {children}
                if let Some(n) = note {
                    div { class: "unit-note", "{n}" }
                }
            }
        }
    }
}

/// A heading, `level` 1 to 6.
#[component]
pub fn Heading(#[props(default = 2)] level: u8, children: Element) -> Element {
    match level.clamp(1, 6) {
        1 => rsx! { h1 { class: "unit-heading", {children} } },
        2 => rsx! { h2 { class: "unit-heading", {children} } },
        3 => rsx! { h3 { class: "unit-heading", {children} } },
        4 => rsx! { h4 { class: "unit-heading", {children} } },
        5 => rsx! { h5 { class: "unit-heading", {children} } },
        _ => rsx! { h6 { class: "unit-heading", {children} } },
    }
}

/// Text, plain or secondary.
#[component]
pub fn Text(#[props(default)] dim: bool, children: Element) -> Element {
    rsx! {
        p { class: if dim { "unit-text dim" } else { "unit-text" }, {children} }
    }
}

/// A small label in a tone: a count, a state.
#[component]
pub fn Badge(#[props(default)] tone: Tone, children: Element) -> Element {
    rsx! { span { class: "unit-badge{tone.class()}", {children} } }
}

/// How far something has got, from 0 to 1.
#[component]
pub fn Progress(value: f64, label: Option<String>) -> Element {
    let v = value.clamp(0.0, 1.0);
    let pct = (v * 100.0).round();
    rsx! {
        div {
            class: "unit-progress",
            role: "progressbar",
            "aria-label": label,
            "aria-valuenow": "{pct}",
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            div { class: "unit-progress-fill", style: "width: {pct}%" }
        }
    }
}

/// A message in the page, in a tone: a warning over a form, a tip. With
/// `action`, a button beside it does `onaction`; with `ondismiss`, a close
/// button takes it away.
#[component]
pub fn Callout(
    #[props(default)] tone: Tone,
    title: Option<String>,
    action: Option<String>,
    onaction: Option<EventHandler<()>>,
    ondismiss: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let role = match tone {
        Tone::Danger | Tone::Warning => "alert",
        _ => "note",
    };
    rsx! {
        div { class: "unit-callout{tone.class()}", role,
            div { class: "unit-callout-body",
                if let Some(t) = title {
                    div { class: "unit-callout-title", "{t}" }
                }
                {children}
            }
            if let Some(a) = action {
                Button { variant: Variant::Ghost, onpress: move |_| if let Some(f) = onaction { f.call(()) }, "{a}" }
            }
            if let Some(d) = ondismiss {
                Button { variant: Variant::Ghost, class: "unit-icon", label: "Dismiss",
                         icon: paths::CLOSE, onpress: move |_| d.call(()) }
            }
        }
    }
}

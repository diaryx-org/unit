//! Choices: a checkbox, a switch, a radio group, a segmented control, a row
//! of colour swatches and a stepper.
//!
//! Each is drawn by the page — a box, a track and knob, rings, segments —
//! rather than by the renderer's `<input type="checkbox">`, so it looks the
//! same in a browser and in Blitz, and carries its ARIA role and state
//! (`role="checkbox"`, `aria-checked`) so both read it as the control it is.

use dioxus::html::{Key, Modifiers, NamedKey};
use dioxus::prelude::*;

use crate::unit::{Kind, next_id, use_focus};

fn is_space(key: &Key) -> bool {
    matches!(key, Key::Character(c) if c == " ")
}

/// A box ticked or not, and its label. `onchange` is given the new state.
#[component]
pub fn Checkbox(
    checked: bool,
    onchange: EventHandler<bool>,
    #[props(default)] disabled: bool,
    children: Element,
) -> Element {
    let focus = use_focus(Kind::Widget, move |key, _| {
        if is_space(key) && !disabled {
            onchange.call(!checked);
            return true;
        }
        false
    });
    let on = if checked { " on" } else { "" };
    let off = if disabled { " off" } else { "" };
    rsx! {
        div {
            class: "unit-check{on}{off}{focus.class()}",
            role: "checkbox",
            tabindex: if disabled { "-1" } else { "0" },
            "aria-checked": if checked { "true" } else { "false" },
            "aria-disabled": disabled.then_some("true"),
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                e.stop_propagation();
                if !disabled {
                    onchange.call(!checked);
                }
            },
            span { class: "unit-check-box", "aria-hidden": "true",
                if checked { "✓" }
            }
            span { class: "unit-check-label", {children} }
        }
    }
}

/// A switch: on or off, at once — a setting, not a choice to submit.
#[component]
pub fn Switch(
    on: bool,
    onchange: EventHandler<bool>,
    #[props(default)] disabled: bool,
    children: Element,
) -> Element {
    let focus = use_focus(Kind::Widget, move |key, _| {
        if (is_space(key) || matches!(key, Key::Named(NamedKey::Enter))) && !disabled {
            onchange.call(!on);
            return true;
        }
        false
    });
    let class = if on { " on" } else { "" };
    let off = if disabled { " off" } else { "" };
    rsx! {
        div {
            class: "unit-switch{class}{off}{focus.class()}",
            role: "switch",
            tabindex: if disabled { "-1" } else { "0" },
            "aria-checked": if on { "true" } else { "false" },
            "aria-disabled": disabled.then_some("true"),
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                e.stop_propagation();
                if !disabled {
                    onchange.call(!on);
                }
            },
            span { class: "unit-switch-label", {children} }
            span { class: "unit-switch-track", "aria-hidden": "true",
                span { class: "unit-switch-knob" }
            }
        }
    }
}

/// Which way a group of choices runs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}

/// The index the arrows move a choice of `n` to from `at`, wrapping, or
/// `None` for a key they do not move it by. Home and End go to the ends.
pub(crate) fn arrow_step(key: &Key, at: Option<usize>, n: usize, o: Orientation) -> Option<usize> {
    if n == 0 {
        return None;
    }
    let (back, on) = match o {
        Orientation::Vertical => (NamedKey::ArrowUp, NamedKey::ArrowDown),
        Orientation::Horizontal => (NamedKey::ArrowLeft, NamedKey::ArrowRight),
    };
    match key {
        Key::Named(k) if *k == on || *k == NamedKey::ArrowRight || *k == NamedKey::ArrowDown => {
            Some(at.map_or(0, |i| (i + 1) % n))
        }
        Key::Named(k) if *k == back || *k == NamedKey::ArrowLeft || *k == NamedKey::ArrowUp => {
            Some(at.map_or(n - 1, |i| (i + n - 1) % n))
        }
        Key::Named(NamedKey::Home) => Some(0),
        Key::Named(NamedKey::End) => Some(n - 1),
        _ => None,
    }
}

/// The group's keys: the arrows, Home and End choose.
fn group_keys(
    key: &Key,
    _: Modifiers,
    selected: Option<usize>,
    n: usize,
    o: Orientation,
    onchange: EventHandler<usize>,
) -> bool {
    match arrow_step(key, selected, n, o) {
        Some(i) => {
            if Some(i) != selected {
                onchange.call(i);
            }
            true
        }
        None => false,
    }
}

/// Radio buttons: one of `options`, chosen. The group has the keys as one
/// control, and its arrows move the choice, as a system's do.
#[component]
pub fn RadioGroup(
    options: Vec<String>,
    selected: Option<usize>,
    onchange: EventHandler<usize>,
    /// What the group chooses, read before its options.
    label: Option<String>,
    #[props(default)] orientation: Orientation,
    #[props(default)] disabled: bool,
) -> Element {
    let n = options.len();
    let focus = use_focus(Kind::Widget, move |key, mods| {
        !disabled && group_keys(key, mods, selected, n, orientation, onchange)
    });
    let dir = match orientation {
        Orientation::Vertical => "",
        Orientation::Horizontal => " unit-row",
    };
    rsx! {
        div {
            class: "unit-radios{dir}{focus.class()}",
            role: "radiogroup",
            tabindex: if disabled { "-1" } else { "0" },
            "aria-label": label,
            "aria-disabled": disabled.then_some("true"),
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            for (i, o) in options.into_iter().enumerate() {
                div {
                    key: "{i}",
                    class: if selected == Some(i) { "unit-radio on" } else { "unit-radio" },
                    role: "radio",
                    "aria-checked": if selected == Some(i) { "true" } else { "false" },
                    onclick: move |e: MouseEvent| {
                        e.stop_propagation();
                        if !disabled {
                            onchange.call(i);
                        }
                    },
                    span { class: "unit-radio-ring", "aria-hidden": "true",
                        span { class: "unit-radio-dot" }
                    }
                    span { "{o}" }
                }
            }
        }
    }
}

/// A segmented control: a row of joined buttons, one of them chosen — a
/// view's mode, a setting with few values.
#[component]
pub fn Segmented(
    options: Vec<String>,
    selected: Option<usize>,
    onchange: EventHandler<usize>,
    label: Option<String>,
    #[props(default)] disabled: bool,
) -> Element {
    let n = options.len();
    let focus = use_focus(Kind::Widget, move |key, mods| {
        !disabled && group_keys(key, mods, selected, n, Orientation::Horizontal, onchange)
    });
    let group = use_hook(next_id);
    rsx! {
        div {
            class: "unit-segmented{focus.class()}",
            role: "radiogroup",
            id: "unit-{group}",
            tabindex: if disabled { "-1" } else { "0" },
            "aria-label": label,
            "aria-disabled": disabled.then_some("true"),
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            for (i, o) in options.into_iter().enumerate() {
                div {
                    key: "{i}",
                    class: if selected == Some(i) { "unit-segment on" } else { "unit-segment" },
                    role: "radio",
                    "aria-checked": if selected == Some(i) { "true" } else { "false" },
                    onclick: move |e: MouseEvent| {
                        e.stop_propagation();
                        if !disabled {
                            onchange.call(i);
                        }
                    },
                    "{o}"
                }
            }
        }
    }
}

/// One colour a [`Swatches`] offers: what it is called, read out in place
/// of the colour, its fill, and the ring round it if not the fill's own.
#[derive(Clone, Debug, PartialEq)]
pub struct Swatch {
    pub name: String,
    pub fill: String,
    pub edge: Option<String>,
}

impl Swatch {
    pub fn new(name: impl Into<String>, fill: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fill: fill.into(),
            edge: None,
        }
    }

    /// A ring of its own: a darker tone of the fill, say.
    pub fn edge(mut self, edge: impl Into<String>) -> Self {
        self.edge = Some(edge.into());
        self
    }
}

/// Colour swatches: one of `swatches`, chosen — a cover's tone, a
/// highlighter's ink. Chosen as a radio group: a tap, or the arrows once it
/// has the keys. `children` go before the swatches, in a swatch's shape: a
/// [`SwatchAction`] such as Random, or a choice of the page's own such as
/// Automatic.
#[component]
pub fn Swatches(
    swatches: Vec<Swatch>,
    selected: Option<usize>,
    onchange: EventHandler<usize>,
    /// What the swatches choose, read before them.
    label: Option<String>,
    #[props(default)] disabled: bool,
    children: Element,
) -> Element {
    let n = swatches.len();
    let focus = use_focus(Kind::Widget, move |key, mods| {
        !disabled && group_keys(key, mods, selected, n, Orientation::Horizontal, onchange)
    });
    rsx! {
        div {
            class: "unit-swatches{focus.class()}",
            role: "radiogroup",
            tabindex: if disabled { "-1" } else { "0" },
            "aria-label": label,
            "aria-disabled": disabled.then_some("true"),
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            {children}
            for (i, s) in swatches.into_iter().enumerate() {
                div {
                    key: "{i}",
                    class: if selected == Some(i) { "unit-swatch on" } else { "unit-swatch" },
                    role: "radio",
                    "aria-checked": if selected == Some(i) { "true" } else { "false" },
                    "aria-label": "{s.name}",
                    style: "background: {s.fill}; border-color: {s.edge.as_deref().unwrap_or(&s.fill)}",
                    onclick: move |e: MouseEvent| {
                        e.stop_propagation();
                        if !disabled {
                            onchange.call(i);
                        }
                    },
                }
            }
        }
    }
}

/// A swatch's place that is not a colour: an icon, or a letter, that acts
/// (Random) or stands for a choice of the page's own (`on`: Automatic).
#[component]
pub fn SwatchAction(
    /// What it does, read out.
    label: String,
    /// An icon's path ([`crate::icon::paths`]); else `children`.
    icon: Option<&'static str>,
    #[props(default)] on: bool,
    onpress: EventHandler<()>,
    children: Element,
) -> Element {
    rsx! {
        crate::Pressable {
            class: if on { "unit-swatch unit-swatch-action on" } else { "unit-swatch unit-swatch-action" },
            "aria-label": "{label}",
            "aria-pressed": on.then_some("true"),
            onpress: move |_| onpress.call(()),
            if let Some(d) = icon {
                crate::icon::Icon { d, size: 18.0 }
            } else {
                {children}
            }
        }
    }
}

/// A stepper: a value made one step less or more — an hour, a count. Its
/// two halves step; with the keys, the arrows and Page Up and Down do. The
/// page shows the value; `value_text` is what assistive tech reads for it.
#[component]
pub fn Stepper(
    ondecrement: EventHandler<()>,
    onincrement: EventHandler<()>,
    /// What it sets, read before the value.
    label: Option<String>,
    value_text: Option<String>,
    #[props(default = true)] can_decrement: bool,
    #[props(default = true)] can_increment: bool,
    #[props(default)] disabled: bool,
) -> Element {
    let down = can_decrement && !disabled;
    let up = can_increment && !disabled;
    let focus = use_focus(Kind::Widget, move |key, _| match key {
        Key::Named(NamedKey::ArrowUp | NamedKey::ArrowRight | NamedKey::PageUp) if up => {
            onincrement.call(());
            true
        }
        Key::Named(NamedKey::ArrowDown | NamedKey::ArrowLeft | NamedKey::PageDown) if down => {
            ondecrement.call(());
            true
        }
        _ => false,
    });
    rsx! {
        div {
            class: "unit-stepper{focus.class()}",
            role: "spinbutton",
            tabindex: if disabled { "-1" } else { "0" },
            "aria-label": label,
            "aria-valuetext": value_text,
            "aria-disabled": disabled.then_some("true"),
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            div {
                class: if down { "unit-step" } else { "unit-step off" },
                "aria-hidden": "true",
                onclick: move |e: MouseEvent| {
                    e.stop_propagation();
                    if down {
                        ondecrement.call(());
                    }
                },
                crate::icon::Icon { d: crate::icon::paths::MINUS, size: 18.0 }
            }
            span { class: "unit-stepper-rule", "aria-hidden": "true" }
            div {
                class: if up { "unit-step" } else { "unit-step off" },
                "aria-hidden": "true",
                onclick: move |e: MouseEvent| {
                    e.stop_propagation();
                    if up {
                        onincrement.call(());
                    }
                },
                crate::icon::Icon { d: crate::icon::paths::PLUS, size: 18.0 }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_arrows_wrap_and_home_and_end_go_to_the_ends() {
        let right = Key::Named(NamedKey::ArrowRight);
        let up = Key::Named(NamedKey::ArrowUp);
        let h = Orientation::Horizontal;
        assert_eq!(arrow_step(&right, Some(2), 3, h), Some(0));
        assert_eq!(arrow_step(&up, Some(0), 3, h), Some(2));
        assert_eq!(arrow_step(&right, None, 3, h), Some(0));
        assert_eq!(arrow_step(&Key::Named(NamedKey::End), None, 3, h), Some(2));
        assert_eq!(arrow_step(&Key::Character("a".into()), Some(0), 3, h), None);
        assert_eq!(arrow_step(&right, None, 0, h), None);
    }
}

//! Choices: a checkbox, a switch, a radio group and a segmented control.
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

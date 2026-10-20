//! Moving among things: tabs, a list to choose from, and a section that
//! opens and closes.

use dioxus::html::{Key, Modifiers, NamedKey};
use dioxus::prelude::*;

use crate::components::button::is_press;
use crate::components::choice::{Orientation, arrow_step};
use crate::unit::{Kind, next_id, use_focus};

/// Tabs: a row of titles, one chosen, and the chosen one's content under
/// them. The row has the keys as one control; the arrows choose.
#[component]
pub fn Tabs(
    tabs: Vec<String>,
    selected: usize,
    onchange: EventHandler<usize>,
    label: Option<String>,
    /// The chosen tab's content.
    children: Element,
) -> Element {
    let n = tabs.len();
    let focus = use_focus(Kind::Widget, move |key, _| {
        match arrow_step(key, Some(selected), n, Orientation::Horizontal) {
            Some(i) => {
                if i != selected {
                    onchange.call(i);
                }
                true
            }
            None => false,
        }
    });
    let base = use_hook(next_id);
    rsx! {
        div { class: "unit-tabs",
            div {
                class: "unit-tablist{focus.class()}",
                role: "tablist",
                tabindex: "0",
                "aria-label": label,
                onpointerdown: move |_| focus.press(),
                onfocus: move |_| focus.dom_focus(),
                onblur: move |_| focus.dom_blur(),
                for (i, t) in tabs.into_iter().enumerate() {
                    div {
                        key: "{i}",
                        class: if i == selected { "unit-tab on" } else { "unit-tab" },
                        role: "tab",
                        id: "unit-{base}-tab-{i}",
                        "aria-selected": if i == selected { "true" } else { "false" },
                        "aria-controls": "unit-{base}-panel",
                        onclick: move |e: MouseEvent| {
                            e.stop_propagation();
                            onchange.call(i);
                        },
                        "{t}"
                    }
                }
            }
            div {
                class: "unit-tabpanel",
                role: "tabpanel",
                id: "unit-{base}-panel",
                "aria-labelledby": "unit-{base}-tab-{selected}",
                {children}
            }
        }
    }
}

/// The next item, from `from`, whose label starts with `typed`, wrapping.
pub(crate) fn typeahead(items: &[String], from: Option<usize>, typed: &str) -> Option<usize> {
    let n = items.len();
    let typed = typed.to_lowercase();
    let start = from.map_or(0, |h| h + 1);
    (0..n)
        .map(|k| (start + k) % n)
        .find(|&k| items[k].to_lowercase().starts_with(&typed))
}

/// A list to choose one item from: a sidebar's, a picker's. The arrows,
/// Home and End move the choice, and a letter jumps to the next item
/// starting with it.
#[component]
pub fn ListBox(
    items: Vec<String>,
    selected: Option<usize>,
    onchange: EventHandler<usize>,
    label: Option<String>,
    /// Return or a double click on an item: open it.
    onactivate: Option<EventHandler<usize>>,
) -> Element {
    let n = items.len();
    let keyed = items.clone();
    let focus = use_focus(Kind::Widget, move |key: &Key, mods: Modifiers| {
        if let Some(i) = arrow_step(key, selected, n, Orientation::Vertical) {
            if Some(i) != selected {
                onchange.call(i);
            }
            return true;
        }
        match key {
            Key::Named(NamedKey::Enter) => {
                if let (Some(a), Some(i)) = (onactivate, selected) {
                    a.call(i);
                }
                true
            }
            Key::Character(c) if mods.is_empty() || mods == Modifiers::SHIFT => {
                if let Some(i) = typeahead(&keyed, selected, c)
                    && Some(i) != selected
                {
                    onchange.call(i);
                }
                true
            }
            _ => false,
        }
    });
    rsx! {
        div {
            class: "unit-listbox{focus.class()}",
            role: "listbox",
            tabindex: "0",
            "aria-label": label,
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            for (i, item) in items.into_iter().enumerate() {
                div {
                    key: "{i}",
                    class: if selected == Some(i) { "unit-option on" } else { "unit-option" },
                    role: "option",
                    "aria-selected": if selected == Some(i) { "true" } else { "false" },
                    onclick: move |e: MouseEvent| {
                        e.stop_propagation();
                        if selected != Some(i) {
                            onchange.call(i);
                        }
                    },
                    ondoubleclick: move |_| {
                        if let Some(a) = onactivate {
                            a.call(i);
                        }
                    },
                    "{item}"
                }
            }
        }
    }
}

/// A section that opens and closes under its title: details a page can
/// hide. It keeps whether it is open itself, starting as `open` says.
#[component]
pub fn Disclosure(title: String, #[props(default)] open: bool, children: Element) -> Element {
    let mut shown = use_signal(|| open);
    let focus = use_focus(Kind::Button, move |key, _| {
        if is_press(key) {
            let mut s = shown;
            s.toggle();
            return true;
        }
        false
    });
    let id = use_hook(next_id);
    let is_open = shown();
    rsx! {
        div { class: if is_open { "unit-disclosure on" } else { "unit-disclosure" },
            button {
                class: "unit-disclosure-title{focus.class()}",
                r#type: "button",
                "aria-expanded": if is_open { "true" } else { "false" },
                "aria-controls": "unit-{id}",
                onpointerdown: move |_| focus.press(),
                onfocus: move |_| focus.dom_focus(),
                onblur: move |_| focus.dom_blur(),
                onclick: move |e: MouseEvent| {
                    e.stop_propagation();
                    shown.toggle();
                },
                span { class: "unit-disclosure-arrow", "aria-hidden": "true",
                    if is_open { "▾" } else { "▸" }
                }
                "{title}"
            }
            div { class: "unit-disclosure-body", id: "unit-{id}",
                style: if is_open { "display: block" } else { "display: none" },
                {children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typeahead_finds_the_next_match_and_wraps() {
        let items: Vec<String> = ["Apple", "Banana", "Avocado"].map(String::from).into();
        assert_eq!(typeahead(&items, None, "a"), Some(0));
        assert_eq!(typeahead(&items, Some(0), "a"), Some(2));
        assert_eq!(typeahead(&items, Some(2), "A"), Some(0));
        assert_eq!(typeahead(&items, Some(0), "z"), None);
    }
}

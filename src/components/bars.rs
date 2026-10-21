//! A phone's bars, and a desktop's sidebar tree: a navigation bar along the
//! top (back, a title, actions), a tab bar along the bottom (a few places,
//! each an icon and a word), and a tree of rows that open and close.
//!
//! The bars keep clear of the window's safe area as the host sets it
//! ([`Unit::set_insets`](crate::Unit::set_insets)): the navigation bar
//! below the status bar, the tab bar above the home indicator, each with its
//! ground drawn under them.

use dioxus::html::{Key, NamedKey};
use dioxus::prelude::*;

use crate::Button;
use crate::components::button::{Variant, is_press};
use crate::icon::{Icon, Ink, paths};
use crate::unit::{Kind, use_focus, use_unit};

/// A bar along the top: a back button at the left, labelled by where it
/// goes back to, the title in the middle (a button with `ontitle`), and
/// `trailing` at the right — a page's own buttons.
#[component]
pub fn NavBar(
    title: String,
    back: Option<String>,
    onback: Option<EventHandler<()>>,
    ontitle: Option<EventHandler<()>>,
    leading: Option<Element>,
    trailing: Option<Element>,
    #[props(default)] class: String,
) -> Element {
    let unit = use_unit();
    let (safe, _) = unit.edges();
    rsx! {
        div { class: "unit-navbar {class}", role: "toolbar", "aria-label": "{title}",
            style: "padding-top: {safe.top}px; padding-left: {safe.left + 4.0}px; padding-right: {safe.right + 4.0}px",
            div { class: "unit-navbar-row",
                div { class: "unit-navbar-side",
                    if let Some(b) = back {
                        Button { variant: Variant::Ghost, class: "unit-back", icon: paths::CHEVRON_LEFT,
                                 label: "Back to {b}",
                                 onpress: move |_| if let Some(f) = onback { f.call(()) }, "{b}" }
                    }
                    {leading}
                }
                if let Some(f) = ontitle {
                    Button { variant: Variant::Ghost, class: "unit-navbar-title", onpress: move |_| f.call(()), "{title}" }
                } else {
                    div { class: "unit-navbar-title", role: "heading", "aria-level": "1", "{title}" }
                }
                div { class: "unit-navbar-side unit-navbar-end", {trailing} }
            }
        }
    }
}

/// One place a [`TabBar`] goes to: an icon over a word.
#[derive(Clone, Debug, PartialEq)]
pub struct BarItem {
    pub label: String,
    /// A path for [`Icon`].
    pub icon: &'static str,
}

impl BarItem {
    pub fn new(label: impl Into<String>, icon: &'static str) -> Self {
        Self {
            label: label.into(),
            icon,
        }
    }
}

/// A bar along the bottom of a few places, the `selected` one lit; read as
/// tabs. `onchange` is given the place pressed. The arrows move along it
/// while it has the keys.
#[component]
pub fn TabBar(
    items: Vec<BarItem>,
    selected: usize,
    onchange: EventHandler<usize>,
    label: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let unit = use_unit();
    let (safe, _) = unit.edges();
    let n = items.len();
    let focus = use_focus(Kind::Widget, move |key, _| {
        let to = match key {
            Key::Named(NamedKey::ArrowRight) => (selected + 1).min(n.saturating_sub(1)),
            Key::Named(NamedKey::ArrowLeft) => selected.saturating_sub(1),
            _ => return false,
        };
        if to != selected {
            onchange.call(to);
        }
        true
    });
    rsx! {
        div { class: "unit-tabbar{focus.class()} {class}", role: "tablist", "aria-label": label,
            style: "padding-bottom: {safe.bottom}px; padding-left: {safe.left}px; padding-right: {safe.right}px",
            tabindex: "0",
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            for (i, item) in items.into_iter().enumerate() {
                div { key: "{i}",
                    class: if i == selected { "unit-tabbar-item on" } else { "unit-tabbar-item" },
                    role: "tab",
                    "aria-selected": if i == selected { "true" } else { "false" },
                    "aria-label": "{item.label}",
                    onclick: move |e: MouseEvent| {
                        e.stop_propagation();
                        onchange.call(i);
                    },
                    Icon { d: item.icon, ink: if i == selected { Ink::Accent } else { Ink::Dim }, size: 24.0 }
                    span { class: "unit-tabbar-label", "{item.label}" }
                }
            }
        }
    }
}

/// A tree of rows: a sidebar's folders. Its rows are [`TreeRow`]s.
#[component]
pub fn Tree(label: Option<String>, #[props(default)] class: String, children: Element) -> Element {
    rsx! {
        div { class: "unit-tree {class}", role: "tree", "aria-label": label, {children} }
    }
}

/// One row of a [`Tree`], `depth` levels in. With `expanded` it has rows
/// under it, shown or not, and an arrow that opens and closes it
/// (`ontoggle`, given the new state); the page draws the rows under it when
/// it is open, after it. `onpress` is pressing the row itself.
///
/// `leading` is drawn before the label in place of an icon (a colour mark),
/// `detail` after it, dim (a count). `onmenu` is the row's menu: a
/// right-click, or the Menu key or Shift-F10 once the row has the keys.
#[component]
pub fn TreeRow(
    label: String,
    #[props(default)] depth: usize,
    expanded: Option<bool>,
    #[props(default)] selected: bool,
    icon: Option<&'static str>,
    leading: Option<Element>,
    detail: Option<String>,
    onpress: EventHandler<()>,
    ontoggle: Option<EventHandler<bool>>,
    onmenu: Option<EventHandler<()>>,
) -> Element {
    let toggle = move |to: bool| {
        if let Some(t) = ontoggle {
            t.call(to);
        }
    };
    let focus = use_focus(Kind::Widget, move |key, mods| match key {
        _ if is_press(key) => {
            onpress.call(());
            true
        }
        Key::Named(NamedKey::ContextMenu) | Key::Named(NamedKey::F10)
            if onmenu.is_some() && (*key == Key::Named(NamedKey::ContextMenu) || mods.shift()) =>
        {
            if let Some(m) = onmenu {
                m.call(());
            }
            true
        }
        Key::Named(NamedKey::ArrowRight) if expanded == Some(false) => {
            toggle(true);
            true
        }
        Key::Named(NamedKey::ArrowLeft) if expanded == Some(true) => {
            toggle(false);
            true
        }
        _ => false,
    });
    let indent = 6.0 + depth as f64 * 16.0;
    let class = format!(
        "unit-tree-row{}{}",
        if selected { " on" } else { "" },
        focus.class()
    );
    rsx! {
        div { class, role: "treeitem", tabindex: "0",
            style: "padding-left: {indent}px",
            "aria-level": "{depth + 1}",
            "aria-expanded": expanded.map(|e| e.to_string()),
            "aria-selected": if selected { "true" } else { "false" },
            "aria-label": "{label}",
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                e.stop_propagation();
                onpress.call(());
            },
            oncontextmenu: move |e: MouseEvent| {
                if let Some(m) = onmenu {
                    e.prevent_default();
                    e.stop_propagation();
                    m.call(());
                }
            },
            match expanded {
                Some(open) => rsx! {
                    span { class: "unit-tree-arrow", "aria-hidden": "true",
                        onclick: move |e: MouseEvent| {
                            e.stop_propagation();
                            toggle(!open);
                        },
                        Icon { d: if open { paths::CHEVRON_DOWN } else { paths::CHEVRON_RIGHT },
                               ink: if selected { Ink::AccentText } else { Ink::Dim }, size: 12.0 }
                    }
                },
                None => rsx! { span { class: "unit-tree-arrow" } },
            }
            if let Some(lead) = leading {
                span { class: "unit-tree-lead", "aria-hidden": "true", {lead} }
            } else if let Some(d) = icon {
                Icon { d, ink: if selected { Ink::AccentText } else { Ink::Accent }, size: 16.0 }
            }
            span { class: "unit-tree-label", "{label}" }
            if let Some(d) = detail {
                span { class: "unit-tree-detail", "{d}" }
            }
        }
    }
}

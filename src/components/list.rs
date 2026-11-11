//! Lists of rows in groups: a settings pane, a list of things to open, a
//! choice of one among a few, each row a line across the group's width.
//!
//! A [`Row`] that does something is pressed as a button is — by the
//! pointer, a finger, assistive tech, or Return and Space while it has the
//! keys — and read as a button; one that holds a control of its own (a
//! field, a switch) is not pressed itself, and leaves its presses to the
//! control. A row ticked or not is read as a radio button in a choice of
//! one, and as a checkbox where each is ticked on its own.

use dioxus::prelude::*;

use crate::components::button::{given, is_press};
use crate::icon::{Icon, Ink, paths};
use crate::layer::Tone;
use crate::unit::{Kind, use_focus};

/// How far a row is set in for each level of its `depth`, as a
/// [`TreeRow`](crate::TreeRow) is.
const INDENT: usize = 16;

/// What a row shows at its right edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Accessory {
    #[default]
    None,
    /// It opens something further: a chevron.
    Chevron,
    /// One of a choice of one: a tick when chosen, read as a radio button
    /// checked or not.
    Check(bool),
    /// Ticked or not on its own, whatever the rows round it are: a tick
    /// when on, read as a checkbox.
    Checkbox(bool),
}

/// Rows under a caption, on a raised surface: a settings pane's group. A
/// `footer` under it says more about them.
#[component]
pub fn Section(
    title: Option<String>,
    footer: Option<String>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    rsx! {
        div { class: "unit-section {class}", role: "group", "aria-label": title.clone(),
            if let Some(t) = &title {
                div { class: "unit-section-title", "aria-hidden": "true", "{t}" }
            }
            div { class: "unit-section-body", {children} }
            if let Some(f) = footer {
                div { class: "unit-section-footer", "{f}" }
            }
        }
    }
}

/// One row: an icon, a title and a line under it, text at the right (a
/// setting's value), and an [`Accessory`]. `depth` sets it in that many
/// levels, as a [`TreeRow`](crate::TreeRow) is: a nested list's rows, a
/// map's keys under the key that holds them. With `onpress` the row is
/// pressed. `trailing` is a control at the right in place of `detail`: in
/// a row that is not pressed, inside it; beside a row that is, so the
/// control is pressed on its own and not inside the row's button (a
/// recent library's Remove beside the library).
///
/// It takes any global attribute — `aria-label` in place of what it reads,
/// `id` — and puts it on the row; `oncontextmenu` is the secondary button's
/// press on it. Its own `title` is its words, so a tooltip is given as
/// `"title": "…"`.
#[component]
pub fn Row(
    title: String,
    subtitle: Option<String>,
    /// What the subtitle says of what the row is: drawn in that tone's
    /// colour, as a verdict's warning.
    #[props(default)]
    tone: Tone,
    /// A third line, under the subtitle: what a finding's fix would do, a
    /// remark's words under who sent it.
    note: Option<String>,
    /// Text at the right: what a setting is set to.
    detail: Option<String>,
    /// An icon at the left: a path for [`Icon`].
    icon: Option<&'static str>,
    /// Something of the page's own at the left, before the icon: a colour's
    /// dot, a picture.
    leading: Option<Element>,
    #[props(default)] accessory: Accessory,
    /// How many levels in it is set.
    #[props(default)]
    depth: usize,
    /// Destroys something: drawn in the danger colour.
    #[props(default)]
    destructive: bool,
    #[props(default)] disabled: bool,
    /// It opens and closes what is under it: read as expanded or not, its
    /// chevron pointing down while it is open.
    expanded: Option<bool>,
    onpress: Option<EventHandler<()>>,
    /// The secondary button: the page's own menu for the row.
    oncontextmenu: Option<EventHandler<MouseEvent>>,
    /// A control at the right: a field, a switch's track.
    trailing: Option<Element>,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let pressable = onpress.is_some();
    let press = move || {
        if !disabled && let Some(p) = onpress {
            p.call(());
        }
    };
    let focus = use_focus(Kind::Button, move |key, _| {
        if pressable && is_press(key) {
            press();
            return true;
        }
        false
    });
    let ink = match () {
        _ if disabled => Ink::Dim,
        _ if destructive => Ink::Danger,
        _ => Ink::Accent,
    };
    let (checked, role) = match accessory {
        Accessory::Check(on) => (Some(on), Some("radio")),
        Accessory::Checkbox(on) => (Some(on), Some("checkbox")),
        _ => (None, pressable.then_some("button")),
    };
    // Read as its title and what it is set to, as a settings row is.
    let (inside, beside) = match trailing {
        Some(t) if pressable => (None, Some(t)),
        t => (t, None),
    };
    let name = match &detail {
        Some(d) if inside.is_none() => format!("{title}, {d}"),
        _ => title.clone(),
    };
    let name = (!given(&attributes, "aria-label")).then_some(name);
    let state = format!(
        "{}{}{}{}",
        if pressable { " press" } else { "" },
        if destructive { " danger" } else { "" },
        if disabled { " off" } else { "" },
        focus.class(),
    );
    let row = rsx! {
        div {
            class: if beside.is_some() { "unit-row{state}" } else { "unit-row{state} {class}" },
            role,
            tabindex: (pressable && !disabled).then_some("0"),
            "aria-checked": checked.map(|c| c.to_string()),
            "aria-expanded": expanded.map(|e| e.to_string()),
            "aria-disabled": disabled.then_some("true"),
            "aria-label": name,
            onpointerdown: move |_| if pressable { focus.press() },
            onfocus: move |_| if pressable { focus.dom_focus() },
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                if pressable {
                    e.stop_propagation();
                    press();
                }
            },
            oncontextmenu: move |e| if let Some(h) = oncontextmenu { h.call(e) },
            ..attributes,
            // A box of its own rather than the row's padding, which a host's
            // stylesheet sets for every row.
            if depth > 0 {
                span { class: "unit-row-indent", style: "width: {depth * INDENT}px" }
            }
            {leading}
            if let Some(d) = icon {
                Icon { d, ink }
            }
            div { class: "unit-row-text",
                span { class: "unit-row-title", "{title}" }
                if let Some(s) = subtitle {
                    span { class: "unit-row-subtitle{tone.class()}", "{s}" }
                }
                if let Some(n) = note {
                    span { class: "unit-row-note", "{n}" }
                }
            }
            if let Some(t) = inside {
                div { class: "unit-row-trailing", {t} }
            } else if let Some(d) = detail {
                span { class: "unit-row-detail", "{d}" }
            }
            match accessory {
                Accessory::None => rsx! {},
                Accessory::Chevron => rsx! {
                    Icon { d: if expanded == Some(true) { paths::CHEVRON_DOWN } else { paths::CHEVRON_RIGHT },
                           ink: Ink::Dim, size: 14.0 }
                },
                Accessory::Check(true) | Accessory::Checkbox(true) => rsx! { Icon { d: paths::CHECK, ink: Ink::Accent, size: 16.0 } },
                Accessory::Check(false) | Accessory::Checkbox(false) => rsx! { span { class: "unit-row-blank" } },
            }
        }
    };
    match beside {
        Some(t) => rsx! {
            div { class: "unit-row-pair {class}",
                {row}
                div { class: "unit-row-beside", {t} }
            }
        },
        None => row,
    }
}

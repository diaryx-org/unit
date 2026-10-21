//! Lists of rows in groups: a settings pane, a list of things to open, a
//! choice of one among a few, each row a line across the group's width.
//!
//! A [`Row`] that does something is pressed as a button is — by the
//! pointer, a finger, assistive tech, or Return and Space while it has the
//! keys — and read as a button; one that holds a control of its own (a
//! field, a switch) is not pressed itself, and leaves its presses to the
//! control. A row ticked or not, in a choice of one, is read as a radio
//! button.

use dioxus::prelude::*;

use crate::components::button::is_press;
use crate::icon::{Icon, Ink, paths};
use crate::unit::{Kind, use_focus};

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
/// setting's value), and an [`Accessory`]. With `onpress` the row is
/// pressed; with `trailing` it holds a control at the right instead of
/// `detail`, and is not.
#[component]
pub fn Row(
    title: String,
    subtitle: Option<String>,
    /// Text at the right: what a setting is set to.
    detail: Option<String>,
    /// An icon at the left: a path for [`Icon`].
    icon: Option<&'static str>,
    #[props(default)] accessory: Accessory,
    /// Destroys something: drawn in the danger colour.
    #[props(default)]
    destructive: bool,
    #[props(default)] disabled: bool,
    onpress: Option<EventHandler<()>>,
    /// A control at the right: a field, a switch's track.
    trailing: Option<Element>,
    #[props(default)] class: String,
) -> Element {
    let pressable = onpress.is_some() && trailing.is_none();
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
    let checked = match accessory {
        Accessory::Check(on) => Some(on),
        _ => None,
    };
    let role = match (checked, pressable) {
        (Some(_), _) => Some("radio"),
        (None, true) => Some("button"),
        (None, false) => None,
    };
    // Read as its title and what it is set to, as a settings row is.
    let name = match &detail {
        Some(d) if trailing.is_none() => format!("{title}, {d}"),
        _ => title.clone(),
    };
    let state = format!(
        "{}{}{}{}",
        if pressable { " press" } else { "" },
        if destructive { " danger" } else { "" },
        if disabled { " off" } else { "" },
        focus.class(),
    );
    rsx! {
        div {
            class: "unit-row{state} {class}",
            role,
            tabindex: (pressable && !disabled).then_some("0"),
            "aria-checked": checked.map(|c| c.to_string()),
            "aria-disabled": disabled.then_some("true"),
            "aria-label": "{name}",
            onpointerdown: move |_| if pressable { focus.press() },
            onfocus: move |_| if pressable { focus.dom_focus() },
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                if pressable {
                    e.stop_propagation();
                    press();
                }
            },
            if let Some(d) = icon {
                Icon { d, ink }
            }
            div { class: "unit-row-text",
                span { class: "unit-row-title", "{title}" }
                if let Some(s) = subtitle {
                    span { class: "unit-row-subtitle", "{s}" }
                }
            }
            if let Some(t) = trailing {
                div { class: "unit-row-trailing", {t} }
            } else if let Some(d) = detail {
                span { class: "unit-row-detail", "{d}" }
            }
            match accessory {
                Accessory::None => rsx! {},
                Accessory::Chevron => rsx! { Icon { d: paths::CHEVRON_RIGHT, ink: Ink::Dim, size: 14.0 } },
                Accessory::Check(true) => rsx! { Icon { d: paths::CHECK, ink: Ink::Accent, size: 16.0 } },
                Accessory::Check(false) => rsx! { span { class: "unit-row-blank" } },
            }
        }
    }
}

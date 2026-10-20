//! Buttons: one that does something, and one that stays pressed.
//!
//! Each is a real `<button>`, so a browser and Blitz's accessibility tree
//! both read it as one. It acts on `click`, which both renderers send for a
//! mouse, a finger and assistive tech's press; Return and Space press it
//! through the page's keys where the renderer does not ([`Kind::Button`]).

use dioxus::html::{Key, NamedKey};
use dioxus::prelude::*;

use crate::unit::{Kind, use_focus};

/// How a button is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    /// A control's face and a border.
    #[default]
    Default,
    /// The accent: the one thing the page most expects.
    Primary,
    /// No face until the pointer is over it: in a toolbar, beside a message.
    Ghost,
    /// Destroys something.
    Danger,
}

impl Variant {
    fn class(self) -> &'static str {
        match self {
            Variant::Default => "",
            Variant::Primary => " unit-primary",
            Variant::Ghost => " unit-ghost",
            Variant::Danger => " unit-danger",
        }
    }
}

pub(crate) fn is_press(key: &Key) -> bool {
    matches!(key, Key::Named(NamedKey::Enter)) || matches!(key, Key::Character(c) if c == " ")
}

/// A button: `onpress` is called when it is clicked, tapped, pressed by
/// assistive tech, or pressed by Return or Space while it has the keys.
#[component]
pub fn Button(
    onpress: EventHandler<()>,
    #[props(default)] variant: Variant,
    #[props(default)] disabled: bool,
    /// What assistive tech reads, where the children are not words: an
    /// icon's button.
    label: Option<String>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let focus = use_focus(Kind::Button, move |key, _| {
        if is_press(key) && !disabled {
            onpress.call(());
            return true;
        }
        false
    });
    rsx! {
        button {
            class: "unit-button{variant.class()}{focus.class()} {class}",
            r#type: "button",
            disabled,
            "aria-label": label,
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                e.stop_propagation();
                if !disabled {
                    onpress.call(());
                }
            },
            {children}
        }
    }
}

/// A button that stays pressed until pressed again: Bold in a toolbar.
/// `onchange` is given its new state.
#[component]
pub fn ToggleButton(
    pressed: bool,
    onchange: EventHandler<bool>,
    #[props(default)] disabled: bool,
    label: Option<String>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let focus = use_focus(Kind::Button, move |key, _| {
        if is_press(key) && !disabled {
            onchange.call(!pressed);
            return true;
        }
        false
    });
    let on = if pressed { " on" } else { "" };
    rsx! {
        button {
            class: "unit-button unit-toggle{on}{focus.class()} {class}",
            r#type: "button",
            disabled,
            "aria-pressed": if pressed { "true" } else { "false" },
            "aria-label": label,
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                e.stop_propagation();
                if !disabled {
                    onchange.call(!pressed);
                }
            },
            {children}
        }
    }
}

//! Buttons: one that does something, and one that stays pressed.
//!
//! Each is a real `<button>`, so a browser and Blitz's accessibility tree
//! both read it as one. It acts on `click`, which both renderers send for a
//! mouse, a finger and assistive tech's press; Return and Space press it
//! through the page's keys where the renderer does not ([`Kind::Button`]).

use dioxus::html::{Key, NamedKey};
use dioxus::prelude::*;

use crate::icon::{Icon, Ink};
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
    /// The colour of an icon on a face of this variant.
    pub(crate) fn ink(self, disabled: bool) -> Ink {
        match self {
            _ if disabled => Ink::Dim,
            Variant::Primary | Variant::Danger => Ink::AccentText,
            Variant::Default | Variant::Ghost => Ink::Text,
        }
    }

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
    /// Takes no keys: a press leaves them where they were, and Tab passes
    /// it by, as a Mac's toolbar buttons do. For a bar over a host's own
    /// editor, whose keys a pressed Bold must not take.
    #[props(default)]
    refuses_keys: bool,
    /// What assistive tech reads, where the children are not words: an
    /// icon's button.
    label: Option<String>,
    /// An icon before the children: a path for [`Icon`]. An icon alone,
    /// with no children, is drawn square with `class: "unit-icon"`, and
    /// named by `label`.
    icon: Option<&'static str>,
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
            tabindex: refuses_keys.then_some("-1"),
            // A browser focuses a pressed button on `mousedown`, Blitz on
            // the click: cancelling both leaves the focus where it was, and
            // a negative `tabindex` keeps the button out of Tab's order.
            onmousedown: move |e: MouseEvent| if refuses_keys { e.prevent_default() },
            onpointerdown: move |_| if refuses_keys { focus.press_refused() } else { focus.press() },
            onfocus: move |_| if !refuses_keys { focus.dom_focus() },
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                e.stop_propagation();
                if refuses_keys {
                    // Blitz focuses what a click lands on as the click's
                    // default action.
                    e.prevent_default();
                }
                if !disabled {
                    onpress.call(());
                }
            },
            if let Some(d) = icon {
                Icon { d, ink: variant.ink(disabled) }
            }
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
    /// Takes no keys: a press leaves them where they were, and Tab passes
    /// it by, as a Mac's toolbar buttons do. For a bar over a host's own
    /// editor, whose keys a pressed Bold must not take.
    #[props(default)]
    refuses_keys: bool,
    label: Option<String>,
    /// An icon before the children: a path for [`Icon`]. An icon alone,
    /// with no children, is drawn square with `class: "unit-icon"`, and
    /// named by `label`.
    icon: Option<&'static str>,
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
            tabindex: refuses_keys.then_some("-1"),
            // A browser focuses a pressed button on `mousedown`, Blitz on
            // the click: cancelling both leaves the focus where it was, and
            // a negative `tabindex` keeps the button out of Tab's order.
            onmousedown: move |e: MouseEvent| if refuses_keys { e.prevent_default() },
            onpointerdown: move |_| if refuses_keys { focus.press_refused() } else { focus.press() },
            onfocus: move |_| if !refuses_keys { focus.dom_focus() },
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                e.stop_propagation();
                if refuses_keys {
                    // Blitz focuses what a click lands on as the click's
                    // default action.
                    e.prevent_default();
                }
                if !disabled {
                    onchange.call(!pressed);
                }
            },
            if let Some(d) = icon {
                Icon { d, ink: if disabled { Ink::Dim } else if pressed { Ink::Accent } else { Ink::Text } }
            }
            {children}
        }
    }
}

/// Something pressed that is drawn as the page's own and not as a button: a
/// sheet's row, a tile, a bar's item, a chip. It is a `div` with
/// `role="button"` (a `span` if `inline`) that Tab reaches, drawn with the
/// ring while it has the keys, and pressed by a click, a tap, assistive
/// tech's press, or Return or Space ([`Kind::Button`]). The ring shows
/// only when the keys came by a key ([`Focus::ring`](crate::Focus::ring)).
///
/// It takes any global attribute — `class`, `style`, `aria-label`,
/// `aria-expanded`, `title`, `role` for another role a press suits (`tab`,
/// `link`) — and puts it on its element. The ring is an attribute,
/// `data-unit-focused`, rather than a class, so the page's selectors for its
/// own classes go on matching as they did.
#[component]
pub fn Pressable(
    onpress: EventHandler<()>,
    /// Not pressed, and passed by: Tab goes past it.
    #[props(default)]
    disabled: bool,
    /// A `span`, for a press inside a line of text.
    #[props(default)]
    inline: bool,
    /// The secondary button: the page's own menu for it.
    oncontextmenu: Option<EventHandler<MouseEvent>>,
    /// The mouse held, moved and let go: a press held for a menu.
    onmousedown: Option<EventHandler<MouseEvent>>,
    onmousemove: Option<EventHandler<MouseEvent>>,
    onmouseup: Option<EventHandler<MouseEvent>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let focus = use_focus(Kind::Button, move |key, _| {
        if is_press(key) && !disabled {
            onpress.call(());
            return true;
        }
        false
    });
    // Only after a key: a page's own rows and tiles are pressed by the
    // mouse and a finger far more than they are reached by Tab.
    let ring = focus.ring().then_some("true");
    let tabindex = if disabled { "-1" } else { "0" };
    // A role the page gives replaces the button's: an element with two is
    // read by its first.
    let role = (!attributes.iter().any(|a| a.name == "role")).then_some("button");
    let disabled_attr = disabled.then_some("true");
    let onclick = move |e: MouseEvent| {
        e.stop_propagation();
        if !disabled {
            onpress.call(());
        }
    };
    if inline {
        rsx! {
            span {
                role,
                tabindex,
                "aria-disabled": disabled_attr,
                "data-unit-focused": ring,
                onpointerdown: move |_| if !disabled { focus.press() },
                onfocus: move |_| focus.dom_focus(),
                onblur: move |_| focus.dom_blur(),
                onclick: onclick,
                oncontextmenu: move |e| if let Some(h) = oncontextmenu { h.call(e) },
                onmousedown: move |e| if let Some(h) = onmousedown { h.call(e) },
                onmousemove: move |e| if let Some(h) = onmousemove { h.call(e) },
                onmouseup: move |e| if let Some(h) = onmouseup { h.call(e) },
                ..attributes,
                {children}
            }
        }
    } else {
        rsx! {
            div {
                role,
                tabindex,
                "aria-disabled": disabled_attr,
                "data-unit-focused": ring,
                onpointerdown: move |_| if !disabled { focus.press() },
                onfocus: move |_| focus.dom_focus(),
                onblur: move |_| focus.dom_blur(),
                onclick: onclick,
                oncontextmenu: move |e| if let Some(h) = oncontextmenu { h.call(e) },
                onmousedown: move |e| if let Some(h) = onmousedown { h.call(e) },
                onmousemove: move |e| if let Some(h) = onmousemove { h.call(e) },
                onmouseup: move |e| if let Some(h) = onmouseup { h.call(e) },
                ..attributes,
                {children}
            }
        }
    }
}

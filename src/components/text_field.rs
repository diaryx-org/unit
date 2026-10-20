//! Text fields: one line, and several.
//!
//! These are the renderer's own `<input>` and `<textarea>`: a browser and
//! Blitz each type in them with the system's input method and soft keyboard,
//! select, and read them out. While one has the keys, its keys are the
//! renderer's ([`Kind::Text`]): a host that reads keys itself asks
//! [`Unit::typing`](crate::Unit::typing) and hands them over.

use dioxus::prelude::*;

use crate::unit::{Kind, next_id, use_focus};

/// What a one-line field holds, which a phone's keyboard is chosen by.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputKind {
    #[default]
    Text,
    Search,
    Email,
    Url,
    Password,
    Number,
}

impl InputKind {
    fn attr(self) -> &'static str {
        match self {
            InputKind::Text => "text",
            InputKind::Search => "search",
            InputKind::Email => "email",
            InputKind::Url => "url",
            InputKind::Password => "password",
            InputKind::Number => "number",
        }
    }
}

/// One line of text. `oninput` is given the whole text after each change.
/// With a `label`, it is drawn above the field and read as its name.
#[component]
pub fn TextField(
    value: String,
    oninput: EventHandler<String>,
    label: Option<String>,
    #[props(default)] placeholder: String,
    #[props(default)] kind: InputKind,
    #[props(default)] disabled: bool,
    /// Return in the field: a search's go.
    onsubmit: Option<EventHandler<String>>,
) -> Element {
    let focus = use_focus(Kind::Text, |_, _| false);
    let id = use_hook(next_id);
    rsx! {
        div { class: "unit-text-field",
            if let Some(l) = &label {
                label { class: "unit-field-label", r#for: "unit-{id}", "{l}" }
            }
            input {
                class: "unit-input{focus.class()}",
                id: "unit-{id}",
                r#type: kind.attr(),
                value: "{value}",
                placeholder: "{placeholder}",
                disabled,
                "aria-label": label.clone(),
                onpointerdown: move |_| focus.press(),
                onfocus: move |_| focus.dom_focus(),
                onblur: move |_| focus.dom_blur(),
                oninput: move |e: FormEvent| oninput.call(e.value()),
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == dioxus::html::Key::Named(dioxus::html::NamedKey::Enter)
                        && let Some(s) = onsubmit
                    {
                        s.call(value.clone());
                    }
                },
            }
        }
    }
}

/// Several lines of text.
#[component]
pub fn TextArea(
    value: String,
    oninput: EventHandler<String>,
    label: Option<String>,
    #[props(default)] placeholder: String,
    #[props(default = 4)] rows: u32,
    #[props(default)] disabled: bool,
) -> Element {
    let focus = use_focus(Kind::Text, |_, _| false);
    let id = use_hook(next_id);
    rsx! {
        div { class: "unit-text-field",
            if let Some(l) = &label {
                label { class: "unit-field-label", r#for: "unit-{id}", "{l}" }
            }
            textarea {
                class: "unit-input unit-textarea{focus.class()}",
                id: "unit-{id}",
                rows: "{rows}",
                value: "{value}",
                placeholder: "{placeholder}",
                disabled,
                "aria-label": label.clone(),
                onpointerdown: move |_| focus.press(),
                onfocus: move |_| focus.dom_focus(),
                onblur: move |_| focus.dom_blur(),
                oninput: move |e: FormEvent| oninput.call(e.value()),
            }
        }
    }
}

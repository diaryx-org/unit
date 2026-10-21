//! Text fields: one line, and several.
//!
//! These are the renderer's own `<input>` and `<textarea>`: a browser and
//! Blitz each type in them with the system's input method and soft keyboard,
//! select, and read them out. While one has the keys, its keys are the
//! renderer's ([`Kind::Text`]): a host that reads keys itself asks
//! [`Unit::typing`](crate::Unit::typing) and hands them over.

use dioxus::prelude::*;

use crate::components::button::{Button, Variant};
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
///
/// With an `action`, a button of that name at the field's end does what the
/// field is for — Add, Save, Use — and Return in the field presses it too,
/// unless the field has an `onsubmit` of its own. `action_disabled` greys
/// it while the text would not do: an empty name, an address not yet
/// whole.
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
    /// Drawn with no box and its text at the right, its `label` read and
    /// not drawn: a field in a [`crate::Row`].
    #[props(default)]
    inline: bool,
    /// A button's name at the field's end: the field's action.
    action: Option<String>,
    /// The action pressed, or Return in the field, given the text.
    onaction: Option<EventHandler<String>>,
    #[props(default)] action_disabled: bool,
) -> Element {
    let focus = use_focus(Kind::Text, |_, _| false);
    let id = use_hook(next_id);
    let inline_class = if inline { " unit-inline" } else { "" };
    let act = (!disabled && !action_disabled)
        .then_some(onaction)
        .flatten();
    let submit = onsubmit.or(act);
    let input = rsx! {
        input {
            class: "unit-input{inline_class}{focus.class()}",
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
            onkeydown: {
                let value = value.clone();
                move |e: KeyboardEvent| {
                    if e.key() == dioxus::html::Key::Named(dioxus::html::NamedKey::Enter)
                        && let Some(s) = submit
                    {
                        s.call(value.clone());
                    }
                }
            },
        }
    };
    rsx! {
        div { class: "unit-text-field{inline_class}",
            if let Some(l) = label.as_ref().filter(|_| !inline) {
                label { class: "unit-field-label", r#for: "unit-{id}", "{l}" }
            }
            if let Some(a) = action {
                div { class: "unit-field-action",
                    {input}
                    Button { variant: Variant::Ghost, disabled: disabled || action_disabled,
                        onpress: move |_| if let Some(f) = onaction { f.call(value.clone()) },
                        "{a}"
                    }
                }
            } else {
                {input}
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

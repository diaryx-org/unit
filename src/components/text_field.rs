//! Text fields: one line, several, and a search.
//!
//! These are the renderer's own `<input>` and `<textarea>`: a browser and
//! Blitz each type in them with the system's input method and soft keyboard,
//! select, and read them out. While one has the keys, its keys are the
//! renderer's ([`Kind::Text`]): a host that reads keys itself asks
//! [`Unit::typing`](crate::Unit::typing) and hands them over.

use dioxus::prelude::*;

use crate::components::button::{Button, Variant};
use crate::icon::{Icon, Ink, paths};
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
///
/// `name`, `autocomplete`, `required`, `minlength` and `maxlength` are the
/// input's own attributes: a browser keeps to the length, and fills and
/// saves the field from a password manager by its `autocomplete`. Blitz does
/// neither yet; `required` is read by assistive tech on both.
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
    /// The field's name in a form.
    name: Option<String>,
    /// What a browser may fill it with: `email`, `current-password`.
    autocomplete: Option<String>,
    #[props(default)] required: bool,
    minlength: Option<u32>,
    maxlength: Option<u32>,
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
            name,
            autocomplete,
            required,
            "aria-required": required.then_some("true"),
            minlength: minlength.map(|n| n.to_string()),
            maxlength: maxlength.map(|n| n.to_string()),
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
    /// The field's name in a form.
    name: Option<String>,
    #[props(default)] required: bool,
    maxlength: Option<u32>,
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
                name,
                required,
                "aria-required": required.then_some("true"),
                maxlength: maxlength.map(|n| n.to_string()),
                "aria-label": label.clone(),
                onpointerdown: move |_| focus.press(),
                onfocus: move |_| focus.dom_focus(),
                onblur: move |_| focus.dom_blur(),
                oninput: move |e: FormEvent| oninput.call(e.value()),
                // Its text as its content too: a `<textarea>` starts from
                // its content, in a browser's HTML and in Blitz's editor,
                // which takes no `value` until it is made. `value` keeps
                // it in step after that.
                "{value}"
            }
        }
    }
}

/// A search: a field with a magnifier at its start and, while it holds
/// text, a button at its end that empties it, as Escape in it does.
/// `oninput` is given the whole text after each change, as a filter wants;
/// `onsubmit` is Return, for a search that runs only when asked.
#[component]
pub fn SearchField(
    value: String,
    oninput: EventHandler<String>,
    /// What assistive tech calls it.
    #[props(default = "Search".to_string())]
    label: String,
    #[props(default = "Search".to_string())] placeholder: String,
    onsubmit: Option<EventHandler<String>>,
    #[props(default)] class: String,
) -> Element {
    let focus = use_focus(Kind::Text, |_, _| false);
    let empty = value.is_empty();
    rsx! {
        div { class: "unit-search {class}", role: "search",
            span { class: "unit-search-icon", "aria-hidden": "true",
                Icon { d: paths::SEARCH, ink: Ink::Dim, size: 14.0 }
            }
            input {
                class: "unit-input{focus.class()}",
                r#type: "search",
                value: "{value}",
                placeholder: "{placeholder}",
                "aria-label": "{label}",
                onpointerdown: move |_| focus.press(),
                onfocus: move |_| focus.dom_focus(),
                onblur: move |_| focus.dom_blur(),
                oninput: move |e: FormEvent| oninput.call(e.value()),
                onkeydown: {
                    let value = value.clone();
                    move |e: KeyboardEvent| match e.key() {
                        dioxus::html::Key::Named(dioxus::html::NamedKey::Enter) => {
                            if let Some(s) = onsubmit {
                                s.call(value.clone());
                            }
                        }
                        // Emptied first; only an empty field lets Escape
                        // go on, to close what it is in.
                        dioxus::html::Key::Named(dioxus::html::NamedKey::Escape) if !value.is_empty() => {
                            e.stop_propagation();
                            oninput.call(String::new());
                        }
                        _ => {}
                    }
                },
            }
            if !empty {
                Button { variant: Variant::Ghost, class: "unit-icon", label: "Clear {label}",
                         icon: paths::CLOSE, onpress: move |_| oninput.call(String::new()) }
            }
        }
    }
}

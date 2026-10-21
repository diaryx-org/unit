//! A one-time code: the digits a sign-in sends by email or text message.
//!
//! One real `<input>`, drawn over by a cell for each digit. Assistive tech
//! reads one text field with the whole code in it, not six; a paste, the
//! system's AutoFill and a phone's QuickType bar each put the whole code in
//! at once, and the field keeps only its digits. The cells are a drawing of
//! the input's text and are hidden from assistive tech.
//!
//! On the web the input says what it is for: `autocomplete="one-time-code"`
//! and `inputmode="numeric"`, which Safari and Chrome offer a code from
//! Messages or Mail by, and a phone's keyboard of digits. Blitz reads
//! neither, nor `maxlength`; the field cuts its text to its length itself.

use dioxus::prelude::*;

use crate::unit::{Kind, next_id, use_focus};

/// The code's digits in `text`, at most `length` of them: a pasted
/// "Your code is 123 456." is `123456`.
fn digits(text: &str, length: usize) -> String {
    text.chars()
        .filter(char::is_ascii_digit)
        .take(length)
        .collect()
}

/// A one-time code of `length` digits, drawn as a cell for each.
/// `oninput` is given the digits after each change, and `oncomplete` the
/// code when its last digit is typed or pasted, or Return is pressed on a
/// whole one. With a `label`, it is drawn above the cells and read as the
/// field's name; without one, the field is read as "Code".
#[component]
pub fn CodeField(
    value: String,
    oninput: EventHandler<String>,
    oncomplete: Option<EventHandler<String>>,
    label: Option<String>,
    #[props(default = 6)] length: usize,
    #[props(default)] disabled: bool,
    /// The code was refused: its cells drawn in the danger colour, and the
    /// field read as invalid.
    #[props(default)]
    invalid: bool,
) -> Element {
    let focus = use_focus(Kind::Text, |_, _| false);
    let id = use_hook(next_id);
    // What was typed, while it is not what the field keeps: drawn for one
    // frame, so the next draws the kept code over it. An input is only
    // written when its value changes, and a seventh digit leaves the code
    // as it was.
    let mut typed = use_signal(|| None::<String>);
    use_effect(move || {
        if typed().is_some() {
            typed.set(None);
        }
    });
    let code = digits(&value, length);
    let shown = typed().unwrap_or_else(|| code.clone());
    let focused = focus.is_focused() && !disabled;
    let at = code.len().min(length.saturating_sub(1));
    let off = if disabled { " off" } else { "" };
    let bad = if invalid { " invalid" } else { "" };
    let cells = (0..length).map(|i| {
        let digit = code.chars().nth(i);
        let on = if focused && i == at { " on" } else { "" };
        let filled = if digit.is_some() { " filled" } else { "" };
        let caret = focused && i == code.len();
        rsx! {
            div { key: "{i}", class: "unit-code-cell{on}{filled}",
                if let Some(d) = digit {
                    "{d}"
                } else if caret {
                    span { class: "unit-caret" }
                }
            }
        }
    });
    rsx! {
        div { class: "unit-text-field unit-code-field",
            if let Some(l) = &label {
                label { class: "unit-field-label", r#for: "unit-{id}", "{l}" }
            }
            div { class: "unit-code{off}{bad}",
                div { class: "unit-code-cells", "aria-hidden": "true", {cells} }
                input {
                    class: "unit-code-input",
                    id: "unit-{id}",
                    r#type: "text",
                    value: "{shown}",
                    disabled,
                    "inputmode": "numeric",
                    "autocomplete": "one-time-code",
                    "maxlength": "{length}",
                    "spellcheck": "false",
                    "aria-label": label.clone().unwrap_or_else(|| "Code".into()),
                    "aria-invalid": invalid.then_some("true"),
                    onpointerdown: move |_| focus.press(),
                    onfocus: move |_| focus.dom_focus(),
                    onblur: move |_| focus.dom_blur(),
                    oninput: {
                        let code = code.clone();
                        move |e: FormEvent| {
                            let raw = e.value();
                            let next = digits(&raw, length);
                            if next != raw {
                                typed.set(Some(raw));
                            }
                            if next == code {
                                return;
                            }
                            oninput.call(next.clone());
                            if next.len() == length
                                && let Some(f) = oncomplete
                            {
                                f.call(next);
                            }
                        }
                    },
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == dioxus::html::Key::Named(dioxus::html::NamedKey::Enter)
                            && code.len() == length
                            && let Some(f) = oncomplete
                        {
                            f.call(code.clone());
                        }
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::digits;

    #[test]
    fn a_code_keeps_its_digits_and_no_more_than_its_length() {
        assert_eq!(digits("123456", 6), "123456");
        assert_eq!(digits("Your code is 123 456.", 6), "123456");
        assert_eq!(digits("123-456-789", 6), "123456");
        assert_eq!(digits("12a", 6), "12");
        assert_eq!(digits("", 6), "");
        assert_eq!(digits("٣٤٥", 6), "", "only ASCII digits");
    }
}

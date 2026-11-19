//! A button that chooses files: the system's picker, by a press or the
//! keys, in a browser and on Blitz alike.
//!
//! It is the renderer's own `<input type=file>`, unseen, laid over a
//! button's face: a press, a tap, Tab, Return and Space all reach the
//! input, and the renderer opens the system's picker for it (a browser's,
//! or on Blitz the platform's, from diaryx-org/blitz's `leaf`) and reads
//! the files back. No JavaScript, no picker of unit's own.

use dioxus::html::FileData;
use dioxus::prelude::*;

use crate::components::button::Variant;
use crate::icon::Icon;
use crate::unit::{Kind, use_focus};

/// A button that opens the system's file picker, and gives the files
/// chosen to `onchoose`: a database to open, a picture to insert.
///
/// - `accept`: the types it takes, as an input's `accept` says them
///   (`.kdbx`, `image/*`, `application/pdf`); any file without.
/// - `multiple`: several at once; `directory`: a folder, its files every
///   one under it.
/// - `oncancel`: the picker was dismissed with nothing chosen.
///
/// Each file is Dioxus's `FileData`: its name, size and type, and its
/// contents read when asked (`read_bytes`). On Blitz a file chosen from
/// disk has its path (`path()`); one a browser or a phone handed over has
/// only its name there.
#[component]
pub fn FileButton(
    /// What the button says, and what assistive tech calls it.
    label: String,
    onchoose: EventHandler<Vec<FileData>>,
    oncancel: Option<EventHandler<()>>,
    accept: Option<String>,
    #[props(default)] multiple: bool,
    #[props(default)] directory: bool,
    #[props(default)] variant: Variant,
    #[props(default)] disabled: bool,
    /// An icon before the label: a path for [`Icon`].
    icon: Option<&'static str>,
    #[props(default)] class: String,
) -> Element {
    // The input takes its own keys, as a text field does: Return and Space
    // open its picker, the renderer's default for a file input.
    let focus = use_focus(Kind::Text, |_, _| false);
    let dim = if disabled { " unit-face-disabled" } else { "" };
    rsx! {
        span { class: "unit-file-button {class}",
            // The face: drawn as a button is, but not one, so that neither
            // presses, Tab nor assistive tech find it; the input over it is
            // the control.
            span {
                class: "unit-button{variant.class()}{focus.class()}{dim}",
                "aria-hidden": "true",
                if let Some(d) = icon {
                    Icon { d, ink: variant.ink(disabled) }
                }
                "{label}"
            }
            input {
                class: "unit-file-input",
                r#type: "file",
                accept,
                multiple,
                "webkitdirectory": directory.then_some("true"),
                disabled,
                "aria-label": "{label}",
                onpointerdown: move |_| focus.press(),
                onfocus: move |_| focus.dom_focus(),
                onblur: move |_| focus.dom_blur(),
                onchange: move |e: FormEvent| {
                    let files = e.files();
                    if !files.is_empty() {
                        onchoose.call(files);
                    }
                },
                oncancel: move |_| {
                    if let Some(h) = oncancel {
                        h.call(());
                    }
                },
            }
        }
    }
}

//! A modal with content of the page's own — a form, a settings pane — over
//! the page, centred or as a sheet from the bottom.
//!
//! It is drawn where the page puts it, not in the overlay, so its content
//! keeps its hooks and its place in the page's tree; and it is in the tree
//! from the first frame, shown and hidden, for the reason the overlay is
//! (`layer.rs`). Escape closes the topmost one open, through the page's
//! keys.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::unit::{next_id, use_unit};

/// Where a modal sits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Placement {
    /// In the middle of the page.
    #[default]
    Center,
    /// Along the bottom, full width: a phone's sheet.
    Sheet,
}

/// A modal over the page while `open`. `onclose` is called when it asks to
/// close — Escape, a press on the scrim, its close button — and the page
/// sets `open` false.
#[component]
pub fn Modal(
    open: bool,
    onclose: EventHandler<()>,
    title: String,
    #[props(default)] placement: Placement,
    /// Its width, in CSS pixels, where the page has room.
    #[props(default = 440.0)]
    width: f64,
    /// Buttons along its bottom.
    actions: Option<Element>,
    children: Element,
) -> Element {
    let unit = use_unit();
    let id = use_hook(next_id);
    // Escape closes the topmost modal open: each says so as it opens and
    // closes, and the last to open is first.
    let close: Rc<dyn Fn()> = Rc::new(move || onclose.call(()));
    unit.modal(id, open.then_some(close));
    use_drop(move || unit.modal(id, None));

    let display = if open { "flex" } else { "none" };
    let place = match placement {
        Placement::Center => "unit-modal-place",
        Placement::Sheet => "unit-modal-place unit-sheet-place",
    };
    let panel = match placement {
        Placement::Center => "unit-modal",
        Placement::Sheet => "unit-modal unit-sheet",
    };
    let style = match placement {
        Placement::Center => format!("width: {width}px"),
        Placement::Sheet => String::new(),
    };
    rsx! {
        div { class: "{place}", style: "display: {display}",
            onpointerdown: move |e: PointerEvent| {
                e.stop_propagation();
                onclose.call(());
            },
            div {
                class: "{panel}",
                style,
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "unit-{id}",
                onpointerdown: move |e: PointerEvent| {
                    e.stop_propagation();
                    unit.pressed_root();
                },
                div { class: "unit-modal-head",
                    div { class: "unit-dialog-title", id: "unit-{id}", "{title}" }
                    button { class: "unit-button unit-ghost unit-icon", r#type: "button",
                        "aria-label": "Close",
                        onclick: move |e: MouseEvent| {
                            e.stop_propagation();
                            onclose.call(());
                        },
                        "✕"
                    }
                }
                div { class: "unit-modal-body", {children} }
                if let Some(a) = actions {
                    div { class: "unit-dialog-buttons", {a} }
                }
            }
        }
    }
}

//! A modal with content of the page's own — a form, a settings pane — over
//! the page, centred or as a sheet from the bottom.
//!
//! It is drawn where the page puts it, not in the overlay, so its content
//! keeps its hooks and its place in the page's tree. It can be in the tree
//! from the first frame, shown and hidden, or put there only while it is
//! open (an app that mounts each sheet on demand): the gallery's tests lay
//! out both under Blitz. Escape closes the topmost one open, through the
//! page's keys.
//!
//! It keeps clear of the window's safe area and the soft keyboard, as the
//! host sets them ([`Unit::set_insets`](crate::Unit::set_insets),
//! [`Unit::set_keyboard`](crate::Unit::set_keyboard)): a sheet sits on the
//! keyboard's top edge while it is up. A sheet is dragged down by its
//! grabber or its title bar to close, as a phone's is: the pointer's moves
//! are followed wherever they go on the page, since the scrim under the
//! sheet covers it all.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::components::button::Variant;
use crate::unit::{next_id, use_unit};
use crate::{Button, Icon, Ink, icon::paths};

/// Where a modal sits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Placement {
    /// In the middle of the page.
    #[default]
    Center,
    /// Along the bottom, full width: a phone's sheet.
    Sheet,
}

/// How far a sheet is dragged down before letting go closes it, in CSS
/// pixels.
pub const DISMISS: f64 = 90.0;

/// A modal over the page while `open`. `onclose` is called when it asks to
/// close — Escape, a press on the scrim, its close button or Cancel, a
/// sheet dragged down — and the page sets `open` false.
///
/// With `cancel` or `done` its title bar is a phone's: Cancel at the left,
/// the title in the middle, Done at the right, which calls `ondone`. With
/// neither, a close button at the right, unless `close_button` is false:
/// while something it started runs, and is not to be closed by a press.
///
/// A sheet with an `inset` floats that far off the window's sides and
/// bottom, a card with every corner rounded, rather than along the bottom
/// edge; `clear` is what it keeps clear of under it while the keyboard is
/// down, a host's own bar, and `max_height` how much of the height it may
/// take, as a fraction.
#[component]
pub fn Modal(
    open: bool,
    onclose: EventHandler<()>,
    title: String,
    #[props(default)] placement: Placement,
    /// Its width, in CSS pixels, where the page has room.
    #[props(default = 440.0)]
    width: f64,
    /// The label of a button at the title bar's left that closes it.
    cancel: Option<String>,
    /// The label of a button at the title bar's right that finishes.
    done: Option<String>,
    ondone: Option<EventHandler<()>>,
    /// Whether a sheet closes when dragged down.
    #[props(default = true)]
    draggable: bool,
    /// Whether its title bar has a close button, with no `cancel` or `done`.
    #[props(default = true)]
    close_button: bool,
    /// How far a sheet floats off the window's sides and bottom, in CSS
    /// pixels: 0 along the bottom edge.
    #[props(default)]
    inset: f64,
    /// What a floating sheet keeps clear of below it while the keyboard is
    /// down, in CSS pixels: a host's bar along the bottom.
    #[props(default)]
    clear: f64,
    /// How much of the height it may take, as a fraction.
    #[props(default = 1.0)]
    max_height: f64,
    /// Buttons along its bottom.
    actions: Option<Element>,
    /// A class of the page's own on the layer it sits in, which covers the
    /// page: for the page's styles of what is inside it.
    #[props(default)]
    class: String,
    children: Element,
) -> Element {
    let unit = use_unit();
    let id = use_hook(next_id);
    // Escape closes the topmost modal open: each says so as it opens and
    // closes, and the last to open is first.
    let close: Rc<dyn Fn()> = Rc::new(move || onclose.call(()));
    unit.modal(id, open.then_some(close));
    use_drop(move || unit.modal(id, None));
    // Where a drag of the sheet began, and how far down it has come.
    let mut drag = use_signal(|| None::<(f64, f64)>);
    if !open && drag.peek().is_some() {
        drag.set(None);
    }

    let sheet = placement == Placement::Sheet;
    let floats = sheet && inset > 0.0;
    let drags = sheet && draggable;
    let (safe, keyboard) = unit.edges();
    let display = if open { "flex" } else { "none" };
    let (place, panel) = match (sheet, floats) {
        (true, true) => (
            "unit-modal-place unit-sheet-place",
            "unit-modal unit-sheet unit-floating",
        ),
        (true, false) => ("unit-modal-place unit-sheet-place", "unit-modal unit-sheet"),
        _ => ("unit-modal-place", "unit-modal"),
    };
    let tall = format!("max-height: {}%", max_height * 100.0);
    // The place keeps the panel off the status bar, and a sheet on the
    // keyboard's top edge; a sheet's own padding keeps its content off the
    // home indicator, which the keyboard covers while it is up.
    // A floating sheet is held off the edges by its place, and off the
    // home indicator and the host's bar, or the keyboard while it is up.
    let place_style = if floats {
        let below = if keyboard > 0.0 {
            keyboard
        } else {
            safe.bottom.max(clear)
        };
        format!(
            "display: {display}; padding: {}px {}px {}px {}px",
            safe.top + 16.0,
            safe.right + inset,
            below + inset,
            safe.left + inset
        )
    } else if sheet {
        format!(
            "display: {display}; padding-top: {}px; padding-bottom: {keyboard}px",
            safe.top + 16.0
        )
    } else {
        format!(
            "display: {display}; padding: {}px {}px {}px {}px",
            safe.top + 16.0,
            safe.right + 16.0,
            safe.bottom.max(keyboard) + 16.0,
            safe.left + 16.0
        )
    };
    let dy = drag().map_or(0.0, |(_, dy)| dy);
    let panel_style = if floats {
        format!("{tall}; transform: translateY({dy}px)")
    } else if sheet {
        let bottom = if keyboard > 0.0 { 0.0 } else { safe.bottom };
        format!(
            "{tall}; padding-bottom: {}px; padding-left: {}px; padding-right: {}px; transform: translateY({dy}px)",
            bottom + 16.0,
            safe.left + 16.0,
            safe.right + 16.0
        )
    } else {
        format!("{tall}; width: {width}px")
    };
    let phone_bar = cancel.is_some() || done.is_some();
    let start = move |e: PointerEvent| {
        if drags {
            drag.clone().set(Some((e.client_coordinates().y, 0.0)));
        }
    };
    rsx! {
        div { class: "{place} {class}", style: "{place_style}",
            onpointerdown: move |e: PointerEvent| {
                e.stop_propagation();
                onclose.call(());
            },
            onpointermove: move |e: PointerEvent| {
                let began = *drag.peek();
                if let Some((y0, _)) = began {
                    let dy = (e.client_coordinates().y - y0).max(0.0);
                    drag.set(Some((y0, dy)));
                }
            },
            onpointerup: move |e: PointerEvent| {
                if let Some((_, dy)) = drag.take() {
                    e.stop_propagation();
                    if dy > DISMISS {
                        onclose.call(());
                    }
                }
            },
            onpointercancel: move |_| drag.set(None),
            div {
                class: "{panel}",
                style: "{panel_style}",
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "unit-{id}",
                onpointerdown: move |e: PointerEvent| {
                    e.stop_propagation();
                    unit.pressed_root();
                },
                if sheet {
                    div { class: "unit-sheet-grabber", "aria-hidden": "true",
                        onpointerdown: move |e: PointerEvent| {
                            e.stop_propagation();
                            unit.pressed_root();
                            start(e);
                        },
                        div { class: "unit-sheet-grabber-bar" }
                    }
                }
                div {
                    class: if phone_bar { "unit-modal-head unit-phone-bar" } else { "unit-modal-head" },
                    onpointerdown: move |e: PointerEvent| {
                        e.stop_propagation();
                        unit.pressed_root();
                        start(e);
                    },
                    if phone_bar {
                        div { class: "unit-modal-side",
                            if let Some(c) = cancel {
                                Button { variant: Variant::Ghost, onpress: move |_| onclose.call(()), "{c}" }
                            }
                        }
                        div { class: "unit-dialog-title", id: "unit-{id}", "{title}" }
                        div { class: "unit-modal-side unit-modal-end",
                            if let Some(d) = done {
                                Button { variant: Variant::Ghost, class: "unit-done",
                                         onpress: move |_| if let Some(f) = ondone { f.call(()) }, "{d}" }
                            }
                        }
                    } else {
                        div { class: "unit-dialog-title", id: "unit-{id}", "{title}" }
                        if close_button { button { class: "unit-button unit-ghost unit-icon", r#type: "button",
                            "aria-label": "Close",
                            onpointerdown: move |e: PointerEvent| e.stop_propagation(),
                            onclick: move |e: MouseEvent| {
                                e.stop_propagation();
                                onclose.call(());
                            },
                            Icon { d: paths::CLOSE, ink: Ink::Dim, size: 14.0 }
                        } }
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

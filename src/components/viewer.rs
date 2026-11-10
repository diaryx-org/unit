//! A photo viewer: one photo at a time over the whole window, on black, as
//! a phone's and a desktop's photo apps show one.
//!
//! It goes from photo to photo by the arrows, by the buttons at its sides,
//! or by a swipe, and closes by Escape or its close button. While it is
//! open it has the keys, and gives them back to what had them when it
//! closes. It is in the page only while it is open, as nothing of it is
//! drawn otherwise.
//!
//! It is `position: fixed`, which Blitz places against the top of the
//! document rather than the window: on a page whose root scrolls it opens
//! off screen once the page is scrolled (`docs/tasks/overlays-on-a-scrolled-page.md`),
//! as a `Modal` does. A page that scrolls in a box of its own does not
//! meet it.
//!
//! It is drawn on black whatever the page's appearance, and sets the
//! theme's text, dim and hover colours for what is in it to match.

use std::rc::Rc;

use dioxus::html::{Key, NamedKey};
use dioxus::prelude::*;

use crate::components::button::{Button, Variant};
use crate::icon::paths;
use crate::unit::{Kind, next_id, use_focus, use_unit};

/// How far a swipe goes, in CSS pixels, before it is a swipe and not a
/// tap.
pub const SWIPE: f64 = 50.0;

/// One photo a [`PhotoViewer`] shows: its address, the words that say what
/// is in it, read by assistive tech, and a caption drawn under it.
#[derive(Clone, Debug, PartialEq)]
pub struct Photo {
    pub src: String,
    pub alt: String,
    pub caption: Option<String>,
}

impl Photo {
    pub fn new(src: impl Into<String>, alt: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            alt: alt.into(),
            caption: None,
        }
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }
}

/// `photos[index]` over the whole window while `open`. `onchange` is given
/// the photo to go to, and the page sets `index`; `onclose` is called when
/// it asks to close, and the page sets `open` false.
#[component]
pub fn PhotoViewer(
    open: bool,
    photos: Vec<Photo>,
    index: usize,
    onchange: EventHandler<usize>,
    onclose: EventHandler<()>,
    #[props(default)] class: String,
) -> Element {
    let unit = use_unit();
    let id = use_hook(next_id);
    let n = photos.len();
    let index = index.min(n.saturating_sub(1));
    let go = move |to: usize| {
        if to != index && to < n {
            onchange.call(to);
        }
    };
    // Escape closes it, as it does a modal.
    let close: Rc<dyn Fn()> = Rc::new(move || onclose.call(()));
    unit.modal(id, open.then_some(close));
    use_drop(move || unit.modal(id, None));

    let focus = use_focus(Kind::Widget, move |key, _| {
        let to = match key {
            Key::Named(NamedKey::ArrowLeft) => index.saturating_sub(1),
            Key::Named(NamedKey::ArrowRight) => index + 1,
            Key::Named(NamedKey::Home) => 0,
            Key::Named(NamedKey::End) => n.saturating_sub(1),
            _ => return false,
        };
        go(to);
        true
    });
    // The keys while it is open, and back where they were after.
    let before = use_hook(|| CopyValue::new(None));
    use_effect(use_reactive!(|open| {
        if open {
            before.clone().set(unit.current());
            focus.take();
        } else if unit.focused() == Some(focus.id) {
            unit.restore(*before.peek());
        }
    }));
    // Where a swipe began.
    let mut swipe = use_signal(|| None::<f64>);

    let Some(photo) = photos.get(index).filter(|_| open).cloned() else {
        return rsx! {};
    };
    let (safe, _) = unit.edges();
    let count = format!("{} of {n}", index + 1);
    rsx! {
        div {
            class: "unit-viewer {class}",
            role: "dialog",
            "aria-modal": "true",
            "aria-label": "{photo.alt}, {count}",
            style: "padding: {safe.top}px {safe.right}px {safe.bottom}px {safe.left}px",
            onpointerdown: move |e: PointerEvent| {
                e.stop_propagation();
                focus.press();
            },
            div { class: "unit-viewer-bar",
                span { class: "unit-viewer-count", "{count}" }
                Button { variant: Variant::Ghost, class: "unit-icon", label: "Close", icon: paths::CLOSE,
                         refuses_keys: true, onpress: move |_| onclose.call(()) }
            }
            div { class: "unit-viewer-stage",
                onpointerdown: move |e: PointerEvent| swipe.set(Some(e.client_coordinates().x)),
                onpointerup: move |e: PointerEvent| {
                    if let Some(x0) = swipe.take() {
                        let dx = e.client_coordinates().x - x0;
                        if dx <= -SWIPE {
                            go(index + 1);
                        } else if dx >= SWIPE {
                            go(index.saturating_sub(1));
                        }
                    }
                },
                onpointercancel: move |_| swipe.set(None),
                // `alt` for a browser; Blitz names an image by its label.
                img { class: "unit-viewer-image", src: "{photo.src}", alt: "{photo.alt}",
                      "aria-label": "{photo.alt}" }
                if index > 0 {
                    Button { variant: Variant::Ghost, class: "unit-icon unit-viewer-prev", label: "Previous photo",
                             icon: paths::CHEVRON_LEFT, refuses_keys: true, onpress: move |_| go(index - 1) }
                }
                if index + 1 < n {
                    Button { variant: Variant::Ghost, class: "unit-icon unit-viewer-next", label: "Next photo",
                             icon: paths::CHEVRON_RIGHT, refuses_keys: true, onpress: move |_| go(index + 1) }
                }
            }
            if let Some(c) = &photo.caption {
                div { class: "unit-viewer-caption", "{c}" }
            }
        }
    }
}

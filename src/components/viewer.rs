//! A photo viewer: one photo at a time over the whole window, on black, as
//! a phone's and a desktop's photo apps show one.
//!
//! It goes from photo to photo by the arrows, by the buttons at its sides,
//! or by a swipe, and closes by Escape or its close button. A photo zooms
//! as a phone's does: a pinch, a double tap, the wheel ([`crate::zoom`]). While it is
//! open it has the keys, and gives them back to what had them when it
//! closes. It is in the page only while it is open, as nothing of it is
//! drawn otherwise.
//!
//! It is drawn on black whatever the page's appearance, and sets the
//! theme's text, dim and hover colours for what is in it to match.

use std::rc::Rc;
use std::time::Duration;

use dioxus::html::{Key, NamedKey};
use dioxus::prelude::*;

use crate::components::button::{Button, Variant};
use crate::icon::paths;
use crate::scroll::{Tracker, wheel_delta, zoom_factor};
use crate::unit::{Kind, Unit, next_id, use_focus, use_unit};
use crate::zoom::{MOST, Zoom};

/// How far a swipe goes, in CSS pixels, before it is a swipe and not a
/// tap.
pub const SWIPE: f64 = 50.0;

/// How far a pointer may move and its lifting still be a tap, in CSS
/// pixels; how soon a second tap follows the first to make a double tap,
/// in milliseconds, and how near it lands.
const TAP: f64 = 10.0;
const DOUBLE_TAP_TIME: f64 = 300.0;
const DOUBLE_TAP_NEAR: f64 = 30.0;

/// One photo a [`PhotoViewer`] shows: its address, the words that say what
/// is in it, read by assistive tech, and a caption drawn under it. `size`
/// is its own width and height, in any unit: its shape, which the viewer
/// needs to know where its edges are when it is zoomed. With none it is
/// taken to be the window's shape.
#[derive(Clone, Debug, PartialEq)]
pub struct Photo {
    pub src: String,
    pub alt: String,
    pub caption: Option<String>,
    pub size: Option<(f64, f64)>,
}

impl Photo {
    pub fn new(src: impl Into<String>, alt: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            alt: alt.into(),
            caption: None,
            size: None,
        }
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn size(mut self, width: f64, height: f64) -> Self {
        self.size = Some((width, height));
        self
    }
}

/// Milliseconds since the first call, for a finger's speed and a frame's
/// length; `None` in a browser, which unit reads no clock in.
fn now_ms() -> Option<f64> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let start = START.get_or_init(std::time::Instant::now);
        Some(start.elapsed().as_secs_f64() * 1000.0)
    }
    #[cfg(target_arch = "wasm32")]
    None
}

/// The pointers on the photo, and what a single one is doing.
#[derive(Default)]
struct Touches {
    /// Each pointer down, by id, and where it is on the stage.
    down: Vec<(i32, (f64, f64))>,
    /// Where a single pointer went down on a photo that fits: a swipe to
    /// the next photo, if it goes far enough.
    swipe: Option<(f64, f64)>,
    /// The last pointer's recent places, across and down, for its speed.
    track: (Tracker, Tracker),
    /// Where a single pointer went down, while it may yet be a tap.
    start: Option<(f64, f64)>,
    /// When and where the last tap was, for a second to make a double tap.
    tap: Option<(f64, (f64, f64))>,
}

impl Touches {
    fn centre_and_apart(&self) -> Option<((f64, f64), f64)> {
        let [(_, a), (_, b), ..] = self.down[..] else {
            return None;
        };
        let centre = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
        let apart = ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
        Some((centre, apart))
    }

    fn track(&mut self, p: (f64, f64)) {
        if let Some(now) = now_ms() {
            self.track.0.push(now, p.0);
            self.track.1.push(now, p.1);
        }
    }

    /// A pointer lifted at `p` after going down at `start`: whether it was
    /// the second tap of a double tap. Blitz sends no `dblclick` for a click
    /// whose default action the viewer cancelled, so the viewer counts its
    /// own taps where it has a clock; a browser, where it has none, sends
    /// `dblclick` as ever.
    fn tapped(&mut self, start: (f64, f64), p: (f64, f64)) -> bool {
        let Some(now) = now_ms() else {
            return false;
        };
        let apart =
            |a: (f64, f64), b: (f64, f64)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
        if apart(start, p) > TAP {
            self.tap = None;
            return false;
        }
        match self.tap.take() {
            Some((t, q)) if now - t <= DOUBLE_TAP_TIME && apart(q, p) <= DOUBLE_TAP_NEAR => true,
            _ => {
                self.tap = Some((now, p));
                false
            }
        }
    }

    fn speed(&self) -> (f64, f64) {
        match now_ms() {
            Some(now) => (self.track.0.velocity(now), self.track.1.velocity(now)),
            None => (0.0, 0.0),
        }
    }
}

/// Run `zoom`'s motion a frame at a time, by the host's sleep, until it
/// rests; with no sleep to time frames by, put it where it rests at once.
fn animate(unit: Unit, mut zoom: Signal<Option<(Key3, Zoom)>>, mut running: Signal<bool>) {
    if *running.peek() {
        return;
    }
    let Some(sleep) = unit.timer.peek().clone() else {
        if let Some((_, z)) = zoom.write().as_mut() {
            z.finish();
        }
        return;
    };
    running.set(true);
    spawn(async move {
        let mut last = now_ms();
        loop {
            sleep(Duration::from_millis(16)).await;
            let now = now_ms();
            let dt = match (last, now) {
                (Some(a), Some(b)) => b - a,
                _ => 16.0,
            };
            last = now;
            let moving = zoom.write().as_mut().is_some_and(|(_, z)| z.step(dt));
            if !moving {
                break;
            }
        }
        running.set(false);
    });
}

/// What a zoom is of: the photo shown, and the stage's size.
type Key3 = (usize, u64, u64);

/// `photos[index]` over the whole window while `open`. `onchange` is given
/// the photo to go to, and the page sets `index`; `onclose` is called when
/// it asks to close, and the page sets `open` false.
///
/// The photo zooms as a phone's does ([`crate::zoom`]): pinched, double
/// tapped, the wheel with ⌘ or Ctrl, or `+`, `-` and `0`; up to `most`
/// times the size that fits. Zoomed, a drag moves it; fitting, a swipe goes
/// to the next. That needs the window's size, which the host gives
/// ([`Unit::set_viewport`]); a browser's root sets it itself. Without it,
/// the photo fits and does not zoom.
#[component]
pub fn PhotoViewer(
    open: bool,
    photos: Vec<Photo>,
    index: usize,
    onchange: EventHandler<usize>,
    onclose: EventHandler<()>,
    /// The most a photo zooms, as a multiple of the size that fits.
    #[props(default = MOST)]
    most: f64,
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

    // The stage: the window inside its safe area, if the host said how
    // large the window is.
    let (safe, _) = unit.edges();
    let viewport = unit.layer.read().viewport;
    let stage = (
        viewport.0 - safe.left - safe.right,
        viewport.1 - safe.top - safe.bottom,
    );
    let size = photos.get(index).and_then(|p| p.size);
    let key: Key3 = (index, stage.0.to_bits(), stage.1.to_bits());
    let fresh = (stage.0 > 0.0 && stage.1 > 0.0).then(|| Zoom::new(stage, size, most));
    // The photo's zoom, for the photo and the stage it was made for: a new
    // photo, or a window of a new size, starts again from fitting.
    let zoom = use_signal(|| None::<(Key3, Zoom)>);
    let running = use_signal(|| false);
    let touches = use_hook(|| CopyValue::new(Touches::default()));
    let current = move || -> Option<Zoom> {
        match *zoom.peek() {
            Some((k, z)) if k == key => Some(z),
            _ => fresh,
        }
    };
    let change = move |f: &mut dyn FnMut(&mut Zoom) -> bool| {
        let Some(mut z) = current() else {
            return;
        };
        let moving = f(&mut z);
        let mut zoom = zoom;
        zoom.set(Some((key, z)));
        if moving {
            animate(unit, zoom, running);
        }
    };
    let shown = match *zoom.read() {
        Some((k, z)) if k == key => Some(z),
        _ => fresh,
    };
    let zoomed = shown.is_some_and(|z| z.is_zoomed());
    let middle = (stage.0 / 2.0, stage.1 / 2.0);

    let focus = use_focus(Kind::Widget, move |key, _| {
        let to = match key {
            Key::Named(NamedKey::ArrowLeft) => index.saturating_sub(1),
            Key::Named(NamedKey::ArrowRight) => index + 1,
            Key::Named(NamedKey::Home) => 0,
            Key::Named(NamedKey::End) => n.saturating_sub(1),
            Key::Character(c) if c == "+" || c == "=" => {
                change(&mut |z| {
                    z.zoom_by(2.0, middle);
                    false
                });
                return true;
            }
            Key::Character(c) if c == "-" => {
                change(&mut |z| {
                    z.zoom_by(0.5, middle);
                    false
                });
                return true;
            }
            Key::Character(c) if c == "0" => {
                change(&mut |z| z.fit());
                return true;
            }
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

    let Some(photo) = photos.get(index).filter(|_| open).cloned() else {
        return rsx! {};
    };
    let count = format!("{} of {n}", index + 1);
    // Where a pointer is on the stage.
    let local = move |e: &PointerEvent| {
        let c = e.client_coordinates();
        (c.x - safe.left, c.y - safe.top)
    };
    let picture = match shown {
        Some(z) => {
            let (x, y, w, h) = z.frame();
            rsx! {
                div { class: "unit-viewer-photo",
                      style: "left: {x}px; top: {y}px; width: {w}px; height: {h}px",
                    // `alt` for a browser; Blitz names an image by its label.
                    img { class: "unit-viewer-image", src: "{photo.src}", alt: "{photo.alt}",
                          "aria-label": "{photo.alt}" }
                }
            }
        }
        None => rsx! {
            img { class: "unit-viewer-image unit-viewer-whole", src: "{photo.src}", alt: "{photo.alt}",
                  "aria-label": "{photo.alt}" }
        },
    };
    rsx! {
        div {
            class: "unit-viewer {class}",
            role: "dialog",
            "aria-modal": "true",
            "aria-label": "{photo.alt}, {count}",
            onpointerdown: move |e: PointerEvent| {
                e.stop_propagation();
                focus.press();
            },
            // A press inside it, a swipe across its photo, leaves the
            // renderer's focus where it was: Blitz focuses what a click
            // lands on as the click's default action, and with nothing
            // focusable under it, the document's root, from where keys no
            // longer reach the page's. A browser does the same on
            // `mousedown`. Its keys are the page's own (`use_focus`).
            onmousedown: move |e: MouseEvent| e.prevent_default(),
            onclick: move |e: MouseEvent| e.prevent_default(),
            // The wheel is the photo's, never the page's under it: with ⌘
            // or Ctrl (a trackpad's pinch, in a browser) it zooms about the
            // pointer; without, it moves a zoomed photo.
            onwheel: move |e: WheelEvent| {
                e.prevent_default();
                e.stop_propagation();
                let (dx, dy) = wheel_delta(e.delta(), stage.1);
                let mods = e.modifiers();
                let c = e.client_coordinates();
                let p = (c.x - safe.left, c.y - safe.top);
                change(&mut |z| {
                    if mods.ctrl() || mods.meta() {
                        z.zoom_by(zoom_factor(dy), p);
                    } else if z.is_zoomed() {
                        z.pan_by(dx, dy);
                    }
                    false
                });
            },
            div { class: "unit-viewer-stage",
                style: "left: {safe.left}px; top: {safe.top}px; right: {safe.right}px; bottom: {safe.bottom}px",
                onpointerdown: move |e: PointerEvent| {
                    let p = local(&e);
                    let mut t = touches;
                    let mut t = t.write();
                    t.down.retain(|(id, _)| *id != e.pointer_id());
                    t.down.push((e.pointer_id(), p));
                    t.track.0.reset();
                    t.track.1.reset();
                    t.track(p);
                    t.start = (t.down.len() == 1).then_some(p);
                    match t.down.len() {
                        1 => {
                            let held = current().is_some_and(|z| z.is_zoomed() || z.is_moving());
                            if held {
                                t.swipe = None;
                                change(&mut |z| {
                                    z.pan_start(p);
                                    false
                                });
                            } else {
                                t.swipe = Some(p);
                            }
                        }
                        _ => {
                            t.swipe = None;
                            if let Some((centre, apart)) = t.centre_and_apart() {
                                change(&mut |z| {
                                    z.pinch_start(centre, apart);
                                    false
                                });
                            }
                        }
                    }
                },
                onpointermove: move |e: PointerEvent| {
                    let p = local(&e);
                    let mut t = touches;
                    let mut t = t.write();
                    let Some(slot) = t.down.iter_mut().find(|(id, _)| *id == e.pointer_id()) else {
                        return;
                    };
                    slot.1 = p;
                    t.track(p);
                    if let Some((centre, apart)) = t.centre_and_apart() {
                        change(&mut |z| {
                            z.pinch_move(centre, apart);
                            false
                        });
                    } else if current().is_some_and(|z| z.is_held()) {
                        change(&mut |z| {
                            z.pan_move(p);
                            false
                        });
                    }
                },
                onpointerup: move |e: PointerEvent| {
                    let p = local(&e);
                    let mut t = touches;
                    let mut t = t.write();
                    let was = t.down.len();
                    t.down.retain(|(id, _)| *id != e.pointer_id());
                    match (was, t.down.as_slice()) {
                        // One of two lifted: the other carries on moving it.
                        (2.., [(_, other)]) => {
                            let other = *other;
                            t.track.0.reset();
                            t.track.1.reset();
                            t.track(other);
                            change(&mut |z| {
                                z.pan_start(other);
                                false
                            });
                        }
                        (_, []) => {
                            let speed = t.speed();
                            let double = t.start.take().is_some_and(|start| t.tapped(start, p));
                            if double {
                                t.swipe = None;
                                change(&mut |z| {
                                    z.release((0.0, 0.0));
                                    z.double_tap(p)
                                });
                            } else if let Some(start) = t.swipe.take() {
                                let dx = p.0 - start.0;
                                if dx <= -SWIPE {
                                    go(index + 1);
                                } else if dx >= SWIPE {
                                    go(index.saturating_sub(1));
                                }
                            } else if current().is_some_and(|z| z.is_held()) {
                                change(&mut |z| z.release(speed));
                            }
                        }
                        _ => {}
                    }
                },
                onpointercancel: move |e: PointerEvent| {
                    let mut t = touches;
                    let mut t = t.write();
                    t.down.retain(|(id, _)| *id != e.pointer_id());
                    t.swipe = None;
                    if t.down.is_empty() && current().is_some_and(|z| z.is_held()) {
                        change(&mut |z| z.release((0.0, 0.0)));
                    }
                },
                ondoubleclick: move |e: MouseEvent| {
                    let c = e.client_coordinates();
                    let p = (c.x - safe.left, c.y - safe.top);
                    change(&mut |z| z.double_tap(p));
                },
                {picture}
                if index > 0 && !zoomed {
                    Button { variant: Variant::Ghost, class: "unit-icon unit-viewer-prev", label: "Previous photo",
                             icon: paths::CHEVRON_LEFT, refuses_keys: true, onpress: move |_| go(index - 1) }
                }
                if index + 1 < n && !zoomed {
                    Button { variant: Variant::Ghost, class: "unit-icon unit-viewer-next", label: "Next photo",
                             icon: paths::CHEVRON_RIGHT, refuses_keys: true, onpress: move |_| go(index + 1) }
                }
            }
            div { class: "unit-viewer-bar", style: "padding-top: {safe.top + 6.0}px; padding-left: {safe.left + 16.0}px; padding-right: {safe.right + 8.0}px",
                span { class: "unit-viewer-count", "{count}" }
                Button { variant: Variant::Ghost, class: "unit-icon", label: "Close", icon: paths::CLOSE,
                         refuses_keys: true, onpress: move |_| onclose.call(()) }
            }
            if let Some(c) = &photo.caption {
                div { class: "unit-viewer-caption", style: "padding-bottom: {safe.bottom + 14.0}px",
                    "{c}"
                }
            }
        }
    }
}

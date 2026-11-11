//! A photo viewer: one photo at a time over the whole window, on black, as
//! a phone's and a desktop's photo apps show one.
//!
//! It goes from photo to photo by the arrows, by the buttons at its sides,
//! or by a swipe, sliding the next photo in beside the last as a phone's
//! does, and closes by Escape or its close button. A photo zooms as a
//! phone's does: a pinch, a double tap, the wheel ([`crate::zoom`]). While
//! it is open it has the keys, and gives them back to what had them when
//! it closes. It is in the page only while it is open, as nothing of it is
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
use crate::zoom::{Dismiss, MOST, PAGE_GAP, Pages, Zoom};

/// How far a pointer may move and its lifting still be a tap, in CSS
/// pixels; how soon a second tap follows the first to make a double tap,
/// in milliseconds, and how near it lands.
const TAP: f64 = 10.0;
const DOUBLE_TAP_TIME: f64 = 300.0;

/// How far a finger moves before what it is doing is settled: sliding to
/// the next photo, or dragging this one down to close.
const LOCK: f64 = 8.0;

/// What a single finger on a photo that fits is doing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Lock {
    /// Not far enough yet to tell.
    #[default]
    Undecided,
    /// Sideways: from photo to photo.
    Pages,
    /// Down: the photo, picked up to close the viewer.
    Dismiss,
}

/// How long the wheel is still, in milliseconds, before a trackpad's swipe
/// (and the momentum the system carries it on with) has ended.
const WHEEL_IDLE: f64 = 100.0;

/// A trackpad's two-finger swipe across a photo that fits: how far it has
/// gone (in the wheel's direction, as a drag's), when it last moved, and its
/// fastest speed, which says which way it was going when the momentum has
/// died away by the time it ends.
#[derive(Clone, Copy, Debug)]
struct WheelSwipe {
    x: f64,
    last: f64,
    peak: f64,
}
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

/// Milliseconds from a start of its own, for a finger's speed, a double tap
/// and a frame's length: the system's clock, or a browser's.
fn now_ms() -> Option<f64> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let start = START.get_or_init(std::time::Instant::now);
        Some(start.elapsed().as_secs_f64() * 1000.0)
    }
    #[cfg(target_arch = "wasm32")]
    Some(js_sys::Date::now())
}

/// The pointers on the photo, and what a single one is doing.
#[derive(Default)]
struct Touches {
    /// Each pointer down, by id, and where it is on the stage.
    down: Vec<(i32, (f64, f64))>,
    /// The last pointer's recent places, across and down, for its speed.
    track: (Tracker, Tracker),
    /// Where a single pointer went down, while it may yet be a tap.
    start: Option<(f64, f64)>,
    /// When and where the last tap was, for a second to make a double tap.
    tap: Option<(f64, (f64, f64))>,
    /// What a single finger on a photo that fits is doing.
    lock: Lock,
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
    /// the second tap of a double tap. The viewer counts its own taps, as a
    /// gesture recogniser does: Blitz sends no `dblclick` for a click whose
    /// default action the viewer cancelled, and a phone's browser sends none
    /// for a finger.
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

/// Run `step` a frame at a time until it says nothing moves: on the host's
/// sleep where it gave one, on a browser's frames in a browser, and with
/// neither, to where it rests at once.
fn frames(unit: Unit, mut running: Signal<bool>, step: impl Fn(f64) -> bool + 'static) {
    if *running.peek() {
        return;
    }
    let sleep = unit.timer.peek().clone();
    if sleep.is_none() && !cfg!(target_arch = "wasm32") {
        for _ in 0..2000 {
            if !step(16.0) {
                break;
            }
        }
        return;
    }
    running.set(true);
    spawn(async move {
        let mut last = now_ms();
        loop {
            match &sleep {
                Some(sleep) => sleep(Duration::from_millis(16)).await,
                None => next_frame().await,
            }
            let now = now_ms();
            let dt = match (last, now) {
                (Some(a), Some(b)) => b - a,
                _ => 16.0,
            };
            last = now;
            if !step(dt) {
                break;
            }
        }
        running.set(false);
    });
}

/// A browser's next frame (`requestAnimationFrame`): what a page in a
/// browser times its motion by, with no sleep from its host.
#[cfg(target_arch = "wasm32")]
async fn next_frame() {
    use wasm_bindgen::JsCast;
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let global = js_sys::global();
        let raf = js_sys::Reflect::get(&global, &"requestAnimationFrame".into())
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok());
        match raf {
            Some(raf) => {
                let _ = raf.call1(&global, &resolve);
            }
            None => {
                let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
            }
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

#[cfg(not(target_arch = "wasm32"))]
async fn next_frame() {}

/// A photo on the stage: its index, how far along its page is, and where
/// the photo is drawn on it.
type Slot = (usize, f64, (f64, f64, f64, f64));

/// What a zoom is of: the photo shown, and the stage's size.
type Key3 = (usize, u64, u64);

/// `photos[index]` over the whole window while `open`. `onchange` is given
/// the photo to go to, and the page sets `index`; `onclose` is called when
/// it asks to close, and the page sets `open` false.
///
/// The photo zooms as a phone's does ([`crate::zoom`]): pinched, double
/// tapped, the wheel with ⌘ or Ctrl, or `+`, `-` and `0`; up to `most`
/// times the size that fits. Zoomed, a drag or a trackpad's two-finger
/// swipe moves it; fitting, either slides it aside with the next beside it
/// ([`Pages`]); dragged down, it closes the viewer ([`Dismiss`]). That needs the
/// window's size, which the host gives ([`Unit::set_viewport`]); a
/// browser's root sets it itself. Without it, the photo fits and does not
/// zoom or slide, and the arrows go from photo to photo.
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
    /// Buttons of the page's own in the bar, before Close: what is done to
    /// the photo shown (`photos[index]`), Share, Favourite, Delete. They fade
    /// with the bar as the photo is dragged down.
    actions: Option<Element>,
    #[props(default)] class: String,
) -> Element {
    let unit = use_unit();
    let id = use_hook(next_id);
    let n = photos.len();
    let index = index.min(n.saturating_sub(1));
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
    let staged = stage.0 > 0.0 && stage.1 > 0.0;
    let stride = stage.0 + PAGE_GAP;
    let (prev, next) = (index > 0, index + 1 < n);
    let sizes: Vec<Option<(f64, f64)>> = photos.iter().map(|p| p.size).collect();
    let fitted = move |i: usize| Zoom::new(stage, sizes.get(i).copied().flatten(), most);
    let key: Key3 = (index, stage.0.to_bits(), stage.1.to_bits());
    let fresh = staged.then(|| fitted(index));
    // The photo's zoom, for the photo and the stage it was made for: a new
    // photo, or a window of a new size, starts again from fitting.
    let zoom = use_signal(|| None::<(Key3, Zoom)>);
    // How far the photos are slid, from where the photo it names rests.
    let pages = use_signal(|| (index, Pages::default()));
    // The photo dragged down to close the viewer.
    let dismiss = use_signal(Dismiss::default);
    let running = use_signal(|| false);
    let touches = use_hook(|| CopyValue::new(Touches::default()));
    let swipe = use_hook(|| CopyValue::new(None::<WheelSwipe>));
    // What the frames read as they run, which may be past the render they
    // began in: the photo shown, whether it has neighbours, the stride.
    let live = use_hook(|| CopyValue::new((index, prev, next, stride)));
    live.clone().set((index, prev, next, stride));
    let current = move || -> Option<Zoom> {
        match *zoom.peek() {
            Some((k, z)) if k == key => Some(z),
            _ => fresh,
        }
    };
    // The slide from where the shown photo rests, whichever photo the
    // pages were last set for: going to the next, the page may not have
    // shown it yet.
    let slid = move |(at, p): (usize, Pages)| -> f64 {
        let apart = at as f64 - index as f64;
        // At rest, the photo shown rests where it rests, whatever photo the
        // pages were last set for (the viewer opened at another).
        if !p.is_moving() && !p.is_dragging() {
            return 0.0;
        }
        if apart.abs() <= 1.0 {
            p.offset + apart * stride
        } else {
            0.0
        }
    };
    // Frames for whatever is moving, until it all rests.
    // A trackpad's swipe has ended once the wheel has been still a while:
    // on to the next photo if it was going that way fast, or past halfway,
    // else back. Whether it is still going.
    let swipe_ended = move |force: bool| -> bool {
        let Some(w) = *swipe.peek() else {
            return false;
        };
        let idle = now_ms().is_none_or(|now| now - w.last >= WHEEL_IDLE);
        if !idle && !force {
            return true;
        }
        swipe.clone().set(None);
        let (index, prev, next, stride) = *live.peek();
        let mut pages = pages;
        let dir = pages.write().1.release(w.peak, stride, prev, next);
        if dir != 0 {
            let to = (index as i64 + dir as i64) as usize;
            let p = pages.peek().1;
            pages.set((to, p));
            onchange.call(to);
        }
        false
    };
    let kick = move || {
        frames(unit, running, move |dt| {
            let (mut zoom, mut pages, mut dismiss) = (zoom, pages, dismiss);
            let swiping = swipe_ended(false);
            let zooming = zoom.write().as_mut().is_some_and(|(_, z)| z.step(dt));
            let sliding = pages.write().1.step(dt);
            let dismissing = dismiss.write().step(dt);
            // Dragged down and gone: the viewer closes.
            if dismiss.peek().is_gone() {
                dismiss.set(Dismiss::default());
                onclose.call(());
                return false;
            }
            swiping || zooming || sliding || dismissing
        });
    };
    let change = move |f: &mut dyn FnMut(&mut Zoom) -> bool| {
        let Some(mut z) = current() else {
            return;
        };
        let moving = f(&mut z);
        let mut zoom = zoom;
        zoom.set(Some((key, z)));
        if moving {
            kick();
        }
    };
    // Go to photo `to`, sliding it in from its side where there is a stage.
    let slide_to = move |to: usize| {
        if to == index || to >= n {
            return;
        }
        if staged {
            let mut p = Pages::at(slid(*pages.peek()));
            p.slide(if to > index { 1 } else { -1 }, stride);
            let mut pages = pages;
            pages.set((to, p));
        }
        onchange.call(to);
        kick();
    };
    let shown = match *zoom.read() {
        Some((k, z)) if k == key => Some(z),
        _ => fresh,
    };
    let offset = slid(*pages.read());
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
        slide_to(to);
        true
    });
    // The keys while it is open, and back where they were after.
    let before = use_hook(|| CopyValue::new(None));
    use_effect(use_reactive!(|open| {
        if open {
            dismiss.clone().set(Dismiss::default());
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
    // Dragged down: the photo smaller, and the viewer's black and its bar
    // fading, the page under it showing through.
    let d = *dismiss.read();
    let fade = d.fade(stage.1);
    let lifted = d.is_active().then(|| {
        format!(
            "; transform: translate({}px, {}px) scale({})",
            d.at.0,
            d.at.1,
            d.scale(stage.1)
        )
    });
    // Where a pointer is on the stage.
    let local = move |e: &PointerEvent| {
        let c = e.client_coordinates();
        (c.x - safe.left, c.y - safe.top)
    };
    // Each photo in view or next to it, side by side: the one shown as it
    // is zoomed, its neighbours fitting, read by assistive tech only as the
    // one shown.
    let slots: Vec<Slot> = if staged {
        (index.saturating_sub(1)..=(index + 1).min(n - 1))
            .map(|i| {
                let x = offset + (i as f64 - index as f64) * stride;
                let frame = match (i == index, shown) {
                    (true, Some(z)) => z.frame(),
                    _ => fitted(i).frame(),
                };
                (i, x, frame)
            })
            .collect()
    } else {
        Vec::new()
    };
    rsx! {
        div {
            class: "unit-viewer {class}",
            style: "background: rgba(0, 0, 0, {fade})",
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
            // pointer; without, it moves a zoomed photo, and across one that
            // fits (a trackpad's two-finger swipe) it slides the photos as a
            // drag does.
            onwheel: move |e: WheelEvent| {
                e.prevent_default();
                e.stop_propagation();
                let (dx, dy) = wheel_delta(e.delta(), stage.1);
                let mods = e.modifiers();
                let c = e.client_coordinates();
                let p = (c.x - safe.left, c.y - safe.top);
                let zoomed = current().is_some_and(|z| z.is_zoomed());
                if mods.ctrl() || mods.meta() || zoomed || !staged {
                    change(&mut |z| {
                        if mods.ctrl() || mods.meta() {
                            z.zoom_by(zoom_factor(dy), p);
                        } else if z.is_zoomed() {
                            z.pan_by(dx, dy);
                        }
                        false
                    });
                    return;
                }
                let mut sw = swipe;
                // Scrolling up and down a photo that fits does nothing; a
                // swipe begins sideways, and goes on whichever way it wavers.
                if sw.peek().is_none() && dx.abs() <= dy.abs() {
                    return;
                }
                let now = now_ms().unwrap_or(0.0);
                let mut pages = pages;
                let w = match *sw.peek() {
                    Some(mut w) => {
                        let dt = now - w.last;
                        if dt > 0.0 && (dx / dt).abs() > w.peak.abs() {
                            w.peak = dx / dt;
                        }
                        w.x += dx;
                        w.last = now;
                        w
                    }
                    None => {
                        let mut s = Pages::at(slid(*pages.peek()));
                        s.drag_start(0.0);
                        pages.set((index, s));
                        WheelSwipe { x: dx, last: now, peak: 0.0 }
                    }
                };
                sw.set(Some(w));
                pages.write().1.drag_move(w.x, stride, prev, next);
                // With no frames to time its end by, it ends at once.
                if unit.timer.peek().is_none() && !cfg!(target_arch = "wasm32") {
                    swipe_ended(true);
                }
                kick();
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
                    t.lock = Lock::Undecided;
                    let mut pages = pages;
                    let mut dismiss = dismiss;
                    match t.down.len() {
                        1 => {
                            let zoomed = current().is_some_and(|z| z.is_zoomed() || z.is_moving());
                            if zoomed {
                                change(&mut |z| {
                                    z.pan_start(p);
                                    false
                                });
                            } else if staged {
                                // The photos stop where they are, and follow
                                // the finger from there.
                                let mut s = Pages::at(slid(*pages.peek()));
                                s.drag_start(p.0);
                                pages.set((index, s));
                            }
                        }
                        _ => {
                            // A second finger: a pinch, and no slide, and the
                            // photo put back.
                            if dismiss.peek().is_dragging() {
                                dismiss.write().cancel();
                                kick();
                            }
                            if pages.peek().1.is_dragging() {
                                pages.write().1.release(0.0, stride, false, false);
                                kick();
                            }
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
                    let mut pages = pages;
                    let mut dismiss = dismiss;
                    // Far enough to tell: down picks the photo up to close
                    // the viewer; any other way slides to the next.
                    if t.lock == Lock::Undecided && pages.peek().1.is_dragging()
                        && let Some(start) = t.start
                    {
                        let (dx, dy) = (p.0 - start.0, p.1 - start.1);
                        if dx.abs().max(dy.abs()) > LOCK {
                            if dy > dx.abs() {
                                t.lock = Lock::Dismiss;
                                pages.write().1.release(0.0, stride, false, false);
                                dismiss.write().drag_start(start);
                                kick();
                            } else {
                                t.lock = Lock::Pages;
                            }
                        }
                    }
                    if let Some((centre, apart)) = t.centre_and_apart() {
                        change(&mut |z| {
                            z.pinch_move(centre, apart);
                            false
                        });
                    } else if dismiss.peek().is_dragging() {
                        dismiss.write().drag_move(p, stage.1);
                    } else if pages.peek().1.is_dragging() {
                        pages.write().1.drag_move(p.0, stride, prev, next);
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
                    let mut pages = pages;
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
                                if pages.peek().1.is_dragging() {
                                    pages.write().1.release(0.0, stride, false, false);
                                }
                                change(&mut |z| {
                                    z.release((0.0, 0.0));
                                    z.double_tap(p)
                                });
                                kick();
                            } else if dismiss.peek().is_dragging() {
                                // Down far or fast enough: on down, and closed;
                                // else back where it rests.
                                let mut dismiss = dismiss;
                                dismiss.write().release(speed);
                                kick();
                            } else if pages.peek().1.is_dragging() {
                                // On to the next photo, or back to this one,
                                // sliding from where the finger left it.
                                let dir = pages.write().1.release(speed.0, stride, prev, next);
                                if dir != 0 {
                                    let to = (index as i64 + dir as i64) as usize;
                                    let s = pages.peek().1;
                                    pages.set((to, s));
                                    onchange.call(to);
                                }
                                kick();
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
                    t.start = None;
                    let mut pages = pages;
                    if pages.peek().1.is_dragging() {
                        pages.write().1.release(0.0, stride, false, false);
                        kick();
                    }
                    let mut dismiss = dismiss;
                    if dismiss.peek().is_dragging() {
                        dismiss.write().cancel();
                        kick();
                    }
                    if t.down.is_empty() && current().is_some_and(|z| z.is_held()) {
                        change(&mut |z| z.release((0.0, 0.0)));
                    }
                },
                if staged {
                    for (i, x, (fx, fy, fw, fh)) in slots {
                        div { key: "{i}", class: "unit-viewer-page",
                              style: "left: {x}px; width: {stage.0}px{lifted.as_deref().filter(|_| i == index).unwrap_or_default()}",
                              "aria-hidden": (i != index).then_some("true"),
                            div { class: if i == index { "unit-viewer-photo current" } else { "unit-viewer-photo" },
                                  style: "left: {fx}px; top: {fy}px; width: {fw}px; height: {fh}px",
                                // `alt` for a browser; Blitz names an image by
                                // its label. Not the browser's to drag away as
                                // an image: a drag moves it.
                                img { class: "unit-viewer-image", src: "{photos[i].src}", alt: "{photos[i].alt}",
                                      "aria-label": "{photos[i].alt}", draggable: "false" }
                            }
                        }
                    }
                } else {
                    img { class: "unit-viewer-image unit-viewer-whole", src: "{photo.src}", alt: "{photo.alt}",
                          "aria-label": "{photo.alt}", draggable: "false" }
                }
                if prev && !zoomed && !d.is_active() {
                    Button { variant: Variant::Ghost, class: "unit-icon unit-viewer-prev", label: "Previous photo",
                             icon: paths::CHEVRON_LEFT, refuses_keys: true, onpress: move |_| slide_to(index - 1) }
                }
                if next && !zoomed && !d.is_active() {
                    Button { variant: Variant::Ghost, class: "unit-icon unit-viewer-next", label: "Next photo",
                             icon: paths::CHEVRON_RIGHT, refuses_keys: true, onpress: move |_| slide_to(index + 1) }
                }
            }
            div { class: "unit-viewer-bar", style: "padding-top: {safe.top + 6.0}px; padding-left: {safe.left + 16.0}px; padding-right: {safe.right + 8.0}px; opacity: {fade}",
                span { class: "unit-viewer-count", "{count}" }
                div { class: "unit-viewer-actions",
                    {actions}
                    Button { variant: Variant::Ghost, class: "unit-icon", label: "Close", icon: paths::CLOSE,
                             refuses_keys: true, onpress: move |_| onclose.call(()) }
                }
            }
            if let Some(c) = &photo.caption {
                div { class: "unit-viewer-caption", style: "padding-bottom: {safe.bottom + 14.0}px; opacity: {fade}",
                    "{c}"
                }
            }
        }
    }
}

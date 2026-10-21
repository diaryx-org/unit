//! A long press: a finger held still on something for [`HOLD`], as a phone
//! opens a context menu.
//!
//! unit keeps no clock of its own, so it cannot tell by itself when the
//! finger has been down long enough. A host gives it one with
//! [`Unit::set_timer`]: a sleep, its own runtime's (tokio's, the
//! browser's). With it, the hold fires while the finger is still down, as
//! a phone's does; without one, it fires when the finger lifts after
//! [`HOLD`] or more, which a page in a browser seldom needs, since a
//! browser sends a long press as `contextmenu` itself.
//!
//! Only a finger or a pen holds: the mouse's long press is its secondary
//! button. A finger that moves more than [`SLOP`] is scrolling, not
//! holding. The click that ends a hold is not a press.

use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use dioxus::prelude::*;

use crate::unit::{Unit, use_unit};

/// How long a finger is held still before it is a long press.
pub const HOLD: Duration = Duration::from_millis(500);

/// How far, in CSS pixels, a finger may move and still be holding.
pub const SLOP: f64 = 10.0;

/// The host's sleep: a future that is ready once `Duration` has passed.
pub type Sleep = Rc<dyn Fn(Duration) -> Pin<Box<dyn Future<Output = ()>>>>;

type OnHold = Rc<dyn Fn((f64, f64))>;

impl Unit {
    /// How the page waits: a long press fires while the finger is still
    /// down once a host has said how to sleep.
    pub fn set_timer(
        &self,
        sleep: impl Fn(Duration) -> Pin<Box<dyn Future<Output = ()>>> + 'static,
    ) {
        self.timer.clone().set(Some(Rc::new(sleep)));
    }
}

/// A press that may become a hold: wired to an element's pointer events by
/// [`LongPress`] and [`crate::ContextMenu`], or by a page's own element.
#[derive(Clone, Copy)]
pub struct Hold {
    unit: Unit,
    /// Where the finger went down, while it may still be a hold.
    at: CopyValue<Option<(f64, f64)>>,
    /// Which press this is, so a sleep that ends after the finger lifted
    /// and went down again does not fire for the new one.
    press: CopyValue<u64>,
    /// The hold fired, and the click that ends it is not a press.
    held: CopyValue<bool>,
    #[cfg(not(target_arch = "wasm32"))]
    since: CopyValue<Option<std::time::Instant>>,
    fire: CopyValue<Option<OnHold>>,
}

/// A [`Hold`] that calls `onhold` with where the finger is.
pub fn use_hold(onhold: impl Fn((f64, f64)) + 'static) -> Hold {
    let unit = use_unit();
    let h = use_hook(|| Hold {
        unit,
        at: CopyValue::new(None),
        press: CopyValue::new(0),
        held: CopyValue::new(false),
        #[cfg(not(target_arch = "wasm32"))]
        since: CopyValue::new(None),
        fire: CopyValue::new(None),
    });
    h.fire.clone().set(Some(Rc::new(onhold)));
    h
}

impl Hold {
    fn fire(&self, at: (f64, f64)) {
        self.at.clone().set(None);
        self.held.clone().set(true);
        let f = self.fire.peek().clone();
        if let Some(f) = f {
            f(at);
        }
    }

    /// The pointer went down.
    pub fn down(&self, e: &PointerEvent) {
        self.held.clone().set(false);
        if e.pointer_type() == "mouse" {
            self.at.clone().set(None);
            return;
        }
        let p = e.client_coordinates();
        let at = (p.x, p.y);
        self.at.clone().set(Some(at));
        let press = *self.press.peek() + 1;
        self.press.clone().set(press);
        #[cfg(not(target_arch = "wasm32"))]
        self.since.clone().set(Some(std::time::Instant::now()));
        let sleep = self.unit.timer.peek().clone();
        if let Some(sleep) = sleep {
            let me = *self;
            spawn(async move {
                sleep(HOLD).await;
                let still = *me.press.peek() == press;
                let at = *me.at.peek();
                if still && let Some(at) = at {
                    me.fire(at);
                }
            });
        }
    }

    /// The pointer moved: far enough, and it is not a hold.
    pub fn moved(&self, e: &PointerEvent) {
        let Some((x, y)) = *self.at.peek() else {
            return;
        };
        let p = e.client_coordinates();
        if (p.x - x).hypot(p.y - y) > SLOP {
            self.at.clone().set(None);
        }
    }

    /// The pointer lifted, or was cancelled. With no timer, a press held
    /// long enough fires now.
    pub fn up(&self, _e: &PointerEvent) {
        let at = self.at.peek().to_owned();
        self.at.clone().set(None);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let since = self.since.peek().to_owned();
            if let (Some(at), Some(t)) = (at, since)
                && self.unit.timer.peek().is_none()
                && t.elapsed() >= HOLD
            {
                self.fire(at);
            }
        }
        #[cfg(target_arch = "wasm32")]
        let _ = at;
    }

    /// Whether the press just ended was a hold, so the click it sends is
    /// not a press: asked once, by the click.
    pub fn take_held(&self) -> bool {
        std::mem::take(&mut *self.held.clone().write())
    }
}

/// An area a finger holds to do `onhold`, given where the finger is; a tap
/// that is not a hold does `onpress`.
#[component]
pub fn LongPress(
    onhold: EventHandler<(f64, f64)>,
    onpress: Option<EventHandler<()>>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let hold = use_hold(move |at| onhold.call(at));
    rsx! {
        div { class: "unit-hold {class}",
            onpointerdown: move |e: PointerEvent| hold.down(&e),
            onpointermove: move |e: PointerEvent| hold.moved(&e),
            onpointerup: move |e: PointerEvent| hold.up(&e),
            onpointercancel: move |e: PointerEvent| hold.up(&e),
            onclick: move |e: MouseEvent| {
                if hold.take_held() {
                    e.stop_propagation();
                } else if let Some(p) = onpress {
                    e.stop_propagation();
                    p.call(());
                }
            },
            {children}
        }
    }
}

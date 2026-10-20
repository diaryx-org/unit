//! A slider: a value between two bounds, dragged along a track or stepped by
//! the arrows.
//!
//! Where the pointer is along the track is read without measuring it. The
//! track is a row of equal stops, one per value it can take (at most
//! [`STOPS`] + 1), and the stop the pointer is over is the value: measuring
//! the track's width would need the DOM, which a page on dioxus-native can
//! only read from a task while Blitz holds it borrowed. So it works at any
//! width, in a browser and in Blitz alike.

use dioxus::html::{Key, NamedKey};
use dioxus::prelude::*;

use crate::unit::{Kind, use_focus};

/// The most stops a track is divided into.
pub const STOPS: usize = 100;

/// The values a slider's track stops at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Track {
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

impl Track {
    /// How many steps from one end to the other, as stops: at most
    /// [`STOPS`].
    pub fn stops(&self) -> usize {
        let range = (self.max - self.min).max(0.0);
        if self.step <= 0.0 {
            return STOPS;
        }
        ((range / self.step).round() as usize).clamp(1, STOPS)
    }

    /// The value at stop `i`.
    pub fn at(&self, i: usize) -> f64 {
        let n = self.stops() as f64;
        self.snap(self.min + (self.max - self.min) * i as f64 / n)
    }

    /// `v` kept within the bounds and on a step.
    pub fn snap(&self, v: f64) -> f64 {
        let v = v.clamp(self.min, self.max);
        if self.step <= 0.0 {
            return v;
        }
        let snapped = self.min + ((v - self.min) / self.step).round() * self.step;
        snapped.clamp(self.min, self.max)
    }

    /// How far along the track `v` is, from 0 to 1.
    pub fn fraction(&self, v: f64) -> f64 {
        if self.max <= self.min {
            return 0.0;
        }
        ((v - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// What a key does to `v`, or `None` for a key that does nothing.
    pub fn key(&self, key: &Key, v: f64) -> Option<f64> {
        let step = if self.step > 0.0 {
            self.step
        } else {
            (self.max - self.min) / STOPS as f64
        };
        let page = step * (self.stops() as f64 / 10.0).ceil().max(1.0);
        let next = match key {
            Key::Named(NamedKey::ArrowRight | NamedKey::ArrowUp) => v + step,
            Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowDown) => v - step,
            Key::Named(NamedKey::PageUp) => v + page,
            Key::Named(NamedKey::PageDown) => v - page,
            Key::Named(NamedKey::Home) => self.min,
            Key::Named(NamedKey::End) => self.max,
            _ => return None,
        };
        Some(self.snap(next))
    }
}

/// A slider from `min` to `max` in steps of `step` (0: as finely as the
/// track's [`STOPS`] allow). `onchange` is given each value it moves to.
#[component]
pub fn Slider(
    value: f64,
    onchange: EventHandler<f64>,
    #[props(default = 0.0)] min: f64,
    #[props(default = 1.0)] max: f64,
    #[props(default = 0.0)] step: f64,
    /// What it sets, for assistive tech.
    label: Option<String>,
    /// The value as words, where a number alone says too little: "50%".
    text: Option<String>,
    #[props(default)] disabled: bool,
) -> Element {
    let track = Track { min, max, step };
    let focus = use_focus(Kind::Widget, move |key, _| {
        if disabled {
            return false;
        }
        match track.key(key, value) {
            Some(v) => {
                if v != value {
                    onchange.call(v);
                }
                true
            }
            None => false,
        }
    });
    let n = track.stops();
    let pct = track.fraction(value) * 100.0;
    let set = move |i: usize| {
        let v = track.at(i);
        if v != value && !disabled {
            onchange.call(v);
        }
    };
    let off = if disabled { " off" } else { "" };
    rsx! {
        div {
            class: "unit-slider{off}{focus.class()}",
            role: "slider",
            tabindex: if disabled { "-1" } else { "0" },
            "aria-label": label,
            "aria-valuenow": "{value}",
            "aria-valuemin": "{min}",
            "aria-valuemax": "{max}",
            "aria-valuetext": text,
            "aria-disabled": disabled.then_some("true"),
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            div { class: "unit-slider-rail", "aria-hidden": "true",
                div { class: "unit-slider-line" }
                div { class: "unit-slider-fill", style: "width: {pct}%" }
                div { class: "unit-slider-thumb", style: "left: {pct}%" }
            }
            // The stops: half a stop at each end, so stop i's middle is at
            // i/n of the way along, where its value is drawn.
            div { class: "unit-slider-stops", "aria-hidden": "true",
                for i in 0..=n {
                    div {
                        key: "{i}",
                        class: "unit-slider-stop",
                        style: if i == 0 || i == n { "flex-grow: 0.5" } else { "flex-grow: 1" },
                        onpointerdown: move |_| set(i),
                        onpointermove: move |e: PointerEvent| {
                            if !e.held_buttons().is_empty() {
                                set(i);
                            }
                        },
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stepped_track_has_a_stop_per_value() {
        let t = Track {
            min: 0.0,
            max: 10.0,
            step: 2.0,
        };
        assert_eq!(t.stops(), 5);
        assert_eq!(t.at(0), 0.0);
        assert_eq!(t.at(3), 6.0);
        assert_eq!(t.at(5), 10.0);
    }

    #[test]
    fn a_continuous_track_is_cut_into_at_most_stops() {
        let t = Track {
            min: 0.0,
            max: 1.0,
            step: 0.0,
        };
        assert_eq!(t.stops(), STOPS);
        assert!((t.at(50) - 0.5).abs() < 1e-9);
        let fine = Track {
            min: 0.0,
            max: 1000.0,
            step: 1.0,
        };
        assert_eq!(fine.stops(), STOPS);
        assert_eq!(fine.at(1), 10.0);
    }

    #[test]
    fn the_keys_step_and_stay_inside() {
        let t = Track {
            min: 0.0,
            max: 10.0,
            step: 1.0,
        };
        let right = Key::Named(NamedKey::ArrowRight);
        assert_eq!(t.key(&right, 3.0), Some(4.0));
        assert_eq!(t.key(&right, 10.0), Some(10.0));
        assert_eq!(t.key(&Key::Named(NamedKey::PageDown), 1.0), Some(0.0));
        assert_eq!(t.key(&Key::Named(NamedKey::End), 1.0), Some(10.0));
        assert_eq!(t.key(&Key::Character("x".into()), 1.0), None);
        assert_eq!(t.snap(3.4), 3.0);
    }
}

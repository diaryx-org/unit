//! Zooming a photo as a phone's photo viewer does: a model a view keeps and
//! draws the photo from, as [`Scroll`](crate::scroll::Scroll) is for a
//! scrolled view. [`PhotoViewer`](crate::PhotoViewer) is built on it.
//!
//! The photo fits the stage at a zoom of 1, its whole self in view, in the
//! middle; it zooms up to [`Zoom::most`]. What it does follows UIKit's
//! `UIScrollView` with `bouncesZoom`, as the Photos app uses it:
//!
//! - **A pinch** zooms about the point between the fingers, and moves the
//!   photo as that point moves: the photo's point that was under the
//!   fingers stays under them. Past the least or the most zoom it gives
//!   less the further it is pinched, and springs back when the fingers
//!   lift, about the point where they were.
//! - **A drag** of a zoomed photo moves it; past an edge it gives as a
//!   scroll view's rubber band does. Let go, it carries on at the finger's
//!   speed, slowing as a scroll view slows, and springs back to the edge.
//! - **A double tap** zooms in, the point tapped brought to the middle (as
//!   Apple's sample does it), or out again to fit if it was zoomed.
//! - **The wheel**, with ⌘ or Ctrl held (what a browser sends for a
//!   trackpad's pinch), zooms about the pointer; without, it moves a zoomed
//!   photo.
//!
//! The numbers are UIKit's where it publishes them, and `scroll`'s throw
//! takes them: the deceleration of 0.998 a millisecond, the rubber band
//! `(1 − 1 / (x·0.55 / d + 1))·d`, a critically damped spring back. How the
//! zoom gives past its limits UIKit does not say; here it is the same
//! rubber band over the zoom's logarithm, a view one *e*-fold "long": a
//! pinch to twice the most shows about 1.3 times it, and no pinch shows
//! more than *e* times it.
//!
//! Pure arithmetic over positions and milliseconds, as `scroll`'s is: the
//! view feeds it the fingers and the frames.

use crate::scroll::{Glide, SPRING, STOPPED, rubber, spring};

/// The most a photo zooms by default: four times the size that fits.
pub const MOST: f64 = 4.0;

/// How far a double tap zooms in at least: further where filling the stage
/// takes more.
pub const DOUBLE_TAP: f64 = 2.0;

/// How near a settling photo is to where it rests, in CSS pixels, and its
/// zoom, as a share, before it is put there.
const NEAR: f64 = 0.5;
const NEAR_ZOOM: f64 = 0.001;

/// What the photo is doing.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Motion {
    Still,
    /// Two fingers on it: the zoom and the distance between them when the
    /// pinch began, the photo's point between them (at zoom 1, from its
    /// top left), and where that point last was.
    Pinch {
        zoom: f64,
        apart: f64,
        point: (f64, f64),
        last: (f64, f64),
    },
    /// One finger on it: where the photo was and the finger was when it
    /// went down.
    Pan {
        from: (f64, f64),
        finger: (f64, f64),
    },
    /// Let go, moving on and slowing, each way on its own.
    Glide {
        x: Glide,
        y: Glide,
    },
    /// On its way to a zoom and a place: its zoom's logarithm's speed, and
    /// its speed each way, a millisecond.
    Settle {
        zoom: f64,
        at: (f64, f64),
        speed: (f64, f64, f64),
    },
}

/// A photo on a stage, zoomed and moved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zoom {
    /// The stage's size, in CSS pixels.
    pub view: (f64, f64),
    /// The photo's size at zoom 1: as large as fits the stage.
    pub fit: (f64, f64),
    /// The most it zooms.
    pub most: f64,
    /// How far it is zoomed: 1 fits the stage.
    pub zoom: f64,
    /// Where its top left is on the stage.
    pub at: (f64, f64),
    motion: Motion,
}

impl Zoom {
    /// A photo fitting `view`, in its middle. `photo` is its own size, for
    /// its shape; with none it is taken to be the stage's shape.
    pub fn new(view: (f64, f64), photo: Option<(f64, f64)>, most: f64) -> Self {
        let (w, h) = (view.0.max(1.0), view.1.max(1.0));
        let aspect = photo
            .filter(|(pw, ph)| *pw > 0.0 && *ph > 0.0)
            .map_or(w / h, |(pw, ph)| pw / ph);
        let fit = if aspect > w / h {
            (w, w / aspect)
        } else {
            (h * aspect, h)
        };
        let mut z = Zoom {
            view: (w, h),
            fit,
            most: most.max(1.0),
            zoom: 1.0,
            at: (0.0, 0.0),
            motion: Motion::Still,
        };
        z.at = z.clamped(z.at, 1.0);
        z
    }

    /// Where the photo is drawn: its left, top, width and height on the
    /// stage.
    pub fn frame(&self) -> (f64, f64, f64, f64) {
        let (w, h) = self.size(self.zoom);
        (self.at.0, self.at.1, w, h)
    }

    /// Whether it is zoomed in past fitting, as a drag moves it rather than
    /// going to the next photo.
    pub fn is_zoomed(&self) -> bool {
        self.zoom > 1.0 + NEAR_ZOOM
    }

    /// Whether it is moving on its own, and wants frames.
    pub fn is_moving(&self) -> bool {
        matches!(self.motion, Motion::Glide { .. } | Motion::Settle { .. })
    }

    /// Whether fingers are on it.
    pub fn is_held(&self) -> bool {
        matches!(self.motion, Motion::Pinch { .. } | Motion::Pan { .. })
    }

    fn size(&self, zoom: f64) -> (f64, f64) {
        (self.fit.0 * zoom, self.fit.1 * zoom)
    }

    /// Where its top left may be at `zoom`, each way: a photo smaller than
    /// the stage that way sits in its middle (both ends the same), a larger
    /// one keeps its edges at or past the stage's.
    pub fn range(&self, zoom: f64) -> ((f64, f64), (f64, f64)) {
        let (w, h) = self.size(zoom);
        let along = |len: f64, view: f64| {
            if len <= view {
                let mid = (view - len) / 2.0;
                (mid, mid)
            } else {
                (view - len, 0.0)
            }
        };
        (along(w, self.view.0), along(h, self.view.1))
    }

    fn clamped(&self, at: (f64, f64), zoom: f64) -> (f64, f64) {
        let ((x0, x1), (y0, y1)) = self.range(zoom);
        (at.0.clamp(x0, x1), at.1.clamp(y0, y1))
    }

    /// Where a photo pulled to `at` at `zoom` shows: the rubber band's
    /// share of the pull past either edge.
    fn banded(&self, at: (f64, f64), zoom: f64) -> (f64, f64) {
        let ((x0, x1), (y0, y1)) = self.range(zoom);
        (
            rubber(at.0, x0, x1, self.view.0),
            rubber(at.1, y0, y1, self.view.1),
        )
    }

    /// The zoom a pinch to `raw` shows: itself between the least and the
    /// most, and past either the rubber band's share, over its logarithm.
    fn banded_zoom(&self, raw: f64) -> f64 {
        rubber(raw.max(1e-6).ln(), 0.0, self.most.ln(), 1.0).exp()
    }

    /// The photo's point (at zoom 1, from its top left) under the stage's
    /// point `p`.
    fn point_under(&self, p: (f64, f64)) -> (f64, f64) {
        ((p.0 - self.at.0) / self.zoom, (p.1 - self.at.1) / self.zoom)
    }

    /// Two fingers went down, `apart` CSS pixels apart with `centre`
    /// between them. Whatever it was doing, it stops.
    pub fn pinch_start(&mut self, centre: (f64, f64), apart: f64) {
        self.motion = Motion::Pinch {
            zoom: self.zoom,
            apart: apart.max(1.0),
            point: self.point_under(centre),
            last: centre,
        };
    }

    /// The fingers moved: `apart`, with `centre` between them.
    pub fn pinch_move(&mut self, centre: (f64, f64), apart: f64) {
        let Motion::Pinch {
            zoom,
            apart: apart0,
            point,
            ..
        } = self.motion
        else {
            return;
        };
        let z = self.banded_zoom(zoom * apart.max(1.0) / apart0);
        let raw = (centre.0 - point.0 * z, centre.1 - point.1 * z);
        self.zoom = z;
        self.at = self.banded(raw, z);
        self.motion = Motion::Pinch {
            zoom,
            apart: apart0,
            point,
            last: centre,
        };
    }

    /// One finger went down at `finger`, or one of two lifted and this one
    /// stayed. Whatever it was doing, it stops.
    pub fn pan_start(&mut self, finger: (f64, f64)) {
        self.motion = Motion::Pan {
            from: self.at,
            finger,
        };
    }

    /// The finger moved to `finger`.
    pub fn pan_move(&mut self, finger: (f64, f64)) {
        let Motion::Pan { from, finger: f0 } = self.motion else {
            return;
        };
        let raw = (from.0 + finger.0 - f0.0, from.1 + finger.1 - f0.1);
        self.at = self.banded(raw, self.zoom);
    }

    /// The fingers lifted, the last moving at `speed`, CSS pixels a
    /// millisecond each way. Past its zoom's limits it springs back to them
    /// about the point it was pinched at; else it carries on and slows, and
    /// springs back inside its edges. Whether it is moving.
    pub fn release(&mut self, speed: (f64, f64)) -> bool {
        let anchor = match self.motion {
            Motion::Pinch { last, .. } => last,
            _ => (self.view.0 / 2.0, self.view.1 / 2.0),
        };
        let target = self.zoom.clamp(1.0, self.most);
        if (target / self.zoom - 1.0).abs() > NEAR_ZOOM {
            let at = self.about(anchor, target);
            self.motion = Motion::Settle {
                zoom: target,
                at,
                speed: (0.0, speed.0, speed.1),
            };
        } else {
            let ((x0, x1), (y0, y1)) = self.range(self.zoom);
            self.motion = Motion::Glide {
                x: Glide {
                    pos: self.at.0,
                    vel: speed.0,
                    min: x0,
                    max: x1,
                },
                y: Glide {
                    pos: self.at.1,
                    vel: speed.1,
                    min: y0,
                    max: y1,
                },
            };
            // Nothing to carry on with, and nothing to spring back from.
            self.step(0.0);
        }
        self.is_moving()
    }

    /// Where its top left goes zoomed to `zoom` about the stage's point
    /// `anchor`, which keeps the photo's point under it, kept within the
    /// edges.
    fn about(&self, anchor: (f64, f64), zoom: f64) -> (f64, f64) {
        let k = zoom / self.zoom;
        let at = (
            anchor.0 - (anchor.0 - self.at.0) * k,
            anchor.1 - (anchor.1 - self.at.1) * k,
        );
        self.clamped(at, zoom)
    }

    /// A double tap at `p`: zoomed in, the point tapped brought to the
    /// middle of the stage as far as the edges let it; or out to fit, if it
    /// was zoomed. Whether it is moving.
    pub fn double_tap(&mut self, p: (f64, f64)) -> bool {
        let (zoom, at) = if self.is_zoomed() {
            (1.0, self.clamped(self.at, 1.0))
        } else {
            let zoom = self.double_tap_zoom();
            let point = self.point_under(p);
            let mid = (self.view.0 / 2.0, self.view.1 / 2.0);
            let at = (mid.0 - point.0 * zoom, mid.1 - point.1 * zoom);
            (zoom, self.clamped(at, zoom))
        };
        self.settle_to(zoom, at)
    }

    /// The zoom a double tap goes to: enough to fill the stage both ways,
    /// and at least [`DOUBLE_TAP`], within the most.
    pub fn double_tap_zoom(&self) -> f64 {
        let fill = (self.view.0 / self.fit.0).max(self.view.1 / self.fit.1);
        fill.max(DOUBLE_TAP).min(self.most)
    }

    /// Go to fit, animated. Whether it is moving.
    pub fn fit(&mut self) -> bool {
        // At zoom 1 the photo fits: its one place is the middle.
        self.settle_to(1.0, self.clamped(self.at, 1.0))
    }

    fn settle_to(&mut self, zoom: f64, at: (f64, f64)) -> bool {
        self.motion = Motion::Settle {
            zoom,
            at,
            speed: (0.0, 0.0, 0.0),
        };
        self.step(0.0);
        self.is_moving()
    }

    /// Zoom by `factor` about the stage's point `p` at once, within the
    /// limits: the wheel's, a key's.
    pub fn zoom_by(&mut self, factor: f64, p: (f64, f64)) {
        let zoom = (self.zoom * factor).clamp(1.0, self.most);
        self.at = self.about(p, zoom);
        self.zoom = zoom;
        self.motion = Motion::Still;
    }

    /// Move by (`dx`, `dy`) at once, within the edges: the wheel's.
    pub fn pan_by(&mut self, dx: f64, dy: f64) {
        self.at = self.clamped((self.at.0 + dx, self.at.1 + dy), self.zoom);
        self.motion = Motion::Still;
    }

    /// Move on by `dt` milliseconds. Whether it is still moving.
    pub fn step(&mut self, dt: f64) -> bool {
        let dt = dt.clamp(0.0, 100.0);
        match self.motion {
            Motion::Glide { mut x, mut y } => {
                let (mx, my) = (x.step(dt), y.step(dt));
                self.at = (x.pos, y.pos);
                self.motion = if mx || my {
                    Motion::Glide { x, y }
                } else {
                    Motion::Still
                };
            }
            Motion::Settle { zoom, at, speed } => {
                let (lz, vz) = spring((self.zoom / zoom).ln(), speed.0, dt);
                let (x, vx) = spring(self.at.0 - at.0, speed.1, dt);
                let (y, vy) = spring(self.at.1 - at.1, speed.2, dt);
                let rested = lz.abs() < NEAR_ZOOM
                    && x.abs() < NEAR
                    && y.abs() < NEAR
                    && vx.abs().max(vy.abs()) < STOPPED;
                if rested {
                    self.zoom = zoom;
                    self.at = at;
                    self.motion = Motion::Still;
                } else {
                    self.zoom = zoom * lz.exp();
                    self.at = (at.0 + x, at.1 + y);
                    self.motion = Motion::Settle {
                        zoom,
                        at,
                        speed: (vz, vx, vy),
                    };
                }
            }
            _ => {}
        }
        self.is_moving()
    }

    /// Move on until it rests, for a view with no frames to show it by.
    pub fn finish(&mut self) {
        for _ in 0..2000 {
            if !self.step(16.0) {
                break;
            }
        }
    }
}

// ── Going from photo to photo ─────────────────────────────────────────────

/// How far apart the photos are as they slide, in CSS pixels: the gap the
/// Photos app leaves between one and the next.
pub const PAGE_GAP: f64 = 20.0;

/// How fast a finger must be going as it lifts, in CSS pixels a
/// millisecond, for a short swipe to go to the next photo all the same.
pub const FLICK: f64 = 0.3;

/// The photos side by side, slid by a finger, as a paging scroll view
/// slides them: the one shown follows the finger with its neighbours
/// beside it, `stride` apart (a stage's width and [`PAGE_GAP`]). Let go
/// more than halfway, or flicked, it goes on to the next, else it springs
/// back; past the first or the last it gives as a rubber band does.
///
/// `offset` is how far the photo shown is from where it rests, negative
/// toward the next. Going to another photo, the view shows that one, and
/// the offset is from where it rests, so it slides on from where it was.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pages {
    pub offset: f64,
    speed: f64,
    /// Where the finger and the offset were when it went down.
    drag: Option<(f64, f64)>,
}

impl Pages {
    /// The photos at rest `offset` from where the one shown rests.
    pub fn at(offset: f64) -> Self {
        Pages {
            offset,
            ..Pages::default()
        }
    }

    /// A finger went down at `x`: the photos stop where they are.
    pub fn drag_start(&mut self, x: f64) {
        self.drag = Some((x, self.offset));
        self.speed = 0.0;
    }

    /// The finger moved to `x`. `prev` and `next` say whether there is a
    /// photo that way to slide to.
    pub fn drag_move(&mut self, x: f64, stride: f64, prev: bool, next: bool) {
        let Some((x0, o0)) = self.drag else {
            return;
        };
        let min = if next { -stride } else { 0.0 };
        let max = if prev { stride } else { 0.0 };
        self.offset = rubber(o0 + x - x0, min, max, stride.max(1.0));
    }

    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// Whether the photos are sliding on their own, and want frames.
    pub fn is_moving(&self) -> bool {
        self.drag.is_none() && (self.offset != 0.0 || self.speed != 0.0)
    }

    /// The finger lifted, going at `speed`: which photo to go to, -1 the one
    /// before, 1 the next, 0 the one shown. The photos then slide to it from
    /// where they are, carrying the finger's speed.
    pub fn release(&mut self, speed: f64, stride: f64, prev: bool, next: bool) -> i32 {
        self.drag = None;
        let o = self.offset;
        let dir = if speed <= -FLICK && next && o < 0.0 {
            1
        } else if speed >= FLICK && prev && o > 0.0 {
            -1
        } else if o < -stride / 2.0 && next {
            1
        } else if o > stride / 2.0 && prev {
            -1
        } else {
            0
        };
        self.offset += dir as f64 * stride;
        self.speed = speed;
        self.no_overshoot();
        dir
    }

    /// Go to the photo `dir` along (a key, a button): it slides in from
    /// that side.
    pub fn slide(&mut self, dir: i32, stride: f64) {
        self.drag = None;
        self.offset += dir as f64 * stride;
        self.no_overshoot();
    }

    /// A speed toward rest no greater than the spring takes without going
    /// past it: a page lands, and does not bounce off where it rests.
    fn no_overshoot(&mut self) {
        if self.offset != 0.0 && self.speed / self.offset < -SPRING {
            self.speed = -SPRING * self.offset;
        }
    }

    /// Move on by `dt` milliseconds. Whether it is still moving.
    pub fn step(&mut self, dt: f64) -> bool {
        if !self.is_moving() {
            return false;
        }
        let (x, v) = spring(self.offset, self.speed, dt.clamp(0.0, 100.0));
        if x.abs() < NEAR && v.abs() < STOPPED {
            self.offset = 0.0;
            self.speed = 0.0;
            return false;
        }
        self.offset = x;
        self.speed = v;
        true
    }
}

// ── Dragging a photo down to close it ─────────────────────────────────────

/// How far down a photo is dragged, in CSS pixels, before letting it go
/// closes the viewer.
pub const DISMISS_DISTANCE: f64 = 100.0;

/// How fast down a finger must be going as it lifts, in CSS pixels a
/// millisecond, for a short drag to close the viewer all the same.
pub const DISMISS_SPEED: f64 = 0.5;

/// How long a photo let go to close goes on down, fading, in milliseconds.
const LEAVING: f64 = 200.0;

/// A photo that fits, dragged down to close the viewer as the Photos app
/// closes one: it follows the finger every way, smaller the further down
/// it goes, and what is behind it shows through as the viewer fades. Let
/// go more than [`DISMISS_DISTANCE`] down, or flicked down, it goes on down
/// and the viewer closes; else it springs back.
///
/// `at` is how far the photo is from where it rests.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Dismiss {
    pub at: (f64, f64),
    speed: (f64, f64),
    /// Where the finger and the photo were when it went down.
    drag: Option<((f64, f64), (f64, f64))>,
    /// Let go to close: how long it has still to go.
    leaving: Option<f64>,
    gone: bool,
}

impl Dismiss {
    /// A finger at `finger` picks the photo up.
    pub fn drag_start(&mut self, finger: (f64, f64)) {
        self.drag = Some((finger, self.at));
        self.speed = (0.0, 0.0);
    }

    /// The finger moved to `finger`. Above where it rests, the photo gives
    /// as a rubber band does: it closes only going down.
    pub fn drag_move(&mut self, finger: (f64, f64), height: f64) {
        let Some((f0, a0)) = self.drag else {
            return;
        };
        let raw = (a0.0 + finger.0 - f0.0, a0.1 + finger.1 - f0.1);
        self.at = (raw.0, rubber(raw.1, 0.0, f64::MAX, height.max(1.0)));
    }

    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// Whether the photo is off where it rests, held or moving.
    pub fn is_active(&self) -> bool {
        self.drag.is_some() || self.leaving.is_some() || self.at != (0.0, 0.0)
    }

    /// Whether it is moving on its own, and wants frames.
    pub fn is_moving(&self) -> bool {
        self.drag.is_none() && (self.leaving.is_some() || self.at != (0.0, 0.0))
    }

    /// Whether it has gone, and the viewer is to close.
    pub fn is_gone(&self) -> bool {
        self.gone
    }

    /// How far down it is, as a share of `height`: 0 at rest, 1 a whole
    /// stage down.
    pub fn progress(&self, height: f64) -> f64 {
        (self.at.1.max(0.0) / height.max(1.0)).min(1.0)
    }

    /// How large it is drawn: smaller the further down, to 70% a stage down.
    pub fn scale(&self, height: f64) -> f64 {
        1.0 - 0.3 * self.progress(height)
    }

    /// How much of the viewer's black, and its bar and caption, still
    /// shows: gone by the time the photo is 40% of a stage down.
    pub fn fade(&self, height: f64) -> f64 {
        (1.0 - self.progress(height) / 0.4).clamp(0.0, 1.0)
    }

    /// The finger lifted, going at `speed`. Whether it closes the viewer.
    pub fn release(&mut self, speed: (f64, f64)) -> bool {
        self.drag = None;
        let far = self.at.1 > DISMISS_DISTANCE && speed.1 > -0.1;
        if far || speed.1 > DISMISS_SPEED {
            self.leaving = Some(LEAVING);
            // On down, at least as fast as a flick, the way it was going.
            self.speed = (speed.0, speed.1.max(1.5));
            true
        } else {
            self.speed = speed;
            false
        }
    }

    /// Put it back where it rests at once: a second finger's pinch.
    pub fn cancel(&mut self) {
        self.drag = None;
        self.speed = (0.0, 0.0);
    }

    /// Move on by `dt` milliseconds. Whether it is still moving.
    pub fn step(&mut self, dt: f64) -> bool {
        if !self.is_moving() {
            return false;
        }
        let dt = dt.clamp(0.0, 100.0);
        if let Some(left) = self.leaving {
            self.at.0 += self.speed.0 * dt;
            self.at.1 += self.speed.1 * dt;
            let left = left - dt;
            if left <= 0.0 {
                self.leaving = None;
                self.gone = true;
                return false;
            }
            self.leaving = Some(left);
            return true;
        }
        let (x, vx) = spring(self.at.0, self.speed.0, dt);
        let (y, vy) = spring(self.at.1, self.speed.1, dt);
        if x.abs() < NEAR && y.abs() < NEAR && vx.abs().max(vy.abs()) < STOPPED {
            *self = Dismiss::default();
            return false;
        }
        self.at = (x, y);
        self.speed = (vx, vy);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A phone's stage, 400 × 800, and a 4:3 photo: it fits 400 × 300,
    /// 250 down.
    fn phone() -> Zoom {
        Zoom::new((400.0, 800.0), Some((4000.0, 3000.0)), MOST)
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    #[test]
    fn a_photo_fits_the_stage_in_its_middle() {
        let z = phone();
        assert_eq!(z.frame(), (0.0, 250.0, 400.0, 300.0));
        let wide = Zoom::new((400.0, 800.0), None, MOST);
        assert_eq!(
            wide.frame(),
            (0.0, 0.0, 400.0, 800.0),
            "the stage's own shape"
        );
    }

    #[test]
    fn a_pinch_keeps_the_point_between_the_fingers_under_them() {
        let mut z = phone();
        z.pinch_start((200.0, 400.0), 100.0);
        // Twice as far apart, and both fingers moved 50 to the left.
        z.pinch_move((150.0, 400.0), 200.0);
        assert!(near(z.zoom, 2.0));
        // The photo's point that was under them, (200, 150) from its top
        // left, is at (150, 400) still: across, inside the edges; down, the
        // photo is 600 tall in an 800 stage, and sits in the middle.
        let (x, y, w, h) = z.frame();
        assert!(near(x + 200.0 * 2.0, 150.0), "{x}");
        assert_eq!((w, h), (800.0, 600.0));
        assert!(
            y > 100.0 - 1.0 && y < 150.0,
            "pulled off the middle, it gives: {y}"
        );
    }

    #[test]
    fn past_the_most_a_pinch_gives_less_and_springs_back_about_the_fingers() {
        let mut z = phone();
        z.pinch_start((200.0, 400.0), 100.0);
        z.pinch_move((200.0, 400.0), 800.0);
        assert!(z.zoom > MOST && z.zoom < 8.0, "{}", z.zoom);
        assert!(
            (z.zoom / MOST - 1.32).abs() < 0.05,
            "twice past, about 1.3 times"
        );
        let point = z.point_under((200.0, 400.0));
        assert!(z.release((0.0, 0.0)));
        z.finish();
        assert!(near(z.zoom, MOST));
        // The point that was under the fingers is there still.
        let (px, py) = (z.at.0 + point.0 * MOST, z.at.1 + point.1 * MOST);
        assert!(near(px, 200.0) && near(py, 400.0), "({px}, {py})");
        // Below the least, too: back to fitting, in the middle.
        z.pinch_start((200.0, 400.0), 400.0);
        z.pinch_move((200.0, 400.0), 50.0);
        assert!(z.zoom < 1.0 && z.zoom > 0.36);
        z.release((0.0, 0.0));
        z.finish();
        assert_eq!(z.frame(), (0.0, 250.0, 400.0, 300.0));
    }

    #[test]
    fn a_zoomed_photo_drags_rubber_bands_at_its_edge_and_glides_back() {
        let mut z = phone();
        z.zoom_by(2.0, (0.0, 0.0));
        assert_eq!(z.at.0, 0.0, "its left edge at the stage's");
        z.pan_start((100.0, 400.0));
        z.pan_move((300.0, 400.0));
        assert!(z.at.0 > 0.0 && z.at.0 < 200.0, "gives: {}", z.at.0);
        assert!(near(z.at.0, rubber(200.0, -400.0, 0.0, 400.0)));
        assert!(z.release((0.0, 0.0)));
        z.finish();
        assert!(near(z.at.0, 0.0), "back to the edge");
        // Thrown to the left, it carries on, slows, and stops inside.
        z.pan_start((300.0, 400.0));
        z.pan_move((250.0, 400.0));
        z.release((-1.0, 0.0));
        let before = z.at.0;
        z.step(16.0);
        assert!(z.at.0 < before, "it carries on");
        z.finish();
        assert!(z.at.0 >= -400.0 - NEAR && z.at.0 < -50.0, "{}", z.at.0);
    }

    #[test]
    fn a_double_tap_zooms_in_to_the_point_tapped_and_back_out() {
        let mut z = phone();
        // Filling an 800-tall stage takes 800 / 300 of it.
        assert!(near(z.double_tap_zoom(), 800.0 / 300.0));
        assert!(z.double_tap((100.0, 400.0)));
        z.finish();
        assert!(near(z.zoom, 800.0 / 300.0));
        // The point tapped, a quarter across, goes to the middle.
        let zoom = 800.0 / 300.0;
        assert!(near(z.at.0 + 100.0 * zoom, 200.0), "{}", z.at.0);
        assert!(z.is_zoomed());
        // One near the edge goes only as far as the edge lets it.
        let mut edge = phone();
        edge.double_tap((10.0, 400.0));
        edge.finish();
        assert!(near(edge.at.0, 0.0));
        z.double_tap((200.0, 400.0));
        z.finish();
        assert_eq!(z.frame(), (0.0, 250.0, 400.0, 300.0));
    }

    #[test]
    fn the_wheel_zooms_about_the_pointer_within_the_limits() {
        let mut z = phone();
        z.zoom_by(100.0, (200.0, 400.0));
        assert_eq!(z.zoom, MOST);
        z.zoom_by(0.01, (200.0, 400.0));
        assert_eq!(z.frame(), (0.0, 250.0, 400.0, 300.0));
        z.zoom_by(2.0, (200.0, 400.0));
        z.pan_by(-1000.0, 0.0);
        assert_eq!(z.at.0, -400.0, "no further than the edge");
    }

    const STRIDE: f64 = 420.0;

    fn run(p: &mut Pages) -> Vec<f64> {
        let mut seen = vec![];
        while p.step(16.0) {
            seen.push(p.offset);
        }
        seen
    }

    #[test]
    fn a_photo_follows_the_finger_and_gives_past_the_last() {
        let mut p = Pages::default();
        p.drag_start(300.0);
        p.drag_move(200.0, STRIDE, true, true);
        assert_eq!(p.offset, -100.0);
        // No photo after it: the drag gives as a rubber band does.
        p.drag_move(200.0, STRIDE, true, false);
        assert!(p.offset < 0.0 && p.offset > -100.0, "{}", p.offset);
    }

    #[test]
    fn let_go_past_halfway_it_slides_on_to_the_next_without_a_bounce() {
        let mut p = Pages::default();
        p.drag_start(400.0);
        p.drag_move(150.0, STRIDE, true, true);
        assert_eq!(p.release(0.0, STRIDE, true, true), 1);
        // The next photo is 170 from where it rests, and slides there.
        assert_eq!(p.offset, -250.0 + STRIDE);
        let seen = run(&mut p);
        assert!(seen.windows(2).all(|w| w[1] <= w[0]), "always toward rest");
        assert!(seen.iter().all(|&o| o >= 0.0), "never past it");
        assert_eq!(p.offset, 0.0);
    }

    #[test]
    fn a_short_flick_goes_on_and_a_short_slow_drag_springs_back() {
        let mut p = Pages::default();
        p.drag_start(400.0);
        p.drag_move(360.0, STRIDE, true, true);
        assert_eq!(p.release(-1.0, STRIDE, true, true), 1, "flicked");
        let mut q = Pages::default();
        q.drag_start(400.0);
        q.drag_move(360.0, STRIDE, true, true);
        assert_eq!(q.release(-0.05, STRIDE, true, true), 0, "slow");
        run(&mut q);
        assert_eq!(q.offset, 0.0);
        // A flick back toward the one shown does not go on.
        let mut r = Pages::default();
        r.drag_start(400.0);
        r.drag_move(360.0, STRIDE, true, true);
        assert_eq!(r.release(1.0, STRIDE, true, true), 0);
    }

    #[test]
    fn a_key_slides_the_next_photo_in_from_its_side() {
        let mut p = Pages::default();
        p.slide(1, STRIDE);
        assert_eq!(p.offset, STRIDE, "from the right");
        assert!(p.is_moving());
        run(&mut p);
        assert_eq!(p.offset, 0.0);
        p.slide(-1, STRIDE);
        assert_eq!(p.offset, -STRIDE, "from the left");
    }

    #[test]
    fn a_photo_dragged_down_shrinks_and_fades_and_far_enough_goes() {
        let mut d = Dismiss::default();
        d.drag_start((200.0, 300.0));
        d.drag_move((230.0, 500.0), 800.0);
        assert_eq!(d.at, (30.0, 200.0), "it follows the finger every way");
        assert!((d.scale(800.0) - (1.0 - 0.3 * 0.25)).abs() < 1e-9);
        assert!((d.fade(800.0) - (1.0 - 0.25 / 0.4)).abs() < 1e-9);
        assert!(d.release((0.0, 0.0)), "200 down: it goes");
        let mut frames = 0;
        while d.step(16.0) {
            frames += 1;
        }
        assert!(d.is_gone() && frames < 20, "{frames}");
        assert!(d.at.1 > 200.0, "on down as it went");
    }

    #[test]
    fn a_short_drag_springs_back_and_a_flick_down_goes() {
        let mut d = Dismiss::default();
        d.drag_start((200.0, 300.0));
        d.drag_move((200.0, 360.0), 800.0);
        assert!(!d.release((0.0, 0.05)), "60 down, slowly: back");
        while d.step(16.0) {}
        assert_eq!(d, Dismiss::default());
        d.drag_start((200.0, 300.0));
        d.drag_move((200.0, 340.0), 800.0);
        assert!(d.release((0.0, 1.0)), "a flick down");
        // Dragged up, it gives, and does not go.
        let mut up = Dismiss::default();
        up.drag_start((200.0, 300.0));
        up.drag_move((200.0, 100.0), 800.0);
        assert!(up.at.1 < 0.0 && up.at.1 > -200.0, "{}", up.at.1);
        assert_eq!(up.fade(800.0), 1.0);
        assert!(!up.release((0.0, -1.0)));
    }
}

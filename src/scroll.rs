//! Scrolling and zooming, as a model a view keeps and draws itself from.
//!
//! A scroll view that wraps its content and moves it does not fit every
//! view: an editor draws only the rows in its viewport, and so has to know
//! where the viewport is to know what to draw. So unit's scrolling is the
//! part every view shares, and no element: a [`Scroll`] holds where the
//! viewport is over the content and how far the content is zoomed, and
//! answers the wheel, a pinch, a move to keep a box in view, and the
//! bounds; [`Glide`] and [`Tracker`] carry a finger's throw on after it lets
//! go, as a phone's scroll view does. A view reads the offset and the zoom
//! and places what it draws by them.
//!
//! Distances are CSS pixels. A content box is given at zoom 1, and the
//! offset is in the zoomed content: the content's point `(x, y)` is drawn at
//! `(x · zoom − offset.x, y · zoom − offset.y)` in the viewport. Times are
//! milliseconds from any start the caller keeps, so that the model reads no
//! clock and works the same in a browser.
//!
//! A wheel's delta is Blitz's (winit's) convention: positive moves toward
//! the top and the left.

use std::collections::VecDeque;

use dioxus::html::geometry::WheelDelta;

/// One line of a wheel that counts in lines, in CSS pixels.
pub const LINE: f64 = 20.0;

/// How far a wheel or a pinch goes to double the zoom, in CSS pixels of
/// its delta.
const PER_DOUBLING: f64 = 140.0;

/// A wheel's delta, in CSS pixels: as it came, a line [`LINE`] tall, or a
/// page `page` tall.
pub fn wheel_delta(delta: WheelDelta, page: f64) -> (f64, f64) {
    match delta {
        WheelDelta::Pixels(v) => (v.x, v.y),
        WheelDelta::Lines(v) => (v.x * LINE, v.y * LINE),
        WheelDelta::Pages(v) => (v.x * page, v.y * page),
    }
}

/// How much a wheel turned `dy` CSS pixels zooms in (or, negative, out): a
/// factor to multiply the zoom by.
pub fn zoom_factor(dy: f64) -> f64 {
    (dy / PER_DOUBLING).exp2()
}

/// Where a viewport is over its content, and how far the content is zoomed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scroll {
    /// The viewport's top left, in the zoomed content.
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
    /// The viewport's size.
    pub view: (f64, f64),
    /// The content's size at zoom 1.
    pub content: (f64, f64),
    /// How far past the content's end the viewport may go, zoomed: room
    /// under the last line, as an editor leaves it.
    pub slack: f64,
    /// The least and the most the content may be zoomed.
    pub zooms: (f64, f64),
}

impl Default for Scroll {
    fn default() -> Self {
        Scroll {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            view: (0.0, 0.0),
            content: (0.0, 0.0),
            slack: 0.0,
            zooms: (1.0, 1.0),
        }
    }
}

impl Scroll {
    /// A viewport `view` at the top of `content`, unzoomed, which does not
    /// zoom.
    pub fn new(view: (f64, f64), content: (f64, f64)) -> Self {
        Scroll {
            view,
            content,
            ..Scroll::default()
        }
    }

    /// The same, zoomable between `least` and `most`.
    pub fn zoomable(mut self, least: f64, most: f64) -> Self {
        self.zooms = (least, most);
        self
    }

    /// The content's size as zoomed.
    pub fn zoomed(&self) -> (f64, f64) {
        (self.content.0 * self.zoom, self.content.1 * self.zoom)
    }

    /// The offsets the viewport may go between, across and down. Content
    /// narrower than the viewport sits in its middle: both ends are the
    /// offset that centres it.
    pub fn range(&self) -> ((f64, f64), (f64, f64)) {
        let (w, h) = self.zoomed();
        let across = if w < self.view.0 {
            let mid = -(self.view.0 - w) / 2.0;
            (mid, mid)
        } else {
            (0.0, w - self.view.0)
        };
        let down = (0.0, (h + self.slack - self.view.1).max(0.0));
        (across, down)
    }

    /// The offset kept within [`Self::range`].
    pub fn clamp(&mut self) {
        let ((x0, x1), (y0, y1)) = self.range();
        self.x = self.x.clamp(x0, x1);
        self.y = self.y.clamp(y0, y1);
    }

    /// Move by a wheel's delta (see [`wheel_delta`]): positive toward the
    /// top and the left. Whether the viewport moved.
    pub fn wheel(&mut self, dx: f64, dy: f64) -> bool {
        let before = (self.x, self.y);
        self.x -= dx;
        self.y -= dy;
        self.clamp();
        (self.x, self.y) != before
    }

    /// Zoom by `factor`, keeping the content under the viewport's point
    /// (`px`, `py`) where it is. Whether the zoom changed.
    pub fn zoom_at(&mut self, factor: f64, px: f64, py: f64) -> bool {
        self.zoom_to(self.zoom * factor, px, py)
    }

    /// Zoom to `zoom`, kept within [`Self::zooms`], keeping the content under
    /// the viewport's point (`px`, `py`) where it is.
    pub fn zoom_to(&mut self, zoom: f64, px: f64, py: f64) -> bool {
        let zoom = zoom.clamp(self.zooms.0, self.zooms.1);
        if zoom == self.zoom {
            return false;
        }
        let (cx, cy) = self.to_content(px, py);
        self.zoom = zoom;
        self.x = cx * zoom - px;
        self.y = cy * zoom - py;
        self.clamp();
        true
    }

    /// The content's point (at zoom 1) under the viewport's (`px`, `py`).
    pub fn to_content(&self, px: f64, py: f64) -> (f64, f64) {
        ((px + self.x) / self.zoom, (py + self.y) / self.zoom)
    }

    /// Where the content's point (`x`, `y`) (at zoom 1) is in the viewport.
    pub fn to_view(&self, x: f64, y: f64) -> (f64, f64) {
        (x * self.zoom - self.x, y * self.zoom - self.y)
    }

    /// Move just far enough that the content's box (`x`, `y`, `w`, `h`) (at
    /// zoom 1) is in view, `margin` CSS pixels clear of the viewport's edges
    /// where there is room for that: as it is, if it is in view already.
    /// Whether the viewport moved.
    pub fn reveal(&mut self, (x, y, w, h): (f64, f64, f64, f64), margin: f64) -> bool {
        let before = (self.x, self.y);
        let z = self.zoom;
        let (x0, y0, x1, y1) = (x * z, y * z, (x + w) * z, (y + h) * z);
        let mx = margin.min((self.view.0 - (x1 - x0)).max(0.0) / 2.0);
        let my = margin.min((self.view.1 - (y1 - y0)).max(0.0) / 2.0);
        if y0 < self.y {
            self.y = y0 - my;
        } else if y1 > self.y + self.view.1 {
            self.y = y1 + my - self.view.1;
        }
        if x0 < self.x {
            self.x = x0 - mx;
        } else if x1 > self.x + self.view.0 {
            self.x = x1 + mx - self.view.0;
        }
        self.clamp();
        (self.x, self.y) != before
    }
}

// ── A throw ───────────────────────────────────────────────────────────────
//
// Scrolling as a phone's own scroll view does it: the finger's speed carried
// on when it lets go and slowing as UIKit's `UIScrollView` slows, a stretch
// past either end that gives less the further it is pulled, and a spring
// back to the end once the finger is gone.
//
// The numbers are UIKit's: a scroll view's normal deceleration rate is 0.998
// — the speed kept per millisecond — and its rubber band gives
// `(1 − 1 / (x·0.55 / d + 1))·d` for a pull of `x` past an end of a view `d`
// tall. The spring back is critically damped, so it settles without a
// wobble, in about a third of a second.
//
// Pure arithmetic over positions and milliseconds, so that a frame that comes
// late moves the page as far as an on-time one would have.

/// The speed kept per millisecond (`UIScrollView.DecelerationRate.normal`).
pub const DECELERATION: f64 = 0.998;

/// How stiff the rubber band is (UIKit's constant).
const RUBBER: f64 = 0.55;

/// How quickly the spring back settles, per millisecond: about a third of a
/// second from a full stretch.
const SPRING: f64 = 0.012;

/// Below this speed, in CSS pixels a millisecond, a glide has stopped.
const STOPPED: f64 = 0.01;

/// How far back the finger's speed is read from, in milliseconds: the last
/// stretch of the drag, as UIKit reads it.
const WINDOW: f64 = 80.0;

/// How long a finger can rest before it lets go and still throw the page:
/// held still longer than this, it lets go of a page that is not moving.
const REST: f64 = 50.0;

/// Where a page pulled to `raw` shows, for a view `d` tall that may go from
/// `min` to `max`: `raw` itself between them, and past either, the rubber
/// band's share of the pull.
pub fn rubber(raw: f64, min: f64, max: f64, d: f64) -> f64 {
    let give = |x: f64| (1.0 - 1.0 / (x * RUBBER / d.max(1.0) + 1.0)) * d.max(1.0);
    if raw < min {
        min - give(min - raw)
    } else if raw > max {
        max + give(raw - max)
    } else {
        raw
    }
}

/// The finger's recent positions, for its speed when it lets go.
#[derive(Default, Debug)]
pub struct Tracker {
    samples: VecDeque<(f64, f64)>,
}

impl Tracker {
    /// Begin again: a finger went down.
    pub fn reset(&mut self) {
        self.samples.clear();
    }

    /// The finger is at `y` at `now`, in milliseconds.
    pub fn push(&mut self, now: f64, y: f64) {
        self.samples.push_back((now, y));
        while self.samples.front().is_some_and(|(t, _)| now - t > WINDOW) {
            self.samples.pop_front();
        }
    }

    /// The finger's speed as it lets go at `now`, in CSS pixels a
    /// millisecond: over the last stretch of the drag, or nothing if it had
    /// come to rest.
    pub fn velocity(&self, now: f64) -> f64 {
        let (Some(&(t0, y0)), Some(&(t1, y1))) = (self.samples.front(), self.samples.back()) else {
            return 0.0;
        };
        if now - t1 > REST {
            return 0.0;
        }
        let span = t1 - t0;
        if span < 1.0 {
            return 0.0;
        }
        (y1 - y0) / span
    }
}

/// A page moving on its own: gliding, slowing, and — past an end — sprung
/// back to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glide {
    /// Where the page is, and its speed in CSS pixels a millisecond.
    pub pos: f64,
    pub vel: f64,
    /// The ends it may rest between.
    pub min: f64,
    pub max: f64,
}

impl Glide {
    /// Move it on by `dt` milliseconds. Whether it is still moving.
    pub fn step(&mut self, dt: f64) -> bool {
        let dt = dt.clamp(0.0, 100.0);
        let edge = if self.pos < self.min {
            Some(self.min)
        } else if self.pos > self.max {
            Some(self.max)
        } else {
            None
        };
        match edge {
            // Between the ends: the speed decays exponentially, and the page
            // goes as far as that speed carries it over `dt`.
            None => {
                let k = DECELERATION.ln();
                let kept = DECELERATION.powf(dt);
                self.pos += self.vel * (kept - 1.0) / k;
                self.vel *= kept;
                if self.vel.abs() < STOPPED {
                    self.vel = 0.0;
                    self.pos = self.pos.clamp(self.min, self.max);
                    return false;
                }
                true
            }
            // Past one: a critically damped spring to it, which takes the
            // speed it came with and turns it round.
            Some(edge) => {
                let x0 = self.pos - edge;
                let v0 = self.vel;
                let w = SPRING;
                let e = (-w * dt).exp();
                let x = (x0 + (v0 + w * x0) * dt) * e;
                let v = (v0 - w * dt * (v0 + w * x0)) * e;
                self.pos = edge + x;
                self.vel = v;
                if x.abs() < 0.5 && v.abs() < STOPPED {
                    self.pos = edge;
                    self.vel = 0.0;
                    return false;
                }
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A view 100 × 100 over content 100 × 400.
    fn tall() -> Scroll {
        Scroll::new((100.0, 100.0), (100.0, 400.0))
    }

    #[test]
    fn a_wheel_turned_down_moves_toward_the_end_and_stops_there() {
        let mut s = tall();
        // Blitz's convention: a negative delta is a wheel turned down.
        assert!(s.wheel(0.0, -30.0));
        assert_eq!(s.y, 30.0);
        s.wheel(0.0, -1000.0);
        assert_eq!(s.y, 300.0);
        assert!(!s.wheel(0.0, -10.0), "at the end already");
        s.wheel(0.0, 1000.0);
        assert_eq!(s.y, 0.0);
    }

    #[test]
    fn slack_lets_the_view_go_past_the_end() {
        let mut s = tall();
        s.slack = 50.0;
        s.wheel(0.0, -1000.0);
        assert_eq!(s.y, 350.0);
    }

    #[test]
    fn a_zoom_keeps_the_point_under_the_pointer() {
        let mut s = tall().zoomable(0.5, 4.0);
        s.wheel(0.0, -40.0);
        let under = s.to_content(30.0, 60.0);
        assert!(s.zoom_at(2.0, 30.0, 60.0));
        assert_eq!(s.zoom, 2.0);
        let (px, py) = s.to_view(under.0, under.1);
        assert!(
            (px - 30.0).abs() < 1e-9 && (py - 60.0).abs() < 1e-9,
            "{px}, {py}"
        );
        // Within its bounds, and no further.
        assert!(s.zoom_at(10.0, 0.0, 0.0));
        assert_eq!(s.zoom, 4.0);
        assert!(!s.zoom_at(2.0, 0.0, 0.0));
    }

    #[test]
    fn content_narrower_than_the_view_sits_in_its_middle() {
        let mut s = tall().zoomable(0.5, 4.0);
        s.zoom_to(0.5, 0.0, 0.0);
        s.clamp();
        assert_eq!(s.x, -25.0);
        // Zoomed in, it scrolls across.
        s.zoom_to(2.0, 0.0, 0.0);
        assert!(s.wheel(-500.0, 0.0));
        assert_eq!(s.x, 100.0);
    }

    #[test]
    fn a_box_out_of_view_is_brought_in_and_one_in_view_is_left() {
        let mut s = tall();
        assert!(!s.reveal((0.0, 40.0, 10.0, 10.0), 8.0));
        assert!(s.reveal((0.0, 250.0, 10.0, 10.0), 8.0));
        assert_eq!(s.y, 250.0 + 10.0 + 8.0 - 100.0);
        assert!(s.reveal((0.0, 20.0, 10.0, 10.0), 8.0));
        assert_eq!(s.y, 12.0);
    }

    #[test]
    fn a_wheel_in_lines_or_pages_is_in_pixels() {
        use dioxus::html::geometry::{LinesVector, PagesVector, PixelsVector3D};
        assert_eq!(
            wheel_delta(
                WheelDelta::Pixels(PixelsVector3D::new(1.0, -3.0, 0.0)),
                500.0
            ),
            (1.0, -3.0)
        );
        assert_eq!(
            wheel_delta(WheelDelta::Lines(LinesVector::new(0.0, -2.0, 0.0)), 500.0),
            (0.0, -40.0)
        );
        assert_eq!(
            wheel_delta(WheelDelta::Pages(PagesVector::new(0.0, 1.0, 0.0)), 500.0),
            (0.0, 500.0)
        );
        assert_eq!(zoom_factor(PER_DOUBLING), 2.0);
        assert_eq!(zoom_factor(-PER_DOUBLING), 0.5);
    }

    fn run(mut g: Glide) -> (Glide, f64) {
        let mut t = 0.0;
        while g.step(8.0) {
            t += 8.0;
            assert!(t < 10_000.0, "a glide ends");
        }
        (g, t)
    }

    #[test]
    fn a_throw_travels_as_far_as_uikits() {
        // UIKit's distance for a throw at v: v·d / (1 − d), with d per ms.
        let v = 2.0;
        let (g, t) = run(Glide {
            pos: 0.0,
            vel: v,
            min: 0.0,
            max: 1e9,
        });
        let expected = v * DECELERATION / (1.0 - DECELERATION);
        assert!(
            (g.pos - expected).abs() < expected * 0.02,
            "{} vs {expected}",
            g.pos
        );
        assert!(t > 1000.0 && t < 4000.0, "about two seconds: {t}");
    }

    #[test]
    fn a_frame_late_goes_as_far_as_two_on_time() {
        let mut a = Glide {
            pos: 0.0,
            vel: 1.0,
            min: 0.0,
            max: 1e9,
        };
        let mut b = a;
        a.step(16.0);
        b.step(8.0);
        b.step(8.0);
        assert!((a.pos - b.pos).abs() < 1e-9);
    }

    #[test]
    fn a_throw_past_the_end_springs_back_to_it() {
        let (g, _) = run(Glide {
            pos: 990.0,
            vel: 3.0,
            min: 0.0,
            max: 1000.0,
        });
        assert_eq!(g.pos, 1000.0);
        let (g, _) = run(Glide {
            pos: -60.0,
            vel: 0.0,
            min: 0.0,
            max: 1000.0,
        });
        assert_eq!(g.pos, 0.0);
    }

    #[test]
    fn the_rubber_band_gives_less_the_further_it_is_pulled() {
        let d = 800.0;
        assert_eq!(rubber(400.0, 0.0, 1000.0, d), 400.0);
        let a = rubber(-100.0, 0.0, 1000.0, d);
        let b = rubber(-200.0, 0.0, 1000.0, d);
        assert!(a < 0.0 && a > -100.0);
        assert!(b < a && (b - a).abs() < 100.0);
        assert!(
            rubber(1e6, 0.0, 1000.0, d) < 1000.0 + d,
            "never past a view's height"
        );
    }

    #[test]
    fn a_finger_at_rest_throws_nothing() {
        let mut t = Tracker::default();
        t.push(0.0, 0.0);
        t.push(40.0, 80.0);
        assert_eq!(t.velocity(40.0), 2.0);
        assert_eq!(t.velocity(40.0 + REST + 1.0), 0.0);
    }
}

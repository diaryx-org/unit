//! A scroll bar: a track along an edge of a view, and a thumb as long as
//! the share of the content in view, as far along as the view is. Dragging
//! the thumb scrolls the view to match; a press on the track scrolls it a
//! screen toward the press.
//!
//! The bar draws a [`Scroll`] and holds no offset of its own: the view
//! keeps its scroll, as it has to anyway to draw (an editor draws only the
//! rows in view), and the bar says what offset it asks for.
//!
//! Nothing is measured. The host places the bar in its view's box, and its
//! length is the viewport's along its axis; a press on the track is placed
//! by where it landed on the track, and a drag goes by how far the pointer
//! moved since the thumb was pressed. A thin bar loses the pointer as soon
//! as a drag leaves it, so a layer over the window takes the pointer for as
//! long as the thumb is held. It is there from the first frame, taking
//! nothing until then: Blitz lays out a fixed box on the first paint, not
//! one a later frame adds.
//!
//! Its look is in its own `style`, so it draws the same under a host
//! without [`UnitRoot`](crate::UnitRoot)'s stylesheet; that adds only the
//! thumb's darker shade under the pointer.

use dioxus::prelude::*;

use crate::Axis;
use crate::scroll::Scroll;

/// How wide a scroll bar is, across its axis, in CSS pixels.
pub const BAR: f64 = 10.0;

/// A view's scroll bar along `axis`.
#[component]
pub fn ScrollBar(
    /// Which way it scrolls: up and down, or across.
    axis: Axis,
    /// The view's scroll, which it draws.
    scroll: Scroll,
    /// Where its track starts in the box it is placed in (the nearest
    /// positioned one, as for any absolutely positioned element), in CSS
    /// pixels: its left and top. Without one, it runs along the box's far
    /// edge, the right or the bottom, from its start. It runs the
    /// viewport's length along `axis`.
    #[props(default)]
    at: Option<(f64, f64)>,
    /// What assistive tech reads it as.
    #[props(default)]
    label: Option<String>,
    /// The offset along `axis` that a drag or a press asks the view for.
    onscroll: EventHandler<f64>,
) -> Element {
    // Where the pointer and the thumb were when the thumb was pressed.
    let mut held = use_signal(|| None::<(f64, f64)>);
    let vertical = axis == Axis::Vertical;
    let thumb = scroll.thumb(axis);
    let track = if vertical {
        scroll.view.1
    } else {
        scroll.view.0
    };
    let (start, len) = thumb.unwrap_or((0.0, 0.0));
    let along = move |e: &PointerEvent| {
        let p = e.client_coordinates();
        if vertical { p.y } else { p.x }
    };
    let on_track = move |e: &PointerEvent| {
        let p = e.element_coordinates();
        if vertical { p.y } else { p.x }
    };
    let shown = if thumb.is_some() { "block" } else { "none" };
    let (w, h) = if vertical { (BAR, track) } else { (track, BAR) };
    let place = match (at, vertical) {
        (Some((x, y)), _) => format!("left: {x}px; top: {y}px;"),
        (None, true) => "right: 0; top: 0;".to_string(),
        (None, false) => "left: 0; bottom: 0;".to_string(),
    };
    let (tx, ty, tw, th) = if vertical {
        (2.0, start, BAR - 4.0, len)
    } else {
        (start, 2.0, len, BAR - 4.0)
    };
    let room = (track - len).max(0.0);
    let percent = if room > 0.0 {
        (start / room * 100.0).round()
    } else {
        0.0
    };
    let dragging = held().is_some();
    let taking = if dragging { "auto" } else { "none" };
    let class = if vertical { "vertical" } else { "horizontal" };
    rsx! {
        div {
            class: "unit-scrollbar {class}",
            role: "scrollbar",
            "aria-orientation": class,
            "aria-label": label,
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            "aria-valuenow": "{percent}",
            style: "display: {shown}; position: absolute; {place} width: {w}px; height: {h}px; z-index: 50;",
            // A press is the bar's, not the view's under it, and leaves the
            // keys where they were.
            onmousedown: move |e: MouseEvent| {
                e.stop_propagation();
                e.prevent_default();
            },
            onmousemove: move |e: MouseEvent| e.stop_propagation(),
            // On the track: a screen toward the press.
            onpointerdown: move |e: PointerEvent| {
                e.stop_propagation();
                let toward_end = on_track(&e) > start + len;
                onscroll.call(scroll.page(axis, toward_end));
            },
            div {
                class: "unit-scrollbar-thumb",
                style: "position: absolute; left: {tx}px; top: {ty}px; width: {tw}px; height: {th}px; border-radius: 4px; background: var(--unit-dim, #8e8e93); opacity: 0.55;",
                onpointerdown: move |e: PointerEvent| {
                    e.stop_propagation();
                    held.set(Some((along(&e), start)));
                },
            }
        }
        // While the thumb is held, the window's pointer is the bar's.
        div {
            class: "unit-scrollbar-drag",
            style: "position: fixed; left: 0; top: 0; right: 0; bottom: 0; z-index: 1000; pointer-events: {taking};",
            onpointermove: move |e: PointerEvent| {
                if let Some((from, was)) = held() {
                    let to = (was + along(&e) - from).clamp(0.0, room);
                    onscroll.call(scroll.offset_at(axis, to));
                }
            },
            onmousemove: move |e: MouseEvent| e.stop_propagation(),
            onmousedown: move |e: MouseEvent| e.stop_propagation(),
            onpointerup: move |_| held.set(None),
            onpointerleave: move |_| held.set(None),
        }
    }
}

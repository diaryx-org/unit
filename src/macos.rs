//! The Mac's own colours, for [`Accent::System`](crate::Accent::System):
//! the accent and the highlight the user chose in System Settings, and the
//! focus ring drawn from them, each in the light appearance and the dark.
//!
//! AppKit's colours are dynamic: one `NSColor` is a different colour in each
//! appearance, so each is resolved to sRGB inside that appearance's drawing.
//! AppKit posts `NSSystemColorsDidChangeNotification` when the user picks
//! another accent or highlight; the page reads them again then, on the main
//! thread, which is where Blitz runs the page.

use std::cell::RefCell;
use std::collections::HashSet;
use std::ptr::NonNull;

use block2::RcBlock;
use dioxus::prelude::*;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{
    NSAppearance, NSAppearanceName, NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSColor,
    NSColorSpace, NSSystemColorsDidChangeNotification,
};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSObjectProtocol};

use crate::theme::SystemColours;

/// The system's colours, light and dark, as last read: read again when the
/// user changes them.
pub(crate) fn use_colours() -> Signal<Option<(SystemColours, SystemColours)>> {
    let mut colours = use_signal(read);
    let changed = use_coroutine(move |mut rx: UnboundedReceiver<()>| async move {
        while rx.recv().await.is_ok() {
            let now = read();
            if *colours.peek() != now {
                colours.set(now);
            }
        }
    });
    // Kept as long as the page: dropped with it, which stops the observing.
    use_hook(|| std::rc::Rc::new(Observer::new(changed.tx())));
    colours
}

/// The accent, focus ring and selection in the light appearance and in the
/// dark; `None` if AppKit has neither appearance.
pub(crate) fn read() -> Option<(SystemColours, SystemColours)> {
    // SAFETY: the names are AppKit's constants, there for as long as it is.
    let (aqua, dark) = unsafe { (NSAppearanceNameAqua, NSAppearanceNameDarkAqua) };
    Some((in_appearance(aqua)?, in_appearance(dark)?))
}

fn in_appearance(name: &NSAppearanceName) -> Option<SystemColours> {
    let appearance = NSAppearance::appearanceNamed(name)?;
    let read = RefCell::new(None);
    let block = RcBlock::new(|| {
        *read.borrow_mut() = Some(SystemColours {
            accent: css(&NSColor::controlAccentColor()),
            focus: css(&NSColor::keyboardFocusIndicatorColor()),
            selection: css(&NSColor::selectedTextBackgroundColor()),
        });
    });
    appearance.performAsCurrentDrawingAppearance(&block);
    drop(block);
    read.into_inner()
}

/// `colour` in the current drawing appearance, as CSS: `#rrggbb`, or
/// `rgba()` where it is not opaque, as the focus ring is not.
fn css(colour: &NSColor) -> &'static str {
    let Some(c) = colour.colorUsingColorSpace(&NSColorSpace::sRGBColorSpace()) else {
        // A pattern or a catalogue colour with no sRGB value: the
        // macOS default accent rather than nothing.
        return "#007aff";
    };
    let [r, g, b] = [c.redComponent(), c.greenComponent(), c.blueComponent()]
        .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
    let a = c.alphaComponent().clamp(0.0, 1.0);
    intern(if a >= 1.0 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("rgba({r}, {g}, {b}, {a:.3})")
    })
}

thread_local! {
    static INTERNED: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new());
}

/// `s`, kept for the life of the process, once however often it is read: a
/// palette's colours are `&'static str`. The user picks among a handful of
/// accents and highlights, so only a handful are ever kept.
fn intern(s: String) -> &'static str {
    INTERNED.with(|set| {
        let mut set = set.borrow_mut();
        if let Some(&kept) = set.get(s.as_str()) {
            return kept;
        }
        let kept: &'static str = Box::leak(s.into_boxed_str());
        set.insert(kept);
        kept
    })
}

/// AppKit's observer of the system's colours, removed when dropped.
struct Observer(Retained<ProtocolObject<dyn NSObjectProtocol>>);

impl Observer {
    fn new(changed: UnboundedSender<()>) -> Self {
        let block = RcBlock::new(move |_: NonNull<NSNotification>| {
            let _ = changed.unbounded_send(());
        });
        // SAFETY: the name is AppKit's constant; no object or queue, so the
        // block runs on the thread that posts, which AppKit's main thread
        // does, and sending on the channel is safe from any thread.
        let token = unsafe {
            NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
                Some(NSSystemColorsDidChangeNotification),
                None,
                None,
                &block,
            )
        };
        Observer(token)
    }
}

impl Drop for Observer {
    fn drop(&mut self) {
        // SAFETY: the token is the one `addObserverForName…` returned.
        unsafe { NSNotificationCenter::defaultCenter().removeObserver(self.0.as_ref()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each appearance resolves to colours CSS reads, and the dark differs
    /// from the light somewhere: the accent is resolved in each, not once.
    #[test]
    fn the_system_colours_resolve_in_each_appearance() {
        let (light, dark) = read().expect("AppKit has Aqua and Dark Aqua");
        for c in [light, dark] {
            for v in [c.accent, c.focus, c.selection] {
                assert!(v.starts_with('#') || v.starts_with("rgba("), "{v}");
            }
        }
        assert_ne!(light, dark);
    }

    #[test]
    fn a_colour_read_twice_is_kept_once() {
        let a = intern("#123456".into());
        let b = intern("#123456".into());
        assert!(std::ptr::eq(a, b));
    }
}

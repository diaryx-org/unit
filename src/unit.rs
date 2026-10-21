//! The [`Unit`]: one per page, made by [`UnitRoot`], and reached by every
//! component under it through context. It holds the page's overlay (menus,
//! the dialog, toasts: `layer.rs`) and which control the keys go to.
//!
//! **Focus is the page's own.** Blitz moves DOM focus only to a text field
//! on a press, never to a button, and a page cannot move it from code
//! (`MountedData::set_focus` from a task finds Blitz's document borrowed and
//! panics). So a control says it has the keys when it is pressed, or when
//! the renderer focuses it (Tab, in a browser and in Blitz), and gives them
//! up when it is blurred or a press lands anywhere else in the page. The
//! control with the keys is drawn with a ring, the same on every renderer.
//!
//! **Keys come one of two ways** ([`Keys`]): as DOM events at the root,
//! which is how a browser and dioxus-native's `launch` deliver them; or
//! from a host that reads them itself — an editor that takes keys from
//! winit — which calls [`Unit::key`] before it uses one. Either way they go
//! to an open menu or dialog first, then to the control with the keys.

use std::cell::Cell;
use std::rc::Rc;

use dioxus::html::{Key, Modifiers, NamedKey};
use dioxus::prelude::*;

use crate::layer::LayerState;
use crate::theme::{Accent, Appearance, Density, SystemColours, Theme};

/// What the host answers for the page: the clipboard, for a dialog's field.
pub trait Clipboard {
    fn read(&self) -> Option<String>;
    fn write(&self, text: &str);
}

/// Room at each edge of the window that the page's content keeps clear of,
/// in CSS pixels: a phone's status bar and home indicator.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

/// How keys reach the page's controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Keys {
    /// As DOM events, bubbling to the root: a browser, dioxus-native's
    /// `launch`.
    #[default]
    Page,
    /// From the host, which calls [`Unit::key`] and [`Unit::text`] first and
    /// uses a key itself only when they did not take it.
    Host,
}

/// What a control does with keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Pressed by Return and Space — by the renderer itself where it does
    /// that (a browser), and by the page where it does not (Blitz).
    Button,
    /// Takes its own keys: the arrows of a slider, a radio group, tabs.
    Widget,
    /// A real `<input>`: its keys are the renderer's to type with, and the
    /// host's to hand over ([`Unit::typing`]).
    Text,
}

pub(crate) type KeyFn = Rc<dyn Fn(&Key, Modifiers) -> bool>;
type Close = Rc<dyn Fn()>;

/// The control that has the keys.
#[derive(Clone, Copy)]
pub(crate) struct Focused {
    pub id: u64,
    pub kind: Kind,
    /// Its key handler, as of its last render.
    pub keys: CopyValue<Option<KeyFn>>,
    /// Whether the renderer focused it too (Tab, a browser's press): then a
    /// renderer that presses buttons on Return does so itself.
    pub dom: bool,
}

/// A page's components' shared state, as every component and the host hold
/// it: a cheap, `Copy` handle.
#[derive(Clone, Copy)]
pub struct Unit {
    pub(crate) layer: Signal<LayerState>,
    focused: Signal<Option<u64>>,
    focus: CopyValue<Option<Focused>>,
    /// Set by a control's press, so the root's handler for the same press,
    /// which bubbles to it after, knows not to take the keys away.
    claimed: CopyValue<bool>,
    pub(crate) clipboard: CopyValue<Option<Rc<dyn Clipboard>>>,
    pub(crate) keys: CopyValue<Keys>,
    activates: CopyValue<bool>,
    pub(crate) density: CopyValue<Density>,
    /// The modals open, by id, and what closes each: the last opened first.
    modals: CopyValue<Vec<(u64, Close)>>,
    /// The page's theme and appearance, which an icon's colour is chosen
    /// from (`icon.rs`).
    pub(crate) theme: CopyValue<Theme>,
    pub(crate) appearance: Signal<Appearance>,
    /// The window's safe area, and how far the soft keyboard covers the
    /// page from the bottom: what sheets, dialogs and toasts keep clear of.
    pub(crate) edges: Signal<(Insets, f64)>,
    /// The host's sleep, which times a long press (`hold.rs`).
    pub(crate) timer: CopyValue<Option<crate::components::hold::Sleep>>,
}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        self.layer == other.layer
    }
}

thread_local! {
    static NEXT_ID: Cell<u64> = const { Cell::new(1) };
}

/// A number no other call in this thread returns: an element's `id`.
pub fn next_id() -> u64 {
    NEXT_ID.with(|n| {
        let id = n.get();
        n.set(id + 1);
        id
    })
}

/// The page's [`Unit`], from a component under [`UnitRoot`].
pub fn use_unit() -> Unit {
    use_context::<Unit>()
}

/// Make the page's [`Unit`] and give it to every component under this one,
/// for a page that lays out its own root rather than using [`UnitRoot`]. It
/// is what [`UnitRoot`] calls.
pub fn use_unit_provider(keys: Keys, activates: bool) -> Unit {
    let unit = use_context_provider(|| Unit {
        layer: Signal::new(LayerState::default()),
        focused: Signal::new(None),
        focus: CopyValue::new(None),
        claimed: CopyValue::new(false),
        clipboard: CopyValue::new(None),
        keys: CopyValue::new(keys),
        activates: CopyValue::new(activates),
        density: CopyValue::new(Density::Regular),
        modals: CopyValue::new(Vec::new()),
        theme: CopyValue::new(Theme::default()),
        appearance: Signal::new(Appearance::System),
        edges: Signal::new((Insets::default(), 0.0)),
        timer: CopyValue::new(None),
    });
    unit.keys.clone().set(keys);
    unit.activates.clone().set(activates);
    unit
}

impl Unit {
    /// The page's size, in CSS pixels, which menus are kept inside: set it
    /// whenever the window's changes. The root sets it itself where the
    /// renderer reports its size.
    pub fn set_viewport(&self, width: f64, height: f64) {
        let mut l = self.layer;
        if l.peek().viewport != (width, height) {
            l.write().viewport = (width, height);
        }
    }

    /// The window's safe area, which sheets, dialogs and toasts keep clear
    /// of: set it where the window has one (a phone) and whenever it
    /// changes. Blitz's viewport is the whole window, so the renderer does
    /// not keep the page out of it.
    pub fn set_insets(&self, safe: Insets) {
        let mut e = self.edges;
        if e.peek().0 != safe {
            e.write().0 = safe;
        }
    }

    /// How far the soft keyboard covers the page from its bottom, in CSS
    /// pixels, or 0 with none: a sheet sits above it, and a toast. A host
    /// reads it from the platform (UIKit's keyboard frame) and sets it as
    /// it changes.
    pub fn set_keyboard(&self, height: f64) {
        let mut e = self.edges;
        if e.peek().1 != height {
            e.write().1 = height;
        }
    }

    /// The safe area, and the keyboard's height, as last set.
    pub fn edges(&self) -> (Insets, f64) {
        *self.edges.read()
    }

    /// Where a dialog's field reads and writes the clipboard, when the host
    /// hands it keys ([`Keys::Host`]).
    pub fn set_clipboard(&self, clipboard: Rc<dyn Clipboard>) {
        self.clipboard.clone().set(Some(clipboard));
    }

    /// Whether a menu or a dialog is open.
    pub fn is_open(&self) -> bool {
        self.layer.peek().is_open()
    }

    /// Whether a real text field has the keys: a host that reads keys itself
    /// gives them to the renderer while it does.
    pub fn typing(&self) -> bool {
        self.focus.peek().is_some_and(|f| f.kind == Kind::Text)
    }

    /// Whether the control `id` has the keys.
    pub fn is_focused(&self, id: u64) -> bool {
        *self.focused.read() == Some(id)
    }

    /// The id of the control that has the keys.
    pub fn focused(&self) -> Option<u64> {
        *self.focused.peek()
    }

    pub(crate) fn take_focus(&self, f: Focused) {
        let same = self
            .focus
            .peek()
            .is_some_and(|g| g.id == f.id && g.dom == f.dom);
        self.focus.clone().set(Some(f));
        if !same && *self.focused.peek() != Some(f.id) {
            self.focused.clone().set(Some(f.id));
        }
    }

    /// The control `id` was blurred by the renderer.
    pub(crate) fn blurred(&self, id: u64) {
        if self.focus.peek().is_some_and(|f| f.id == id) {
            self.blur();
        }
    }

    /// No control has the keys. A host calls it when a press goes to
    /// something of its own.
    pub fn blur(&self) {
        if self.focus.peek().is_some() {
            self.focus.clone().set(None);
        }
        if self.focused.peek().is_some() {
            self.focused.clone().set(None);
        }
    }

    /// A press in the page is a control's, and keeps the keys where it put
    /// them.
    pub(crate) fn claim(&self) {
        self.claimed.clone().set(true);
    }

    /// The root's own handler for a press, after any control's.
    pub(crate) fn pressed_root(&self) {
        let claimed = *self.claimed.peek();
        self.claimed.clone().set(false);
        if !claimed {
            self.blur();
        }
    }

    /// A key, before anything else has it: whether the page took it. An open
    /// menu takes every key, as a system menu does, but those it has no use
    /// for: those close it and go on (⌘S with a menu open still saves). A
    /// dialog takes every key. With neither, the control with the keys has
    /// it.
    pub fn key(&self, key: &Key, mods: Modifiers) -> bool {
        if self.layer.peek().is_open() {
            return self.layer_key(key, mods);
        }
        if matches!(key, Key::Named(NamedKey::Escape)) {
            let top = self.modals.peek().last().map(|(_, c)| c.clone());
            if let Some(close) = top {
                close();
                return true;
            }
        }
        let Some(f) = *self.focus.peek() else {
            return false;
        };
        match f.kind {
            Kind::Text => return false,
            Kind::Button => {
                let press = matches!(key, Key::Named(NamedKey::Enter))
                    || matches!(key, Key::Character(c) if c == " ");
                if press && f.dom && *self.activates.peek() {
                    return false;
                }
            }
            Kind::Widget => {}
        }
        if matches!(key, Key::Named(NamedKey::Escape)) {
            self.blur();
            return true;
        }
        let keys = f.keys.peek().clone();
        keys.is_some_and(|k| k(key, mods))
    }

    /// Text from the input method — an accent, a composition's commit, the
    /// soft keyboard's — when the host hands it over: whether the page took
    /// it.
    pub fn text(&self, text: &str) -> bool {
        self.layer_text(text)
    }

    /// The modal `id` is open, closed by `close`, or (`None`) not open.
    pub(crate) fn modal(&self, id: u64, close: Option<Rc<dyn Fn()>>) {
        let mut m = self.modals;
        let at = m.peek().iter().position(|(i, _)| *i == id);
        match (at, close) {
            (Some(at), Some(c)) => m.write()[at].1 = c,
            (None, Some(c)) => m.write().push((id, c)),
            (Some(at), None) => {
                m.write().remove(at);
            }
            (None, None) => {}
        }
    }

    /// The theme and appearance the page is drawn in, for what is coloured
    /// from Rust rather than CSS: an icon. [`UnitRoot`] sets them; a page
    /// that lays out its own root calls it.
    pub fn set_theme(&self, theme: &Theme, appearance: Appearance) {
        if *self.theme.peek() != *theme {
            self.theme.clone().set(theme.clone());
        }
        if *self.appearance.peek() != appearance {
            self.appearance.clone().set(appearance);
        }
    }

    /// The page's stylesheet: `theme`'s properties and every component's
    /// styles. [`UnitRoot`] puts it in the page.
    pub fn stylesheet(theme: &Theme) -> String {
        format!("{}{}", theme.css(), crate::style::CSS)
    }
}

/// A focusable control's link to the page's [`Unit`]: made by
/// [`use_focus`], and wired to the control's element.
#[derive(Clone, Copy, PartialEq)]
pub struct Focus {
    pub id: u64,
    kind: Kind,
    unit: Unit,
    keys: CopyValue<Option<KeyFn>>,
}

/// A control that can have the keys: `keys` is called with each key while it
/// does, and says whether it used it. Call it on every render, so the
/// handler sees the control's latest props.
pub fn use_focus(kind: Kind, keys: impl Fn(&Key, Modifiers) -> bool + 'static) -> Focus {
    let unit = use_unit();
    let f = use_hook(|| Focus {
        id: next_id(),
        kind,
        unit,
        keys: CopyValue::new(None),
    });
    f.keys.clone().set(Some(Rc::new(keys)));
    // A control taken out of the page gives up the keys: its handler is
    // dropped with it, and a key for it would find nothing to call.
    use_drop(move || f.unit.blurred(f.id));
    f
}

impl Focus {
    /// Whether this control has the keys — read, so the control redraws its
    /// ring when that changes.
    pub fn is_focused(&self) -> bool {
        self.unit.is_focused(self.id)
    }

    fn focused(&self, dom: bool) -> Focused {
        Focused {
            id: self.id,
            kind: self.kind,
            keys: self.keys,
            dom,
        }
    }

    /// The control was pressed: it has the keys.
    pub fn press(&self) {
        self.unit.claim();
        let dom = self
            .unit
            .focus
            .peek()
            .is_some_and(|f| f.id == self.id && f.dom);
        self.unit.take_focus(self.focused(dom));
    }

    /// The control was pressed, and refuses the keys: they stay with
    /// whatever had them, which may be the host's own.
    pub fn press_refused(&self) {
        self.unit.claim();
    }

    /// The renderer focused the control.
    pub fn dom_focus(&self) {
        self.unit.take_focus(self.focused(true));
    }

    /// The renderer blurred the control.
    pub fn dom_blur(&self) {
        self.unit.blurred(self.id);
    }

    /// The class the control's element adds while it has the keys.
    pub fn class(&self) -> &'static str {
        if self.is_focused() {
            " unit-focused"
        } else {
            ""
        }
    }
}

/// The system's colours, light and dark, as a page follows them; `None`
/// where unit reads none.
type Colours = Signal<Option<(SystemColours, SystemColours)>>;

#[cfg(all(feature = "apple", target_os = "macos"))]
fn use_system_colours() -> Option<Colours> {
    Some(crate::macos::use_colours())
}

#[cfg(not(all(feature = "apple", target_os = "macos")))]
fn use_system_colours() -> Option<Colours> {
    None
}

/// The root of a page of components: the theme's stylesheet, the page, and
/// over it the overlay where menus, dialogs and toasts are drawn. Everything
/// that uses a component goes under it.
#[component]
pub fn UnitRoot(
    /// How keys reach the controls.
    #[props(default)]
    keys: Keys,
    #[props(default)] appearance: Appearance,
    #[props(default)] density: Density,
    /// Where the accent comes from: the theme's, or the system's where unit
    /// can read one ([`Accent::System`]).
    #[props(default)]
    accent: Accent,
    /// Whether the renderer presses a focused button on Return and Space
    /// itself, as a browser does and Blitz does not.
    #[props(default = cfg!(target_arch = "wasm32"))]
    activates: bool,
    /// The theme; [`Theme::default`] unless given.
    theme: Option<Theme>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let unit = use_unit_provider(keys, activates);
    unit.density.clone().set(density);
    let system = use_system_colours();
    let mut theme = theme.unwrap_or_default();
    if accent == Accent::System
        && let Some((light, dark)) = system.and_then(|colours| colours())
    {
        theme.light = theme.light.with(light);
        theme.dark = theme.dark.with(dark);
    }
    unit.set_theme(&theme, appearance);
    let css = use_memo(use_reactive!(|theme| Unit::stylesheet(&theme)));
    let page_keys = keys == Keys::Page;
    rsx! {
        style { {css()} }
        div {
            class: "unit-root {class}",
            "data-appearance": appearance.attr(),
            "data-density": density.attr(),
            // A browser sends keys to the focused element, and to the body
            // with none: a press inside the page focuses the root, so its
            // keys reach it here. Blitz focuses nothing on a press but a
            // field, and sends keys to the document's root element with
            // nothing focused: a host that needs every key reads them itself.
            tabindex: page_keys.then_some("-1"),
            onpointerdown: move |_| unit.pressed_root(),
            onkeydown: move |e: KeyboardEvent| {
                if page_keys && unit.key(&e.key(), e.modifiers()) {
                    e.prevent_default();
                    e.stop_propagation();
                }
            },
            onresize: move |e: ResizeEvent| {
                if let Ok(size) = e.get_border_box_size() {
                    unit.set_viewport(size.width, size.height);
                }
            },
            {children}
            {crate::layer::render(unit)}
        }
    }
}

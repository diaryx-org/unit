//! Controls that open a menu: a button, a select, and an area with a context
//! menu — and chips, a row of choices that each toggle.
//!
//! A menu opens on the press, not the click, so it can be chosen from by
//! dragging to an item and letting go, as a Mac's can; a click that no press
//! began — Return on a browser's focused button — opens it too. Where it
//! opens is read from the pointer's event (where the element starts, in the
//! page); opened by a key, it opens where the pointer last found the
//! control.

use dioxus::html::{Key, NamedKey};
use dioxus::prelude::*;

use crate::components::button::{Variant, is_press};
use crate::components::hold::use_hold;
use crate::icon::Icon;
use crate::menu::{Entry, Item};
use crate::unit::{Kind, Unit, use_focus, use_unit};

/// Where the element an event is on starts, in the page: the pointer less
/// its offset into the element.
pub fn left_top(e: &PointerEvent) -> (f64, f64) {
    let page = e.client_coordinates();
    let inside = e.element_coordinates();
    // A renderer that reports no offset gives the pointer itself.
    let ok = |v: f64| (0.0..4000.0).contains(&v);
    let x = if ok(inside.x) {
        page.x - inside.x
    } else {
        page.x
    };
    let y = if ok(inside.y) {
        page.y - inside.y
    } else {
        page.y
    };
    (x, y)
}

/// A control that opens a menu, and where.
#[derive(Clone, Copy)]
struct Opener {
    unit: Unit,
    /// Where the control starts, as the pointer last found it.
    at: CopyValue<Option<(f64, f64)>>,
    /// A press opened the menu, and the click that ends it opens nothing.
    pressed: CopyValue<bool>,
    /// The control's focus id, which the menu is read as opened by.
    owner: CopyValue<u64>,
}

impl Opener {
    fn new(unit: Unit) -> Self {
        Self {
            unit,
            at: CopyValue::new(None),
            pressed: CopyValue::new(false),
            owner: CopyValue::new(0),
        }
    }

    fn seen(&self, e: &PointerEvent) {
        self.at.clone().set(Some(left_top(e)));
    }

    /// Open `entries` under the control, `height` tall; with `first`, its
    /// first item highlighted, as a menu opened by a key is.
    fn open(
        &self,
        entries: Vec<Entry>,
        height: f64,
        top: Option<f64>,
        first: bool,
        pick: EventHandler<usize>,
    ) {
        let (x, y) = self.at.peek().unwrap_or_else(|| {
            let (w, h) = self.unit.layer.peek().viewport;
            ((w / 2.0 - 80.0).max(4.0), (h / 3.0).max(4.0))
        });
        let top = top.unwrap_or(y + height);
        let owner = *self.owner.peek();
        self.unit
            .open_menu_from(owner, entries, (x, top), height, move |i| pick.call(i));
        if first {
            self.unit.hot_first();
        }
    }
}

/// A button that opens `entries` under itself; `onpick` is given the chosen
/// item's id. `top` is where the menu's top goes, in the page — the bottom
/// edge of the bar the button sits in, where the host knows it — and by
/// default the button's own bottom.
#[component]
pub fn MenuButton(
    entries: Vec<Entry>,
    onpick: EventHandler<usize>,
    #[props(default)] class: String,
    /// The button's height, in CSS pixels: how far below its top the menu
    /// opens.
    #[props(default = 26.0)]
    height: f64,
    top: Option<f64>,
    label: Option<String>,
    /// An icon before the children, as [`crate::Button`]'s.
    icon: Option<&'static str>,
    #[props(default)] disabled: bool,
    /// Takes no keys, as [`crate::Button`]'s `refuses_keys`: the menu
    /// still opens on a press, and its keys are the menu's while it is open.
    #[props(default)]
    refuses_keys: bool,
    children: Element,
) -> Element {
    let unit = use_unit();
    let opener = use_hook(|| Opener::new(unit));
    let menu = entries.clone();
    let focus = use_focus(Kind::Button, move |key, _| {
        let open = is_press(key) || matches!(key, Key::Named(NamedKey::ArrowDown));
        if open && !disabled {
            opener.open(menu.clone(), height, top, true, onpick);
            return true;
        }
        false
    });
    opener.owner.clone().set(focus.id);
    let expanded = unit.layer.read().menu_owner() == Some(focus.id);
    let entries2 = entries.clone();
    rsx! {
        button {
            class: "unit-button unit-menu-button{focus.class()} {class}",
            r#type: "button",
            disabled,
            "aria-haspopup": "menu",
            "aria-expanded": if expanded { "true" } else { "false" },
            "aria-label": label,
            tabindex: refuses_keys.then_some("-1"),
            onpointerenter: move |e: PointerEvent| opener.seen(&e),
            onpointermove: move |e: PointerEvent| opener.seen(&e),
            // Focus left where it was, as `Button`'s `refuses_keys` leaves it.
            onmousedown: move |e: MouseEvent| if refuses_keys { e.prevent_default() },
            onpointerdown: move |e: PointerEvent| {
                if refuses_keys {
                    focus.press_refused();
                } else {
                    focus.press();
                }
                if disabled {
                    return;
                }
                // Not cancelled: a browser focuses the button on the
                // `mousedown` a cancelled press never sends, and its keys
                // reach the page from there.
                opener.seen(&e);
                opener.pressed.clone().set(true);
                opener.open(entries.clone(), height, top, false, onpick);
            },
            onfocus: move |_| if !refuses_keys { focus.dom_focus() },
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                e.stop_propagation();
                if refuses_keys {
                    // Blitz focuses what a click lands on as the click's
                    // default action.
                    e.prevent_default();
                }
                if std::mem::take(&mut *opener.pressed.clone().write()) || disabled {
                    return;
                }
                opener.open(entries2.clone(), height, top, true, onpick);
            },
            if let Some(d) = icon {
                Icon { d, ink: Variant::Default.ink(disabled) }
            }
            {children}
        }
    }
}

/// One of a list of options, as a select or chips show it.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub label: String,
    /// A style for its label: a font's chip in that font.
    pub style: Option<String>,
    /// A colour's dot before a chip's label, a CSS colour: the circle or
    /// the ink it names.
    pub dot: Option<String>,
    /// A few words after a chip's label, a count: how many it holds. Read
    /// after the label.
    pub badge: Option<String>,
}

impl Choice {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            style: None,
            dot: None,
            badge: None,
        }
    }

    pub fn dot(mut self, colour: impl Into<String>) -> Self {
        self.dot = Some(colour.into());
        self
    }

    pub fn badge(mut self, text: impl Into<String>) -> Self {
        self.badge = Some(text.into());
        self
    }

    pub fn styled(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }
}

impl<S: Into<String>> From<S> for Choice {
    fn from(s: S) -> Self {
        Choice::new(s)
    }
}

/// A select: the chosen option's label, and a menu of them all, ticked at
/// the chosen one. `onpick` is given the index chosen.
#[component]
pub fn Select(
    choices: Vec<Choice>,
    selected: Option<usize>,
    onpick: EventHandler<usize>,
    /// What it chooses, for assistive tech; the chosen label follows it.
    label: Option<String>,
    #[props(default = 24.0)] height: f64,
    #[props(default)] disabled: bool,
) -> Element {
    let shown = selected
        .and_then(|i| choices.get(i))
        .map_or(String::new(), |c| c.label.clone());
    let entries: Vec<Entry> = choices
        .iter()
        .enumerate()
        .map(|(i, c)| {
            Item::new(i, c.label.clone())
                .checked(selected == Some(i))
                .into()
        })
        .collect();
    let name = match &label {
        Some(l) => format!("{l}: {shown}"),
        None => shown.clone(),
    };
    rsx! {
        MenuButton { class: "unit-select", entries, onpick, height, label: name, disabled,
            span { class: "unit-select-label", "{shown}" }
            span { class: "unit-select-arrows", "aria-hidden": "true", "▾" }
        }
    }
}

/// An area whose context menu is `entries`: opened by the secondary button,
/// a two-finger click, or a finger held on it ([`crate::LongPress`]'s
/// hold), at the pointer. `onopen` is called first, with where, so the page
/// can choose what the menu holds.
#[component]
pub fn ContextMenu(
    entries: Vec<Entry>,
    onpick: EventHandler<usize>,
    onopen: Option<EventHandler<(f64, f64)>>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let unit = use_unit();
    let menu = entries.clone();
    let open = move |(x, y): (f64, f64), entries: Vec<Entry>| {
        if let Some(o) = onopen {
            o.call((x, y));
        }
        unit.open_menu(entries, (x, y), 0.0, move |i| onpick.call(i));
    };
    let hold = use_hold(move |at| open(at, menu.clone()));
    rsx! {
        div { class: "unit-context {class}",
            oncontextmenu: move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                let p = e.client_coordinates();
                open((p.x, p.y), entries.clone());
            },
            onpointerdown: move |e: PointerEvent| hold.down(&e),
            onpointermove: move |e: PointerEvent| hold.moved(&e),
            onpointerup: move |e: PointerEvent| hold.up(&e),
            onpointercancel: move |e: PointerEvent| hold.up(&e),
            onclick: move |e: MouseEvent| {
                if hold.take_held() {
                    e.stop_propagation();
                }
            },
            {children}
        }
    }
}

/// Chips: each option a pill that toggles, the ones `on` says lit. `onpick`
/// is given the index pressed.
#[component]
pub fn Chips(
    choices: Vec<Choice>,
    on: Vec<bool>,
    onpick: EventHandler<usize>,
    label: Option<String>,
) -> Element {
    rsx! {
        div { class: "unit-chips", role: "group", "aria-label": label,
            for (i, c) in choices.into_iter().enumerate() {
                Chip { key: "{i}", choice: c, on: on.get(i).copied().unwrap_or(false),
                       onpress: move |_| onpick.call(i) }
            }
        }
    }
}

#[component]
fn Chip(choice: Choice, on: bool, onpress: EventHandler<()>) -> Element {
    let focus = use_focus(Kind::Button, move |key, _| {
        if is_press(key) {
            onpress.call(());
            return true;
        }
        false
    });
    let marked = if choice.dot.is_some() || choice.badge.is_some() {
        " unit-marked"
    } else {
        ""
    };
    let ring = focus.class();
    let pressed = on;
    let on = if on { " on" } else { "" };
    rsx! {
        button {
            class: "unit-chip{on}{marked}{ring}",
            r#type: "button",
            "aria-pressed": if pressed { "true" } else { "false" },
            "aria-label": choice.badge.as_ref().map(|b| format!("{}, {b}", choice.label)),
            style: choice.style.clone().unwrap_or_default(),
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                e.stop_propagation();
                onpress.call(());
            },
            if let Some(c) = &choice.dot {
                span { class: "unit-chip-dot", "aria-hidden": "true", style: "background: {c}" }
            }
            "{choice.label}"
            if let Some(b) = &choice.badge {
                span { class: "unit-chip-badge", "{b}" }
            }
        }
    }
}

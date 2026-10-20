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

use crate::components::button::is_press;
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
    #[props(default)] disabled: bool,
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
            onpointerenter: move |e: PointerEvent| opener.seen(&e),
            onpointermove: move |e: PointerEvent| opener.seen(&e),
            onpointerdown: move |e: PointerEvent| {
                focus.press();
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
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                e.stop_propagation();
                if std::mem::take(&mut *opener.pressed.clone().write()) || disabled {
                    return;
                }
                opener.open(entries2.clone(), height, top, true, onpick);
            },
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
}

impl Choice {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            style: None,
        }
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
/// or a two-finger click, at the pointer. `onopen` is called first, with
/// where, so the page can choose what the menu holds.
#[component]
pub fn ContextMenu(
    entries: Vec<Entry>,
    onpick: EventHandler<usize>,
    onopen: Option<EventHandler<(f64, f64)>>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let unit = use_unit();
    rsx! {
        div { class: "unit-context {class}",
            oncontextmenu: move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                let p = e.client_coordinates();
                if let Some(o) = onopen {
                    o.call((p.x, p.y));
                }
                unit.open_menu(entries.clone(), (p.x, p.y), 0.0, move |i| onpick.call(i));
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
    rsx! {
        button {
            class: if on { "unit-chip on{focus.class()}" } else { "unit-chip{focus.class()}" },
            r#type: "button",
            "aria-pressed": if on { "true" } else { "false" },
            style: choice.style.clone().unwrap_or_default(),
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            onclick: move |e: MouseEvent| {
                e.stop_propagation();
                onpress.call(());
            },
            "{choice.label}"
        }
    }
}

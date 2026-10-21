//! The overlay: one per page, over everything else, where its menus, its
//! dialog and its toasts are drawn.
//!
//! It is mounted with the page and never unmounted: its backdrop, a fixed
//! number of menu levels, the dialog and the toasts' stack are always in the
//! tree, shown or hidden, and moved by their style. Blitz lays out an
//! absolutely positioned element on the first paint and not when a later
//! frame inserts it, and skips painting a child that hangs past its parent's
//! box — so an overlay made when a menu opens, inside the button that opened
//! it, would not be drawn. One made at the page's root from the start is.
//!
//! The overlay itself is never hidden: showing a box in the same frame as
//! rows arrive in an absolutely positioned child of it leaves blitz-dom
//! 0.3.0-beta.4 a freed node in its layout tree, and every layout after
//! panics. It lets the pointer through instead (`pointer-events: none`), and
//! what it holds is shown and hidden.
//!
//! Everything here acts on pointer events, not mouse events: Blitz sends a
//! finger's press as pointer and touch events and no mouse events.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use dioxus::html::{Key, Modifiers, NamedKey};
use dioxus::prelude::*;

use crate::actions::{ActionSheet, SheetAction};
use crate::dialog::{Answer, Dialog, Field, FieldKey};
use crate::icon::{Icon, Ink};
use crate::menu::{Entry, Level, Menu, Outcome, Sizes};
use crate::unit::{Keys, Unit, next_id};

/// How many levels deep a menu may go: the first and three submenus.
pub const LEVELS: usize = 4;

/// How many toasts are shown at once; older ones wait.
pub const TOASTS: usize = 3;

type Pick = Rc<dyn Fn(usize)>;
type Done = Rc<dyn Fn(Answer)>;
type Action = Rc<dyn Fn()>;
type Choose = Rc<dyn Fn(Option<usize>)>;

/// A short notice at the bottom of the page, until it is dismissed or, with
/// `lasts`, until that long has gone by.
#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub message: String,
    /// A button beside the message, and the id [`Unit::toast`]'s handler is
    /// given when it is pressed.
    pub action: Option<String>,
    pub tone: Tone,
    /// How long it is shown, timed by the host's sleep
    /// ([`Unit::set_timer`]); with none, or no sleep, until it is dismissed.
    pub lasts: Option<Duration>,
}

/// What a toast or a badge says about its message.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Neutral,
    Success,
    Warning,
    Danger,
}

impl Tone {
    pub(crate) fn class(self) -> &'static str {
        match self {
            Tone::Neutral => "",
            Tone::Success => " unit-success",
            Tone::Warning => " unit-warning",
            Tone::Danger => " unit-danger",
        }
    }
}

impl Toast {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            action: None,
            tone: Tone::Neutral,
            lasts: None,
        }
    }

    /// Gone by itself after `time`: an Undo offered for a few seconds.
    pub fn lasts(mut self, time: Duration) -> Self {
        self.lasts = Some(time);
        self
    }

    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.action = Some(label.into());
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}

/// What the overlay shows.
#[derive(Clone, Default)]
pub(crate) struct LayerState {
    menu: Option<(Menu, Pick)>,
    /// The control that opened the menu, by its focus id: read out as
    /// expanded.
    owner: Option<u64>,
    dialog: Option<(Dialog, Done)>,
    actions: Option<(ActionSheet, Choose)>,
    /// Each toast, its id and its action's handler, oldest first.
    toasts: Vec<(u64, Toast, Option<Action>)>,
    /// The page's size, which a menu is kept inside.
    pub viewport: (f64, f64),
}

impl LayerState {
    pub fn is_open(&self) -> bool {
        self.menu.is_some() || self.dialog.is_some() || self.actions.is_some()
    }

    /// The control whose menu is open.
    pub fn menu_owner(&self) -> Option<u64> {
        self.menu.as_ref().and(self.owner)
    }
}

impl Unit {
    /// Open `entries` with the first level's top-left at `at`; `above` is
    /// the height of the control it opened under, which it opens above
    /// instead where there is no room below. `pick` is given the chosen
    /// item's id, after the menu has closed.
    pub fn open_menu(
        &self,
        entries: Vec<Entry>,
        at: (f64, f64),
        above: f64,
        pick: impl FnMut(usize) + 'static,
    ) {
        self.open_menu_from(0, entries, at, above, pick);
    }

    /// [`Unit::open_menu`], for the control `owner`.
    pub(crate) fn open_menu_from(
        &self,
        owner: u64,
        entries: Vec<Entry>,
        at: (f64, f64),
        above: f64,
        pick: impl FnMut(usize) + 'static,
    ) {
        let pick = RefCell::new(pick);
        let mut l = self.layer;
        let viewport = l.peek().viewport;
        let viewport = if viewport.0 > 0.0 {
            viewport
        } else {
            (1e5, 1e5)
        };
        let sizes = match *self.density.peek() {
            crate::Density::Regular => Sizes::REGULAR,
            crate::Density::Touch => Sizes::TOUCH,
            crate::Density::Cell => Sizes::CELL,
        };
        let menu = Menu::open(entries, at, above, viewport, sizes);
        let mut w = l.write();
        w.menu = Some((menu, Rc::new(move |i| (pick.borrow_mut())(i))));
        w.owner = (owner != 0).then_some(owner);
    }

    /// Highlight the open menu's first item, as a menu opened by a key does:
    /// for a host that opens [`Unit::open_menu`] from a key of its own.
    pub fn hot_first(&self) {
        let mut l = self.layer;
        if let Some((m, _)) = l.write().menu.as_mut() {
            m.hot_first();
        }
    }

    /// Open `dialog`; `done` is given how it ended, after it has closed.
    pub fn open_dialog(&self, dialog: Dialog, done: impl FnMut(Answer) + 'static) {
        let done = RefCell::new(done);
        let mut l = self.layer;
        let mut w = l.write();
        w.menu = None;
        w.dialog = Some((dialog, Rc::new(move |a| (done.borrow_mut())(a))));
    }

    /// Open `sheet`; `choose` is given the chosen action's id, or `None`
    /// when it was cancelled, after it has closed.
    pub fn open_actions(&self, sheet: ActionSheet, choose: impl FnMut(Option<usize>) + 'static) {
        let choose = RefCell::new(choose);
        let mut l = self.layer;
        let mut w = l.write();
        w.menu = None;
        w.actions = Some((sheet, Rc::new(move |a| (choose.borrow_mut())(a))));
    }

    fn step_actions(&self, down: bool) {
        let mut l = self.layer;
        if let Some((s, _)) = l.write().actions.as_mut() {
            s.step(down);
        }
    }

    /// Choose what the keys have highlighted, if anything.
    fn choose_hot(&self) {
        let chosen = self
            .layer
            .peek()
            .actions
            .as_ref()
            .and_then(|(s, _)| s.chosen());
        if let Some(c) = chosen {
            self.choose(c);
        }
    }

    fn choose(&self, action: Option<usize>) {
        let mut l = self.layer;
        let choose = l.write().actions.take().map(|(_, c)| c);
        if let Some(c) = choose {
            c(action);
        }
    }

    /// Close whatever is open, choosing nothing.
    pub fn close(&self) {
        let mut l = self.layer;
        if l.peek().menu.is_some() {
            l.write().menu = None;
        } else if l.peek().actions.is_some() {
            self.choose(None);
        } else if l.peek().dialog.is_some() {
            self.answer(Answer::Cancel);
        }
    }

    /// Show `toast` until it is dismissed. The id dismisses it from code
    /// ([`Unit::dismiss`]).
    pub fn toast(&self, toast: Toast) -> u64 {
        self.push_toast(toast, None)
    }

    /// Show `toast`, and call `action` when its action is pressed, which
    /// dismisses it too.
    pub fn toast_with(&self, toast: Toast, action: impl FnMut() + 'static) -> u64 {
        let action = RefCell::new(action);
        self.push_toast(toast, Some(Rc::new(move || (action.borrow_mut())())))
    }

    fn push_toast(&self, toast: Toast, action: Option<Action>) -> u64 {
        let id = next_id();
        let lasts = toast.lasts;
        self.layer.clone().write().toasts.push((id, toast, action));
        let sleep = self.timer.peek().clone();
        if let (Some(time), Some(sleep)) = (lasts, sleep) {
            let unit = *self;
            // The root's, not the caller's: the sheet that showed it may
            // be gone before the time is up.
            dioxus::core::spawn_forever(async move {
                sleep(time).await;
                unit.dismiss(id);
            });
        }
        id
    }

    /// Take the toast `id` away.
    pub fn dismiss(&self, id: u64) {
        let mut l = self.layer;
        if l.peek().toasts.iter().any(|(t, ..)| *t == id) {
            l.write().toasts.retain(|(t, ..)| *t != id);
        }
    }

    pub(crate) fn layer_key(&self, key: &Key, mods: Modifiers) -> bool {
        let mut l = self.layer;
        if l.peek().menu.is_some() {
            let outcome = l
                .write()
                .menu
                .as_mut()
                .map_or(Outcome::Ignored, |(m, _)| m.key(key, mods));
            return self.settle(outcome);
        }
        if l.peek().actions.is_some() {
            let shift = mods.contains(Modifiers::SHIFT);
            match key {
                Key::Named(NamedKey::Escape) => self.choose(None),
                Key::Named(NamedKey::ArrowDown) => self.step_actions(true),
                Key::Named(NamedKey::ArrowUp) => self.step_actions(false),
                Key::Named(NamedKey::Tab) => self.step_actions(!shift),
                Key::Named(NamedKey::Enter) => self.choose_hot(),
                Key::Character(c) if c == " " => self.choose_hot(),
                _ => {}
            }
            return true;
        }
        if l.peek().dialog.is_none() {
            return false;
        }
        match key {
            Key::Named(NamedKey::Escape) => self.answer(Answer::Cancel),
            Key::Named(NamedKey::Enter) => {
                let text = l
                    .peek()
                    .dialog
                    .as_ref()
                    .map(|(d, _)| d.text())
                    .unwrap_or_default();
                self.answer(Answer::Ok(text));
            }
            // A real `<input>` types for itself.
            _ if *self.keys.peek() == Keys::Page => return false,
            _ => {
                let outcome = l
                    .write()
                    .dialog
                    .as_mut()
                    .and_then(|(d, _)| d.field.as_mut())
                    .map(|f| f.key(key, mods));
                match outcome {
                    Some(FieldKey::Paste) => {
                        if let Some(t) = self.clipboard.peek().as_ref().and_then(|c| c.read()) {
                            self.type_text(&t);
                        }
                    }
                    Some(FieldKey::Copy(t)) => {
                        if let Some(c) = self.clipboard.peek().as_ref() {
                            c.write(&t);
                        }
                    }
                    _ => {}
                }
            }
        }
        true
    }

    pub(crate) fn layer_text(&self, text: &str) -> bool {
        let l = self.layer.peek();
        if l.dialog.is_none() {
            return l.menu.is_some() || l.actions.is_some();
        }
        drop(l);
        self.type_text(text);
        true
    }

    fn type_text(&self, text: &str) {
        let mut l = self.layer;
        if let Some(f) = l
            .write()
            .dialog
            .as_mut()
            .and_then(|(d, _)| d.field.as_mut())
        {
            f.insert(text);
        }
    }

    /// Carry out what the menu said.
    fn settle(&self, outcome: Outcome) -> bool {
        let mut l = self.layer;
        match outcome {
            Outcome::Ignored => false,
            Outcome::Handled => true,
            Outcome::Close => {
                l.write().menu = None;
                true
            }
            Outcome::Pick(id) => {
                let pick = l.write().menu.take().map(|(_, p)| p);
                if let Some(p) = pick {
                    p(id);
                }
                true
            }
        }
    }

    fn answer(&self, a: Answer) {
        let mut l = self.layer;
        let done = l.write().dialog.take().map(|(_, d)| d);
        if let Some(d) = done {
            d(a);
        }
    }
}

/// The overlay's markup: last in the root's.
pub(crate) fn render(unit: Unit) -> Element {
    rsx! { LayerView { unit } }
}

#[component]
fn LayerView(unit: Unit) -> Element {
    let l = unit.layer.read();
    let menu = l.menu.is_some();
    let levels: Vec<Option<Level>> = (0..LEVELS)
        .map(|i| l.menu.as_ref().and_then(|(m, _)| m.levels.get(i).cloned()))
        .collect();
    let dialog = l.dialog.as_ref().map(|(d, _)| d.clone());
    let actions = l.actions.as_ref().map(|(a, _)| a.clone());
    let toasts: Vec<(u64, Toast)> = l
        .toasts
        .iter()
        .take(TOASTS)
        .map(|(id, t, _)| (*id, t.clone()))
        .collect();
    drop(l);
    let (safe, keyboard) = unit.edges();
    let toasts_bottom = safe.bottom.max(keyboard) + 16.0;
    let backdrop = if menu || dialog.is_some() || actions.is_some() {
        "block"
    } else {
        "none"
    };
    let scrim = if dialog.is_some() || actions.is_some() {
        " unit-scrim"
    } else {
        ""
    };
    // The layer itself is always displayed, and lets the pointer through to
    // the page; what it holds is shown and hidden.
    rsx! {
        div { class: "unit-layer",
            // A press anywhere outside a menu closes it, and is not also a
            // press on whatever is under it.
            div { class: "unit-backdrop{scrim}", style: "display: {backdrop}",
                "aria-hidden": "true",
                onpointerdown: move |e: PointerEvent| {
                    e.prevent_default();
                    e.stop_propagation();
                    let mut l = unit.layer;
                    if l.peek().menu.is_some() {
                        l.write().menu = None;
                    } else if l.peek().actions.is_some() {
                        unit.choose(None);
                    }
                },
            }
            for (depth, level) in levels.into_iter().enumerate() {
                MenuLevel { key: "{depth}", unit, depth, level }
            }
            DialogView { unit, dialog, top: safe.top }
            ActionsView { unit, sheet: actions, bottom: safe.bottom }
            div { class: "unit-toasts", role: "status", "aria-live": "polite",
                style: "bottom: {toasts_bottom}px",
                for (id, t) in toasts {
                    ToastView { key: "{id}", unit, id, toast: t }
                }
            }
        }
    }
}

#[component]
fn MenuLevel(unit: Unit, depth: usize, level: Option<Level>) -> Element {
    // One template whether the level is open or not: Blitz does not lay out
    // an absolutely positioned element a later frame inserts, so this one is
    // only ever restyled, and its items changed.
    let (style, entries, hot) = match &level {
        Some(l) => (
            format!(
                "display: block; left: {}px; top: {}px; width: {}px",
                l.left, l.top, l.width
            ),
            l.entries.clone(),
            l.hot,
        ),
        None => ("display: none".to_string(), Vec::new(), None),
    };
    rsx! {
        div { class: "unit-menu", role: "menu", style,
            "aria-hidden": if level.is_none() { "true" } else { "false" },
            onpointerdown: move |e: PointerEvent| {
                e.prevent_default();
                e.stop_propagation();
                let mut l = unit.layer;
                if let Some((m, _)) = l.write().menu.as_mut() {
                    m.press();
                }
            },
            for (i, entry) in entries.iter().enumerate() {
                {entry_view(unit, depth, i, entry, hot == Some(i))}
            }
        }
    }
}

fn entry_view(unit: Unit, depth: usize, i: usize, entry: &Entry, hot: bool) -> Element {
    let hover = move |_: PointerEvent| {
        let mut l = unit.layer;
        let now = l
            .peek()
            .menu
            .as_ref()
            .and_then(|(m, _)| m.levels.get(depth).map(|l| l.hot));
        let armed = l.peek().menu.as_ref().is_some_and(|(m, _)| m.armed);
        if (now != Some(Some(i)) || !armed)
            && let Some((m, _)) = l.write().menu.as_mut()
        {
            m.hover(depth, i);
        }
    };
    let release = move |e: PointerEvent| {
        e.stop_propagation();
        let mut l = unit.layer;
        let outcome = l
            .write()
            .menu
            .as_mut()
            .map_or(Outcome::Ignored, |(m, _)| m.release(depth, i));
        unit.settle(outcome);
    };
    let class = |enabled: bool, destructive: bool| {
        format!(
            "unit-item{}{}{}",
            if hot { " hot" } else { "" },
            if enabled { "" } else { " off" },
            if destructive { " danger" } else { "" }
        )
    };
    match entry {
        Entry::Separator => rsx! {
            div { class: "unit-sep", role: "separator", div { class: "unit-rule" } }
        },
        Entry::Item(item) => {
            let tick = if item.checked { "✓" } else { "" };
            let shortcut = item.shortcut.clone().unwrap_or_default();
            let role = if item.checkable {
                "menuitemcheckbox"
            } else {
                "menuitem"
            };
            let ink = match () {
                _ if !item.enabled => Ink::Dim,
                _ if hot => Ink::AccentText,
                _ if item.destructive => Ink::Danger,
                _ => Ink::Text,
            };
            rsx! {
                div { class: class(item.enabled, item.destructive), role,
                    "aria-checked": item.checkable.then(|| item.checked.to_string()),
                    "aria-disabled": (!item.enabled).then_some("true"),
                    "aria-label": "{item.label}",
                    onpointermove: hover, onpointerup: release,
                    span { class: "unit-tick", "aria-hidden": "true", "{tick}" }
                    if let Some(d) = item.icon {
                        Icon { d, ink, size: 16.0 }
                    }
                    span { class: "unit-label", "{item.label}" }
                    span { class: "unit-key", "aria-hidden": "true", "{shortcut}" }
                }
            }
        }
        Entry::Submenu { label, enabled, .. } => rsx! {
            div { class: class(*enabled, false), role: "menuitem", "aria-haspopup": "menu",
                "aria-disabled": (!enabled).then_some("true"),
                "aria-label": "{label}",
                onpointermove: hover, onpointerup: release,
                span { class: "unit-tick" }
                span { class: "unit-label", "{label}" }
                span { class: "unit-key", "aria-hidden": "true", "▸" }
            }
        },
    }
}

#[component]
fn DialogView(unit: Unit, dialog: Option<Dialog>, top: f64) -> Element {
    // One template open or closed, for the reason a menu level has one.
    let ids = use_hook(|| (next_id(), next_id()));
    let display = if dialog.is_some() { "flex" } else { "none" };
    let d = dialog.unwrap_or_else(|| Dialog::new(""));
    let page = *unit.keys.peek() == Keys::Page;
    let button = move |label: String, ok: bool, destructive: bool| {
        let class = match (ok, destructive) {
            (true, true) => "unit-button unit-danger",
            (true, false) => "unit-button unit-primary",
            _ => "unit-button",
        };
        rsx! {
            button { class, r#type: "button",
                onpointerdown: move |e: PointerEvent| {
                    e.stop_propagation();
                },
                onclick: move |e: MouseEvent| {
                    e.stop_propagation();
                    if ok {
                        let text = unit.layer.peek().dialog.as_ref()
                            .map(|(d, _)| d.text()).unwrap_or_default();
                        unit.answer(Answer::Ok(text));
                    } else {
                        unit.answer(Answer::Cancel);
                    }
                },
                "{label}"
            }
        }
    };
    let (title_id, message_id) = (format!("unit-{}", ids.0), format!("unit-{}", ids.1));
    rsx! {
        div { class: "unit-dialog-place", style: "display: {display}; padding-top: {top + 72.0}px",
            div { class: "unit-dialog", role: "dialog", "aria-modal": "true",
                "aria-labelledby": "{title_id}",
                "aria-describedby": d.message.is_some().then(|| message_id.clone()),
                onpointerdown: move |e: PointerEvent| {
                    e.stop_propagation();
                },
                div { class: "unit-dialog-title", id: "{title_id}", "{d.title}" }
                if let Some(m) = &d.message {
                    div { class: "unit-dialog-message", id: "{message_id}", "{m}" }
                }
                if let Some(f) = &d.field {
                    if page {
                        input { class: "unit-input", r#type: "text",
                            value: "{f.text}", placeholder: "{f.placeholder}",
                            "aria-label": "{d.title}",
                            oninput: move |e: FormEvent| {
                                let mut l = unit.layer;
                                if let Some(f) = l.write().dialog.as_mut().and_then(|(d, _)| d.field.as_mut()) {
                                    f.set(e.value());
                                }
                            },
                        }
                    } else {
                        {field_view(f, &d.title)}
                    }
                }
                div { class: "unit-dialog-buttons",
                    if let Some(c) = d.cancel.clone() {
                        {button(c, false, false)}
                    }
                    {button(d.ok.clone(), true, d.destructive)}
                }
            }
        }
    }
}

#[component]
fn ActionsView(unit: Unit, sheet: Option<ActionSheet>, bottom: f64) -> Element {
    // One template open or closed, for the reason a menu level has one.
    let title_id = use_hook(|| format!("unit-{}", next_id()));
    let display = if sheet.is_some() { "flex" } else { "none" };
    let s = sheet.unwrap_or_else(|| ActionSheet::new(Vec::new()));
    rsx! {
        div { class: "unit-actions-place", style: "display: {display}; padding-bottom: {bottom + 8.0}px",
            div { class: "unit-actions", role: "dialog", "aria-modal": "true",
                "aria-labelledby": s.title.is_some().then(|| title_id.clone()),
                onpointerdown: move |e: PointerEvent| e.stop_propagation(),
                div { class: "unit-actions-group", role: "menu",
                    if s.title.is_some() || s.message.is_some() {
                        div { class: "unit-actions-head",
                            if let Some(t) = &s.title {
                                div { class: "unit-actions-title", id: "{title_id}", "{t}" }
                            }
                            if let Some(m) = &s.message {
                                div { class: "unit-actions-message", "{m}" }
                            }
                        }
                    }
                    for (i, a) in s.actions.iter().cloned().enumerate() {
                        {action_view(unit, a, s.hot == Some(i))}
                    }
                }
                button {
                    class: if s.hot == Some(s.actions.len()) { "unit-actions-cancel hot" } else { "unit-actions-cancel" },
                    r#type: "button",
                    "aria-label": "{s.cancel}",
                    onpointerdown: move |e: PointerEvent| e.stop_propagation(),
                    onclick: move |e: MouseEvent| {
                        e.stop_propagation();
                        unit.choose(None);
                    },
                    "{s.cancel}"
                }
            }
        }
    }
}

fn action_view(unit: Unit, a: SheetAction, hot: bool) -> Element {
    let ink = match () {
        _ if !a.enabled => Ink::Dim,
        _ if a.destructive => Ink::Danger,
        _ => Ink::Accent,
    };
    let class = format!(
        "unit-action{}{}{}",
        if a.destructive { " danger" } else { "" },
        if a.enabled { "" } else { " off" },
        if hot { " hot" } else { "" }
    );
    let id = a.id;
    let enabled = a.enabled;
    rsx! {
        button { key: "{id}", class, r#type: "button", role: "menuitem",
            disabled: !enabled,
            "aria-label": "{a.label}",
            onpointerdown: move |e: PointerEvent| e.stop_propagation(),
            onclick: move |e: MouseEvent| {
                e.stop_propagation();
                if enabled {
                    unit.choose(Some(id));
                }
            },
            if let Some(d) = a.icon {
                Icon { d, ink }
            }
            span { "{a.label}" }
        }
    }
}

/// The page's own field, drawn: the text, a caret, the selection.
fn field_view(f: &Field, label: &str) -> Element {
    let (before, after) = f.split();
    let class = if f.all {
        "unit-input unit-field all"
    } else {
        "unit-input unit-field"
    };
    if f.text.is_empty() {
        return rsx! {
            div { class, role: "textbox", "aria-label": "{label}",
                span { class: "unit-caret" }
                span { class: "unit-placeholder", "{f.placeholder}" }
            }
        };
    }
    rsx! {
        div { class, role: "textbox", "aria-label": "{label}",
            span { "{before}" }
            span { class: "unit-caret" }
            span { "{after}" }
        }
    }
}

#[component]
fn ToastView(unit: Unit, id: u64, toast: Toast) -> Element {
    let act = move |e: MouseEvent| {
        e.stop_propagation();
        let action = unit
            .layer
            .peek()
            .toasts
            .iter()
            .find(|(t, ..)| *t == id)
            .and_then(|(_, _, a)| a.clone());
        unit.dismiss(id);
        if let Some(a) = action {
            a();
        }
    };
    rsx! {
        div { class: "unit-toast{toast.tone.class()}",
            onpointerdown: move |e: PointerEvent| {
                e.stop_propagation();
            },
            span { class: "unit-toast-message", "{toast.message}" }
            if let Some(a) = &toast.action {
                button { class: "unit-button unit-ghost", r#type: "button", onclick: act, "{a}" }
            }
            button { class: "unit-button unit-ghost unit-icon", r#type: "button",
                "aria-label": "Dismiss",
                onclick: move |e: MouseEvent| {
                    e.stop_propagation();
                    unit.dismiss(id);
                },
                "✕"
            }
        }
    }
}

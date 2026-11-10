//! Which of a collection's items are selected: a [`TileGrid`]'s tiles, a
//! [`Table`]'s rows. The page keeps it, as it keeps every control's value,
//! and hands it back to the control each render; the control gives it a new
//! one when a press or a key changes it.
//!
//! Items are named by a key of the page's own rather than by where they
//! are, so a selection outlives the collection being sorted or filtered: a
//! file stays selected when its folder is sorted by size, or a search hides
//! the files round it.
//!
//! A selection keeps three things: the items chosen, the **cursor** (the
//! item the arrows move from, drawn with a ring while the control has the
//! keys) and the **anchor** (where a Shift-press's range starts: the item
//! last pressed alone).
//!
//! [`TileGrid`]: crate::TileGrid
//! [`Table`]: crate::Table

use std::collections::BTreeSet;

use dioxus::html::{Key, Modifiers, NamedKey};

use crate::components::navigation::typeahead;

/// What a press does to a selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pick {
    /// The item alone: a plain press.
    Only,
    /// The item in or out, the rest kept: ⌘ on a Mac, Ctrl elsewhere, or
    /// any press while a control `toggles` (a phone's Select mode).
    Toggle,
    /// Every item from the anchor to this one: Shift.
    Range,
}

impl Pick {
    /// The pick a press with `mods` held makes. On a Mac, Ctrl-click is the
    /// secondary button, so only ⌘ toggles there; in a browser, where unit
    /// cannot tell the system, either does.
    pub fn from_modifiers(mods: Modifiers) -> Pick {
        let toggle = if cfg!(target_os = "macos") {
            mods.meta()
        } else {
            mods.meta() || mods.ctrl()
        };
        if mods.shift() {
            Pick::Range
        } else if toggle {
            Pick::Toggle
        } else {
            Pick::Only
        }
    }
}

/// The items chosen in a collection, by key, and its cursor and anchor.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    keys: BTreeSet<String>,
    cursor: Option<String>,
    anchor: Option<String>,
}

impl Selection {
    /// Nothing selected.
    pub fn new() -> Self {
        Self::default()
    }

    /// `key` alone, the cursor and anchor on it.
    pub fn only(key: impl Into<String>) -> Self {
        let key = key.into();
        Self {
            keys: BTreeSet::from([key.clone()]),
            cursor: Some(key.clone()),
            anchor: Some(key),
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.keys.contains(key)
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// The keys selected, in key order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.keys.iter().map(String::as_str)
    }

    /// The keys selected, in the order `order` shows them.
    pub fn in_order<'a>(&'a self, order: &'a [String]) -> impl Iterator<Item = &'a str> {
        order
            .iter()
            .filter(|k| self.keys.contains(*k))
            .map(String::as_str)
    }

    /// The item the arrows move from.
    pub fn cursor(&self) -> Option<&str> {
        self.cursor.as_deref()
    }

    /// Nothing selected; the cursor stays where it was.
    pub fn clear(&mut self) {
        self.keys.clear();
        self.anchor = None;
    }

    /// Every item in `order`.
    pub fn select_all(&mut self, order: &[String]) {
        self.keys = order.iter().cloned().collect();
        if self.cursor.is_none() {
            self.cursor = order.first().cloned();
        }
    }

    /// A press on `key`, among the items `order` shows.
    pub fn press(&mut self, order: &[String], key: &str, pick: Pick) {
        match pick {
            Pick::Only => *self = Self::only(key),
            Pick::Toggle => {
                if !self.keys.remove(key) {
                    self.keys.insert(key.to_string());
                }
                self.cursor = Some(key.to_string());
                self.anchor = Some(key.to_string());
            }
            Pick::Range => {
                let anchor = self
                    .anchor
                    .clone()
                    .filter(|a| order.contains(a))
                    .unwrap_or_else(|| key.to_string());
                self.keys = range(order, &anchor, key);
                self.cursor = Some(key.to_string());
                self.anchor = Some(anchor);
            }
        }
    }

    /// The arrows moved the cursor to `key`: it alone is selected, or with
    /// Shift (`extend`) the range from the anchor to it.
    pub fn moved(&mut self, order: &[String], key: &str, extend: bool) {
        self.press(order, key, if extend { Pick::Range } else { Pick::Only });
    }

    /// Forget what `order` no longer shows: a search narrowed the items, or
    /// some were deleted. The cursor goes with them.
    pub fn retain(&mut self, order: &[String]) {
        self.keys.retain(|k| order.contains(k));
        if self.cursor.as_ref().is_some_and(|c| !order.contains(c)) {
            self.cursor = None;
        }
        if self.anchor.as_ref().is_some_and(|a| !order.contains(a)) {
            self.anchor = None;
        }
    }

    /// Where the cursor is in `order`.
    pub(crate) fn cursor_in(&self, order: &[String]) -> Option<usize> {
        let c = self.cursor.as_ref()?;
        order.iter().position(|k| k == c)
    }
}

/// Every key in `order` from `a` to `b`, whichever comes first.
fn range(order: &[String], a: &str, b: &str) -> BTreeSet<String> {
    let at = |k: &str| order.iter().position(|o| o == k);
    match (at(a), at(b)) {
        (Some(i), Some(j)) => order[i.min(j)..=i.max(j)].iter().cloned().collect(),
        _ => BTreeSet::from([b.to_string()]),
    }
}

/// What a key did to a collection.
#[derive(Debug, PartialEq)]
pub(crate) enum Did {
    Select(Selection),
    /// Return on the cursor's item: open it.
    Activate(String),
    /// The Menu key, or Shift-F10: the cursor's item's menu.
    Menu,
}

/// A collection as its keys see it: its items' keys and labels in the order
/// shown, whether it is a grid (whose items left and right move along), how
/// many a row holds (a grid's columns, as far as it knows; 1 for a list),
/// and how many a page is.
pub(crate) struct Keyed<'a> {
    pub order: &'a [String],
    pub labels: &'a [String],
    pub selection: &'a Selection,
    pub multiple: bool,
    pub grid: bool,
    pub columns: usize,
    pub page: usize,
}

impl Keyed<'_> {
    /// What `key` does, if anything: the arrows move the cursor (by a row
    /// up and down, an item left and right in a grid), Home, End, Page Up
    /// and Page Down too, Shift extending the selection to it; ⌘A or
    /// Ctrl-A selects everything; Space puts the cursor's item in or out;
    /// Return opens it; a letter moves to the next item it starts.
    pub fn key(&self, key: &Key, mods: Modifiers) -> Option<Did> {
        let n = self.order.len();
        if n == 0 {
            return None;
        }
        let at = self.selection.cursor_in(self.order);
        let last = n - 1;
        let back = |by: usize| at.map_or(last, |i| i.saturating_sub(by));
        let on = |by: usize| at.map_or(0, |i| (i + by).min(last));
        let grid = self.grid;
        let to = match key {
            Key::Named(NamedKey::ArrowLeft) if grid => back(1),
            Key::Named(NamedKey::ArrowRight) if grid => on(1),
            Key::Named(NamedKey::ArrowUp) => back(self.columns),
            Key::Named(NamedKey::ArrowDown) => on(self.columns),
            Key::Named(NamedKey::PageUp) => back(self.page),
            Key::Named(NamedKey::PageDown) => on(self.page),
            Key::Named(NamedKey::Home) => 0,
            Key::Named(NamedKey::End) => last,
            Key::Named(NamedKey::Enter) => {
                return at.map(|i| Did::Activate(self.order[i].clone()));
            }
            Key::Named(NamedKey::ContextMenu) => return Some(Did::Menu),
            Key::Named(NamedKey::F10) if mods.shift() => return Some(Did::Menu),
            Key::Character(c) if c == " " => {
                let i = at.unwrap_or(0);
                let mut s = self.selection.clone();
                let pick = if self.multiple {
                    Pick::Toggle
                } else {
                    Pick::Only
                };
                s.press(self.order, &self.order[i], pick);
                return Some(Did::Select(s));
            }
            Key::Character(c) if c.eq_ignore_ascii_case("a") && (mods.meta() || mods.ctrl()) => {
                if !self.multiple {
                    return None;
                }
                let mut s = self.selection.clone();
                s.select_all(self.order);
                return Some(Did::Select(s));
            }
            Key::Character(c) if mods.is_empty() || mods == Modifiers::SHIFT => {
                typeahead(self.labels, at, c)?
            }
            _ => return None,
        };
        let mut s = self.selection.clone();
        s.moved(self.order, &self.order[to], self.multiple && mods.shift());
        Some(Did::Select(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order() -> Vec<String> {
        ["a", "b", "c", "d", "e"].map(String::from).into()
    }

    fn keys(s: &Selection) -> Vec<&str> {
        s.keys().collect()
    }

    #[test]
    fn a_plain_press_selects_the_item_alone() {
        let mut s = Selection::only("a");
        s.press(&order(), "c", Pick::Only);
        assert_eq!(keys(&s), ["c"]);
        assert_eq!(s.cursor(), Some("c"));
    }

    #[test]
    fn a_toggle_adds_and_removes_and_moves_the_anchor() {
        let o = order();
        let mut s = Selection::only("a");
        s.press(&o, "c", Pick::Toggle);
        assert_eq!(keys(&s), ["a", "c"]);
        s.press(&o, "a", Pick::Toggle);
        assert_eq!(keys(&s), ["c"]);
        // The range runs from the item last toggled, in or out.
        s.press(&o, "e", Pick::Range);
        assert_eq!(keys(&s), ["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn a_range_runs_from_the_anchor_either_way_and_is_redrawn_from_it() {
        let o = order();
        let mut s = Selection::only("c");
        s.press(&o, "e", Pick::Range);
        assert_eq!(keys(&s), ["c", "d", "e"]);
        // A second Shift-press redraws the range from the same anchor.
        s.press(&o, "a", Pick::Range);
        assert_eq!(keys(&s), ["a", "b", "c"]);
        assert_eq!(s.cursor(), Some("a"));
    }

    #[test]
    fn a_range_with_no_anchor_is_the_item_alone() {
        let mut s = Selection::new();
        s.press(&order(), "b", Pick::Range);
        assert_eq!(keys(&s), ["b"]);
    }

    #[test]
    fn a_selection_follows_its_keys_through_a_sort() {
        let mut s = Selection::only("b");
        s.press(&order(), "d", Pick::Range);
        let sorted: Vec<String> = ["e", "d", "c", "b", "a"].map(String::from).into();
        assert_eq!(s.in_order(&sorted).collect::<Vec<_>>(), ["d", "c", "b"]);
        assert_eq!(s.cursor_in(&sorted), Some(1));
    }

    #[test]
    fn retain_forgets_what_is_no_longer_shown() {
        let mut s = Selection::new();
        s.select_all(&order());
        s.press(&order(), "e", Pick::Toggle);
        let narrowed: Vec<String> = ["a", "b"].map(String::from).into();
        s.retain(&narrowed);
        assert_eq!(keys(&s), ["a", "b"]);
        assert_eq!(s.cursor(), None);
    }

    #[test]
    fn modifiers_choose_the_pick() {
        assert_eq!(Pick::from_modifiers(Modifiers::empty()), Pick::Only);
        assert_eq!(Pick::from_modifiers(Modifiers::SHIFT), Pick::Range);
        assert_eq!(Pick::from_modifiers(Modifiers::META), Pick::Toggle);
        let ctrl = Pick::from_modifiers(Modifiers::CONTROL);
        if cfg!(target_os = "macos") {
            assert_eq!(ctrl, Pick::Only, "Ctrl-click is a Mac's secondary button");
        } else {
            assert_eq!(ctrl, Pick::Toggle);
        }
    }

    fn keyed<'a>(order: &'a [String], s: &'a Selection, columns: usize) -> Keyed<'a> {
        Keyed {
            order,
            labels: order,
            selection: s,
            multiple: true,
            grid: columns > 1,
            columns,
            page: 10,
        }
    }

    fn selected(did: Option<Did>) -> Selection {
        match did {
            Some(Did::Select(s)) => s,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_arrows_move_by_a_row_in_a_grid_and_stop_at_the_ends() {
        let o = order();
        let s = Selection::only("b");
        let k = keyed(&o, &s, 2);
        let down = selected(k.key(&Key::Named(NamedKey::ArrowDown), Modifiers::empty()));
        assert_eq!(down.cursor(), Some("d"));
        let k = keyed(&o, &down, 2);
        let down = selected(k.key(&Key::Named(NamedKey::ArrowDown), Modifiers::empty()));
        assert_eq!(down.cursor(), Some("e"), "short of a row: the last");
        let k = keyed(&o, &down, 2);
        let right = selected(k.key(&Key::Named(NamedKey::ArrowRight), Modifiers::empty()));
        assert_eq!(right.cursor(), Some("e"), "no wrapping past the end");
        // In a list, left and right are not the list's.
        let k = keyed(&o, &s, 1);
        assert_eq!(
            k.key(&Key::Named(NamedKey::ArrowRight), Modifiers::empty()),
            None
        );
    }

    #[test]
    fn shift_extends_space_toggles_and_return_opens() {
        let o = order();
        let s = Selection::only("b");
        let k = keyed(&o, &s, 1);
        let ext = selected(k.key(&Key::Named(NamedKey::ArrowDown), Modifiers::SHIFT));
        assert_eq!(keys(&ext), ["b", "c"]);
        let k = keyed(&o, &ext, 1);
        let off = selected(k.key(&Key::Character(" ".into()), Modifiers::empty()));
        assert_eq!(keys(&off), ["b"], "Space takes the cursor's item out");
        let k = keyed(&o, &off, 1);
        assert_eq!(
            k.key(&Key::Named(NamedKey::Enter), Modifiers::empty()),
            Some(Did::Activate("c".into()))
        );
        let all = selected(k.key(&Key::Character("a".into()), Modifiers::META));
        assert_eq!(all.len(), 5);
        let typed = selected(k.key(&Key::Character("e".into()), Modifiers::empty()));
        assert_eq!(keys(&typed), ["e"]);
    }
}

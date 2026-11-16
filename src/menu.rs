//! A menu, as data and as a state machine: what it holds ([`Entry`]), where
//! each of its open levels stands, which item is highlighted, and what a key
//! or the pointer does to it. Nothing here draws; `layer.rs` does, from this.
//!
//! Every geometry is computed rather than measured: a menu's levels are
//! absolutely positioned boxes in the page's overlay, and a submenu has to
//! know where its parent's item is. Measuring the parent after layout would
//! need the DOM, which a page on dioxus-native can only read from a task
//! while Blitz holds it borrowed. So an item is [`Sizes::item`] tall, a
//! separator [`Sizes::separator`], and a level's width is estimated from its
//! longest label.

use dioxus::html::{Key, Modifiers};

/// A menu's sizes, in CSS pixels — the stylesheet's `.unit-item`,
/// `.unit-sep` and `.unit-menu`, for each density.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sizes {
    /// An item's height.
    pub item: f64,
    pub separator: f64,
    /// A level's padding, top and bottom.
    pub pad: f64,
    /// An average character of the menus' face, for estimating width.
    pub char: f64,
    pub min_width: f64,
    /// How far a level is kept off the viewport's edges.
    pub margin: f64,
    /// How far a submenu overlaps its parent, as the system's do.
    pub overlap: f64,
    /// A palette's height: a row of items side by side, each an icon over
    /// a short label ([`Entry::Palette`]).
    pub palette: f64,
    /// A palette's narrowest cell.
    pub cell: f64,
}

impl Sizes {
    pub const REGULAR: Sizes = Sizes {
        item: 24.0,
        separator: 9.0,
        pad: 5.0,
        char: 7.0,
        min_width: 160.0,
        margin: 4.0,
        overlap: 4.0,
        palette: 44.0,
        cell: 52.0,
    };

    /// For a finger: rows tall enough to tap.
    pub const TOUCH: Sizes = Sizes {
        item: 44.0,
        separator: 13.0,
        pad: 6.0,
        char: 9.0,
        min_width: 200.0,
        margin: 4.0,
        overlap: 4.0,
        palette: 64.0,
        cell: 64.0,
    };

    /// For a terminal's cells: an item and a separator a row each, and a
    /// row above and below the items, where the border is drawn — `pad` is
    /// the stylesheet's 23px padding and the 1px border together, so a level
    /// is exactly as tall as its rows. `char` is a 16px monospace face's
    /// advance. A level keeps to the grid: no margin off the edges, and a
    /// submenu overlapping its parent by one cell, so they share a border.
    pub const CELL: Sizes = Sizes {
        item: 24.0,
        separator: 24.0,
        pad: 24.0,
        char: 9.7,
        min_width: 160.0,
        margin: 0.0,
        overlap: 9.7,
        // A row: the labels side by side, no icons.
        palette: 24.0,
        cell: 48.0,
    };
}

const MAX_WIDTH: f64 = 380.0;
/// Room for the tick on the left and the submenu arrow on the right.
const GUTTERS: f64 = 44.0;
/// An item's icon and the gap after it: `.unit-item .unit-svg`.
const ICON: f64 = 22.0;

/// One thing a menu shows.
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    Item(Item),
    Separator,
    /// A heading over the entries after it: what a menu is about ("The
    /// entry"), or a group's name. It is not chosen, and the arrows pass it.
    Header(String),
    /// A label that opens another menu beside it.
    Submenu {
        label: String,
        entries: Vec<Entry>,
        enabled: bool,
    },
    /// Items side by side in one row, each an icon over a short label: the
    /// few things done most, at the top of a long menu (Copy, Share,
    /// Duplicate, Delete), or a choice of a few, ticked where it stands
    /// (Reflow, Pages, Source) instead of a submenu of them. One entry for
    /// the arrows up and down; left and right go along it.
    Palette(Vec<Item>),
}

/// An item that does something when chosen: its `id` is what the menu's
/// pick handler is given.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub id: usize,
    pub label: String,
    /// The key that does the same, shown at the right: `⌘B`.
    pub shortcut: Option<String>,
    pub checked: bool,
    /// Whether it is ticked or not, rather than an action: read out as a
    /// checkbox. Set by [`Item::checked`].
    pub checkable: bool,
    pub enabled: bool,
    /// An icon before the label: a path for [`crate::Icon`].
    pub icon: Option<&'static str>,
    /// Destroys something: drawn in the danger colour.
    pub destructive: bool,
}

impl Item {
    pub fn new(id: usize, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            shortcut: None,
            checked: false,
            checkable: false,
            enabled: true,
            icon: None,
            destructive: false,
        }
    }

    pub fn icon(mut self, d: &'static str) -> Self {
        self.icon = Some(d);
        self
    }

    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    pub fn shortcut(mut self, s: impl Into<String>) -> Self {
        let s = s.into();
        self.shortcut = (!s.is_empty()).then_some(s);
        self
    }

    pub fn checked(mut self, yes: bool) -> Self {
        self.checked = yes;
        self.checkable = true;
        self
    }

    pub fn enabled(mut self, yes: bool) -> Self {
        self.enabled = yes;
        self
    }
}

impl From<Item> for Entry {
    fn from(i: Item) -> Self {
        Entry::Item(i)
    }
}

impl Entry {
    /// A heading over the entries after it.
    pub fn header(label: impl Into<String>) -> Self {
        Entry::Header(label.into())
    }

    /// A submenu of `entries`, enabled while any of them is.
    pub fn submenu(label: impl Into<String>, entries: Vec<Entry>) -> Self {
        let enabled = entries.iter().any(Entry::enabled);
        Entry::Submenu {
            label: label.into(),
            entries,
            enabled,
        }
    }

    /// A row of `items` side by side ([`Entry::Palette`]).
    pub fn palette(items: Vec<Item>) -> Self {
        Entry::Palette(items)
    }

    pub fn enabled(&self) -> bool {
        match self {
            Entry::Item(i) => i.enabled,
            Entry::Separator | Entry::Header(_) => false,
            Entry::Submenu { enabled, .. } => *enabled,
            Entry::Palette(items) => items.iter().any(|i| i.enabled),
        }
    }

    fn height(&self, z: Sizes) -> f64 {
        match self {
            Entry::Separator => z.separator,
            Entry::Palette(_) => z.palette,
            _ => z.item,
        }
    }

    fn width(&self, z: Sizes) -> f64 {
        let chars = |s: &str| s.chars().count() as f64;
        match self {
            Entry::Item(i) => {
                i.icon.map_or(0.0, |_| ICON)
                    + chars(&i.label) * z.char
                    + i.shortcut
                        .as_deref()
                        .map_or(0.0, |s| chars(s) * z.char + 24.0)
            }
            Entry::Separator => 0.0,
            Entry::Header(label) => chars(label) * z.char,
            Entry::Submenu { label, .. } => chars(label) * z.char + 16.0,
            // The cells share the row equally, so each is as wide as the
            // widest label in the palette's smaller face, or its narrowest;
            // less the gutters a row leaves for a tick and an arrow, which a
            // palette has no need of.
            Entry::Palette(items) => {
                let widest = items
                    .iter()
                    .map(|i| (chars(&i.label) * z.char * 0.9 + 14.0).max(z.cell))
                    .fold(0.0, f64::max);
                widest * items.len() as f64 - GUTTERS + 12.0
            }
        }
    }
}

/// One open level of a menu: its entries, its box, and its highlighted
/// item.
#[derive(Clone, Debug, PartialEq)]
pub struct Level {
    pub entries: Vec<Entry>,
    pub left: f64,
    pub top: f64,
    pub width: f64,
    /// The height it is drawn at where its entries are taller than the room
    /// the page has for it: it scrolls inside that. `None` where they fit.
    pub clip: Option<f64>,
    pub hot: Option<usize>,
    /// Which of a palette's items is highlighted, where [`Level::hot`] is
    /// a palette.
    pub cell: usize,
    pub sizes: Sizes,
    /// Which opening this is, of every level any menu has opened: the box
    /// its rows scroll in is drawn anew for each (`layer.rs`), so a level
    /// opens at its first row however far the last one was scrolled.
    pub opening: u64,
}

/// The next level's [`Level::opening`].
static OPENINGS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Level {
    fn new(entries: Vec<Entry>, z: Sizes) -> Self {
        let width = (entries.iter().map(|e| e.width(z)).fold(0.0, f64::max) + GUTTERS)
            .clamp(z.min_width, MAX_WIDTH);
        Self {
            entries,
            left: 0.0,
            top: 0.0,
            width,
            clip: None,
            hot: None,
            cell: 0,
            sizes: z,
            opening: OPENINGS.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// Stand the level between `top` and `bottom`, the page's room for a
    /// menu, with its top at `at` where that fits: moved up where it would
    /// run past `bottom`, and where it is taller than the room, as tall as
    /// the room and scrolling.
    fn stand(&mut self, at: f64, top: f64, bottom: f64) {
        let h = self.height();
        if h > bottom - top {
            self.clip = Some((bottom - top).max(self.sizes.item));
            self.top = top;
        } else {
            self.clip = None;
            self.top = at.min(bottom - h).max(top);
        }
    }

    pub fn height(&self) -> f64 {
        let z = self.sizes;
        2.0 * z.pad + self.entries.iter().map(|e| e.height(z)).sum::<f64>()
    }

    /// The top of entry `i`, from the level's top.
    fn entry_top(&self, i: usize) -> f64 {
        let z = self.sizes;
        z.pad + self.entries[..i].iter().map(|e| e.height(z)).sum::<f64>()
    }

    /// The next entry that can be highlighted from `from`, going `down` —
    /// wrapping, as a system menu does.
    fn step(&self, from: Option<usize>, down: bool) -> Option<usize> {
        let n = self.entries.len();
        if n == 0 {
            return None;
        }
        let mut i = from;
        for _ in 0..n {
            let next = match (i, down) {
                (None, true) => 0,
                (None, false) => n - 1,
                (Some(i), true) => (i + 1) % n,
                (Some(i), false) => (i + n - 1) % n,
            };
            if self.entries[next].enabled() {
                return Some(next);
            }
            i = Some(next);
        }
        None
    }

    /// Highlight entry `i`, and where it is a palette, its first item that
    /// can be chosen at or after `cell`. Over a row, `cell` is kept: the
    /// column the arrows go on down in, into the next palette.
    fn light(&mut self, i: Option<usize>, cell: usize) {
        self.hot = i;
        self.cell = cell;
        if let Some(Entry::Palette(items)) = i.and_then(|i| self.entries.get(i)) {
            let n = items.len();
            self.cell = (0..n)
                .map(|k| (cell.min(n.saturating_sub(1)) + k) % n)
                .find(|&k| items[k].enabled)
                .unwrap_or(0);
        }
    }

    /// Move along the highlighted palette, `right` or left, past what
    /// cannot be chosen: whether there was anywhere to go.
    fn along(&mut self, right: bool) -> bool {
        let Some(Entry::Palette(items)) = self.hot.and_then(|i| self.entries.get(i)) else {
            return false;
        };
        let mut k = self.cell;
        loop {
            k = match (right, k) {
                (true, k) if k + 1 < items.len() => k + 1,
                (false, k) if k > 0 => k - 1,
                _ => return false,
            };
            if items[k].enabled {
                self.cell = k;
                return true;
            }
        }
    }
}

/// What a key or a press did to the menu.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// Not the menu's: the key goes on to whoever is under it.
    Ignored,
    /// The menu changed, or the key was swallowed.
    Handled,
    /// This item was chosen; the menu closes.
    Pick(usize),
    /// The menu closes with nothing chosen.
    Close,
}

/// An open menu: its levels, the first one's and each submenu after it.
#[derive(Clone, Debug, PartialEq)]
pub struct Menu {
    pub levels: Vec<Level>,
    /// The page's size, which no level is let past.
    viewport: (f64, f64),
    /// What is kept clear at the page's top and bottom: a phone's safe
    /// area, and its keyboard.
    clear: (f64, f64),
    /// Whether a level has scrolled since the last press: the finger that
    /// lets go then was scrolling the menu, not choosing from it.
    scrolled: bool,
    /// Whether the pointer has moved over the menu, or pressed in it, since
    /// it opened: until then a release is the end of the press that opened
    /// it — a right-click's — and picks nothing.
    pub armed: bool,
}

impl Menu {
    /// `entries`, opened with the first level's top-left at `at`, kept in a
    /// page of `viewport`'s size: moved left, or above `at` (less `above`,
    /// the height of whatever it opened under), where it would run off.
    pub fn open(
        entries: Vec<Entry>,
        at: (f64, f64),
        above: f64,
        viewport: (f64, f64),
        sizes: Sizes,
    ) -> Self {
        Self::open_clear(entries, at, above, viewport, (0.0, 0.0), sizes)
    }

    /// [`Menu::open`], with `clear` kept free at the page's top and bottom,
    /// a phone's safe area: no level stands in it, and one with more
    /// entries than the room between holds is that tall and scrolls.
    pub fn open_clear(
        entries: Vec<Entry>,
        at: (f64, f64),
        above: f64,
        viewport: (f64, f64),
        clear: (f64, f64),
        sizes: Sizes,
    ) -> Self {
        let mut level = Level::new(entries, sizes);
        let (w, h) = (level.width, level.height());
        let margin = sizes.margin;
        let (top, bottom) = (clear.0 + margin, viewport.1 - clear.1 - margin);
        level.left = at.0.min(viewport.0 - w - margin).max(margin);
        if at.1 + h > bottom && at.1 - above - h > top {
            level.top = at.1 - above - h;
        } else {
            level.stand(at.1, top, bottom);
        }
        Self {
            levels: vec![level],
            viewport,
            clear,
            scrolled: false,
            armed: false,
        }
    }

    /// A level scrolled under the pointer.
    pub fn scroll(&mut self) {
        self.scrolled = true;
    }

    /// Highlight entry `i` of level `depth`, opening its submenu or closing
    /// any deeper level.
    pub fn hover(&mut self, depth: usize, i: usize) {
        self.hover_at(depth, i, 0);
    }

    /// Highlight item `cell` of the palette at entry `i` of level `depth`
    /// ([`Menu::hover`] for any other entry). A cell that cannot be chosen
    /// highlights nothing, as such an item does.
    pub fn hover_at(&mut self, depth: usize, i: usize, cell: usize) {
        self.armed = true;
        if depth >= self.levels.len() {
            return;
        }
        self.levels.truncate(depth + 1);
        let level = &mut self.levels[depth];
        let enabled = match level.entries.get(i) {
            Some(Entry::Palette(items)) => items.get(cell).is_some_and(|c| c.enabled),
            entry => entry.is_some_and(Entry::enabled),
        };
        level.hot = enabled.then_some(i);
        level.cell = if enabled { cell } else { 0 };
        if enabled {
            self.open_submenu(depth, i);
        }
    }

    /// Open the submenu at entry `i` of level `depth`, if it is one.
    fn open_submenu(&mut self, depth: usize, i: usize) -> bool {
        let parent = &self.levels[depth];
        let Some(Entry::Submenu {
            entries,
            enabled: true,
            ..
        }) = parent.entries.get(i)
        else {
            return false;
        };
        let mut level = Level::new(entries.clone(), parent.sizes);
        let w = level.width;
        let z = parent.sizes;
        let right = parent.left + parent.width - z.overlap;
        level.left = if right + w > self.viewport.0 - z.margin {
            // No room to the right: to the left of the parent instead.
            (parent.left - w + z.overlap).max(z.margin)
        } else {
            right
        };
        // Its first item level with the parent's.
        level.stand(
            parent.top + parent.entry_top(i) - z.pad,
            self.clear.0 + z.margin,
            self.viewport.1 - self.clear.1 - z.margin,
        );
        self.levels.truncate(depth + 1);
        self.levels.push(level);
        true
    }

    /// Highlight the first level's first item that can be chosen.
    pub fn hot_first(&mut self) {
        let level = &mut self.levels[0];
        let first = level.step(None, true);
        level.light(first, 0);
    }

    /// The pointer pressed in the menu.
    pub fn press(&mut self) {
        self.armed = true;
        self.scrolled = false;
    }

    /// The pointer let go over entry `i` of level `depth`.
    pub fn release(&mut self, depth: usize, i: usize) -> Outcome {
        self.release_at(depth, i, 0)
    }

    /// The pointer let go over item `cell` of the palette at entry `i` of
    /// level `depth` ([`Menu::release`] for any other entry).
    pub fn release_at(&mut self, depth: usize, i: usize, cell: usize) -> Outcome {
        if !self.armed || std::mem::take(&mut self.scrolled) {
            return Outcome::Handled;
        }
        if let Some(level) = self.levels.get_mut(depth)
            && matches!(level.entries.get(i), Some(Entry::Palette(_)))
        {
            level.cell = cell;
        }
        self.activate(depth, i)
    }

    fn activate(&mut self, depth: usize, i: usize) -> Outcome {
        let level = self.levels.get(depth);
        let cell = level.map_or(0, |l| l.cell);
        match level.and_then(|l| l.entries.get(i)) {
            Some(Entry::Item(item)) if item.enabled => Outcome::Pick(item.id),
            Some(Entry::Palette(items)) => match items.get(cell) {
                Some(item) if item.enabled => Outcome::Pick(item.id),
                _ => Outcome::Handled,
            },
            Some(Entry::Submenu { enabled: true, .. }) => {
                self.levels[depth].hot = Some(i);
                if self.open_submenu(depth, i) {
                    let deeper = depth + 1;
                    let first = self.levels[deeper].step(None, true);
                    self.levels[deeper].light(first, 0);
                }
                Outcome::Handled
            }
            _ => Outcome::Handled,
        }
    }

    /// A key, while the menu is open. Every key is the menu's — as a system
    /// menu's are — but those it does nothing with, which close it and go on.
    pub fn key(&mut self, key: &Key, mods: Modifiers) -> Outcome {
        use dioxus::html::NamedKey::*;
        let depth = self.levels.len() - 1;
        match key {
            Key::Named(Escape) => Outcome::Close,
            Key::Named(ArrowDown) | Key::Named(ArrowUp) => {
                let down = *key == Key::Named(ArrowDown);
                let level = &mut self.levels[depth];
                let next = level.step(level.hot, down);
                // Down a column of a palette and back up it.
                level.light(next, level.cell);
                Outcome::Handled
            }
            Key::Named(ArrowRight) => {
                if self.levels[depth].along(true) {
                    return Outcome::Handled;
                }
                if let Some(i) = self.levels[depth].hot
                    && !matches!(self.levels[depth].entries.get(i), Some(Entry::Palette(_)))
                {
                    self.activate(depth, i);
                }
                Outcome::Handled
            }
            Key::Named(ArrowLeft) => {
                if !self.levels[depth].along(false) && depth > 0 {
                    self.levels.pop();
                }
                Outcome::Handled
            }
            Key::Named(Enter) => match self.levels[depth].hot {
                Some(i) => self.activate(depth, i),
                None => Outcome::Close,
            },
            Key::Character(c) if c == " " && mods.is_empty() => match self.levels[depth].hot {
                Some(i) => self.activate(depth, i),
                None => Outcome::Handled,
            },
            // A letter jumps to the next item starting with it.
            Key::Character(c) if mods.is_empty() || mods == Modifiers::SHIFT => {
                let c = c.to_lowercase();
                let level = &mut self.levels[depth];
                let starts = |label: &str| label.to_lowercase().starts_with(&c);
                // Each item that can be chosen, as (entry, cell), after the
                // highlighted one and round: a palette's in its order.
                let mut stops = Vec::new();
                for (k, entry) in level.entries.iter().enumerate() {
                    match entry {
                        Entry::Item(i) if i.enabled && starts(&i.label) => stops.push((k, 0)),
                        Entry::Submenu {
                            label,
                            enabled: true,
                            ..
                        } if starts(label) => stops.push((k, 0)),
                        Entry::Palette(items) => stops.extend(
                            items
                                .iter()
                                .enumerate()
                                .filter(|(_, i)| i.enabled && starts(&i.label))
                                .map(|(cell, _)| (k, cell)),
                        ),
                        _ => {}
                    }
                }
                let now = level.hot.map(|h| (h, level.cell));
                let found = stops
                    .iter()
                    .find(|&&stop| now.is_none_or(|now| stop > now))
                    .or(stops.first());
                if let Some(&(k, cell)) = found {
                    level.hot = Some(k);
                    level.cell = cell;
                }
                Outcome::Handled
            }
            _ => Outcome::Close,
        }
    }
}

#[cfg(test)]
mod tests {
    use dioxus::html::NamedKey;

    use super::*;

    fn entries() -> Vec<Entry> {
        vec![
            Item::new(1, "Bold").shortcut("⌘B").into(),
            Item::new(2, "Italic").enabled(false).into(),
            Entry::Separator,
            Entry::submenu(
                "Heading",
                vec![
                    Item::new(10, "Heading 1").into(),
                    Item::new(11, "Heading 2").into(),
                ],
            ),
        ]
    }

    const PAGE: (f64, f64) = (800.0, 600.0);

    #[test]
    fn a_heading_is_an_item_tall_and_never_highlighted() {
        let headed = vec![
            Entry::header("The entry"),
            Item::new(1, "Open").into(),
            Entry::Separator,
            Entry::header("Danger"),
            Item::new(2, "Delete").destructive().into(),
        ];
        let l = Level::new(headed, Sizes::REGULAR);
        let z = Sizes::REGULAR;
        assert_eq!(l.height(), 2.0 * z.pad + 4.0 * z.item + z.separator);
        assert_eq!(
            l.step(None, true),
            Some(1),
            "the first item, under the heading"
        );
        assert_eq!(
            l.step(Some(1), true),
            Some(4),
            "past the rule and the heading"
        );
        assert_eq!(l.step(Some(4), true), Some(1), "round, past the heading");
    }

    #[test]
    fn a_menu_opens_where_it_was_asked_and_inside_the_page() {
        let m = Menu::open(entries(), (100.0, 50.0), 0.0, PAGE, Sizes::REGULAR);
        assert_eq!((m.levels[0].left, m.levels[0].top), (100.0, 50.0));
        let m = Menu::open(entries(), (790.0, 50.0), 0.0, PAGE, Sizes::REGULAR);
        let l = &m.levels[0];
        assert!(l.left + l.width <= PAGE.0, "{}", l.left);
        // Near the bottom it opens above the point instead.
        let m = Menu::open(entries(), (100.0, 590.0), 20.0, PAGE, Sizes::REGULAR);
        let l = &m.levels[0];
        assert_eq!(l.top + l.height(), 570.0);
    }

    #[test]
    fn a_menu_taller_than_the_room_is_that_tall_and_scrolls() {
        // A phone: 440 by 956, the status bar's 62 and the home bar's 34
        // kept clear, and more rows than fit between them.
        let many: Vec<Entry> = (0..30).map(|i| Item::new(i, "Row").into()).collect();
        let page = (440.0, 956.0);
        let z = Sizes::TOUCH;
        let mut m = Menu::open_clear(many.clone(), (40.0, 300.0), 0.0, page, (62.0, 34.0), z);
        let l = &m.levels[0];
        assert!(l.height() > page.1);
        assert_eq!(l.top, 62.0 + z.margin);
        assert_eq!(l.clip, Some(956.0 - 62.0 - 34.0 - 2.0 * z.margin));
        // A finger that scrolled it chooses nothing as it lets go; the next
        // press and release does.
        m.press();
        m.scroll();
        assert_eq!(m.release(0, 3), Outcome::Handled);
        m.press();
        assert_eq!(m.release(0, 3), Outcome::Pick(3));
        // One that fits stands clear of the edges kept, and is not clipped.
        let few: Vec<Entry> = many.into_iter().take(4).collect();
        let m = Menu::open_clear(few, (40.0, 0.0), 0.0, page, (62.0, 34.0), z);
        assert_eq!((m.levels[0].top, m.levels[0].clip), (62.0 + z.margin, None));
    }

    #[test]
    fn the_arrows_skip_what_cannot_be_chosen_and_wrap() {
        let mut m = Menu::open(entries(), (0.0, 0.0), 0.0, PAGE, Sizes::REGULAR);
        let down = Key::Named(NamedKey::ArrowDown);
        let none = Modifiers::empty();
        m.key(&down, none);
        assert_eq!(m.levels[0].hot, Some(0));
        m.key(&down, none);
        // Past the disabled item and the separator.
        assert_eq!(m.levels[0].hot, Some(3));
        m.key(&down, none);
        assert_eq!(m.levels[0].hot, Some(0));
        m.key(&Key::Named(NamedKey::ArrowUp), none);
        assert_eq!(m.levels[0].hot, Some(3));
    }

    #[test]
    fn a_submenu_opens_beside_its_item_and_picks_from_there() {
        let mut m = Menu::open(entries(), (100.0, 50.0), 0.0, PAGE, Sizes::REGULAR);
        m.hover(0, 3);
        assert_eq!(m.levels.len(), 2);
        let (parent, sub) = (&m.levels[0], &m.levels[1]);
        assert_eq!(
            sub.left,
            parent.left + parent.width - Sizes::REGULAR.overlap
        );
        assert_eq!(sub.top, parent.top + parent.entry_top(3) - parent.sizes.pad);
        assert_eq!(m.release(1, 1), Outcome::Pick(11));
        // Back to the parent: the submenu closes.
        m.hover(0, 0);
        assert_eq!(m.levels.len(), 1);
    }

    #[test]
    fn the_keyboard_walks_into_a_submenu_and_out() {
        let mut m = Menu::open(entries(), (0.0, 0.0), 0.0, PAGE, Sizes::REGULAR);
        let none = Modifiers::empty();
        m.key(&Key::Character("h".into()), none);
        assert_eq!(m.levels[0].hot, Some(3));
        m.key(&Key::Named(NamedKey::ArrowRight), none);
        assert_eq!(m.levels.len(), 2);
        assert_eq!(m.levels[1].hot, Some(0));
        assert_eq!(m.key(&Key::Named(NamedKey::Enter), none), Outcome::Pick(10));
        m.key(&Key::Named(NamedKey::ArrowLeft), none);
        assert_eq!(m.levels.len(), 1);
        assert_eq!(m.key(&Key::Named(NamedKey::Escape), none), Outcome::Close);
    }

    #[test]
    fn the_release_that_ends_the_opening_press_picks_nothing() {
        let mut m = Menu::open(entries(), (0.0, 0.0), 0.0, PAGE, Sizes::REGULAR);
        assert_eq!(m.release(0, 0), Outcome::Handled);
        m.hover(0, 0);
        assert_eq!(m.release(0, 0), Outcome::Pick(1));
        // A disabled item never picks.
        assert_eq!(m.release(0, 1), Outcome::Handled);
    }

    fn palette() -> Vec<Entry> {
        vec![
            Entry::header("Notes.md"),
            Entry::palette(vec![
                Item::new(1, "Copy"),
                Item::new(2, "Share").enabled(false),
                Item::new(3, "Duplicate"),
                Item::new(4, "Delete").destructive(),
            ]),
            Entry::Separator,
            Item::new(5, "Rename…").into(),
            Entry::palette(vec![
                Item::new(6, "Reflow").checked(true),
                Item::new(7, "Pages").checked(false),
            ]),
        ]
    }

    #[test]
    fn a_palette_is_one_entry_a_palette_tall_and_as_wide_as_its_cells() {
        let z = Sizes::REGULAR;
        let l = Level::new(palette(), z);
        assert_eq!(
            l.height(),
            2.0 * z.pad + 2.0 * z.item + 2.0 * z.palette + z.separator
        );
        assert!(l.width >= 4.0 * z.cell, "{}", l.width);
    }

    #[test]
    fn the_arrows_go_along_a_palette_and_down_out_of_it() {
        let mut m = Menu::open(palette(), (0.0, 0.0), 0.0, PAGE, Sizes::REGULAR);
        let none = Modifiers::empty();
        let key = |m: &mut Menu, k: NamedKey| m.key(&Key::Named(k), none);
        m.hot_first();
        assert_eq!((m.levels[0].hot, m.levels[0].cell), (Some(1), 0));
        key(&mut m, NamedKey::ArrowRight);
        // Past Share, which cannot be chosen.
        assert_eq!(m.levels[0].cell, 2);
        key(&mut m, NamedKey::ArrowRight);
        key(&mut m, NamedKey::ArrowRight);
        assert_eq!(m.levels[0].cell, 3, "it stops at the end");
        assert_eq!(key(&mut m, NamedKey::Enter), Outcome::Pick(4));
        key(&mut m, NamedKey::ArrowLeft);
        assert_eq!(m.levels[0].cell, 2);
        key(&mut m, NamedKey::ArrowDown);
        assert_eq!(m.levels[0].hot, Some(3));
        // Down into the next palette, as near the column as it has.
        m.levels[0].cell = 0;
        key(&mut m, NamedKey::ArrowUp);
        assert_eq!(m.levels[0].hot, Some(1));
        m.levels[0].cell = 3;
        key(&mut m, NamedKey::ArrowDown);
        key(&mut m, NamedKey::ArrowDown);
        assert_eq!((m.levels[0].hot, m.levels[0].cell), (Some(4), 1));
        assert_eq!(key(&mut m, NamedKey::Enter), Outcome::Pick(7));
        // A letter finds a palette's item too.
        m.key(&Key::Character("d".into()), none);
        assert_eq!((m.levels[0].hot, m.levels[0].cell), (Some(1), 2));
        m.key(&Key::Character("d".into()), none);
        assert_eq!((m.levels[0].hot, m.levels[0].cell), (Some(1), 3));
    }

    #[test]
    fn the_pointer_picks_a_palettes_cell_but_never_one_that_is_off() {
        let mut m = Menu::open(palette(), (0.0, 0.0), 0.0, PAGE, Sizes::REGULAR);
        m.hover_at(0, 1, 1);
        assert_eq!(m.levels[0].hot, None, "Share cannot be chosen");
        assert_eq!(m.release_at(0, 1, 1), Outcome::Handled);
        m.hover_at(0, 1, 2);
        assert_eq!((m.levels[0].hot, m.levels[0].cell), (Some(1), 2));
        assert_eq!(m.release_at(0, 1, 2), Outcome::Pick(3));
        assert_eq!(m.release_at(0, 4, 0), Outcome::Pick(6));
    }

    #[test]
    fn a_submenu_with_nothing_to_choose_is_disabled() {
        let s = Entry::submenu(
            "Table",
            vec![Item::new(1, "Delete Row").enabled(false).into()],
        );
        assert!(!s.enabled());
    }
}

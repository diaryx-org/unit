//! Collections of things to choose among and open: a grid of tiles (a
//! folder's icons, a library of photos) and a table of rows (a folder's
//! files by name, size and date).
//!
//! Each takes a [`Selection`] the page keeps and gives back a new one, and
//! chooses as a desktop's file browser does: a press selects an item alone,
//! ⌘ (Ctrl off a Mac) adds or removes it, Shift selects the range to it;
//! the arrows move, Shift extending, ⌘A selects everything, Space puts an
//! item in or out, Return or a double click opens it, and a letter jumps
//! to the next item starting with it. A press on nothing clears the
//! selection. At a finger's density a tap opens what it lands on, as a
//! phone's does, unless the collection `toggles`, a phone's Select mode,
//! where every tap puts an item in or out.
//!
//! With a `menu`, the secondary button, a finger held still, or the Menu
//! key opens it over the selection; a press on an item not selected
//! selects it first, so the menu is always about what is selected.
//!
//! The collection is one control to Tab and to assistive tech, its items
//! inside it, as a list box is: the keys go to it, and it draws a ring on
//! its cursor's item. Nothing is measured: a grid lays its tiles out at
//! their `size`, and reads how many fit a row from its own width as the
//! renderer reports it.

use dioxus::html::Modifiers;
use dioxus::prelude::*;

use crate::components::menus::left_top;
use crate::icon::{Icon, Ink, paths};
use crate::menu::Entry;
use crate::selection::{Did, Keyed, Pick, Selection, Shape};
use crate::theme::Density;
use crate::unit::{Kind, Unit, use_focus, use_unit};

/// How far apart a grid's tiles are, and its padding, in CSS pixels.
const GAP: f64 = 8.0;
const PAD: f64 = 4.0;

/// The height of a tile's text under its picture: its label's line and its
/// detail's, at each density. The CSS draws them these heights.
fn tile_text(density: Density) -> f64 {
    match density {
        Density::Touch => 48.0,
        _ => 40.0,
    }
}

/// A table row's height: the theme's control height at the page's density.
fn row_height(unit: &Unit) -> f64 {
    let theme = unit.theme.peek();
    match *unit.density.peek() {
        Density::Touch => theme.touch.control,
        Density::Cell => theme.cell.control,
        Density::Regular => theme.regular.control,
    }
}

/// One tile of a [`TileGrid`]: a picture or an icon over a label, and a
/// line under it (a size, a date). `key` is the page's name for it, which a
/// [`Selection`] holds.
#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub key: String,
    pub label: String,
    pub detail: Option<String>,
    /// An icon drawn large in place of a picture: a path for [`Icon`].
    pub icon: Option<&'static str>,
    /// A picture's address, drawn to fill the tile's square: a photo's
    /// thumbnail. Read by the tile's label, not as an image of its own.
    pub image: Option<String>,
    /// The heading of the section it is in: a heading is drawn where a
    /// tile's section differs from the one before it.
    pub section: Option<String>,
    /// Whether it may be chosen. One that may not is opened all the same,
    /// and left out of every selection: a press, a range, ⌘A.
    pub selectable: bool,
    /// A button in the tile's corner, its label and icon: what is done to
    /// this tile alone (Place, Remove), given to the grid's `onaction`.
    pub action: Option<(String, &'static str)>,
}

impl Tile {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            detail: None,
            icon: None,
            image: None,
            section: None,
            selectable: true,
            action: None,
        }
    }

    /// In the section headed `heading`.
    pub fn section(mut self, heading: impl Into<String>) -> Self {
        self.section = Some(heading.into());
        self
    }

    /// Opened, but never chosen.
    pub fn unselectable(mut self) -> Self {
        self.selectable = false;
        self
    }

    /// A button in its corner, named `label`, drawn as `icon`.
    pub fn action(mut self, label: impl Into<String>, icon: &'static str) -> Self {
        self.action = Some((label.into(), icon));
        self
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn icon(mut self, d: &'static str) -> Self {
        self.icon = Some(d);
        self
    }

    pub fn image(mut self, src: impl Into<String>) -> Self {
        self.image = Some(src.into());
        self
    }
}

/// What a collection does with a press on one of its items, and with its
/// menu: shared by a grid's tiles and a table's rows.
#[derive(Clone, PartialEq)]
struct Chooser {
    order: Vec<String>,
    selection: Selection,
    onselect: EventHandler<Selection>,
    onactivate: Option<EventHandler<String>>,
    multiple: bool,
    toggles: bool,
    /// A press alone opens what it lands on, as well as choosing it.
    opens: bool,
    /// The items that may be opened and not chosen.
    fixed: Vec<String>,
    menu: Vec<Entry>,
    onpick: Option<EventHandler<usize>>,
}

impl Chooser {
    fn pick(&self, mods: Modifiers) -> Pick {
        match Pick::from_modifiers(mods) {
            _ if !self.multiple => Pick::Only,
            Pick::Only if self.toggles => Pick::Toggle,
            p => p,
        }
    }

    /// Give the page `s`, less what may not be chosen, if it is new.
    fn select(&self, mut s: Selection) {
        s.exclude(&self.fixed);
        if s != self.selection {
            self.onselect.call(s);
        }
    }

    /// A click on `key`, with `mods` held.
    fn press(&self, unit: Unit, key: &str, mods: Modifiers) {
        let pick = self.pick(mods);
        let mut s = self.selection.clone();
        s.press(&self.order, key, pick);
        self.select(s);
        let finger = *unit.density.peek() == Density::Touch;
        if (finger || self.opens) && pick == Pick::Only {
            self.activate(key);
        }
    }

    fn activate(&self, key: &str) {
        if let Some(a) = self.onactivate {
            a.call(key.to_string());
        }
    }

    /// The menu asked for over `key`, at `at` in the page.
    fn menu(&self, unit: Unit, key: &str, at: (f64, f64)) {
        if !self.selection.contains(key) {
            self.select(Selection::only(key));
        }
        self.open(unit, at, false);
    }

    /// Open the menu at `at`; `first` highlights its first item, as a menu
    /// opened by a key does.
    fn open(&self, unit: Unit, at: (f64, f64), first: bool) {
        let Some(pick) = self.onpick.filter(|_| !self.menu.is_empty()) else {
            return;
        };
        unit.open_menu(self.menu.clone(), at, 0.0, move |i| pick.call(i));
        if first {
            unit.hot_first();
        }
    }

    fn has_menu(&self) -> bool {
        self.onpick.is_some() && !self.menu.is_empty()
    }

    /// A press on the collection's ground, between and after its items.
    fn ground(&self) {
        if !self.toggles && !self.selection.is_empty() {
            let mut s = self.selection.clone();
            s.clear();
            self.onselect.call(s);
        }
    }

    /// What a key does, given what the keys see.
    fn key(&self, unit: Unit, did: Option<Did>, at: impl Fn() -> (f64, f64)) -> bool {
        match did {
            Some(Did::Select(s)) => {
                self.select(s);
                true
            }
            Some(Did::Activate(k)) => {
                self.activate(&k);
                true
            }
            Some(Did::Menu) if self.has_menu() => {
                self.open(unit, at(), true);
                true
            }
            _ => false,
        }
    }
}

/// Where the pointer has found a grid's tiles: each by its place in the
/// order, where it starts in the page (which a scroll leaves as it was)
/// and in the window (where a menu opens). Renderers that report an
/// element's size tell the grid how many tiles fit a row; Blitz reports
/// none, and the grid learns it from where the pointer finds its tiles:
/// how far apart two on one row are, if the tiles stretch to fill the row,
/// and from two on different rows of one section, how many a row holds.
#[derive(Clone, Default)]
struct Sightings {
    order: Vec<String>,
    seen: Vec<Sighting>,
}

/// One tile the pointer found: its section and its place in it, and where
/// it starts in the page and in the window.
#[derive(Clone, Copy)]
struct Sighting {
    index: usize,
    section: usize,
    within: usize,
    page: (f64, f64),
    window: (f64, f64),
}

impl Sightings {
    /// How far apart the tiles are across, from two seen on one row of one
    /// section; `None` until there are two.
    fn stride(&self) -> Option<f64> {
        for (k, a) in self.seen.iter().enumerate() {
            for b in &self.seen[k + 1..] {
                let apart = b.within as f64 - a.within as f64;
                if a.section == b.section && (a.page.1 - b.page.1).abs() < 1.0 && apart != 0.0 {
                    let s = (b.page.0 - a.page.0) / apart;
                    if s > 0.0 {
                        return Some(s);
                    }
                }
            }
        }
        None
    }

    /// How many tiles a row holds, if two tiles seen on different rows of
    /// one section say, the tiles `across` apart and their rows `down`.
    fn columns(&self, across: f64, down: f64) -> Option<usize> {
        for (k, a) in self.seen.iter().enumerate().rev() {
            for b in self.seen[..k].iter().rev() {
                if a.section != b.section {
                    continue;
                }
                let rows = ((b.page.1 - a.page.1) / down).round() as i64;
                let cols = ((b.page.0 - a.page.0) / across).round() as i64;
                let along = b.within as i64 - a.within as i64 - cols;
                if rows != 0 && along % rows == 0 && along / rows > 0 {
                    return Some((along / rows) as usize);
                }
            }
        }
        None
    }

    fn saw(&mut self, sighting: Sighting) {
        self.seen.retain(|s| s.index != sighting.index);
        self.seen.push(sighting);
        if self.seen.len() > 16 {
            self.seen.remove(0);
        }
    }
}

/// A grid of tiles to choose among and open: a folder's icons, a library
/// of photos. See the [module](self) for how it chooses.
///
/// Every tile is `size` CSS pixels wide, its picture square, and as many fit
/// a row as the grid's width holds; with `fill`, `size` is the least a tile
/// is, and the tiles stretch to fill the row, as a photo library's do.
/// `labels: false` draws the pictures alone, each named by its label to
/// assistive tech: a photo library's grid. A tile's `section` puts a
/// heading over it where the section changes; the grid is one grid still,
/// a range and ⌘A running across its sections.
///
/// `onselect` is given each new selection, `onactivate` the key of a tile
/// opened, `onaction` the key of a tile whose corner button was pressed.
/// `multiple: false` keeps one tile chosen at most; `toggles` makes every
/// press put a tile in or out; `opens` makes a press alone (a click, a tap)
/// open the tile, as a photo library's does, where a folder's opens by a
/// double click.
///
/// The arrows up and down move a row, across into the next section in the
/// same column. A browser says how wide the grid is; under Blitz, which
/// does not, the grid learns how many tiles a row holds from where the
/// pointer finds them, and until it has, up and down move one tile, as
/// left and right do.
#[component]
pub fn TileGrid(
    tiles: Vec<Tile>,
    selection: Selection,
    onselect: EventHandler<Selection>,
    onactivate: Option<EventHandler<String>>,
    onaction: Option<EventHandler<String>>,
    label: Option<String>,
    #[props(default = true)] multiple: bool,
    #[props(default)] toggles: bool,
    #[props(default)] opens: bool,
    /// A tile's width, and its picture's side, in CSS pixels: with `fill`,
    /// the least they are.
    #[props(default = 112.0)]
    size: f64,
    /// Tiles that stretch to fill the row, at least `size` wide.
    #[props(default)]
    fill: bool,
    /// Whether each tile's label and detail are drawn under it.
    #[props(default = true)]
    labels: bool,
    /// The selection's menu: its items' ids are given to `onpick`.
    #[props(default)]
    menu: Vec<Entry>,
    onpick: Option<EventHandler<usize>>,
    #[props(default)] class: String,
) -> Element {
    let unit = use_unit();
    let density = *unit.density.peek();
    let text = if labels { tile_text(density) } else { 0.0 };
    // How many tiles a row holds, where the renderer reports the width.
    let mut measured = use_signal(|| None::<usize>);
    let sightings = use_hook(|| CopyValue::new(Sightings::default()));

    let order: Vec<String> = tiles.iter().map(|t| t.key.clone()).collect();
    if sightings.peek().order != order {
        sightings.clone().set(Sightings {
            order: order.clone(),
            seen: Vec::new(),
        });
    }
    // Where each section starts, and each tile's section and place in it.
    let mut starts: Vec<usize> = Vec::new();
    let mut places: Vec<(usize, usize)> = Vec::with_capacity(tiles.len());
    for (i, t) in tiles.iter().enumerate() {
        if i == 0 || tiles[i - 1].section != t.section {
            starts.push(i);
        }
        let s = starts.len() - 1;
        places.push((s, i - starts[s]));
    }
    let labels_said: Vec<String> = tiles.iter().map(|t| t.label.clone()).collect();
    let fixed: Vec<String> = tiles
        .iter()
        .filter(|t| !t.selectable)
        .map(|t| t.key.clone())
        .collect();
    let chooser = Chooser {
        order,
        selection: selection.clone(),
        onselect,
        onactivate,
        multiple,
        toggles,
        opens,
        fixed,
        menu,
        onpick,
    };
    let focus = use_focus(Kind::Widget, {
        let chooser = chooser.clone();
        let starts = starts.clone();
        move |key, mods| {
            let seen = sightings.peek().clone();
            // Tiles `size` and the gap apart, or as far apart as two on
            // one row were seen to be where they stretch.
            let across = if fill {
                seen.stride().unwrap_or(size + GAP)
            } else {
                size + GAP
            };
            let down = across + text;
            let columns = measured.peek().or_else(|| seen.columns(across, down));
            let cols = columns.unwrap_or(1);
            let shape = Shape {
                starts: starts.clone(),
                columns: cols,
                len: chooser.order.len(),
            };
            let keyed = Keyed {
                order: &chooser.order,
                labels: &labels_said,
                selection: &chooser.selection,
                multiple: chooser.multiple,
                grid: true,
                columns: cols,
                page: cols * 3,
                shape: Some(&shape),
            };
            // A menu opened by a key opens at the cursor's tile, where the
            // pointer saw it, or at the tile it last saw.
            let at = || {
                let to = chooser.selection.cursor_in(&chooser.order).unwrap_or(0);
                let s = seen
                    .seen
                    .iter()
                    .find(|s| s.index == to)
                    .or(seen.seen.last());
                s.map_or((0.0, 0.0), |s| (s.window.0, s.window.1 + across - GAP))
            };
            chooser.key(unit, keyed.key(key, mods), at)
        }
    });
    let ring = focus.ring();
    let cursor = selection.cursor().map(str::to_string);
    let layout = if fill {
        format!("grid-template-columns: repeat(auto-fill, minmax({size}px, 1fr))")
    } else {
        String::new()
    };
    rsx! {
        div {
            class: if fill { "unit-tilegrid fill {class}" } else { "unit-tilegrid {class}" },
            style: "{layout}",
            role: "listbox",
            tabindex: "0",
            "aria-label": label,
            "aria-multiselectable": if multiple { "true" } else { "false" },
            "data-unit-focused": ring.then_some("true"),
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            onresize: move |e: ResizeEvent| {
                if let Ok(b) = e.get_border_box_size() {
                    // As many as fit at `size`: what CSS's auto-fill makes
                    // of `fill`'s columns too.
                    let fit = ((b.width - 2.0 * PAD + GAP) / (size + GAP)).floor().max(1.0) as usize;
                    if *measured.peek() != Some(fit) {
                        measured.set(Some(fit));
                    }
                }
            },
            onclick: {
                let chooser = chooser.clone();
                move |_| chooser.ground()
            },
            for (i, tile) in tiles.into_iter().enumerate() {
                TileView {
                    key: "{tile.key}",
                    heading: tile.section.clone().filter(|_| starts.contains(&i)),
                    selected: selection.contains(&tile.key),
                    cursor: ring && cursor.as_deref() == Some(tile.key.as_str()),
                    tile,
                    index: i,
                    place: places[i],
                    size,
                    fill,
                    text,
                    sightings,
                    chooser: chooser.clone(),
                    onaction,
                }
            }
        }
    }
}

#[component]
fn TileView(
    /// The section's heading, drawn before the tile, where a section
    /// starts with it.
    heading: Option<String>,
    tile: Tile,
    index: usize,
    place: (usize, usize),
    selected: bool,
    cursor: bool,
    size: f64,
    fill: bool,
    text: f64,
    sightings: CopyValue<Sightings>,
    chooser: Chooser,
    onaction: Option<EventHandler<String>>,
) -> Element {
    let unit = use_unit();
    let key = tile.key.clone();
    let held = {
        let (chooser, key) = (chooser.clone(), key.clone());
        crate::components::hold::use_hold(move |at| chooser.menu(unit, &key, at))
    };
    let seen = move |e: &PointerEvent| {
        let window = left_top(e);
        let inside = e.element_coordinates();
        let page = e.page_coordinates();
        sightings.clone().write().saw(Sighting {
            index,
            section: place.0,
            within: place.1,
            page: (page.x - inside.x, page.y - inside.y),
            window,
        });
    };
    let class = format!(
        "unit-tile{}{}{}",
        if selected { " on" } else { "" },
        if cursor { " cursor" } else { "" },
        if tile.selectable { "" } else { " fixed" }
    );
    let name = match &tile.detail {
        Some(d) => format!("{}, {d}", tile.label),
        None => tile.label.clone(),
    };
    // Its own size, or its cell's where the tiles stretch, the picture
    // square either way.
    let (style, face) = if fill {
        (String::new(), "aspect-ratio: 1 / 1".to_string())
    } else {
        (
            format!("width: {size}px; height: {}px", size + text),
            format!("height: {size}px"),
        )
    };
    let glyph = (size * 0.42).round();
    rsx! {
        if let Some(h) = heading {
            div { class: "unit-tilegrid-heading", role: "presentation", "{h}" }
        }
        div {
            class,
            role: "option",
            style,
            "aria-selected": if selected { "true" } else { "false" },
            "aria-label": "{name}",
            onpointerdown: move |e: PointerEvent| {
                seen(&e);
                held.down(&e);
            },
            onpointermove: move |e: PointerEvent| {
                seen(&e);
                held.moved(&e);
            },
            onpointerup: move |e: PointerEvent| held.up(&e),
            onpointercancel: move |e: PointerEvent| held.up(&e),
            onclick: {
                let (chooser, key) = (chooser.clone(), key.clone());
                move |e: MouseEvent| {
                    e.stop_propagation();
                    if held.take_held() {
                        return;
                    }
                    chooser.press(unit, &key, e.modifiers());
                }
            },
            ondoubleclick: {
                let (chooser, key) = (chooser.clone(), key.clone());
                move |e: MouseEvent| {
                    e.stop_propagation();
                    // A press already opened it where one does.
                    if *unit.density.peek() != Density::Touch && !chooser.opens {
                        chooser.activate(&key);
                    }
                }
            },
            oncontextmenu: {
                let (chooser, key) = (chooser.clone(), key.clone());
                move |e: MouseEvent| {
                    if chooser.has_menu() {
                        e.prevent_default();
                        e.stop_propagation();
                        let p = e.client_coordinates();
                        chooser.menu(unit, &key, (p.x, p.y));
                    }
                }
            },
            div { class: "unit-tile-face", style: "{face}",
                if let Some(src) = &tile.image {
                    img { class: "unit-tile-image", src: "{src}", alt: "", "aria-hidden": "true" }
                } else {
                    Icon { d: tile.icon.unwrap_or(paths::DOCUMENT), ink: Ink::Accent, size: glyph }
                }
                if let (Some((label, icon)), Some(f)) = (tile.action.clone(), onaction) {
                    crate::Button {
                        variant: crate::Variant::Ghost,
                        class: "unit-icon unit-tile-action",
                        label: "{label}",
                        icon,
                        onpress: {
                            let key = key.clone();
                            move |_| f.call(key.clone())
                        },
                    }
                }
            }
            if text > 0.0 {
                div { class: "unit-tile-text",
                    span { class: "unit-tile-label", "{tile.label}" }
                    span { class: "unit-tile-detail", {tile.detail.clone().unwrap_or_default()} }
                }
            }
        }
    }
}

/// One column of a [`Table`]: its title, and how it is laid out.
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub title: String,
    /// Its width in CSS pixels; with none, it takes what the others leave.
    pub width: Option<f64>,
    /// Its text at the right: a size, a count.
    pub end: bool,
    /// Its title is a button that sorts by it.
    pub sortable: bool,
    /// Left out at a finger's density, where a phone has no room for it.
    pub secondary: bool,
}

impl Column {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: None,
            end: false,
            sortable: false,
            secondary: false,
        }
    }

    pub fn width(mut self, px: f64) -> Self {
        self.width = Some(px);
        self
    }

    pub fn end(mut self) -> Self {
        self.end = true;
        self
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn secondary(mut self) -> Self {
        self.secondary = true;
        self
    }

    fn style(&self) -> String {
        match self.width {
            Some(w) => format!("flex: 0 0 {w}px; width: {w}px"),
            None => "flex: 1 1 0; min-width: 0".to_string(),
        }
    }

    fn class(&self, base: &str) -> String {
        format!(
            "{base}{}{}",
            if self.end { " end" } else { "" },
            if self.secondary {
                " unit-secondary"
            } else {
                ""
            }
        )
    }
}

/// One row of a [`Table`]: a cell for each column, in order. `key` is the
/// page's name for it, which a [`Selection`] holds; `icon` is drawn before
/// its first cell.
#[derive(Clone, Debug, PartialEq)]
pub struct TableRow {
    pub key: String,
    pub cells: Vec<String>,
    pub icon: Option<&'static str>,
}

impl TableRow {
    pub fn new(key: impl Into<String>, cells: Vec<String>) -> Self {
        Self {
            key: key.into(),
            cells,
            icon: None,
        }
    }

    pub fn icon(mut self, d: &'static str) -> Self {
        self.icon = Some(d);
        self
    }
}

/// Which column a table is sorted by, and which way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sort {
    pub column: usize,
    pub ascending: bool,
}

/// A table of rows to choose among and open, under a row of column titles:
/// a folder's files by name, size and date. A sortable column's title is a
/// button: pressed, `onsort` is given that column, ascending, or the other
/// way if it was already sorted by it, and the page sorts its rows. See the
/// [module](self) for how it chooses.
#[component]
pub fn Table(
    columns: Vec<Column>,
    rows: Vec<TableRow>,
    selection: Selection,
    onselect: EventHandler<Selection>,
    onactivate: Option<EventHandler<String>>,
    label: Option<String>,
    sort: Option<Sort>,
    onsort: Option<EventHandler<Sort>>,
    #[props(default = true)] multiple: bool,
    #[props(default)] toggles: bool,
    /// The selection's menu: its items' ids are given to `onpick`.
    #[props(default)]
    menu: Vec<Entry>,
    onpick: Option<EventHandler<usize>>,
    #[props(default)] class: String,
) -> Element {
    let unit = use_unit();
    let height = row_height(&unit);
    let origin = use_hook(|| CopyValue::new(None::<(f64, f64)>));
    let order: Vec<String> = rows.iter().map(|r| r.key.clone()).collect();
    let labels: Vec<String> = rows
        .iter()
        .map(|r| r.cells.first().cloned().unwrap_or_default())
        .collect();
    let chooser = Chooser {
        order: order.clone(),
        selection: selection.clone(),
        onselect,
        onactivate,
        multiple,
        toggles,
        opens: false,
        fixed: Vec::new(),
        menu,
        onpick,
    };
    let focus = use_focus(Kind::Widget, {
        let chooser = chooser.clone();
        move |key, mods| {
            let keyed = Keyed {
                order: &chooser.order,
                labels: &labels,
                selection: &chooser.selection,
                multiple: chooser.multiple,
                grid: false,
                columns: 1,
                page: 10,
                shape: None,
            };
            let at = || {
                let (x0, y0) = origin.peek().unwrap_or((0.0, 0.0));
                let i = chooser.selection.cursor_in(&chooser.order).unwrap_or(0);
                (x0 + 24.0, y0 + (i + 1) as f64 * height)
            };
            chooser.key(unit, keyed.key(key, mods), at)
        }
    });
    let ring = focus.ring();
    let cursor = selection.cursor().map(str::to_string);
    let count = rows.len() + 1;
    rsx! {
        div {
            class: "unit-table {class}",
            role: "grid",
            tabindex: "0",
            "aria-label": label,
            "aria-multiselectable": if multiple { "true" } else { "false" },
            "aria-rowcount": "{count}",
            "data-unit-focused": ring.then_some("true"),
            onpointerdown: move |_| focus.press(),
            onfocus: move |_| focus.dom_focus(),
            onblur: move |_| focus.dom_blur(),
            onclick: {
                let chooser = chooser.clone();
                move |_| chooser.ground()
            },
            div { class: "unit-thead", role: "row",
                for (i, c) in columns.iter().enumerate() {
                    HeaderCell { key: "{i}", column: c.clone(), index: i, sort, onsort }
                }
            }
            for (i, row) in rows.into_iter().enumerate() {
                RowView {
                    key: "{row.key}",
                    selected: selection.contains(&row.key),
                    cursor: ring && cursor.as_deref() == Some(row.key.as_str()),
                    row,
                    columns: columns.clone(),
                    height,
                    index: i,
                    origin,
                    chooser: chooser.clone(),
                }
            }
        }
    }
}

#[component]
fn HeaderCell(
    column: Column,
    index: usize,
    sort: Option<Sort>,
    onsort: Option<EventHandler<Sort>>,
) -> Element {
    let sorted = sort.filter(|s| s.column == index);
    let aria = match sorted {
        Some(Sort {
            ascending: true, ..
        }) => "ascending",
        Some(_) => "descending",
        None => "none",
    };
    let next = Sort {
        column: index,
        ascending: sorted.is_none_or(|s| !s.ascending),
    };
    let title = column.title.clone();
    rsx! {
        div {
            class: column.class("unit-th"),
            role: "columnheader",
            style: column.style(),
            "aria-sort": column.sortable.then_some(aria),
            if let (true, Some(f)) = (column.sortable, onsort) {
                crate::Button {
                    variant: crate::Variant::Ghost,
                    class: "unit-th-button",
                    onpress: move |_| f.call(next),
                    span { class: "unit-th-title", "{title}" }
                    if let Some(s) = sorted {
                        Icon { d: if s.ascending { paths::CHEVRON_UP } else { paths::CHEVRON_DOWN },
                               ink: Ink::Dim, size: 12.0 }
                    }
                }
            } else {
                span { class: "unit-th-title", "{title}" }
            }
        }
    }
}

#[component]
fn RowView(
    row: TableRow,
    columns: Vec<Column>,
    selected: bool,
    cursor: bool,
    height: f64,
    index: usize,
    origin: CopyValue<Option<(f64, f64)>>,
    chooser: Chooser,
) -> Element {
    let unit = use_unit();
    let key = row.key.clone();
    let held = {
        let (chooser, key) = (chooser.clone(), key.clone());
        crate::components::hold::use_hold(move |at| chooser.menu(unit, &key, at))
    };
    let seen = move |e: &PointerEvent| {
        let (x, y) = left_top(e);
        origin.clone().set(Some((x, y - index as f64 * height)));
    };
    let class = format!(
        "unit-tr{}{}",
        if selected { " on" } else { "" },
        if cursor { " cursor" } else { "" }
    );
    rsx! {
        div {
            class,
            role: "row",
            style: "min-height: {height}px",
            "aria-selected": if selected { "true" } else { "false" },
            "aria-rowindex": "{index + 2}",
            onpointerdown: move |e: PointerEvent| {
                seen(&e);
                held.down(&e);
            },
            onpointermove: move |e: PointerEvent| held.moved(&e),
            onpointerup: move |e: PointerEvent| held.up(&e),
            onpointercancel: move |e: PointerEvent| held.up(&e),
            onclick: {
                let (chooser, key) = (chooser.clone(), key.clone());
                move |e: MouseEvent| {
                    e.stop_propagation();
                    if held.take_held() {
                        return;
                    }
                    chooser.press(unit, &key, e.modifiers());
                }
            },
            ondoubleclick: {
                let (chooser, key) = (chooser.clone(), key.clone());
                move |e: MouseEvent| {
                    e.stop_propagation();
                    // A press already opened it where one does.
                    if *unit.density.peek() != Density::Touch && !chooser.opens {
                        chooser.activate(&key);
                    }
                }
            },
            oncontextmenu: {
                let (chooser, key) = (chooser.clone(), key.clone());
                move |e: MouseEvent| {
                    if chooser.has_menu() {
                        e.prevent_default();
                        e.stop_propagation();
                        let p = e.client_coordinates();
                        chooser.menu(unit, &key, (p.x, p.y));
                    }
                }
            },
            for (i, c) in columns.iter().enumerate() {
                div { key: "{i}", class: c.class("unit-td"), role: "gridcell", style: c.style(),
                    if let (0, Some(d)) = (i, row.icon) {
                        Icon { d, ink: Ink::Accent, size: 16.0 }
                    }
                    span { class: "unit-td-text", {row.cells.get(i).cloned().unwrap_or_default()} }
                }
            }
        }
    }
}

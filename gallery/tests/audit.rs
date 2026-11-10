//! The audit: every check, on everything the gallery draws, in every mode,
//! read from what Blitz computed and laid out and from the tree assistive
//! tech reads, not from the theme's values.
//!
//! A cell is one check of one thing in one mode: `text-contrast dark/touch/390
//! unit-note`. `gallery/audit.baseline` lists the cells known to fail, and it
//! only shrinks:
//!
//! - a cell that fails and is not listed fails this test: a regression;
//! - a cell that is listed and now passes fails it too, until
//!   `UNIT_AUDIT=tighten` takes it off the list, so a gain cannot slip back.
//!
//! Tighten only ever removes. A cell goes onto the list by hand, where a
//! reviewer sees it.
//!
//! ```sh
//! corgi test -p unit-gallery --test audit --force                     # the score, in target/audit.txt
//! UNIT_AUDIT=tighten corgi test -p unit-gallery --test audit --force  # take fixed cells off
//! ```
//!
//! `--force` because corgi keeps a pass until something the test read changes,
//! and the environment is not such a thing. Under cargo, the same with
//! `cargo test -p unit-gallery --test audit`.
//!
//! Scoring how a thing looks is not here: that is a person's to judge.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;
use std::task::{Context, Waker};

use accesskit::{Node, Role};
use atomic_refcell::AtomicRefCell;
use blitz_dom::{Document, DocumentConfig};
use blitz_traits::events::{
    BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, KeyState, MouseEventButton,
    MouseEventButtons, PointerCoords, PointerDetails, UiEvent,
};
use blitz_traits::node_id::NodeId;
use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::*;
use dioxus_native::DioxusDocument;
use keyboard_types::{Code, Key, Location, Modifiers, NamedKey};
use unit::prelude::*;
use unit_gallery::GalleryAt;

/// What the page is drawn at. The name is the middle of a cell's name.
#[derive(Clone, Copy)]
struct Mode {
    name: &'static str,
    appearance: Appearance,
    density: Density,
    width: u32,
    height: u32,
}

const fn mode(
    name: &'static str,
    appearance: Appearance,
    density: Density,
    width: u32,
    height: u32,
) -> Mode {
    Mode {
        name,
        appearance,
        density,
        width,
        height,
    }
}

const MODES: [Mode; 6] = [
    mode(
        "light/regular/1200",
        Appearance::Light,
        Density::Regular,
        1200,
        800,
    ),
    mode(
        "dark/regular/1200",
        Appearance::Dark,
        Density::Regular,
        1200,
        800,
    ),
    mode(
        "light/touch/390",
        Appearance::Light,
        Density::Touch,
        390,
        844,
    ),
    mode("dark/touch/390", Appearance::Dark, Density::Touch, 390, 844),
    mode(
        "light/touch/320",
        Appearance::Light,
        Density::Touch,
        320,
        640,
    ),
    mode("dark/touch/320", Appearance::Dark, Density::Touch, 320, 640),
];

/// An overlay opened by the button reading `button`, and whether the page
/// is scrolled to that button first.
const SCENES: [(&str, &str, bool); 11] = [
    ("menu", "Edit ▾", false),
    ("menu-scrolled", "Edit ▾", true),
    ("prompt", "Prompt…", false),
    ("alert", "Confirm…", false),
    ("actions", "Actions…", false),
    ("modal", "Modal…", false),
    ("sheet", "Sheet…", false),
    ("toast", "Toast", false),
    ("alert-scrolled", "Confirm…", true),
    ("actions-scrolled", "Actions…", true),
    ("viewer", "View Photos…", false),
];

/// What opens over the page: each must be on screen once it is open.
const OVERLAYS: &str =
    ".unit-dialog, .unit-actions, .unit-menu, .unit-modal, .unit-toast, .unit-viewer";

/// The controls whose edge is what shows where they are (WCAG 1.4.11).
const EDGED: &str = ".unit-check-box, .unit-radio-ring, .unit-switch-track, .unit-input, \
                     .unit-code-cell, .unit-slider-line";

/// The roles a person presses, types in or changes.
const INTERACTIVE: [Role; 19] = [
    Role::Button,
    Role::Link,
    Role::CheckBox,
    Role::Switch,
    Role::RadioButton,
    Role::Slider,
    Role::SpinButton,
    Role::Tab,
    Role::ListBoxOption,
    Role::MenuItem,
    Role::MenuItemCheckBox,
    Role::MenuItemRadio,
    Role::TreeItem,
    Role::ComboBox,
    Role::TextInput,
    Role::MultilineTextInput,
    Role::SearchInput,
    Role::EmailInput,
    Role::PasswordInput,
];

/// The checks, for the score's table and to prove that each one ran.
const CHECKS: [&str; 9] = [
    "name",
    "state",
    "target-size",
    "text-contrast",
    "edge-contrast",
    "keyboard",
    "focus-ring",
    "overflow",
    "on-screen",
];

/// The props of a page the audit builds.
#[derive(Props, Clone, PartialEq)]
struct At {
    appearance: Appearance,
    density: Density,
}

impl At {
    fn of(m: Mode) -> At {
        At {
            appearance: m.appearance,
            density: m.density,
        }
    }
}

#[allow(non_snake_case)]
fn WholeGallery(at: At) -> Element {
    rsx! {
        GalleryAt { appearance: at.appearance, density: at.density }
    }
}

/// Labels far longer than the gallery's: what a translation or a person's
/// own data puts in a control.
#[allow(non_snake_case)]
fn LongText(at: At) -> Element {
    const LONG: &str = "A label a good deal longer than any in the gallery, as a translation has";
    const WORD: &str = "Donaudampfschifffahrtsgesellschaftskapitänsmützenabzeichen";
    let options: Vec<String> = vec![LONG.into(), WORD.into()];
    rsx! {
        UnitRoot { appearance: at.appearance, density: at.density,
            Stack { gap: 12.0,
                Heading { level: 1, "{LONG}" }
                Heading { level: 1, "{WORD}" }
                Text { "{WORD}" }
                Button { onpress: |_| {}, "{LONG}" }
                Button { onpress: |_| {}, "{WORD}" }
                Checkbox { checked: false, onchange: |_| {}, "{LONG}" }
                Switch { on: false, onchange: |_| {}, "{WORD}" }
                Segmented { label: "Long", options: options.clone(), selected: Some(0), onchange: |_| {} }
                Tabs { label: "Long", tabs: options.clone(), selected: 0, onchange: |_| {}, Text { "{LONG}" } }
                Chips { label: "Long", choices: vec![Choice::new(LONG), Choice::new(WORD)], on: vec![false, false], onpick: |_| {} }
                Card { title: LONG, Text { "{WORD}" } }
                Callout { title: LONG, "{WORD}" }
                Section { title: LONG, footer: LONG,
                    Row { title: LONG, subtitle: LONG, detail: WORD, onpress: |_| {} }
                    Row { title: WORD, accessory: Accessory::Chevron, onpress: |_| {} }
                }
                Badge { "{WORD}" }
                NavBar { title: LONG, back: WORD, onback: |_| {} }
            }
        }
    }
}

fn settle(doc: &mut DioxusDocument) {
    for _ in 0..3 {
        Document::poll(doc, Some(Context::from_waker(Waker::noop())));
        doc.inner.borrow_mut().resolve(0.0);
    }
}

fn laid_out(vdom: VirtualDom, m: Mode) -> DioxusDocument {
    let mut doc = DioxusDocument::new(vdom, DocumentConfig::default());
    let scheme = match m.appearance {
        Appearance::Dark => ColorScheme::Dark,
        _ => ColorScheme::Light,
    };
    doc.inner
        .borrow_mut()
        .set_viewport(Viewport::new(m.width, m.height, 1.0, scheme));
    doc.initial_build();
    settle(&mut doc);
    doc
}

fn gallery(m: Mode) -> DioxusDocument {
    laid_out(VirtualDom::new_with_props(WholeGallery, At::of(m)), m)
}

fn long_text(m: Mode) -> DioxusDocument {
    laid_out(VirtualDom::new_with_props(LongText, At::of(m)), m)
}

fn rect(doc: &DioxusDocument, node: NodeId) -> (f64, f64, f64, f64) {
    let r = doc.inner.borrow().get_client_bounding_rect(node).unwrap();
    (r.x, r.y, r.width, r.height)
}

/// A pointer at `x`, `y` in the window, on a page scrolled down by `top`.
fn pointer(x: f64, y: f64, top: f64, buttons: MouseEventButtons) -> BlitzPointerEvent {
    let (x, y, top) = (x as f32, y as f32, top as f32);
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y + top,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons,
        mods: Modifiers::empty(),
        details: PointerDetails::default(),
        element: Default::default(),
        active_pointers: Arc::new(AtomicRefCell::new(Vec::new())),
    }
}

fn click(doc: &mut DioxusDocument, node: NodeId) {
    let (x, y, w, h) = rect(doc, node);
    let (x, y) = (x + w / 2.0, y + h / 2.0);
    let top = doc.inner.borrow().viewport_scroll().y;
    let none = MouseEventButtons::None;
    doc.handle_ui_event(UiEvent::PointerMove(pointer(x, y, top, none)));
    doc.handle_ui_event(UiEvent::PointerDown(pointer(
        x,
        y,
        top,
        MouseEventButtons::Primary,
    )));
    settle(doc);
    doc.handle_ui_event(UiEvent::PointerUp(pointer(x, y, top, none)));
    settle(doc);
}

fn tab(doc: &mut DioxusDocument) {
    let event = |state| BlitzKeyEvent {
        key: Key::Named(NamedKey::Tab),
        code: Code::Tab,
        modifiers: Modifiers::empty(),
        location: Location::Standard,
        is_auto_repeating: false,
        is_composing: false,
        state,
        text: None,
    };
    doc.handle_ui_event(UiEvent::KeyDown(event(KeyState::Pressed)));
    doc.handle_ui_event(UiEvent::KeyUp(event(KeyState::Released)));
    settle(doc);
}

/// One computed property of an element, empty for one under `display: none`,
/// which has no stored style: Blitz computing one there trips a debug
/// assertion in stylo, and nothing hidden is drawn for a check to judge.
fn css(doc: &DioxusDocument, node: NodeId, property: &str) -> String {
    let inner = doc.inner.borrow();
    if inner
        .get_node(node)
        .is_none_or(|n| n.primary_styles().is_none())
    {
        return String::new();
    }
    inner.resolved_style_value(node, property)
}

fn px(value: &str) -> f64 {
    value.trim_end_matches("px").parse().unwrap_or(0.0)
}

fn parent(doc: &DioxusDocument, node: NodeId) -> Option<NodeId> {
    doc.inner.borrow().get_node(node)?.parent
}

fn attr(doc: &DioxusDocument, node: NodeId, name: &str) -> Option<String> {
    let inner = doc.inner.borrow();
    inner.get_node(node)?.element_data().and_then(|e| {
        e.attrs()
            .iter()
            .find(|a| &*a.name.local == name)
            .map(|a| a.value.to_string())
    })
}

/// `node` and each element round it, nearest first.
fn lineage(doc: &DioxusDocument, node: NodeId) -> Vec<NodeId> {
    let mut all = vec![node];
    while let Some(p) = parent(doc, *all.last().unwrap()) {
        all.push(p);
    }
    all.retain(|&n| {
        let inner = doc.inner.borrow();
        inner.get_node(n).is_some_and(|n| n.is_element())
    });
    all
}

/// An sRGB colour with its alpha, each 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Colour(f64, f64, f64, f64);

impl Colour {
    const WHITE: Colour = Colour(1.0, 1.0, 1.0, 1.0);

    /// A colour as `resolved_style_value` writes it.
    fn parse(value: &str) -> Option<Colour> {
        if value == "transparent" {
            return Some(Colour(0.0, 0.0, 0.0, 0.0));
        }
        let inner = value
            .strip_prefix("rgba(")
            .or_else(|| value.strip_prefix("rgb("))?
            .strip_suffix(')')?;
        let v: Vec<f64> = inner
            .split([',', ' ', '/'])
            .filter(|s| !s.is_empty())
            .map(|s| s.parse())
            .collect::<Result<_, _>>()
            .ok()?;
        match v[..] {
            [r, g, b] => Some(Colour(r / 255.0, g / 255.0, b / 255.0, 1.0)),
            [r, g, b, a] => Some(Colour(r / 255.0, g / 255.0, b / 255.0, a)),
            _ => None,
        }
    }

    /// This colour, faded by `alpha`, drawn over `under`.
    fn over(self, under: Colour, alpha: f64) -> Colour {
        let a = self.3 * alpha;
        let mix = |c: f64, u: f64| c * a + u * (1.0 - a);
        Colour(
            mix(self.0, under.0),
            mix(self.1, under.1),
            mix(self.2, under.2),
            1.0,
        )
    }

    fn luminance(self) -> f64 {
        let linear = |c: f64| {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(self.0) + 0.7152 * linear(self.1) + 0.0722 * linear(self.2)
    }

    /// WCAG's contrast ratio, 1 to 21.
    fn contrast(self, other: Colour) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
}

fn colour(doc: &DioxusDocument, node: NodeId, property: &str) -> Colour {
    let value = css(doc, node, property);
    if value == "currentcolor" {
        return colour(doc, node, "color");
    }
    Colour::parse(&value).unwrap_or_else(|| panic!("{property} is {value:?}"))
}

/// What is painted behind `node`: the backgrounds of the elements round it,
/// `node`'s own among them if `own`.
fn backing(doc: &DioxusDocument, node: NodeId, own: bool) -> Colour {
    let line = lineage(doc, node);
    let line = if own { &line[..] } else { &line[1..] };
    let mut layers = Vec::new();
    for &n in line {
        let c = colour(doc, n, "background-color");
        layers.push(c);
        if c.3 >= 1.0 {
            break;
        }
    }
    layers
        .iter()
        .rev()
        .fold(Colour::WHITE, |under, c| c.over(under, 1.0))
}

/// The opacity `node` is drawn at, its parents' included.
fn opacity(doc: &DioxusDocument, node: NodeId) -> f64 {
    lineage(doc, node)
        .iter()
        .map(|&n| css(doc, n, "opacity").parse().unwrap_or(1.0))
        .product()
}

/// Whether `node` is in something disabled, which WCAG leaves out.
fn disabled(doc: &DioxusDocument, node: NodeId) -> bool {
    lineage(doc, node).iter().any(|&n| {
        attr(doc, n, "disabled").is_some()
            || attr(doc, n, "aria-disabled").as_deref() == Some("true")
            || attr(doc, n, "class").is_some_and(|c| c.split_whitespace().any(|c| c == "off"))
    })
}

/// A name for what `node` is in, by the classes of the nearest element that
/// has any: the same from run to run, whatever a check measured of it. None
/// for the gallery's own elements, which are not unit's to answer for.
fn owner(doc: &DioxusDocument, node: NodeId) -> Option<String> {
    for n in lineage(doc, node) {
        let Some(class) = attr(doc, n, "class") else {
            continue;
        };
        if class.split_whitespace().any(|c| c.starts_with("gallery-")) {
            return None;
        }
        let kept: Vec<_> = class
            .split_whitespace()
            .filter(|c| *c != "unit-focused")
            .collect();
        if !kept.is_empty() {
            return Some(kept.join("."));
        }
    }
    None
}

/// What a control is called: its label, or the text in it.
fn name_of(doc: &DioxusDocument, id: NodeId, node: &Node) -> String {
    let text = node.label().map(str::to_string).unwrap_or_else(|| {
        let inner = doc.inner.borrow();
        inner
            .get_node(id)
            .map_or(String::new(), |n| n.text_content())
    });
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    text.chars().take(32).collect()
}

fn control(doc: &DioxusDocument, id: NodeId, node: &Node) -> String {
    format!(
        "{:?}:{}",
        node.role(),
        name_of(doc, id, node).replace(' ', "_")
    )
}

/// The tree assistive tech reads, by the DOM's node ids, which are its ids.
fn tree(doc: &DioxusDocument) -> Vec<(NodeId, Node)> {
    let nodes = doc.inner.borrow().build_accessibility_tree().nodes;
    nodes
        .into_iter()
        .filter(|(id, _)| id.0 != u64::MAX)
        .map(|(id, n)| (NodeId::from_u64(id.0), n))
        .collect()
}

/// Every cell seen, and why each that failed did.
#[derive(Default)]
struct Audit {
    seen: BTreeSet<String>,
    failed: BTreeMap<String, String>,
}

impl Audit {
    fn cell(
        &mut self,
        check: &str,
        m: Mode,
        subject: &str,
        ok: bool,
        why: impl FnOnce() -> String,
    ) {
        assert!(CHECKS.contains(&check), "{check} is not in CHECKS");
        let cell = format!("{check} {} {}", m.name, subject.replace(' ', "_"));
        self.seen.insert(cell.clone());
        if !ok {
            self.failed.entry(cell).or_insert_with(why);
        }
    }

    /// What can be asked of any page: what it names, how it reads, how large
    /// its controls are, and whether its text can be read.
    fn page(&mut self, doc: &DioxusDocument, m: Mode, scene: &str) {
        let nodes = tree(doc);
        for (id, node) in &nodes {
            if INTERACTIVE.contains(&node.role()) {
                self.control(doc, m, scene, *id, node);
            }
            if node.role() == Role::Label {
                self.text(doc, m, scene, *id);
            }
            self.overflow(doc, m, scene, *id);
        }
        self.edges(doc, m, scene, &nodes);
    }

    fn control(&mut self, doc: &DioxusDocument, m: Mode, scene: &str, id: NodeId, node: &Node) {
        let (_, _, w, h) = rect(doc, id);
        if w == 0.0 && h == 0.0 {
            return;
        }
        let subject = control(doc, id, node);
        let role = node.role();
        let named = !name_of(doc, id, node).is_empty();
        self.cell("name", m, &subject, named, || format!("{scene}: no name"));

        let read = match role {
            Role::CheckBox | Role::Switch | Role::RadioButton | Role::MenuItemCheckBox => {
                node.toggled().is_some()
            }
            Role::Tab | Role::ListBoxOption => node.is_selected().is_some(),
            Role::Slider => node.numeric_value().is_some(),
            _ => true,
        };
        self.cell("state", m, &subject, read, || {
            format!("{scene}: {role:?} says nothing of its state")
        });

        if !node.is_disabled() && owner(doc, id).is_some() {
            // Apple's 44 points for a finger, WCAG 2.5.8's 24 for a pointer.
            let least = if m.density == Density::Touch {
                44.0
            } else {
                24.0
            };
            self.cell("target-size", m, &subject, w >= least && h >= least, || {
                format!("{scene}: {w:.0}×{h:.0}, under {least}")
            });
        }
    }

    fn text(&mut self, doc: &DioxusDocument, m: Mode, scene: &str, id: NodeId) {
        let Some(element) = parent(doc, id) else {
            return;
        };
        let element = lineage(doc, element)[0];
        let (_, _, w, h) = rect(doc, element);
        if w == 0.0 || h == 0.0 || disabled(doc, element) {
            return;
        }
        let Some(subject) = owner(doc, element) else {
            return;
        };
        let behind = backing(doc, element, true);
        let ink = colour(doc, element, "color").over(behind, opacity(doc, element));
        let ratio = ink.contrast(behind);
        let size = px(&css(doc, element, "font-size"));
        let weight: f64 = css(doc, element, "font-weight").parse().unwrap_or(400.0);
        // WCAG's large text: 18 points, or 14 and bold, at 4/3 of a pixel each.
        let large = size >= 24.0 || (size >= 18.66 && weight >= 700.0);
        // A glyph hidden from assistive tech is a drawing, not text: a
        // select's arrow, held to 1.4.11's 3:1 as an icon is.
        let drawn = lineage(doc, element)
            .iter()
            .any(|&n| attr(doc, n, "aria-hidden").as_deref() == Some("true"));
        let least = if large || drawn { 3.0 } else { 4.5 };
        self.cell("text-contrast", m, &subject, ratio >= least, || {
            format!("{scene}: {ratio:.2}:1, under {least}:1")
        });
    }

    fn edges(&mut self, doc: &DioxusDocument, m: Mode, scene: &str, nodes: &[(NodeId, Node)]) {
        let shown: BTreeSet<NodeId> = nodes.iter().map(|(id, _)| *id).collect();
        let all = doc.inner.borrow().query_selector_all(EDGED).unwrap();
        for id in all {
            let (_, _, w, h) = rect(doc, id);
            if !shown.contains(&id) || w == 0.0 || h == 0.0 || disabled(doc, id) {
                continue;
            }
            let Some(subject) = owner(doc, id) else {
                continue;
            };
            let behind = backing(doc, id, false);
            let face = colour(doc, id, "background-color").over(behind, 1.0);
            let edge = if px(&css(doc, id, "border-top-width")) > 0.0 {
                colour(doc, id, "border-top-color").over(behind, 1.0)
            } else {
                face
            };
            let ratio = face.contrast(behind).max(edge.contrast(behind));
            self.cell("edge-contrast", m, &subject, ratio >= 3.0, || {
                format!("{scene}: {ratio:.2}:1, under 3:1")
            });
        }
    }

    /// Whether `id`'s box stays inside the page's width, unless something
    /// round it scrolls or clips it there.
    fn overflow(&mut self, doc: &DioxusDocument, m: Mode, scene: &str, id: NodeId) {
        let is_element = doc
            .inner
            .borrow()
            .get_node(id)
            .is_some_and(|n| n.is_element());
        if !is_element {
            return;
        }
        let (x, _, w, h) = rect(doc, id);
        if w == 0.0 || h == 0.0 {
            return;
        }
        let clipped = lineage(doc, id).iter().skip(1).any(|&n| {
            !matches!(css(doc, n, "overflow-x").as_str(), "visible" | "")
                && attr(doc, n, "class").is_some()
        });
        let Some(subject) = owner(doc, id) else {
            return;
        };
        if clipped {
            return;
        }
        let page = m.width as f64;
        self.cell(
            "overflow",
            m,
            &subject,
            x >= -1.0 && x + w <= page + 1.0,
            || format!("{scene}: {x:.0} to {:.0} of {page}", x + w),
        );
    }

    /// Whether what `scene` opened is wholly in the window.
    fn on_screen(&mut self, doc: &DioxusDocument, m: Mode, scene: &str) {
        let all = doc.inner.borrow().query_selector_all(OVERLAYS).unwrap();
        let shown: BTreeSet<NodeId> = tree(doc).into_iter().map(|(id, _)| id).collect();
        let mut opened = false;
        for id in all {
            let (x, y, w, h) = rect(doc, id);
            if !shown.contains(&id) || w == 0.0 || h == 0.0 {
                continue;
            }
            opened = true;
            let (pw, ph) = (m.width as f64, m.height as f64);
            let inside = x >= -1.0 && y >= -1.0 && x + w <= pw + 1.0 && y + h <= ph + 1.0;
            self.cell("on-screen", m, scene, inside, || {
                format!("{x:.0},{y:.0} {w:.0}×{h:.0} in {pw}×{ph}")
            });
        }
        assert!(opened, "{scene} opened nothing at {}", m.name);
    }

    /// Tab from the top of the page to its end: every control reached, each
    /// showing a ring that can be seen. A list's rows and a group's choices
    /// are reached through the list or the group, by the arrows.
    fn keyboard(&mut self, doc: &mut DioxusDocument, m: Mode) {
        let first = {
            let inner = doc.inner.borrow();
            let all = inner
                .query_selector_all("button, input, textarea, [tabindex]")
                .unwrap();
            *all.iter()
                .find(|&&id| inner.get_node(id).is_some_and(|n| n.is_focussable()))
                .expect("something to focus")
        };
        doc.inner.borrow_mut().set_focus_to(first);
        settle(doc);

        let nodes: BTreeMap<NodeId, Node> = tree(doc).into_iter().collect();
        let mut visited = BTreeSet::new();
        loop {
            let at = doc.inner.borrow().get_focussed_node_id();
            let Some(at) = at.filter(|&at| visited.insert(at)) else {
                break;
            };
            match nodes.get(&at) {
                Some(node) => self.ring(doc, m, at, node),
                // Out of the tree: under `display: none`, and drawn nowhere.
                None => {
                    let hidden = owner(doc, at).unwrap_or_default();
                    self.cell("keyboard", m, &format!("hidden:{hidden}"), false, || {
                        "Tab stopped on something not shown".to_string()
                    });
                }
            }
            tab(doc);
        }

        for (&id, node) in &nodes {
            if !INTERACTIVE.contains(&node.role()) || node.is_disabled() {
                continue;
            }
            let (_, _, w, h) = rect(doc, id);
            if w == 0.0 || h == 0.0 {
                continue;
            }
            let reached = lineage(doc, id).iter().any(|n| visited.contains(n))
                || visited.iter().any(|&v| lineage(doc, v).contains(&id));
            let subject = control(doc, id, node);
            self.cell("keyboard", m, &subject, reached, || {
                "Tab never reached it".to_string()
            });
        }
    }

    fn ring(&mut self, doc: &DioxusDocument, m: Mode, at: NodeId, node: &Node) {
        let subject = control(doc, at, node);
        let marked = doc
            .inner
            .borrow()
            .query_selector_all("[data-unit-focused], .unit-focused")
            .unwrap();
        let ring = lineage(doc, at).into_iter().chain(marked).find(|&n| {
            css(doc, n, "outline-style") != "none" && px(&css(doc, n, "outline-width")) > 0.0
        });
        let Some(ring) = ring else {
            self.cell("focus-ring", m, &subject, false, || "no ring".to_string());
            return;
        };
        let behind = backing(doc, ring, false);
        let ratio = colour(doc, ring, "outline-color")
            .over(behind, 1.0)
            .contrast(behind);
        self.cell("focus-ring", m, &subject, ratio >= 3.0, || {
            format!("{ratio:.2}:1, under 3:1")
        });
    }
}

/// Every check in every mode.
fn audit() -> Audit {
    let mut a = Audit::default();
    for m in MODES {
        let mut doc = gallery(m);
        a.page(&doc, m, "page");
        a.keyboard(&mut doc, m);

        for (scene, button, scrolled) in SCENES {
            let mut doc = gallery(m);
            let target = {
                let inner = doc.inner.borrow();
                let all = inner.query_selector_all("button").unwrap();
                *all.iter()
                    .find(|&&id| inner.get_node(id).unwrap().text_content().trim() == button)
                    .unwrap_or_else(|| panic!("no button reading {button:?}"))
            };
            if scrolled {
                let (_, y, _, _) = rect(&doc, target);
                let to = blitz_dom::Point {
                    x: 0.0,
                    y: y - 100.0,
                };
                doc.inner.borrow_mut().set_viewport_scroll(to);
                settle(&mut doc);
                let (_, moved, _, _) = rect(&doc, target);
                assert!(moved < y, "{scene} did not scroll the page at {}", m.name);
            }
            // A menu opens at its button: only as on screen as the button.
            let (_, y, _, h) = rect(&doc, target);
            let opener_shown = y >= 0.0 && y + h <= m.height as f64;
            click(&mut doc, target);
            a.page(&doc, m, scene);
            if opener_shown || !scene.starts_with("menu") {
                a.on_screen(&doc, m, scene);
            }
        }

        let doc = long_text(m);
        a.page(&doc, m, "long-text");
    }
    a
}

const BASELINE: &str = "audit.baseline";
const REPORT: &str = "../target/audit.txt";

fn listed(path: &Path) -> BTreeSet<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// `baseline`'s text with `fixed` taken out and everything else as it was.
fn tightened(baseline: &str, fixed: &BTreeSet<String>) -> String {
    baseline
        .lines()
        .filter(|l| !fixed.contains(l.trim()))
        .map(|l| format!("{l}\n"))
        .collect()
}

/// The score, a row for each check, and each failure with why it failed.
fn report(a: &Audit) -> String {
    let total = a.seen.len();
    let passing = total - a.failed.len();
    let percent = passing as f64 * 100.0 / total as f64;
    let mut out = format!("unit audit: {passing} of {total} cells pass ({percent:.1}%)\n\n");
    out += &format!("{:<15}{:>6}{:>6}\n", "check", "pass", "fail");
    for check in CHECKS {
        let of = |c: &&String| c.split(' ').next() == Some(check);
        let seen = a.seen.iter().filter(of).count();
        let failed = a.failed.keys().filter(of).count();
        out += &format!("{check:<15}{:>6}{failed:>6}\n", seen - failed);
    }
    out.push('\n');
    for (cell, why) in &a.failed {
        out += &format!("{cell}  # {why}\n");
    }
    out
}

/// The list of failures only shrinks. See the top of this file.
#[test]
fn the_audit_holds() {
    let a = audit();
    // A file, as corgi shows nothing a passing test prints.
    std::fs::create_dir_all("../target").unwrap();
    std::fs::write(REPORT, report(&a)).unwrap();
    let mode = std::env::var("UNIT_AUDIT").unwrap_or_default();

    for check in CHECKS {
        let ran = a.seen.iter().any(|c| c.split(' ').next() == Some(check));
        assert!(ran, "{check} looked at nothing: a check gone quiet");
    }

    let path = Path::new(BASELINE);
    let known = listed(path);
    let failed: BTreeSet<String> = a.failed.keys().cloned().collect();
    let new: Vec<String> = failed
        .difference(&known)
        .map(|c| format!("  {c}  # {}", a.failed[c]))
        .collect();
    let fixed: BTreeSet<String> = known.difference(&failed).cloned().collect();

    assert!(
        new.is_empty(),
        "{} new failures, not in {BASELINE}:\n{}",
        new.len(),
        new.join("\n")
    );
    if mode == "tighten" {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        std::fs::write(path, tightened(&text, &fixed)).unwrap();
        return;
    }
    assert!(
        fixed.is_empty(),
        "{} cells now pass: UNIT_AUDIT=tighten takes them off {BASELINE}:\n  {}",
        fixed.len(),
        fixed.iter().cloned().collect::<Vec<_>>().join("\n  ")
    );
}

/// A component the gallery does not show is one the audit cannot see.
#[test]
fn the_gallery_shows_every_component() {
    let page = std::fs::read_to_string("src/lib.rs").unwrap();
    let mut missing = Vec::new();
    for file in std::fs::read_dir("../src/components").unwrap() {
        let source = std::fs::read_to_string(file.unwrap().path()).unwrap();
        let names = source.lines().filter_map(|l| {
            let name = l.trim().strip_prefix("pub fn ")?.split('(').next()?;
            name.starts_with(char::is_uppercase).then_some(name)
        });
        for name in names {
            if !page.contains(&format!("{name} {{")) {
                missing.push(name.to_string());
            }
        }
    }
    assert!(missing.is_empty(), "not in the gallery: {missing:?}");
}

#[test]
fn contrast_is_wcags() {
    let black = Colour(0.0, 0.0, 0.0, 1.0);
    assert!((black.contrast(Colour::WHITE) - 21.0).abs() < 1e-9);
    let dim = Colour::parse("rgb(134, 134, 139)").unwrap();
    assert!((dim.contrast(Colour::WHITE) - 3.62).abs() < 0.01);
}

#[test]
fn a_colour_is_read_as_blitz_writes_it() {
    assert_eq!(
        Colour::parse("rgba(0, 0, 0, 0.25)"),
        Some(Colour(0.0, 0.0, 0.0, 0.25))
    );
    assert_eq!(Colour::parse("transparent").map(|c| c.3), Some(0.0));
    assert_eq!(Colour::parse("hsl(0 0% 0%)"), None);
}

#[test]
fn half_a_black_over_white_is_grey() {
    let grey = Colour(0.0, 0.0, 0.0, 0.5).over(Colour::WHITE, 1.0);
    assert_eq!(grey, Colour(0.5, 0.5, 0.5, 1.0));
}

#[test]
fn tightening_only_removes() {
    let fixed = BTreeSet::from(["a".to_string()]);
    assert_eq!(tightened("# why\na\nb\n", &fixed), "# why\nb\n");
}

//! The gallery laid out by Blitz with no window, and pressed, tapped and
//! typed at as a window's events would: what breaks Blitz's layout, a
//! control that a finger cannot press, or a tree that assistive tech cannot
//! read breaks here, in a test.

use std::sync::Arc;
use std::task::{Context, Waker};

use accesskit::{Node, NodeId as AxId, Role, Toggled};
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

/// The gallery, `width` × `height` CSS pixels, built and laid out.
fn page(width: u32, height: u32) -> DioxusDocument {
    let vdom = VirtualDom::new(unit_gallery::Gallery);
    let mut doc = DioxusDocument::new(vdom, DocumentConfig::default());
    doc.inner
        .borrow_mut()
        .set_viewport(Viewport::new(width, height, 1.0, ColorScheme::Light));
    doc.initial_build();
    settle(&mut doc);
    doc
}

/// Poll the page and lay it out, as a window's turn and its next frame do.
fn settle(doc: &mut DioxusDocument) {
    for _ in 0..3 {
        Document::poll(doc, Some(Context::from_waker(Waker::noop())));
        doc.inner.borrow_mut().resolve(0.0);
    }
}

/// The first element matching `selector` whose text is `text`.
fn find(doc: &DioxusDocument, selector: &str, text: &str) -> NodeId {
    let inner = doc.inner.borrow();
    let all = inner.query_selector_all(selector).unwrap();
    let text_of = |id: NodeId| inner.get_node(id).unwrap().text_content();
    // Exactly, or after a mark drawn before it: a checkbox's tick.
    *all.iter()
        .find(|&&id| text_of(id).trim() == text)
        .or_else(|| all.iter().find(|&&id| text_of(id).trim().ends_with(text)))
        .unwrap_or_else(|| panic!("no {selector} reading {text:?}"))
}

/// The box of `node`. A box under a hidden one keeps the box it last had:
/// ask whether the box that is hidden is displayed.: left, top, width, height.
fn rect(doc: &DioxusDocument, node: NodeId) -> (f64, f64, f64, f64) {
    let r = doc.inner.borrow().get_client_bounding_rect(node).unwrap();
    (r.x, r.y, r.width, r.height)
}

fn pointer(id: BlitzPointerId, x: f64, y: f64, buttons: MouseEventButtons) -> BlitzPointerEvent {
    let (x, y) = (x as f32, y as f32);
    BlitzPointerEvent {
        id,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
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

/// A press and release at the middle of `node`, by `id`: the mouse or a
/// finger.
fn tap_by(doc: &mut DioxusDocument, node: NodeId, id: BlitzPointerId) {
    let (x, y, w, h) = rect(doc, node);
    let (x, y) = (x + w / 2.0, y + h / 2.0);
    doc.handle_ui_event(UiEvent::PointerMove(pointer(
        id,
        x,
        y,
        MouseEventButtons::None,
    )));
    doc.handle_ui_event(UiEvent::PointerDown(pointer(
        id,
        x,
        y,
        MouseEventButtons::Primary,
    )));
    settle(doc);
    doc.handle_ui_event(UiEvent::PointerUp(pointer(
        id,
        x,
        y,
        MouseEventButtons::None,
    )));
    settle(doc);
}

fn click(doc: &mut DioxusDocument, node: NodeId) {
    tap_by(doc, node, BlitzPointerId::Mouse);
}

fn tap(doc: &mut DioxusDocument, node: NodeId) {
    tap_by(doc, node, BlitzPointerId::Finger(1));
}

fn key(doc: &mut DioxusDocument, key: Key) {
    let event = |state| BlitzKeyEvent {
        key: key.clone(),
        code: Code::Unidentified,
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

/// What the gallery says the last control did.
fn said(doc: &DioxusDocument) -> String {
    let inner = doc.inner.borrow();
    let node = inner.query_selector(".gallery-said").unwrap().unwrap();
    inner
        .get_node(node)
        .unwrap()
        .text_content()
        .trim()
        .to_string()
}

fn displayed(doc: &DioxusDocument, node: NodeId) -> bool {
    let (_, _, w, h) = rect(doc, node);
    w > 0.0 && h > 0.0
}

#[test]
fn the_gallery_lays_out_on_a_desktop_and_a_phone() {
    for (w, h) in [(1200, 800), (390, 844)] {
        let doc = page(w, h);
        let root = doc
            .inner
            .borrow()
            .query_selector(".unit-root")
            .unwrap()
            .unwrap();
        let (_, _, rw, _) = rect(&doc, root);
        assert_eq!(rw, w as f64, "the root fills the page's width");
    }
}

#[test]
fn a_button_is_pressed_by_the_mouse_and_by_a_finger() {
    let mut doc = page(1200, 800);
    let primary = find(&doc, "button", "Primary");
    click(&mut doc, primary);
    assert_eq!(said(&doc), "Primary pressed.");
    let ghost = find(&doc, "button", "Ghost");
    tap(&mut doc, ghost);
    assert_eq!(said(&doc), "Ghost pressed.");
    let disabled = find(&doc, "button", "Disabled");
    click(&mut doc, disabled);
    assert_eq!(
        said(&doc),
        "Ghost pressed.",
        "a disabled button does nothing"
    );
}

#[test]
fn a_menu_opens_under_its_button_inside_the_page_and_picks() {
    for finger in [false, true] {
        let mut doc = page(1200, 800);
        let edit = find(&doc, "button", "Edit ▾");
        let (bx, by, _, bh) = rect(&doc, edit);
        if finger {
            tap(&mut doc, edit)
        } else {
            click(&mut doc, edit)
        }
        let menu = doc
            .inner
            .borrow()
            .query_selector(".unit-menu")
            .unwrap()
            .unwrap();
        assert!(displayed(&doc, menu), "the menu is shown");
        let (mx, my, mw, mh) = rect(&doc, menu);
        assert_eq!(mx, bx, "under the button's left edge");
        assert!(
            (my - (by + bh)).abs() < 4.0,
            "under the button: {my} vs {}",
            by + bh
        );
        assert!(mx + mw <= 1200.0 && my + mh <= 800.0);
        let copy = find(&doc, ".unit-item", "Copy⌘C");
        if finger {
            tap(&mut doc, copy)
        } else {
            click(&mut doc, copy)
        }
        assert_eq!(said(&doc), "Menu item 2 chosen.");
        assert!(!displayed(&doc, menu), "the menu closed");
    }
}

#[test]
fn a_press_outside_a_menu_closes_it_and_presses_nothing() {
    let mut doc = page(1200, 800);
    let edit = find(&doc, "button", "Edit ▾");
    click(&mut doc, edit);
    let primary = find(&doc, "button", "Primary");
    click(&mut doc, primary);
    let menu = doc
        .inner
        .borrow()
        .query_selector(".unit-menu")
        .unwrap()
        .unwrap();
    assert!(!displayed(&doc, menu));
    assert_eq!(said(&doc), "Nothing yet.");
}

#[test]
fn a_submenu_opens_beside_its_item() {
    let mut doc = page(1200, 800);
    let edit = find(&doc, "button", "Edit ▾");
    click(&mut doc, edit);
    let heading = find(&doc, ".unit-item", "Heading▸");
    let (hx, hy, hw, hh) = rect(&doc, heading);
    doc.handle_ui_event(UiEvent::PointerMove(pointer(
        BlitzPointerId::Mouse,
        hx + hw / 2.0,
        hy + hh / 2.0,
        MouseEventButtons::None,
    )));
    settle(&mut doc);
    let inner = doc.inner.borrow();
    let levels = inner.query_selector_all(".unit-menu").unwrap();
    drop(inner);
    let (px, _, pw, _) = rect(&doc, levels[0]);
    let (sx, sy, _, _) = rect(&doc, levels[1]);
    assert!(displayed(&doc, levels[1]), "the submenu is shown");
    assert!(sx > px + pw - 10.0, "beside its parent");
    assert!(
        (sy - (hy - 5.0)).abs() < 3.0,
        "its first item level with the parent's"
    );
}

#[test]
fn a_dialog_opens_centred_and_its_buttons_answer() {
    let mut doc = page(1200, 800);
    let confirm = find(&doc, "button", "Confirm…");
    click(&mut doc, confirm);
    let place = doc
        .inner
        .borrow()
        .query_selector(".unit-dialog-place")
        .unwrap()
        .unwrap();
    let dialog = doc
        .inner
        .borrow()
        .query_selector(".unit-dialog")
        .unwrap()
        .unwrap();
    assert!(displayed(&doc, dialog));
    let (dx, _, dw, _) = rect(&doc, dialog);
    assert!(
        (dx + dw / 2.0 - 600.0).abs() < 2.0,
        "centred: {dx} + {dw}/2"
    );
    let delete = find(&doc, ".unit-dialog button", "Delete");
    tap(&mut doc, delete);
    assert_eq!(said(&doc), "The alert answered Ok(\"\").");
    assert!(!displayed(&doc, place));
}

#[test]
fn a_dialog_fits_a_phone() {
    let mut doc = page(390, 844);
    let confirm = find(&doc, "button", "Confirm…");
    tap(&mut doc, confirm);
    let dialog = doc
        .inner
        .borrow()
        .query_selector(".unit-dialog")
        .unwrap()
        .unwrap();
    let (dx, _, dw, _) = rect(&doc, dialog);
    assert!(dx >= 0.0 && dx + dw <= 390.0, "{dx} + {dw}");
}

#[test]
fn a_modal_of_the_pages_own_opens_and_closes() {
    let mut doc = page(1200, 800);
    let open = find(&doc, "button", "Modal…");
    click(&mut doc, open);
    let modal = doc
        .inner
        .borrow()
        .query_selector(".unit-modal-place")
        .unwrap()
        .unwrap();
    assert!(displayed(&doc, modal));
    let done = find(&doc, ".unit-modal button", "Done");
    click(&mut doc, done);
    assert_eq!(said(&doc), "Modal done.");
    assert!(!displayed(&doc, modal));
}

#[test]
fn checkboxes_switches_and_radios_change_on_a_tap() {
    let mut doc = page(1200, 800);
    let check = find(&doc, "[role=checkbox]", "Check spelling");
    let attr = |doc: &DioxusDocument, n: NodeId, a: &str| {
        let inner = doc.inner.borrow();
        let el = inner.get_node(n).unwrap();
        el.element_data().and_then(|e| {
            e.attrs()
                .iter()
                .find(|x| &*x.name.local == a)
                .map(|x| x.value.clone())
        })
    };
    assert_eq!(attr(&doc, check, "aria-checked").as_deref(), Some("true"));
    tap(&mut doc, check);
    assert_eq!(attr(&doc, check, "aria-checked").as_deref(), Some("false"));
    let large = find(&doc, "[role=radio]", "Large");
    tap(&mut doc, large);
    assert_eq!(attr(&doc, large, "aria-checked").as_deref(), Some("true"));
}

#[test]
fn touch_density_draws_menu_rows_the_height_their_geometry_assumes() {
    let mut doc = page(390, 844);
    let touch = find(&doc, ".unit-segment", "Touch");
    tap(&mut doc, touch);
    let edit = find(&doc, "button", "Edit ▾");
    tap(&mut doc, edit);
    let cut = find(&doc, ".unit-item", "Cut⌘X");
    let (_, _, w, h) = rect(&doc, cut);
    assert_eq!(h, unit::menu::Sizes::TOUCH.item);
    let menu = doc
        .inner
        .borrow()
        .query_selector(".unit-menu")
        .unwrap()
        .unwrap();
    let (mx, _, mw, _) = rect(&doc, menu);
    assert!(mx >= 0.0 && mx + mw <= 390.0, "inside a phone's width");
    assert!(w > 0.0);
}

/// A key at the page, where Blitz sends it: to its focused element, or the
/// document's root with none.
#[test]
fn the_arrows_move_a_list_after_it_is_pressed() {
    let mut doc = page(1200, 800);
    let cherry = find(&doc, "[role=option]", "Cherry");
    click(&mut doc, cherry);
    key(&mut doc, Key::Named(NamedKey::ArrowDown));
    let damson = find(&doc, "[role=option]", "Damson");
    let inner = doc.inner.borrow();
    let selected = inner
        .get_node(damson)
        .unwrap()
        .element_data()
        .and_then(|e| {
            e.attrs()
                .iter()
                .find(|a| &*a.name.local == "aria-selected")
                .map(|a| a.value.clone())
        });
    assert_eq!(selected.as_deref(), Some("true"));
}

/// A toolbar's button that refuses the keys is pressed, and the keys stay
/// with the control that had them, as a host's editor keeps its own.
#[test]
fn a_button_that_refuses_the_keys_leaves_them_where_they_were() {
    let mut doc = page(1200, 800);
    let cherry = find(&doc, "[role=option]", "Cherry");
    click(&mut doc, cherry);
    let italic = find(&doc, "button", "I");
    click(&mut doc, italic);
    assert_eq!(said(&doc), "Italic on.");
    assert!(!class_of(&doc, italic).contains("unit-focused"));
    key(&mut doc, Key::Named(NamedKey::ArrowDown));
    let damson = find(&doc, "[role=option]", "Damson");
    assert_eq!(attr(&doc, damson, "aria-selected").as_deref(), Some("true"));
    assert_eq!(
        said(&doc),
        "Italic on.",
        "the list had the arrow, not the button"
    );
}

/// An element of the page's own keeps the box model the page gave it:
/// unit sets `border-box` on its own elements, not on the host's.
#[test]
fn the_pages_own_elements_keep_their_box() {
    let doc = page(1200, 800);
    let own = doc
        .inner
        .borrow()
        .query_selector(".gallery-own")
        .unwrap()
        .unwrap();
    let (_, _, w, _) = rect(&doc, own);
    assert_eq!(w, 120.0, "a width of 100 and padding of 10 each side");
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

fn class_of(doc: &DioxusDocument, node: NodeId) -> String {
    attr(doc, node, "class").unwrap_or_default()
}

/// The accessibility tree Blitz builds for the page, by node.
fn tree(doc: &DioxusDocument) -> Vec<(AxId, Node)> {
    doc.inner.borrow().build_accessibility_tree().nodes
}

fn with_role(nodes: &[(AxId, Node)], role: Role) -> Vec<&Node> {
    nodes
        .iter()
        .map(|(_, n)| n)
        .filter(|n| n.role() == role)
        .collect()
}

#[test]
fn assistive_tech_reads_every_control_as_what_it_is() {
    let mut doc = page(1200, 800);
    // A closed dialog is hidden, and out of the tree: open one.
    let confirm = find(&doc, "button", "Confirm…");
    click(&mut doc, confirm);
    let nodes = tree(&doc);
    for role in [
        Role::Button,
        Role::CheckBox,
        Role::Switch,
        Role::RadioGroup,
        Role::RadioButton,
        Role::Slider,
        Role::TabList,
        Role::Tab,
        Role::TabPanel,
        Role::ListBox,
        Role::ListBoxOption,
        Role::ProgressIndicator,
        Role::TextInput,
        Role::Dialog,
    ] {
        assert!(
            !with_role(&nodes, role).is_empty(),
            "no {role:?} in the tree"
        );
    }
    let bold = with_role(&nodes, Role::Button)
        .into_iter()
        .find(|n| n.label() == Some("Bold"));
    assert!(bold.is_some(), "the toggle is named by its label");
    let tabs = with_role(&nodes, Role::Tab);
    assert_eq!(
        tabs.iter()
            .filter(|n| n.is_selected() == Some(true))
            .count(),
        1
    );
}

/// The states ARIA gives a control — ticked, pressed, its value — read as
/// AccessKit's.
#[test]
fn assistive_tech_reads_each_controls_state() {
    let mut doc = page(1200, 800);
    let confirm = find(&doc, "button", "Confirm…");
    click(&mut doc, confirm);
    let nodes = tree(&doc);
    let checks = with_role(&nodes, Role::CheckBox);
    assert!(
        checks.iter().any(|n| n.toggled() == Some(Toggled::True)),
        "a ticked box"
    );
    let switches = with_role(&nodes, Role::Switch);
    assert!(
        switches.iter().any(|n| n.toggled() == Some(Toggled::False)),
        "a switch off"
    );
    // The tree's nodes come in no order: find the slider by its name.
    let slider = with_role(&nodes, Role::Slider)
        .into_iter()
        .find(|n| n.label() == Some("Volume") && n.value() == Some("40%"))
        .expect("the volume slider, its value as words");
    assert_eq!(slider.numeric_value(), Some(0.4));
    assert_eq!(slider.max_numeric_value(), Some(1.0));
    let dialogs = with_role(&nodes, Role::Dialog);
    assert_eq!(dialogs.len(), 1, "only the open dialog is in the tree");
    assert!(dialogs[0].is_modal());
}

/// The first element matching `selector` named `label` by `aria-label`.
fn labelled(doc: &DioxusDocument, selector: &str, label: &str) -> NodeId {
    let all = doc.inner.borrow().query_selector_all(selector).unwrap();
    *all.iter()
        .find(|&&id| attr(doc, id, "aria-label").as_deref() == Some(label))
        .unwrap_or_else(|| panic!("no {selector} labelled {label:?}"))
}

/// The `stroke` of the first icon under `node`.
fn stroke_under(doc: &DioxusDocument, node: NodeId) -> String {
    let svg = {
        let inner = doc.inner.borrow();
        let all = inner.query_selector_all(".unit-svg").unwrap();
        *all.iter()
            .find(|&&id| {
                let mut n = inner.get_node(id).unwrap().parent;
                while let Some(p) = n {
                    if p == node {
                        return true;
                    }
                    n = inner.get_node(p).unwrap().parent;
                }
                false
            })
            .expect("an icon under the node")
    };
    attr(doc, svg, "stroke").unwrap()
}

#[test]
fn an_icon_is_stroked_in_the_themes_colour_and_its_button_named_by_its_label() {
    let mut doc = page(1200, 800);
    let share = labelled(&doc, "button", "Share");
    assert_eq!(stroke_under(&doc, share), unit::Palette::LIGHT.text);
    let (_, _, w, h) = rect(&doc, share);
    assert_eq!(w, h, "an icon alone draws its button square");
    tap(&mut doc, share);
    assert_eq!(said(&doc), "Share pressed.");
    let nodes = tree(&doc);
    assert!(
        with_role(&nodes, Role::Button)
            .iter()
            .any(|n| n.label() == Some("Share")),
        "the icon's button is read by its label"
    );
    let add = find(&doc, "button", "Add");
    assert_eq!(stroke_under(&doc, add), unit::Palette::LIGHT.text);
}

#[test]
fn a_destructive_menu_item_is_drawn_in_the_danger_colour_with_its_icon() {
    let mut doc = page(1200, 800);
    let edit = find(&doc, "button", "Edit ▾");
    click(&mut doc, edit);
    let delete = labelled(&doc, ".unit-item", "Delete");
    assert!(class_of(&doc, delete).contains("danger"));
    assert_eq!(stroke_under(&doc, delete), unit::Palette::LIGHT.danger);
    let (_, _, w, _) = rect(&doc, delete);
    let level = doc
        .inner
        .borrow()
        .query_selector(".unit-menu")
        .unwrap()
        .unwrap();
    let (_, _, lw, _) = rect(&doc, level);
    assert!(w < lw, "the item and its icon fit in the level");
}

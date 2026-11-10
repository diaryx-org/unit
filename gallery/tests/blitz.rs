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
fn a_swatch_and_a_step_change_on_a_tap() {
    let mut doc = page(1200, 800);
    let only = |doc: &DioxusDocument, selector: &str| {
        doc.inner
            .borrow()
            .query_selector(selector)
            .unwrap()
            .unwrap_or_else(|| panic!("no {selector}"))
    };
    let sky = only(&doc, "[role=radio][aria-label=Sky]");
    assert_eq!(attr(&doc, sky, "aria-checked").as_deref(), Some("false"));
    tap(&mut doc, sky);
    assert_eq!(attr(&doc, sky, "aria-checked").as_deref(), Some("true"));
    // Random moves the choice on from Sky to Ink.
    let random = only(&doc, ".unit-swatch-action");
    tap(&mut doc, random);
    let ink = only(&doc, "[role=radio][aria-label=Ink]");
    assert_eq!(attr(&doc, ink, "aria-checked").as_deref(), Some("true"));
    // The stepper's second half is +.
    let stepper = only(&doc, "[role=spinbutton]");
    assert_eq!(
        attr(&doc, stepper, "aria-valuetext").as_deref(),
        Some("9:00")
    );
    let plus = doc.inner.borrow().query_selector_all(".unit-step").unwrap()[1];
    tap(&mut doc, plus);
    assert_eq!(
        attr(&doc, stepper, "aria-valuetext").as_deref(),
        Some("10:00")
    );
    // Its keys, once it is pressed: Down steps back.
    key(&mut doc, Key::Named(NamedKey::ArrowDown));
    assert_eq!(
        attr(&doc, stepper, "aria-valuetext").as_deref(),
        Some("9:00")
    );
}

#[test]
fn a_tree_row_shows_its_count_and_opens_its_menu_on_a_right_click() {
    let mut doc = page(1200, 800);
    let journal = doc
        .inner
        .borrow()
        .query_selector("[role=treeitem][aria-label=Journal]")
        .unwrap()
        .unwrap();
    assert!(
        doc.inner
            .borrow()
            .get_node(journal)
            .unwrap()
            .text_content()
            .contains('2')
    );
    let (x, y, w, h) = rect(&doc, journal);
    let (x, y) = (x + w / 2.0, y + h / 2.0);
    let mut right = pointer(BlitzPointerId::Mouse, x, y, MouseEventButtons::Secondary);
    right.button = MouseEventButton::Secondary;
    doc.handle_ui_event(UiEvent::PointerDown(right.clone()));
    doc.handle_ui_event(UiEvent::PointerUp(right));
    settle(&mut doc);
    assert_eq!(said(&doc), "Journal's menu.");
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
    // The Navigation card's tabs, read by their text; the tab bar's are
    // named by their labels.
    let bar = ["Shelf", "Search", "Share"];
    let tabs: Vec<_> = with_role(&nodes, Role::Tab)
        .into_iter()
        .filter(|n| n.label().is_none_or(|l| !bar.contains(&l)))
        .collect();
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

/// The first icon under `node`: its stroke and its class.
fn icon_under(doc: &DioxusDocument, node: NodeId) -> (String, String) {
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
    (attr(doc, svg, "stroke").unwrap(), class_of(doc, svg))
}

#[test]
fn an_icon_is_stroked_in_its_buttons_colour_and_the_button_named_by_its_label() {
    let mut doc = page(1200, 800);
    let share = labelled(&doc, "button", "Share");
    assert_eq!(
        icon_under(&doc, share),
        ("currentColor".into(), "unit-svg".into()),
        "in the button's text colour"
    );
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
    assert_eq!(icon_under(&doc, add).0, "currentColor");
}

#[test]
fn a_destructive_menu_item_is_drawn_in_the_danger_colour_with_its_icon() {
    let mut doc = page(1200, 800);
    let edit = find(&doc, "button", "Edit ▾");
    click(&mut doc, edit);
    let delete = labelled(&doc, ".unit-item", "Delete");
    assert!(class_of(&doc, delete).contains("danger"));
    assert_eq!(
        icon_under(&doc, delete),
        ("currentColor".into(), "unit-svg unit-ink-danger".into())
    );
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

#[test]
fn a_row_is_pressed_by_a_finger_and_read_as_a_button_with_its_value() {
    let mut doc = page(1200, 800);
    let row = labelled(&doc, ".unit-row", "Appearance, System");
    tap(&mut doc, row);
    assert_eq!(said(&doc), "Appearance opened.");
    let signout = labelled(&doc, ".unit-row", "Sign Out");
    assert!(class_of(&doc, signout).contains("danger"));
    click(&mut doc, signout);
    assert_eq!(said(&doc), "Signed out.");
    let nodes = tree(&doc);
    assert!(
        with_role(&nodes, Role::Button)
            .iter()
            .any(|n| n.label() == Some("Appearance, System")),
        "a row that opens something is read as a button, with its value"
    );
}

#[test]
fn a_rows_choice_of_one_ticks_the_row_pressed_and_reads_as_radios() {
    let mut doc = page(1200, 800);
    let org = labelled(&doc, ".unit-row", "Org");
    assert_eq!(attr(&doc, org, "aria-checked").as_deref(), Some("false"));
    tap(&mut doc, org);
    let org = labelled(&doc, ".unit-row", "Org");
    assert_eq!(attr(&doc, org, "aria-checked").as_deref(), Some("true"));
    let md = labelled(&doc, ".unit-row", "Markdown");
    assert_eq!(attr(&doc, md, "aria-checked").as_deref(), Some("false"));
    let nodes = tree(&doc);
    let on: Vec<_> = with_role(&nodes, Role::RadioButton)
        .into_iter()
        .filter(|n| n.label() == Some("Org"))
        .collect();
    assert_eq!(on.len(), 1);
    assert_eq!(on[0].toggled(), Some(Toggled::True));
}

#[test]
fn a_row_holds_a_field_at_its_right_and_is_a_fingers_height_at_touch_density() {
    let mut doc = page(390, 844);
    let touch = find(&doc, ".unit-segment", "Touch");
    tap(&mut doc, touch);
    let row = labelled(&doc, ".unit-row", "Name");
    let (rx, _, rw, rh) = rect(&doc, row);
    assert!(rh >= 44.0, "a row {rh} tall");
    // The inline field, in the row, not the Text card's.
    let field = doc
        .inner
        .borrow()
        .query_selector("input.unit-inline")
        .unwrap()
        .unwrap();
    let (fx, _, fw, _) = rect(&doc, field);
    assert!(fw > 40.0, "the field has room: {fw}");
    assert!(fx > rx + 40.0, "the field sits right of the title");
    assert!(fx + fw <= rx + rw, "and inside the row");
}

/// A finger pressed at `(x, y)`, moved by `dy` in steps, and lifted.
fn drag_by(doc: &mut DioxusDocument, x: f64, y: f64, dy: f64) {
    let id = BlitzPointerId::Finger(1);
    doc.handle_ui_event(UiEvent::PointerDown(pointer(
        id,
        x,
        y,
        MouseEventButtons::Primary,
    )));
    settle(doc);
    for i in 1..=5 {
        let at = y + dy * i as f64 / 5.0;
        doc.handle_ui_event(UiEvent::PointerMove(pointer(
            id,
            x,
            at,
            MouseEventButtons::Primary,
        )));
        settle(doc);
    }
    doc.handle_ui_event(UiEvent::PointerUp(pointer(
        id,
        x,
        y + dy,
        MouseEventButtons::None,
    )));
    settle(doc);
}

fn sheet_open(doc: &DioxusDocument) -> bool {
    let place = doc
        .inner
        .borrow()
        .query_selector(".unit-sheet-place")
        .unwrap()
        .unwrap();
    displayed(doc, place)
}

fn sheet_panel(doc: &DioxusDocument) -> NodeId {
    doc.inner
        .borrow()
        .query_selector(".unit-sheet")
        .unwrap()
        .unwrap()
}

#[test]
fn a_sheet_dragged_down_far_enough_closes_and_a_short_drag_springs_back() {
    let mut doc = page(390, 844);
    let open = find(&doc, "button", "Sheet…");
    tap(&mut doc, open);
    assert!(sheet_open(&doc));
    let grabber = doc
        .inner
        .borrow()
        .query_selector(".unit-sheet-grabber")
        .unwrap()
        .unwrap();
    let (gx, gy, gw, gh) = rect(&doc, grabber);
    let (_, top, _, _) = rect(&doc, sheet_panel(&doc));
    drag_by(&mut doc, gx + gw / 2.0, gy + gh / 2.0, 30.0);
    assert!(sheet_open(&doc), "a short drag leaves it open");
    let (_, after, _, _) = rect(&doc, sheet_panel(&doc));
    assert_eq!(after, top, "and back where it was");
    drag_by(&mut doc, gx + gw / 2.0, gy + gh / 2.0, 160.0);
    assert!(!sheet_open(&doc), "a long one closes it");
}

#[test]
fn a_sheets_title_bar_has_cancel_and_done() {
    let mut doc = page(390, 844);
    let open = find(&doc, "button", "Sheet…");
    tap(&mut doc, open);
    let done = find(&doc, ".unit-sheet button", "Done");
    tap(&mut doc, done);
    assert!(!sheet_open(&doc));
    assert_eq!(said(&doc), "Sheet done.");
    let open = find(&doc, "button", "Sheet…");
    tap(&mut doc, open);
    let cancel = find(&doc, ".unit-sheet button", "Cancel");
    tap(&mut doc, cancel);
    assert!(!sheet_open(&doc));
    assert_eq!(said(&doc), "Sheet done.", "Cancel is not Done");
}

#[test]
fn a_sheet_keeps_off_the_home_indicator_and_sits_on_the_keyboard() {
    let mut doc = page(390, 844);
    let open = find(&doc, "button", "Sheet…");
    let phone = find(&doc, ".unit-segment", "Phone");
    tap(&mut doc, phone);
    tap(&mut doc, open);
    let panel = sheet_panel(&doc);
    let (_, y, _, h) = rect(&doc, panel);
    assert_eq!(y + h, 844.0, "the sheet runs to the bottom edge");
    let body = doc
        .inner
        .borrow()
        .query_selector(".unit-sheet .unit-modal-body")
        .unwrap()
        .unwrap();
    let (_, by, _, bh) = rect(&doc, body);
    assert!(
        by + bh <= 844.0 - 34.0,
        "its content clears the home indicator"
    );
    // The keyboard, up.
    let cancel = find(&doc, ".unit-sheet button", "Cancel");
    tap(&mut doc, cancel);
    let keyboard = find(&doc, ".unit-segment", "Keyboard");
    tap(&mut doc, keyboard);
    let open = find(&doc, "button", "Sheet…");
    tap(&mut doc, open);
    let (_, y, _, h) = rect(&doc, sheet_panel(&doc));
    assert_eq!(y + h, 844.0 - 336.0, "the sheet sits on the keyboard");
}

#[test]
fn a_toast_sits_above_the_keyboard() {
    let mut doc = page(390, 844);
    let keyboard = find(&doc, ".unit-segment", "Keyboard");
    tap(&mut doc, keyboard);
    let toast = find(&doc, "button", "Toast");
    tap(&mut doc, toast);
    let shown = doc
        .inner
        .borrow()
        .query_selector(".unit-toast")
        .unwrap()
        .unwrap();
    let (_, y, _, h) = rect(&doc, shown);
    assert!(
        y + h <= 844.0 - 336.0,
        "the toast's bottom {} is under the keyboard",
        y + h
    );
}

/// A finger down at the middle of `node`, held `ms` with the page polled
/// meanwhile, then lifted.
fn hold(doc: &mut DioxusDocument, node: NodeId, ms: u64) -> bool {
    let (x, y, w, h) = rect(doc, node);
    let (x, y) = (x + w / 2.0, y + h / 2.0);
    let id = BlitzPointerId::Finger(1);
    doc.handle_ui_event(UiEvent::PointerDown(pointer(
        id,
        x,
        y,
        MouseEventButtons::Primary,
    )));
    settle(doc);
    std::thread::sleep(std::time::Duration::from_millis(ms));
    settle(doc);
    let open_before_lift = menu_open(doc);
    doc.handle_ui_event(UiEvent::PointerUp(pointer(
        id,
        x,
        y,
        MouseEventButtons::None,
    )));
    settle(doc);
    open_before_lift
}

fn menu_open(doc: &DioxusDocument) -> bool {
    let menu = doc
        .inner
        .borrow()
        .query_selector(".unit-menu")
        .unwrap()
        .unwrap();
    displayed(doc, menu)
}

#[test]
fn a_finger_held_still_opens_the_context_menu_while_it_is_down_and_lifting_picks_nothing() {
    let mut doc = page(390, 844);
    let area = find(
        &doc,
        ".unit-callout",
        "Right-click or hold here for a context menu.",
    );
    assert!(
        hold(&mut doc, area, 650),
        "the menu opens before the finger lifts"
    );
    assert!(menu_open(&doc), "and stays open when it lifts");
    assert_eq!(said(&doc), "Nothing yet.", "lifting chose nothing");
}

#[test]
fn a_tap_is_not_a_hold_and_a_hold_is_not_a_tap() {
    let mut doc = page(390, 844);
    let area = find(&doc, ".unit-callout", "Tap or hold here.");
    tap(&mut doc, area);
    assert_eq!(said(&doc), "Tapped, not held.");
    hold(&mut doc, area, 650);
    assert_eq!(
        said(&doc),
        "Held.",
        "the click that ends a hold is not a tap"
    );
}

#[test]
fn the_mouse_held_down_is_not_a_long_press() {
    let mut doc = page(1200, 800);
    let area = find(&doc, ".unit-callout", "Tap or hold here.");
    let (x, y, w, h) = rect(&doc, area);
    let (x, y) = (x + w / 2.0, y + h / 2.0);
    doc.handle_ui_event(UiEvent::PointerDown(pointer(
        BlitzPointerId::Mouse,
        x,
        y,
        MouseEventButtons::Primary,
    )));
    settle(&mut doc);
    std::thread::sleep(std::time::Duration::from_millis(650));
    settle(&mut doc);
    doc.handle_ui_event(UiEvent::PointerUp(pointer(
        BlitzPointerId::Mouse,
        x,
        y,
        MouseEventButtons::None,
    )));
    settle(&mut doc);
    assert_eq!(said(&doc), "Tapped, not held.");
}

fn actions_open(doc: &DioxusDocument) -> bool {
    let place = doc
        .inner
        .borrow()
        .query_selector(".unit-actions-place")
        .unwrap()
        .unwrap();
    displayed(doc, place)
}

#[test]
fn an_action_sheet_rises_from_the_bottom_and_picks_or_cancels() {
    let mut doc = page(390, 844);
    let open = find(&doc, "button", "Actions…");
    tap(&mut doc, open);
    assert!(actions_open(&doc));
    let cancel = find(&doc, ".unit-actions-cancel", "Cancel");
    let (_, y, _, h) = rect(&doc, cancel);
    assert_eq!(y + h, 844.0 - 8.0, "Cancel sits at the bottom");
    let delete = find(&doc, ".unit-action", "Delete");
    assert!(class_of(&doc, delete).contains("danger"));
    tap(&mut doc, delete);
    assert!(!actions_open(&doc));
    assert_eq!(said(&doc), "Action Some(3) chosen.");

    let open = find(&doc, "button", "Actions…");
    tap(&mut doc, open);
    tap(&mut doc, cancel);
    assert_eq!(said(&doc), "Action None chosen.");

    // A press on the scrim cancels it, and Escape does.
    let open = find(&doc, "button", "Actions…");
    tap(&mut doc, open);
    let backdrop = doc
        .inner
        .borrow()
        .query_selector(".unit-backdrop")
        .unwrap()
        .unwrap();
    doc.handle_ui_event(UiEvent::PointerDown(pointer(
        BlitzPointerId::Finger(1),
        195.0,
        40.0,
        MouseEventButtons::Primary,
    )));
    settle(&mut doc);
    let _ = backdrop;
    assert!(!actions_open(&doc), "a press above the sheet cancels it");
    let open = find(&doc, "button", "Actions…");
    tap(&mut doc, open);
    key(&mut doc, Key::Named(NamedKey::Escape));
    assert!(!actions_open(&doc));
}

#[test]
fn assistive_tech_reads_an_action_sheet_as_a_modal_menu() {
    let mut doc = page(390, 844);
    let open = find(&doc, "button", "Actions…");
    tap(&mut doc, open);
    let nodes = tree(&doc);
    let items = with_role(&nodes, Role::MenuItem);
    for label in ["Share", "Duplicate", "Delete"] {
        assert!(
            items.iter().any(|n| n.label() == Some(label)),
            "{label} is a menu item"
        );
    }
    assert!(
        with_role(&nodes, Role::Dialog)
            .iter()
            .any(|n| n.is_modal() && n.label() == Some("The entry")),
        "the sheet is a modal named by its title"
    );
}

#[test]
fn a_callout_is_acted_on_and_put_away() {
    let mut doc = page(1200, 800);
    let show = find(&doc, ".unit-callout button", "Show me");
    tap(&mut doc, show);
    assert_eq!(said(&doc), "Tip shown.");
    let dismiss = labelled(&doc, ".unit-callout button", "Dismiss");
    click(&mut doc, dismiss);
    let titles: Vec<String> = doc
        .inner
        .borrow()
        .query_selector_all(".unit-callout-title")
        .unwrap()
        .iter()
        .map(|&id| doc.inner.borrow().get_node(id).unwrap().text_content())
        .collect();
    assert!(!titles.iter().any(|t| t == "A tip"), "{titles:?}");
}

#[test]
fn a_nav_bar_goes_back_and_keeps_below_the_status_bar() {
    let mut doc = page(390, 844);
    let back = labelled(&doc, "button", "Back to Library");
    tap(&mut doc, back);
    assert_eq!(said(&doc), "Back to Library pressed.");
    let bar = doc
        .inner
        .borrow()
        .query_selector(".unit-navbar")
        .unwrap()
        .unwrap();
    let row = doc
        .inner
        .borrow()
        .query_selector(".unit-navbar-row")
        .unwrap()
        .unwrap();
    let (_, top, _, _) = rect(&doc, bar);
    let (_, row_top, _, _) = rect(&doc, row);
    assert_eq!(row_top, top, "no status bar, no room for one");
    let phone = find(&doc, ".unit-segment", "Phone");
    tap(&mut doc, phone);
    let (_, top, _, _) = rect(&doc, bar);
    let (_, row_top, _, _) = rect(&doc, row);
    assert_eq!(row_top - top, 47.0, "the bar's row clears the status bar");
    let nodes = tree(&doc);
    assert!(
        with_role(&nodes, Role::Toolbar)
            .iter()
            .any(|n| n.label() == Some("Entries"))
    );
}

#[test]
fn a_tab_bar_picks_a_place_by_a_tap_and_the_arrows() {
    let mut doc = page(390, 844);
    let search = labelled(&doc, ".unit-tabbar-item", "Search");
    tap(&mut doc, search);
    let search = labelled(&doc, ".unit-tabbar-item", "Search");
    assert_eq!(attr(&doc, search, "aria-selected").as_deref(), Some("true"));
    key(&mut doc, Key::Named(NamedKey::ArrowRight));
    let share = labelled(&doc, ".unit-tabbar-item", "Share");
    assert_eq!(attr(&doc, share, "aria-selected").as_deref(), Some("true"));
    let nodes = tree(&doc);
    let tabs: Vec<_> = with_role(&nodes, Role::Tab)
        .into_iter()
        .filter(|n| n.label() == Some("Share"))
        .collect();
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0].is_selected(), Some(true));
}

#[test]
fn a_tree_opens_by_its_arrow_and_the_keys_and_reads_its_levels() {
    let mut doc = page(1200, 800);
    let count = |doc: &DioxusDocument| {
        doc.inner
            .borrow()
            .query_selector_all(".unit-tree-row")
            .unwrap()
            .len()
    };
    assert_eq!(count(&doc), 2);
    let arrow = doc
        .inner
        .borrow()
        .query_selector(".unit-tree-row .unit-tree-arrow")
        .unwrap()
        .unwrap();
    click(&mut doc, arrow);
    assert_eq!(count(&doc), 4, "Journal open, its two rows under it");
    let journal = labelled(&doc, ".unit-tree-row", "Journal");
    assert_eq!(
        attr(&doc, journal, "aria-selected").as_deref(),
        Some("false"),
        "opening is not choosing"
    );
    let drafts = labelled(&doc, ".unit-tree-row", "Drafts");
    let (jx, _, _, _) = rect(&doc, journal);
    let (dx, _, _, _) = rect(&doc, drafts);
    assert_eq!(dx, jx, "a row's indent is inside its box");
    click(&mut doc, drafts);
    let drafts = labelled(&doc, ".unit-tree-row", "Drafts");
    assert_eq!(attr(&doc, drafts, "aria-selected").as_deref(), Some("true"));
    let nodes = tree(&doc);
    let items = with_role(&nodes, Role::TreeItem);
    let d = items
        .iter()
        .find(|n| n.label() == Some("Drafts"))
        .expect("Drafts in the tree");
    // AccessKit counts levels from 0: aria-level 2.
    assert_eq!(d.level(), Some(1));
    let j = items
        .iter()
        .find(|n| n.label() == Some("Journal"))
        .expect("Journal in the tree");
    assert_eq!(j.is_expanded(), Some(true));
    // The Left arrow closes Journal once it has the keys.
    let journal = labelled(&doc, ".unit-tree-row", "Journal");
    click(&mut doc, journal);
    key(&mut doc, Key::Named(NamedKey::ArrowLeft));
    assert_eq!(count(&doc), 2);
}

#[test]
fn a_pressed_row_holds_a_control_beside_it_that_is_pressed_on_its_own() {
    let mut doc = page(390, 844);
    let remove = find(&doc, ".unit-row-pair button", "Remove");
    tap(&mut doc, remove);
    assert_eq!(said(&doc), "Archive removed.", "Remove, not the row");
    let row = labelled(&doc, ".unit-row", "Archive");
    tap(&mut doc, row);
    assert_eq!(said(&doc), "Archive opened.");
    let (rx, _, rw, _) = rect(&doc, row);
    let (bx, _, _, _) = rect(&doc, remove);
    assert!(bx >= rx + rw, "Remove sits beside the row, not in it");
    let dot = doc
        .inner
        .borrow()
        .query_selector(".unit-row .gallery-dot")
        .unwrap();
    assert!(dot.is_some(), "the row's own leading element is in it");
    let nodes = tree(&doc);
    let buttons = with_role(&nodes, Role::Button);
    assert!(buttons.iter().any(|n| n.label() == Some("Archive")));
}

#[test]
fn a_sheet_mounted_only_while_open_is_laid_out_and_closes() {
    let mut doc = page(390, 844);
    let open = find(&doc, "button", "Mounted sheet…");
    tap(&mut doc, open);
    let place = doc
        .inner
        .borrow()
        .query_selector(".gallery-mounted")
        .unwrap()
        .expect("the page's class on the modal's layer");
    assert!(class_of(&doc, place).contains("unit-modal-place"));
    let panel = doc
        .inner
        .borrow()
        .query_selector(".gallery-mounted .unit-sheet")
        .unwrap()
        .unwrap();
    let (_, y, w, h) = rect(&doc, panel);
    assert_eq!(w, 390.0, "laid out the frame it was mounted in");
    assert_eq!(y + h, 844.0, "along the bottom");
    key(&mut doc, Key::Named(NamedKey::Escape));
    assert!(
        doc.inner
            .borrow()
            .query_selector(".gallery-mounted")
            .unwrap()
            .is_none(),
        "Escape closed it, and the page took it out"
    );
}

/// Each of `text`'s characters typed, as a keyboard types them into the
/// field that has the keys.
fn type_text(doc: &mut DioxusDocument, text: &str) {
    for c in text.chars() {
        let event = |state| BlitzKeyEvent {
            key: Key::Character(c.to_string()),
            code: Code::Unidentified,
            modifiers: Modifiers::empty(),
            location: Location::Standard,
            is_auto_repeating: false,
            is_composing: false,
            state,
            text: Some(c.to_string().into()),
        };
        doc.handle_ui_event(UiEvent::KeyDown(event(KeyState::Pressed)));
        doc.handle_ui_event(UiEvent::KeyUp(event(KeyState::Released)));
        settle(doc);
    }
}

#[test]
fn a_row_is_set_in_by_its_depth_and_reads_its_note() {
    let doc = page(1200, 800);
    let x = |title| rect(&doc, find(&doc, ".unit-row-title", title)).0;
    let (part, chapter, scene) = (x("Part One"), x("The Harbour"), x("Low tide"));
    assert!((chapter - part - 16.0).abs() < 0.5, "{part} {chapter}");
    assert!((scene - chapter - 16.0).abs() < 0.5, "{chapter} {scene}");
    let note = find(
        &doc,
        ".unit-row-note",
        "\u{201c}The boats lean over on the mud.\u{201d}",
    );
    assert!(displayed(&doc, note));
}

#[test]
fn a_fields_action_waits_for_its_text_and_is_pressed_by_return_too() {
    let mut doc = page(1200, 800);
    let add = find(&doc, ".unit-field-action button", "Add");
    assert!(
        attr(&doc, add, "disabled").is_some(),
        "greyed while the field is empty"
    );
    let field = doc
        .inner
        .borrow()
        .query_selector("input[aria-label='Add a reader']")
        .unwrap()
        .unwrap();
    click(&mut doc, field);
    type_text(&mut doc, "Ana");
    let add = find(&doc, ".unit-field-action button", "Add");
    assert!(attr(&doc, add, "disabled").is_none());
    click(&mut doc, add);
    assert_eq!(said(&doc), "Added Ana.");
    click(&mut doc, field);
    type_text(&mut doc, "Bo");
    key(&mut doc, Key::Named(NamedKey::Enter));
    assert_eq!(said(&doc), "Added Bo.");
}

#[test]
fn a_brief_toast_goes_by_itself() {
    let mut doc = page(1200, 800);
    let show = find(&doc, "button", "Brief toast");
    click(&mut doc, show);
    let shown = |doc: &DioxusDocument| {
        doc.inner
            .borrow()
            .query_selector(".unit-toast")
            .unwrap()
            .is_some()
    };
    assert!(shown(&doc));
    std::thread::sleep(std::time::Duration::from_millis(2300));
    settle(&mut doc);
    assert!(!shown(&doc), "gone after its two seconds");
}

#[test]
fn a_floating_sheet_is_held_off_the_edges_over_the_pages_bar_and_on_the_keyboard() {
    let mut doc = page(390, 844);
    let phone = find(&doc, ".unit-segment", "Phone");
    tap(&mut doc, phone);
    let open = find(&doc, "button", "Floating sheet…");
    tap(&mut doc, open);
    let (x, y, w, h) = rect(&doc, floating_panel(&doc));
    assert_eq!((x, w), (8.0, 374.0), "held 8px off each side");
    let style = attr(&doc, floating_panel(&doc), "style").unwrap();
    assert!(style.contains("max-height: 70%"), "{style}");
    assert_eq!(
        y + h,
        844.0 - 66.0 - 8.0,
        "over the page's bar, not the home indicator"
    );
    assert!(
        doc.inner
            .borrow()
            .query_selector(".unit-floating [aria-label=Close]")
            .unwrap()
            .is_none(),
        "no close button"
    );
    key(&mut doc, Key::Named(NamedKey::Escape));
    let place = doc
        .inner
        .borrow()
        .query_selector(".gallery-floating")
        .unwrap()
        .unwrap();
    assert!(!displayed(&doc, place), "Escape still closes it");
    let keyboard = find(&doc, ".unit-segment", "Keyboard");
    tap(&mut doc, keyboard);
    let open = find(&doc, "button", "Floating sheet…");
    tap(&mut doc, open);
    let (_, y, _, h) = rect(&doc, floating_panel(&doc));
    assert_eq!(y + h, 844.0 - 336.0 - 8.0, "on the keyboard");
}

fn floating_panel(doc: &DioxusDocument) -> NodeId {
    doc.inner
        .borrow()
        .query_selector(".unit-floating")
        .unwrap()
        .unwrap()
}

#[test]
fn a_chip_shows_its_colour_and_count_and_is_read_by_its_label() {
    let mut doc = page(1200, 800);
    let family = find(&doc, ".unit-chip", "Family4");
    let dot = doc
        .inner
        .borrow()
        .query_selector(".unit-chip .unit-chip-dot")
        .unwrap()
        .unwrap();
    let ((cx, cy, _, ch), (dx, dy, dw, dh)) = (rect(&doc, family), rect(&doc, dot));
    assert_eq!((dw, dh), (8.0, 8.0));
    assert!(
        dx > cx && (dy + dh / 2.0 - (cy + ch / 2.0)).abs() < 1.0,
        "a dot before the label, centred"
    );
    assert_eq!(
        attr(&doc, family, "aria-label").as_deref(),
        Some("Family, 4")
    );
    click(&mut doc, family);
    assert_eq!(attr(&doc, family, "aria-pressed").as_deref(), Some("true"));
}

/// Backspace, as the window sends it: on a Mac, AppKit's `deleteBackward:`.
fn backspace(doc: &mut DioxusDocument) {
    #[cfg(target_os = "macos")]
    {
        doc.handle_ui_event(UiEvent::AppleStandardKeybinding("deleteBackward:".into()));
        settle(doc);
    }
    #[cfg(not(target_os = "macos"))]
    key(doc, Key::Named(NamedKey::Backspace));
}

/// The text the field's cells draw, one cell at a time.
fn code_cells(doc: &DioxusDocument) -> String {
    let inner = doc.inner.borrow();
    inner
        .query_selector_all(".unit-code-cell")
        .unwrap()
        .iter()
        .map(|&id| inner.get_node(id).unwrap().text_content())
        .collect()
}

#[test]
fn a_code_is_typed_or_pasted_whole_into_one_field_and_keeps_only_its_digits() {
    use blitz_traits::events::BlitzImeEvent;
    let mut doc = page(1200, 800);
    let field = labelled(&doc, "input", "Code");
    // The field lies over every cell, so a press on any lands in it.
    let cells = doc
        .inner
        .borrow()
        .query_selector(".unit-code-cells")
        .unwrap()
        .unwrap();
    assert_eq!(rect(&doc, field), rect(&doc, cells));
    assert_eq!(code_cells(&doc), "");
    tap(&mut doc, field);
    type_text(&mut doc, "12a3");
    assert_eq!(code_cells(&doc), "123", "a letter is left out");
    assert_eq!(attr(&doc, field, "value").as_deref(), Some("123"));
    type_text(&mut doc, "999");
    assert_eq!(said(&doc), "123999 is not the code.");
    let code = doc
        .inner
        .borrow()
        .query_selector(".unit-code")
        .unwrap()
        .unwrap();
    assert!(class_of(&doc, code).contains("invalid"));
    // A seventh digit is not kept, in the cells or in the field.
    type_text(&mut doc, "7");
    assert_eq!(code_cells(&doc), "123999");
    assert_eq!(attr(&doc, field, "value").as_deref(), Some("123999"));
    // Emptied, and the whole code put in at once, as a paste or the
    // system's AutoFill does, with what came round it in the message.
    for _ in 0..6 {
        backspace(&mut doc);
    }
    assert_eq!(code_cells(&doc), "");
    assert!(
        !class_of(&doc, code).contains("invalid"),
        "typing again clears it"
    );
    doc.handle_ui_event(UiEvent::Ime(BlitzImeEvent::Commit(
        "Your code: 123 456".into(),
    )));
    settle(&mut doc);
    assert_eq!(code_cells(&doc), "123456");
    assert_eq!(said(&doc), "Signed in.");

    // Assistive tech reads one field, named, and no cells.
    let nodes = tree(&doc);
    let inputs: Vec<_> = with_role(&nodes, Role::TextInput)
        .into_iter()
        .filter(|n| n.label() == Some("Code"))
        .collect();
    // Blitz gives an <input>'s node no value yet, a TextField's neither:
    // the field is found by its name.
    assert_eq!(inputs.len(), 1, "one field, not one for each digit");
    // The cells are hidden, which every adapter leaves out with all that
    // is under them: their digits too.
    let (_, cells) = nodes
        .iter()
        .find(|(id, _)| *id == AxId(cells.as_u64()))
        .expect("the cells' node");
    assert!(cells.is_hidden());
    assert_eq!(cells.children().len(), 6);
}

/// A pressable of the page's own is reached by Tab from the button before
/// it, drawn with the ring, and pressed by Return and by Space; pressed by
/// the mouse, it has no ring.
#[test]
fn tab_reaches_a_pressable_and_return_presses_it() {
    let mut doc = page(1200, 800);
    let bold = find(&doc, "button", "B");
    click(&mut doc, bold);
    key(&mut doc, Key::Named(NamedKey::Tab));
    let tile = find(&doc, "[role=button]", "Tile");
    assert_eq!(
        doc.inner.borrow().get_focussed_node_id(),
        Some(tile),
        "Tab passes Italic, which refuses the keys, to the tile"
    );
    assert!(attr(&doc, tile, "data-unit-focused").is_some(), "the ring");
    key(&mut doc, Key::Named(NamedKey::Enter));
    assert_eq!(said(&doc), "Tile pressed.");
    let other = find(&doc, "button", "Default");
    click(&mut doc, other);
    assert_eq!(said(&doc), "Default pressed.");
    // Primary, Ghost, Delete, Add, Share, Bold: Disabled and Italic passed by.
    for _ in 0..7 {
        key(&mut doc, Key::Named(NamedKey::Tab));
    }
    assert_eq!(doc.inner.borrow().get_focussed_node_id(), Some(tile));
    key(&mut doc, Key::Character(" ".into()));
    assert_eq!(said(&doc), "Tile pressed.");
    // Pressed by the mouse, it has the keys but no ring.
    click(&mut doc, other);
    click(&mut doc, tile);
    assert_eq!(said(&doc), "Tile pressed.");
    assert!(
        attr(&doc, tile, "data-unit-focused").is_none(),
        "no ring after a press"
    );
}

/// The first element matching `selector`.
fn first(doc: &DioxusDocument, selector: &str) -> NodeId {
    doc.inner
        .borrow()
        .query_selector(selector)
        .unwrap()
        .unwrap_or_else(|| panic!("no {selector}"))
}

/// A press at (`x`, `y`), a drag with the button held to (`x2`, `y2`), and
/// the release there: the mouse's.
fn drag(doc: &mut DioxusDocument, (x, y): (f64, f64), (x2, y2): (f64, f64)) {
    let id = BlitzPointerId::Mouse;
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
    for i in 1..=4 {
        let t = i as f64 / 4.0;
        let at = (x + (x2 - x) * t, y + (y2 - y) * t);
        doc.handle_ui_event(UiEvent::PointerMove(pointer(
            id,
            at.0,
            at.1,
            MouseEventButtons::Primary,
        )));
        settle(doc);
    }
    doc.handle_ui_event(UiEvent::PointerUp(pointer(
        id,
        x2,
        y2,
        MouseEventButtons::None,
    )));
    settle(doc);
}

/// The scroll box's bar, laid out and pressed by Blitz: a drag of its thumb
/// scrolls the lines, and goes on when the pointer leaves the thin bar; a
/// press on the track below the thumb scrolls the box a screen on.
#[test]
fn a_scroll_bars_thumb_drags_and_its_track_pages() {
    let mut doc = page(900, 6000);
    let thumb = first(&doc, ".unit-scrollbar-thumb");
    let bar = first(&doc, ".unit-scrollbar");
    let line = find(&doc, ".gallery-line", "Line 1");
    let (tx, ty, tw, th) = rect(&doc, thumb);
    let (_, y0, _, _) = rect(&doc, line);
    assert!(th > 20.0 && tw > 0.0, "a thumb a hand can take: {th}");

    // Down 40 pixels, wandering off the bar to the left on the way.
    let from = (tx + tw / 2.0, ty + th / 2.0);
    drag(&mut doc, from, (from.0 - 200.0, from.1 + 40.0));
    let (_, ty2, _, _) = rect(&doc, thumb);
    let (_, y1, _, _) = rect(&doc, line);
    assert!(
        (ty2 - ty - 40.0).abs() < 1.0,
        "the thumb followed: {ty} -> {ty2}"
    );
    assert!(y1 < y0 - 40.0, "the lines scrolled up: {y0} -> {y1}");

    // A press on the track, below the thumb: a screen on.
    let (bx, by, bw, bh) = rect(&doc, bar);
    drag(
        &mut doc,
        (bx + bw / 2.0, by + bh - 3.0),
        (bx + bw / 2.0, by + bh - 3.0),
    );
    let (_, ty3, _, _) = rect(&doc, thumb);
    let (_, y2, _, _) = rect(&doc, line);
    assert!(ty3 > ty2, "the thumb moved on: {ty2} -> {ty3}");
    assert!(y2 < y1, "the lines scrolled on: {y1} -> {y2}");
}

/// A page with nothing on it but a text area already holding a note.
#[allow(non_snake_case)]
fn AWrittenNote() -> Element {
    use unit::prelude::*;
    let mut note = use_signal(|| "Bought the bread.\nForgot the milk.".to_string());
    rsx! {
        UnitRoot {
            TextArea { label: "Notes", value: note(), rows: 3, oninput: move |v| note.set(v) }
        }
    }
}

/// A text area shows what it already holds from its first frame, not
/// only once it is tapped: Blitz's editor takes a `<textarea>`'s text from
/// its content, as a browser's does, and not from a `value`.
#[test]
fn a_text_area_shows_what_it_already_holds() {
    let mut doc = DioxusDocument::new(VirtualDom::new(AWrittenNote), DocumentConfig::default());
    doc.inner
        .borrow_mut()
        .set_viewport(Viewport::new(800, 600, 1.0, ColorScheme::Light));
    doc.initial_build();
    settle(&mut doc);
    let area = first(&doc, "textarea");
    let shown = |doc: &DioxusDocument| {
        doc.inner
            .borrow()
            .get_node(area)
            .and_then(|n| n.element_data())
            .and_then(|e| e.text_input_data())
            .map(|t| t.editor.raw_text().to_string())
            .unwrap_or_default()
    };
    assert_eq!(shown(&doc), "Bought the bread.\nForgot the milk.");

    // Typed in twice, what was typed stays: the page's new value, written
    // back into the area, neither empties it nor types it again.
    use blitz_traits::events::BlitzImeEvent;
    tap(&mut doc, area);
    for typed in ["!", "?"] {
        doc.handle_ui_event(UiEvent::Ime(BlitzImeEvent::Commit(typed.into())));
        settle(&mut doc);
    }
    let after = shown(&doc);
    assert_eq!(after.len(), "Bought the bread.\nForgot the milk.".len() + 2);
    assert_eq!(after.matches('!').count(), 1, "{after:?}");
    assert_eq!(after.matches('?').count(), 1, "{after:?}");
}

/// The option of a grid named `name`, its detail after it or not.
fn option_named(doc: &DioxusDocument, name: &str) -> NodeId {
    let all = doc
        .inner
        .borrow()
        .query_selector_all("[role=option]")
        .unwrap();
    *all.iter()
        .find(|&&id| {
            attr(doc, id, "aria-label")
                .is_some_and(|l| l == name || l.starts_with(&format!("{name}, ")))
        })
        .unwrap_or_else(|| panic!("no option named {name:?}"))
}

/// The row of a table whose first cell is `name`.
fn row_named(doc: &DioxusDocument, name: &str) -> NodeId {
    let inner = doc.inner.borrow();
    let all = inner.query_selector_all(".unit-tr").unwrap();
    *all.iter()
        .find(|&&id| {
            let first = inner.get_node(id).unwrap().children[0];
            inner.get_node(first).unwrap().text_content().trim() == name
        })
        .unwrap_or_else(|| panic!("no row {name:?}"))
}

/// The first cell of each of a table's rows, in order.
fn table_names(doc: &DioxusDocument) -> Vec<String> {
    let inner = doc.inner.borrow();
    inner
        .query_selector_all(".unit-tr")
        .unwrap()
        .iter()
        .map(|&id| {
            let first = inner.get_node(id).unwrap().children[0];
            inner
                .get_node(first)
                .unwrap()
                .text_content()
                .trim()
                .to_string()
        })
        .collect()
}

fn selected(doc: &DioxusDocument, node: NodeId) -> bool {
    attr(doc, node, "aria-selected").as_deref() == Some("true")
}

/// What the file browser says it has chosen.
fn chosen(doc: &DioxusDocument) -> String {
    let inner = doc.inner.borrow();
    let all = inner.query_selector_all(".unit-text").unwrap();
    all.iter()
        .map(|&id| inner.get_node(id).unwrap().text_content())
        .find(|t| t.contains(" chosen."))
        .expect("the file browser's count")
        .trim()
        .to_string()
}

/// A click at (`x`, `y`) with `mods` held: the mouse's.
fn click_at(doc: &mut DioxusDocument, x: f64, y: f64, mods: Modifiers) {
    let id = BlitzPointerId::Mouse;
    let at = |buttons| {
        let mut p = pointer(id, x, y, buttons);
        p.mods = mods;
        p
    };
    doc.handle_ui_event(UiEvent::PointerMove(at(MouseEventButtons::None)));
    doc.handle_ui_event(UiEvent::PointerDown(at(MouseEventButtons::Primary)));
    settle(doc);
    doc.handle_ui_event(UiEvent::PointerUp(at(MouseEventButtons::None)));
    settle(doc);
}

fn click_with(doc: &mut DioxusDocument, node: NodeId, mods: Modifiers) {
    let (x, y, w, h) = rect(doc, node);
    click_at(doc, x + w / 2.0, y + h / 2.0, mods);
}

fn right_click(doc: &mut DioxusDocument, node: NodeId) {
    let (x, y, w, h) = rect(doc, node);
    let (x, y) = (x + w / 2.0, y + h / 2.0);
    let mut right = pointer(BlitzPointerId::Mouse, x, y, MouseEventButtons::Secondary);
    right.button = MouseEventButton::Secondary;
    doc.handle_ui_event(UiEvent::PointerDown(right.clone()));
    doc.handle_ui_event(UiEvent::PointerUp(right));
    settle(doc);
}

fn key_with(doc: &mut DioxusDocument, key: Key, modifiers: Modifiers) {
    let event = |state| BlitzKeyEvent {
        key: key.clone(),
        code: Code::Unidentified,
        modifiers,
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

/// Where the file browser is: the breadcrumb read as the current page.
fn folder_here(doc: &DioxusDocument) -> String {
    let inner = doc.inner.borrow();
    let here = inner
        .query_selector("[aria-current=page]")
        .unwrap()
        .expect("a current folder");
    inner
        .get_node(here)
        .unwrap()
        .text_content()
        .trim()
        .to_string()
}

#[test]
fn a_tile_grid_chooses_as_a_file_browser_does() {
    let mut doc = page(1200, 800);
    let notes = option_named(&doc, "Notes.md");
    assert!(selected(&doc, notes), "chosen from the start");
    let letters = option_named(&doc, "Letters.md");
    click(&mut doc, letters);
    assert!(
        selected(&doc, letters) && !selected(&doc, notes),
        "a press chooses one alone"
    );
    // ⌘ adds, on any system.
    let harbour = option_named(&doc, "Harbour.md");
    click_with(&mut doc, harbour, Modifiers::META);
    assert!(selected(&doc, letters) && selected(&doc, harbour));
    assert_eq!(chosen(&doc), "2 of 7 chosen.");
    // Shift chooses the range from the last pressed, in the order shown:
    // Harbour, Letters, Low tide, Notes, Plans.
    let plans = option_named(&doc, "Plans.md");
    click_with(&mut doc, plans, Modifiers::SHIFT);
    assert_eq!(chosen(&doc), "5 of 7 chosen.");
    assert!(selected(&doc, option_named(&doc, "Low tide.md")));
    assert!(!selected(&doc, option_named(&doc, "Boats.jpg")));
    // A press on the grid's ground, past its last tile, chooses nothing.
    let grid = labelled(&doc, "[role=listbox]", "Files in Journal");
    let (x, y, w, h) = rect(&doc, grid);
    click_at(&mut doc, x + w - 6.0, y + h - 6.0, Modifiers::empty());
    assert_eq!(chosen(&doc), "0 of 7 chosen.");
}

#[test]
fn the_arrows_move_through_a_grid_by_its_rows_and_return_opens_a_folder() {
    let mut doc = page(1200, 800);
    let boats = option_named(&doc, "Boats.jpg");
    click(&mut doc, boats);
    // As many to a row as the grid's width holds at 96px and 8px apart.
    let grid = labelled(&doc, "[role=listbox]", "Files in Journal");
    let (_, _, w, _) = rect(&doc, grid);
    let columns = ((w - 8.0 + 8.0) / 104.0).floor() as usize;
    assert!(columns > 1, "{w}");
    let (bx, by, _, _) = rect(&doc, boats);
    let below = option_named(&doc, "Plans.md");
    let (_, py, _, _) = rect(&doc, below);
    if columns == 6 {
        assert!(py > by, "Plans.md, the seventh, on the second row");
    }
    // The mouse passes over a tile on the next row, as it does on its way
    // anywhere: Blitz reports no element's size, and the grid learns from
    // that how many tiles a row holds.
    let (px, py, pw, ph) = rect(&doc, below);
    doc.handle_ui_event(UiEvent::PointerMove(pointer(
        BlitzPointerId::Mouse,
        px + pw / 2.0,
        py + ph / 2.0,
        MouseEventButtons::None,
    )));
    settle(&mut doc);
    key(&mut doc, Key::Named(NamedKey::ArrowDown));
    let order = [
        "Boats.jpg",
        "Drafts",
        "Harbour.md",
        "Letters.md",
        "Low tide.md",
        "Notes.md",
        "Plans.md",
    ];
    let down = order[columns.min(6)];
    assert!(
        selected(&doc, option_named(&doc, down)),
        "a row down: {down}"
    );
    let (dx, _, _, _) = rect(&doc, option_named(&doc, down));
    assert!((dx - bx).abs() < 1.0 || columns >= 7, "straight down");
    key(&mut doc, Key::Named(NamedKey::Home));
    key_with(&mut doc, Key::Named(NamedKey::ArrowRight), Modifiers::SHIFT);
    assert_eq!(chosen(&doc), "2 of 7 chosen.", "Shift extends");
    key(&mut doc, Key::Named(NamedKey::ArrowRight));
    assert!(selected(&doc, option_named(&doc, "Harbour.md")));
    key(&mut doc, Key::Named(NamedKey::ArrowLeft));
    // Return opens the folder the cursor is on.
    key(&mut doc, Key::Named(NamedKey::Enter));
    assert_eq!(folder_here(&doc), "Drafts");
    let empty = first(&doc, ".unit-empty");
    assert!(displayed(&doc, empty), "an empty folder says so");
    let back = find(&doc, "button", "Journal");
    click(&mut doc, back);
    assert_eq!(folder_here(&doc), "Journal");
    // A double click opens a file.
    let letters = option_named(&doc, "Letters.md");
    click(&mut doc, letters);
    click(&mut doc, letters);
    assert_eq!(said(&doc), "Opened Letters.md.");
}

#[test]
fn a_collections_menu_is_about_its_selection() {
    let mut doc = page(1200, 800);
    // A right-click on a tile not chosen chooses it alone first.
    let letters = option_named(&doc, "Letters.md");
    right_click(&mut doc, letters);
    assert!(selected(&doc, letters));
    assert!(!selected(&doc, option_named(&doc, "Notes.md")));
    assert!(menu_open(&doc));
    let trash = find(&doc, ".unit-item", "Move to Trash");
    click(&mut doc, trash);
    assert_eq!(said(&doc), "Move 1 item to the Trash.");
    // On one of several chosen, it keeps them all.
    let plans = option_named(&doc, "Plans.md");
    click_with(&mut doc, plans, Modifiers::META);
    right_click(&mut doc, plans);
    assert_eq!(chosen(&doc), "2 of 7 chosen.");
    let open = find(&doc, ".unit-item", "Open");
    click(&mut doc, open);
    assert_eq!(said(&doc), "Open 2 items.");
    // The Menu key opens it too, its first item lit as a key's menu is.
    let letters = option_named(&doc, "Letters.md");
    click(&mut doc, letters);
    key(&mut doc, Key::Named(NamedKey::ContextMenu));
    assert!(menu_open(&doc));
    key(&mut doc, Key::Named(NamedKey::Enter));
    assert_eq!(said(&doc), "Open 1 item.");
}

#[test]
fn a_table_sorts_by_its_columns_and_chooses_its_rows() {
    let mut doc = page(1200, 800);
    let list = find(&doc, ".unit-segment", "List");
    click(&mut doc, list);
    assert_eq!(table_names(&doc)[0], "Boats.jpg", "by name");
    let size = find(&doc, "button", "Size");
    let header = parent_of(&doc, size);
    assert_eq!(attr(&doc, header, "aria-sort").as_deref(), Some("none"));
    click(&mut doc, size);
    let header = parent_of(&doc, find(&doc, "button", "Size"));
    assert_eq!(
        attr(&doc, header, "aria-sort").as_deref(),
        Some("ascending")
    );
    assert_eq!(
        &table_names(&doc)[..2],
        ["Drafts", "Plans.md"],
        "smallest first"
    );
    let size = find(&doc, "button", "Size");
    click(&mut doc, size);
    assert_eq!(table_names(&doc)[0], "Boats.jpg", "largest first");
    // Notes.md is still chosen, wherever the sort put it.
    assert!(selected(&doc, row_named(&doc, "Notes.md")));
    let letters = row_named(&doc, "Letters.md");
    click(&mut doc, letters);
    assert!(selected(&doc, letters) && !selected(&doc, row_named(&doc, "Notes.md")));
    key(&mut doc, Key::Named(NamedKey::ArrowDown));
    let names = table_names(&doc);
    let next = &names[names.iter().position(|n| n == "Letters.md").unwrap() + 1];
    assert!(selected(&doc, row_named(&doc, next)), "the row under it");
    let low = row_named(&doc, "Low tide.md");
    click(&mut doc, low);
    click(&mut doc, low);
    assert_eq!(said(&doc), "Opened Low tide.md.");
    // Every column fits its row, the date included, at a desktop's width.
    let modified = find(&doc, "button", "Modified");
    assert!(displayed(&doc, modified));
}

fn parent_of(doc: &DioxusDocument, node: NodeId) -> NodeId {
    doc.inner.borrow().get_node(node).unwrap().parent.unwrap()
}

#[test]
fn a_phone_opens_what_it_taps_and_leaves_a_tables_secondary_columns_out() {
    let mut doc = page(390, 844);
    let touch = find(&doc, ".unit-segment", "Touch");
    tap(&mut doc, touch);
    let drafts = option_named(&doc, "Drafts");
    let (_, _, w, h) = rect(&doc, drafts);
    assert!(w >= 44.0 && h >= 44.0, "a finger's target: {w}×{h}");
    tap(&mut doc, drafts);
    assert_eq!(folder_here(&doc), "Drafts", "a tap opens");
    let home = find(&doc, "button", "Home");
    tap(&mut doc, home);
    assert_eq!(folder_here(&doc), "Home");
    let list = find(&doc, ".unit-segment", "List");
    tap(&mut doc, list);
    let modified = find(&doc, ".unit-th", "Modified");
    assert!(!displayed(&doc, modified), "no room on a phone");
    let photos = row_named(&doc, "Photos");
    let (_, _, _, rh) = rect(&doc, photos);
    assert_eq!(rh, 44.0, "a finger's row");
    tap(&mut doc, photos);
    assert_eq!(folder_here(&doc), "Photos");
    let table = first(&doc, ".unit-table");
    let (tx, _, tw, _) = rect(&doc, table);
    assert!(tx >= 0.0 && tx + tw <= 390.0, "inside the phone");
}

#[test]
fn a_search_narrows_the_files_and_says_when_nothing_matches() {
    let mut doc = page(1200, 800);
    let field = labelled(&doc, "input", "Search files");
    click(&mut doc, field);
    type_text(&mut doc, "tide");
    option_named(&doc, "Low tide.md");
    assert_eq!(
        chosen(&doc),
        "0 of 1 chosen.",
        "Notes.md, hidden, is let go"
    );
    let clear = labelled(&doc, "button", "Clear Search files");
    click(&mut doc, clear);
    assert_eq!(chosen(&doc), "0 of 7 chosen.");
    let field = labelled(&doc, "input", "Search files");
    click(&mut doc, field);
    type_text(&mut doc, "zzz");
    let empty = first(&doc, ".unit-empty");
    assert!(displayed(&doc, empty));
    assert_eq!(attr(&doc, empty, "role").as_deref(), Some("status"));
    let again = find(&doc, "button", "Clear Search");
    click(&mut doc, again);
    assert_eq!(chosen(&doc), "0 of 7 chosen.");
    // Escape in the field empties it.
    let field = labelled(&doc, "input", "Search files");
    click(&mut doc, field);
    type_text(&mut doc, "plan");
    assert_eq!(chosen(&doc), "0 of 1 chosen.");
    key(&mut doc, Key::Named(NamedKey::Escape));
    assert_eq!(chosen(&doc), "0 of 7 chosen.");
}

fn viewer(doc: &DioxusDocument) -> Option<NodeId> {
    doc.inner.borrow().query_selector(".unit-viewer").unwrap()
}

fn viewer_count(doc: &DioxusDocument) -> String {
    let node = first(doc, ".unit-viewer-count");
    doc.inner
        .borrow()
        .get_node(node)
        .unwrap()
        .text_content()
        .trim()
        .to_string()
}

#[test]
fn the_photo_viewer_goes_from_photo_to_photo_and_gives_the_keys_back() {
    let mut doc = page(1200, 800);
    assert!(viewer(&doc).is_none(), "in the page only while open");
    // The keys are with the photos when it opens.
    let photo = option_named(&doc, "Noon on the hill");
    click(&mut doc, photo);
    click(&mut doc, photo);
    let v = viewer(&doc).expect("a double click opens the viewer");
    assert_eq!(rect(&doc, v), (0.0, 0.0, 1200.0, 800.0), "the whole window");
    assert_eq!(viewer_count(&doc), "2 of 6");
    let image = first(&doc, ".unit-viewer-image");
    assert_eq!(
        attr(&doc, image, "alt").as_deref(),
        Some("Noon on the hill")
    );
    key(&mut doc, Key::Named(NamedKey::ArrowRight));
    assert_eq!(viewer_count(&doc), "3 of 6");
    let next = labelled(&doc, "button", "Next photo");
    click(&mut doc, next);
    assert_eq!(viewer_count(&doc), "4 of 6");
    // The arrows still go to it after its button is pressed.
    key(&mut doc, Key::Named(NamedKey::ArrowLeft));
    assert_eq!(viewer_count(&doc), "3 of 6");
    // A swipe to the right goes back, one to the left on.
    drag(&mut doc, (500.0, 400.0), (640.0, 400.0));
    assert_eq!(viewer_count(&doc), "2 of 6");
    drag(&mut doc, (640.0, 400.0), (500.0, 400.0));
    assert_eq!(viewer_count(&doc), "3 of 6");
    key(&mut doc, Key::Named(NamedKey::Home));
    assert!(
        doc.inner
            .borrow()
            .query_selector("[aria-label='Previous photo']")
            .unwrap()
            .is_none(),
        "nothing before the first"
    );
    key(&mut doc, Key::Named(NamedKey::Escape));
    assert!(viewer(&doc).is_none(), "Escape closes it");
    // The keys are back with the photos: the arrows choose among them.
    key(&mut doc, Key::Named(NamedKey::ArrowRight));
    assert!(selected(&doc, option_named(&doc, "A grey afternoon")));
    // And its button opens it at the first, closed by its own button.
    let view = find(&doc, "button", "View Photos…");
    click(&mut doc, view);
    assert_eq!(viewer_count(&doc), "1 of 6");
    key(&mut doc, Key::Named(NamedKey::ArrowRight));
    assert_eq!(
        viewer_count(&doc),
        "2 of 6",
        "the keys are its, opened by a button too"
    );
    let close = labelled(&doc, ".unit-viewer button", "Close");
    click(&mut doc, close);
    assert!(viewer(&doc).is_none());
}

#[test]
fn assistive_tech_reads_the_collections() {
    let mut doc = page(1200, 800);
    let nodes = tree(&doc);
    let lists: Vec<_> = with_role(&nodes, Role::ListBox)
        .into_iter()
        .filter(|n| n.label() == Some("Files in Journal"))
        .collect();
    assert_eq!(lists.len(), 1, "the grid is a list box");
    let notes = nodes
        .iter()
        .map(|(_, n)| n)
        .find(|n| n.role() == Role::ListBoxOption && n.label() == Some("Notes.md, 13 KB"))
        .expect("a tile, named by its label and its size");
    assert_eq!(notes.is_selected(), Some(true));
    assert!(
        with_role(&nodes, Role::Navigation)
            .iter()
            .any(|n| n.label() == Some("Folders")),
        "the path is navigation"
    );
    let list = find(&doc, ".unit-segment", "List");
    click(&mut doc, list);
    let nodes = tree(&doc);
    assert!(
        with_role(&nodes, Role::Grid)
            .iter()
            .any(|n| n.label() == Some("Files in Journal")),
        "the table is a grid"
    );
    assert_eq!(with_role(&nodes, Role::ColumnHeader).len(), 3);
    let rows = with_role(&nodes, Role::Row);
    assert_eq!(rows.len(), 8, "a header and seven files");
    assert!(rows.iter().any(|r| r.is_selected() == Some(true)));
    let view = find(&doc, "button", "View Photos…");
    click(&mut doc, view);
    let nodes = tree(&doc);
    assert!(
        with_role(&nodes, Role::Image)
            .iter()
            .any(|n| n.label() == Some("Dawn over the harbour")),
        "the photo is read by what is in it"
    );
}

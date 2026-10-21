//! The gallery's page: every unit component, working, on one page. The
//! binary shows it in a window or a browser; the tests lay it out headless.

use dioxus::prelude::*;
use unit::Insets;
use unit::icon::paths;
use unit::prelude::*;

/// Ids of the menus' items.
const CUT: usize = 1;
const COPY: usize = 2;
const PASTE: usize = 3;
const WRAP: usize = 4;
const H1: usize = 10;
const H2: usize = 11;
const DELETE: usize = 12;

/// The page, at the appearance and density the page itself chooses.
#[allow(non_snake_case)]
pub fn Gallery() -> Element {
    // A window can start dark, or at a finger's density, for a look at each:
    // UNIT_APPEARANCE=dark, UNIT_DENSITY=touch.
    let env = |_name: &str| {
        #[cfg(not(target_arch = "wasm32"))]
        return std::env::var(_name).unwrap_or_default();
        #[cfg(target_arch = "wasm32")]
        return String::new();
    };
    let appearance = use_signal(|| match env("UNIT_APPEARANCE").as_str() {
        "dark" => Appearance::Dark,
        "light" => Appearance::Light,
        _ => Appearance::System,
    });
    let density = use_signal(|| match env("UNIT_DENSITY").as_str() {
        "touch" => Density::Touch,
        _ => Density::Regular,
    });
    rsx! {
        UnitRoot { appearance: appearance(), density: density(), class: "gallery",
            style { {GALLERY_CSS} }
            Page { appearance, density }
            // The page's own element, outside any component, as a host's
            // editor is: unit's styles leave its box as the page drew it.
            div { class: "gallery-own" }
        }
    }
}

/// A sleep on a thread of its own, for a long press while the finger is
/// still down: the gallery has no runtime of its own to time it with. With
/// UNIT_NO_TIMER set, the page has none, and a hold fires on the release.
#[cfg(not(target_arch = "wasm32"))]
fn thread_sleep(
    d: std::time::Duration,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>> {
    use std::sync::{Arc, Mutex};
    use std::task::{Poll, Waker};
    struct Sleep(Arc<Mutex<(bool, Option<Waker>)>>);
    impl std::future::Future for Sleep {
        type Output = ();
        fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<()> {
            let mut s = self.0.lock().unwrap();
            if s.0 {
                Poll::Ready(())
            } else {
                s.1 = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
    let state = Arc::new(Mutex::new((false, None::<Waker>)));
    let s = state.clone();
    std::thread::spawn(move || {
        std::thread::sleep(d);
        let mut s = s.lock().unwrap();
        s.0 = true;
        if let Some(w) = s.1.take() {
            w.wake();
        }
    });
    Box::pin(Sleep(state))
}

const GALLERY_CSS: &str = r#"
html, body { margin: 0; }
.gallery { min-height: 100vh; padding: 20px 24px 80px; }
.gallery-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: flex-start; }
.gallery-grid > .unit-card { width: 340px; }
.gallery-said { color: var(--unit-dim); margin-top: 8px; }
.gallery-own { width: 100px; height: 1px; padding: 0 10px; }
.gallery-dot { width: 10px; height: 10px; border-radius: 5px; background: #b4541a; }
"#;

#[component]
fn Page(appearance: Signal<Appearance>, density: Signal<Density>) -> Element {
    let unit = use_unit();
    #[cfg(not(target_arch = "wasm32"))]
    use_hook(move || {
        if std::env::var_os("UNIT_NO_TIMER").is_none() {
            unit.set_timer(thread_sleep);
        }
    });
    let mut said = use_signal(|| String::from("Nothing yet."));
    let mut bold = use_signal(|| false);
    let mut italic = use_signal(|| false);
    let mut spelling = use_signal(|| true);
    let mut wrap = use_signal(|| false);
    let mut size = use_signal(|| Some(1usize));
    let mut view = use_signal(|| Some(0usize));
    let mut volume = use_signal(|| 0.4f64);
    let mut stepped = use_signal(|| 3.0f64);
    let mut name = use_signal(String::new);
    let mut reader = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut font = use_signal(|| Some(0usize));
    let mut circle = use_signal(|| 0usize);
    let mut faces = use_signal(|| vec![true, false, false]);
    let mut tab = use_signal(|| 0usize);
    let mut picked = use_signal(|| Some(2usize));
    let mut modal = use_signal(|| false);
    let mut sheet = use_signal(|| false);
    let mut floating = use_signal(|| false);
    // A sheet put in the page only while it is open, as an app that
    // mounts each of its sheets on demand does.
    let mut mounted = use_signal(|| false);
    let mut markup = use_signal(|| 0usize);
    // What a phone would set: its safe area, and the keyboard up.
    let mut edges = use_signal(|| 0usize);
    let mut tip = use_signal(|| true);
    let mut place = use_signal(|| 0usize);
    let mut journal = use_signal(|| false);
    let mut folder = use_signal(|| String::from("Archive"));

    let edit = vec![
        Item::new(CUT, "Cut").shortcut("⌘X").into(),
        Item::new(COPY, "Copy").shortcut("⌘C").into(),
        Item::new(PASTE, "Paste")
            .shortcut("⌘V")
            .enabled(false)
            .into(),
        Entry::Separator,
        Item::new(WRAP, "Wrap Lines").checked(wrap()).into(),
        Item::new(DELETE, "Delete")
            .icon(paths::TRASH)
            .destructive()
            .into(),
        Entry::submenu(
            "Heading",
            vec![
                Item::new(H1, "Heading 1").into(),
                Item::new(H2, "Heading 2").into(),
            ],
        ),
    ];
    let pick = move |id: usize| {
        if id == WRAP {
            wrap.toggle();
        }
        said.set(format!("Menu item {id} chosen."));
    };
    let fonts: Vec<Choice> = vec!["System".into(), "Serif".into(), "Mono".into()];
    let face_choices: Vec<Choice> = vec![
        Choice::new("Sans").styled("font-family: sans-serif"),
        Choice::new("Serif").styled("font-family: serif"),
        Choice::new("Mono").styled("font-family: monospace"),
    ];
    let fruit: Vec<String> = ["Apple", "Apricot", "Banana", "Cherry", "Damson"]
        .map(String::from)
        .into();

    rsx! {
        Stack { gap: 16.0,
            Stack { axis: Axis::Horizontal, align: Align::Center, wrap: true,
                Heading { level: 1, "unit" }
                Spacer {}
                Segmented {
                    label: "Appearance",
                    options: vec!["System".into(), "Light".into(), "Dark".into()],
                    selected: Some(match appearance() { Appearance::System => 0, Appearance::Light => 1, Appearance::Dark => 2 }),
                    onchange: move |i| appearance.clone().set([Appearance::System, Appearance::Light, Appearance::Dark][i]),
                }
                Segmented {
                    label: "Edges",
                    options: vec!["Window".into(), "Phone".into(), "Keyboard".into()],
                    selected: Some(edges()),
                    onchange: move |i| {
                        edges.set(i);
                        let phone = Insets { top: 47.0, bottom: 34.0, ..Insets::default() };
                        unit.set_insets(if i == 0 { Insets::default() } else { phone });
                        unit.set_keyboard(if i == 2 { 336.0 } else { 0.0 });
                    },
                }
                Segmented {
                    label: "Density",
                    options: vec!["Pointer".into(), "Touch".into()],
                    selected: Some(if density() == Density::Touch { 1 } else { 0 }),
                    onchange: move |i| density.clone().set(if i == 1 { Density::Touch } else { Density::Regular }),
                }
            }
            Text { dim: true, "Every component, working. What the last one did:" }
            div { class: "gallery-said", role: "status", "{said}" }

            div { class: "gallery-grid",
                Card { title: "Buttons",
                    Stack { axis: Axis::Horizontal, wrap: true,
                        Button { onpress: move |_| said.set("Default pressed.".into()), "Default" }
                        Button { variant: Variant::Primary, onpress: move |_| said.set("Primary pressed.".into()), "Primary" }
                        Button { variant: Variant::Ghost, onpress: move |_| said.set("Ghost pressed.".into()), "Ghost" }
                        Button { variant: Variant::Danger, onpress: move |_| said.set("Delete pressed.".into()), "Delete" }
                        Button { disabled: true, onpress: move |_| said.set("Disabled pressed!".into()), "Disabled" }
                        Button { icon: paths::PLUS, onpress: move |_| said.set("Add pressed.".into()), "Add" }
                        Button { icon: paths::SHARE, label: "Share", variant: Variant::Ghost, class: "unit-icon",
                                 onpress: move |_| said.set("Share pressed.".into()) }
                        ToggleButton { pressed: bold(), label: "Bold", onchange: move |v| bold.set(v), b { "B" } }
                        // A toolbar's: pressed, it leaves the keys where they were.
                        ToggleButton { pressed: italic(), label: "Italic", refuses_keys: true,
                                       onchange: move |v| {
                                           italic.set(v);
                                           said.set(format!("Italic {}.", if v { "on" } else { "off" }));
                                       },
                                       i { "I" } }
                    }
                }
                Card { title: "Choices",
                    Checkbox { checked: spelling(), onchange: move |v| spelling.set(v), "Check spelling" }
                    Checkbox { checked: false, disabled: true, onchange: move |_| {}, "Unavailable" }
                    Switch { on: wrap(), onchange: move |v| wrap.set(v), "Wrap lines" }
                    Divider {}
                    RadioGroup {
                        label: "Size",
                        options: vec!["Small".into(), "Medium".into(), "Large".into()],
                        selected: size(),
                        onchange: move |i| size.set(Some(i)),
                    }
                    Divider {}
                    Segmented {
                        label: "View",
                        options: vec!["Day".into(), "Week".into(), "Month".into()],
                        selected: view(),
                        onchange: move |i| view.set(Some(i)),
                    }
                }
                Card { title: "Sliders",
                    Field { label: "Volume", note: format!("{:.0}%", volume() * 100.0),
                        Slider { label: "Volume", value: volume(), text: format!("{:.0}%", volume() * 100.0),
                                 onchange: move |v| volume.set(v) }
                    }
                    Field { label: "Steps", note: format!("{}", stepped()),
                        Slider { label: "Steps", value: stepped(), min: 0.0, max: 10.0, step: 1.0,
                                 onchange: move |v| stepped.set(v) }
                    }
                    Progress { label: "Progress", value: volume() }
                }
                Card { title: "Text",
                    TextField { label: "Name", value: name(), placeholder: "Your name",
                                oninput: move |v| name.set(v) }
                    TextArea { label: "Notes", value: notes(), rows: 3, oninput: move |v| notes.set(v) }
                    Text { dim: true, "Hello, {name}." }
                }
                Card { title: "Menus",
                    Stack { axis: Axis::Horizontal, wrap: true, align: Align::Center,
                        MenuButton { entries: edit.clone(), onpick: pick, "Edit ▾" }
                        Select { label: "Font", choices: fonts.clone(), selected: font(),
                                 onpick: move |i| font.set(Some(i)) }
                    }
                    ContextMenu { entries: edit.clone(), onpick: pick,
                        div { class: "unit-callout", style: "margin-top: 10px",
                            "Right-click or hold here for a context menu." }
                    }
                    LongPress { onhold: move |_| said.set("Held.".into()),
                                onpress: move |_| said.set("Tapped, not held.".into()),
                        div { class: "unit-callout", "Tap or hold here." }
                    }
                    Chips { label: "Faces", choices: face_choices, on: faces(),
                            onpick: move |i: usize| faces.with_mut(|f: &mut Vec<bool>| f[i] = !f[i]) }
                    Chips { label: "Circle", on: (0..3).map(|i| circle() == i).collect::<Vec<_>>(),
                            choices: vec![Choice::new("Only You"),
                                          Choice::new("Family").dot("#b4541a").badge("4"),
                                          Choice::new("Friends").dot("#3a7bd5").badge("12")],
                            onpick: move |i: usize| circle.set(i) }
                }
                Card { title: "Navigation",
                    Tabs { label: "Sections", tabs: vec!["General".into(), "Editor".into(), "About".into()],
                           selected: tab(), onchange: move |i| tab.set(i),
                        match tab() {
                            0 => rsx! { Text { "General settings would go here." } },
                            1 => rsx! { Text { "Editor settings would go here." } },
                            _ => rsx! { Text { "unit, a component library." } },
                        }
                    }
                    ListBox { label: "Fruit", items: fruit.clone(), selected: picked(),
                              onchange: move |i| picked.set(Some(i)),
                              onactivate: move |i: usize| said.set(format!("Opened {}.", fruit[i])) }
                    Disclosure { title: "More details",
                        Text { "Hidden until opened." }
                    }
                }
                Card { title: "Lists",
                    Section { title: "Account", footer: "Rows in a group, as a settings pane has them.",
                        Row { title: "Name", trailing: rsx! {
                            TextField { inline: true, label: "Name", value: name(), placeholder: "Your name",
                                        oninput: move |v| name.set(v) }
                        } }
                        Row { title: "Appearance", detail: "System", icon: paths::SHARE,
                              accessory: Accessory::Chevron,
                              onpress: move |_| said.set("Appearance opened.".into()) }
                        Switch { on: wrap(), onchange: move |v| wrap.set(v), "Wrap lines" }
                        TextField { inline: true, label: "Add a reader", placeholder: "Add a reader…",
                                    value: reader(), oninput: move |v| reader.set(v),
                                    action: "Add", action_disabled: reader().trim().is_empty(),
                                    onaction: move |v: String| { said.set(format!("Added {v}.")); reader.set(String::new()); } }
                        Row { title: "Sign Out", destructive: true,
                              onpress: move |_| said.set("Signed out.".into()) }
                    }
                    Section { title: "Libraries",
                        Row { title: "Journal", accessory: Accessory::Check(true),
                              onpress: move |_| said.set("Journal opened.".into()) }
                        Row { title: "Archive", onpress: move |_| said.set("Archive opened.".into()),
                              leading: rsx! { div { class: "gallery-dot" } },
                              trailing: rsx! {
                                  Button { variant: Variant::Ghost, onpress: move |_| said.set("Archive removed.".into()), "Remove" }
                              } }
                    }
                    Section { title: "Contents", footer: "Rows set in by their depth, as a nested list's are.",
                        Row { title: "Part One", onpress: move |_| said.set("Part One opened.".into()) }
                        Row { title: "The Harbour", depth: 1, subtitle: "Chapter 1",
                              onpress: move |_| said.set("The Harbour opened.".into()) }
                        Row { title: "Low tide", depth: 2, subtitle: "From Ana",
                              note: "\u{201c}The boats lean over on the mud.\u{201d}",
                              onpress: move |_| said.set("Low tide opened.".into()) }
                    }
                    Section { title: "Markup",
                        for (i, m) in ["Markdown", "Plain text", "Org"].into_iter().enumerate() {
                            Row { key: "{i}", title: m, subtitle: (i == 0).then(|| "What the editor writes".to_string()),
                                  accessory: Accessory::Check(markup() == i),
                                  onpress: move |_| markup.set(i) }
                        }
                    }
                }
                Card { title: "Bars",
                    NavBar { title: "Entries", back: "Library",
                             onback: move |_| said.set("Back to Library pressed.".into()),
                             trailing: rsx! {
                                 Button { icon: paths::PLUS, label: "New", variant: Variant::Ghost, class: "unit-icon",
                                          onpress: move |_| said.set("New pressed.".into()) }
                             } }
                    TabBar { label: "Places", selected: place(), onchange: move |i| place.set(i),
                             items: vec![
                                 BarItem::new("Shelf", paths::MORE),
                                 BarItem::new("Search", paths::SEARCH),
                                 BarItem::new("Share", paths::SHARE),
                             ] }
                    Tree { label: "Folders",
                        TreeRow { label: "Journal", expanded: journal(), selected: folder() == "Journal",
                                  ontoggle: move |v| journal.set(v), onpress: move |_| folder.set("Journal".into()) }
                        if journal() {
                            for f in ["2026", "Drafts"] {
                                TreeRow { key: "{f}", label: f, depth: 1, selected: folder() == f,
                                          onpress: move |_| folder.set(f.into()) }
                            }
                        }
                        TreeRow { label: "Archive", selected: folder() == "Archive",
                                  onpress: move |_| folder.set("Archive".into()) }
                    }
                }
                Card { title: "Overlays",
                    Stack { axis: Axis::Horizontal, wrap: true,
                        Button { onpress: move |_| {
                            unit.open_dialog(
                                Dialog::new("Add Link").message("The address the text links to.")
                                    .field("", "https://").buttons("Add", "Cancel"),
                                move |a| said.set(format!("The dialog answered {a:?}.")),
                            );
                        }, "Prompt…" }
                        Button { onpress: move |_| {
                            unit.open_dialog(
                                Dialog::new("Delete the draft?").message("This cannot be undone.")
                                    .buttons("Delete", "Cancel").destructive(),
                                move |a| said.set(format!("The alert answered {a:?}.")),
                            );
                        }, "Confirm…" }
                        Button { onpress: move |_| {
                            unit.open_actions(
                                ActionSheet::new(vec![
                                    SheetAction::new(1, "Share").icon(paths::SHARE),
                                    SheetAction::new(2, "Duplicate"),
                                    SheetAction::new(3, "Delete").icon(paths::TRASH).destructive(),
                                ]).title("The entry").message("What to do with it."),
                                move |a| said.set(format!("Action {a:?} chosen.")),
                            );
                        }, "Actions…" }
                        Button { onpress: move |_| modal.set(true), "Modal…" }
                        Button { onpress: move |_| sheet.set(true), "Sheet…" }
                        Button { onpress: move |_| floating.set(true), "Floating sheet…" }
                        Button { onpress: move |_| mounted.set(true), "Mounted sheet…" }
                        Button { onpress: move |_| {
                            unit.toast_with(Toast::new("Saved.").tone(Tone::Success).action("Undo"),
                                move || said.set("Undone.".into()));
                        }, "Toast" }
                        Button { onpress: move |_| {
                            unit.toast_with(Toast::new("Moved.").action("Undo").lasts(std::time::Duration::from_secs(2)),
                                move || said.set("Move undone.".into()));
                        }, "Brief toast" }
                    }
                    Stack { axis: Axis::Horizontal, wrap: true,
                        Badge { "3" }
                        Badge { tone: Tone::Success, "Synced" }
                        Badge { tone: Tone::Warning, "Offline" }
                        Badge { tone: Tone::Danger, "Error" }
                    }
                    Callout { tone: Tone::Warning, title: "Heads up", "A callout in the page." }
                    if tip() {
                        Callout { title: "A tip", action: "Show me",
                                  onaction: move |_| said.set("Tip shown.".into()),
                                  ondismiss: move |_| tip.set(false),
                                  "One that can be acted on, or put away." }
                    }
                }
            }
        }
        Modal { open: modal(), title: "A modal", onclose: move |_| modal.set(false),
            actions: rsx! {
                Button { onpress: move |_| modal.set(false), "Cancel" }
                Button { variant: Variant::Primary, onpress: move |_| { modal.set(false); said.set("Modal done.".into()); }, "Done" }
            },
            Text { "Content of the page's own, with its own state:" }
            Checkbox { checked: spelling(), onchange: move |v| spelling.set(v), "Check spelling" }
            Select { label: "Font", choices: fonts.clone(), selected: font(), onpick: move |i| font.set(Some(i)) }
        }
        if mounted() {
            Modal { open: true, title: "Mounted", placement: Placement::Sheet, class: "gallery-mounted",
                    onclose: move |_| mounted.set(false),
                Text { "In the page only while open." }
            }
        }
        Modal { open: sheet(), title: "A sheet", placement: Placement::Sheet,
                cancel: "Cancel", done: "Done", onclose: move |_| sheet.set(false),
                ondone: move |_| { sheet.set(false); said.set("Sheet done.".into()); },
            Switch { on: wrap(), onchange: move |v| wrap.set(v), "Wrap lines" }
            Slider { label: "Volume", value: volume(), onchange: move |v| volume.set(v) }
        }
        // A card held off the edges and over a bar of the page's own along
        // the bottom, with no close button: it goes by a drag, the scrim or
        // Escape.
        Modal { open: floating(), title: "A floating sheet", placement: Placement::Sheet,
                inset: 8.0, clear: 66.0, max_height: 0.7, close_button: false, class: "gallery-floating",
                onclose: move |_| floating.set(false),
            Text { "Held off the window's edges, above the page's own bar." }
        }
    }
}

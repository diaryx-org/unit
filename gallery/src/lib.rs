//! The gallery's page: every unit component, working, on one page. The
//! binary shows it in a window or a browser; the tests lay it out headless.

use dioxus::prelude::*;
use unit::prelude::*;

/// Ids of the menus' items.
const CUT: usize = 1;
const COPY: usize = 2;
const PASTE: usize = 3;
const WRAP: usize = 4;
const H1: usize = 10;
const H2: usize = 11;

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
        }
    }
}

const GALLERY_CSS: &str = r#"
html, body { margin: 0; }
.gallery { min-height: 100vh; padding: 20px 24px 80px; }
.gallery-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: flex-start; }
.gallery-grid > .unit-card { width: 340px; }
.gallery-said { color: var(--unit-dim); margin-top: 8px; }
"#;

#[component]
fn Page(appearance: Signal<Appearance>, density: Signal<Density>) -> Element {
    let unit = use_unit();
    let mut said = use_signal(|| String::from("Nothing yet."));
    let mut bold = use_signal(|| false);
    let mut spelling = use_signal(|| true);
    let mut wrap = use_signal(|| false);
    let mut size = use_signal(|| Some(1usize));
    let mut view = use_signal(|| Some(0usize));
    let mut volume = use_signal(|| 0.4f64);
    let mut stepped = use_signal(|| 3.0f64);
    let mut name = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut font = use_signal(|| Some(0usize));
    let mut faces = use_signal(|| vec![true, false, false]);
    let mut tab = use_signal(|| 0usize);
    let mut picked = use_signal(|| Some(2usize));
    let mut modal = use_signal(|| false);
    let mut sheet = use_signal(|| false);

    let edit = vec![
        Item::new(CUT, "Cut").shortcut("⌘X").into(),
        Item::new(COPY, "Copy").shortcut("⌘C").into(),
        Item::new(PASTE, "Paste")
            .shortcut("⌘V")
            .enabled(false)
            .into(),
        Entry::Separator,
        Item::new(WRAP, "Wrap Lines").checked(wrap()).into(),
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
                        ToggleButton { pressed: bold(), label: "Bold", onchange: move |v| bold.set(v), b { "B" } }
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
                            "Right-click here for a context menu." }
                    }
                    Chips { label: "Faces", choices: face_choices, on: faces(),
                            onpick: move |i: usize| faces.with_mut(|f: &mut Vec<bool>| f[i] = !f[i]) }
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
                        Button { onpress: move |_| modal.set(true), "Modal…" }
                        Button { onpress: move |_| sheet.set(true), "Sheet…" }
                        Button { onpress: move |_| {
                            unit.toast_with(Toast::new("Saved.").tone(Tone::Success).action("Undo"),
                                move || said.set("Undone.".into()));
                        }, "Toast" }
                    }
                    Stack { axis: Axis::Horizontal, wrap: true,
                        Badge { "3" }
                        Badge { tone: Tone::Success, "Synced" }
                        Badge { tone: Tone::Warning, "Offline" }
                        Badge { tone: Tone::Danger, "Error" }
                    }
                    Callout { tone: Tone::Warning, title: "Heads up", "A callout in the page." }
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
        Modal { open: sheet(), title: "A sheet", placement: Placement::Sheet, onclose: move |_| sheet.set(false),
            Switch { on: wrap(), onchange: move |v| wrap.set(v), "Wrap lines" }
            Slider { label: "Volume", value: volume(), onchange: move |v| volume.set(v) }
        }
    }
}

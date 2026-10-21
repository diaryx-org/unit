//! unit — components for Dioxus that work the same on the web and on
//! dioxus-native (Blitz), on a desktop and on a phone, and that assistive
//! tech reads on each.
//!
//! Buttons, toggles, checkboxes, switches, radio groups, segmented
//! controls, sliders, text fields, selects, menus with submenus, context
//! menus, chips, tabs, list boxes, disclosures, dialogs, modals and sheets,
//! toasts, and the layout and text round them — in a theme of CSS custom
//! properties, light and dark, at a pointer's density or a finger's.
//!
//! # Why it is built the way it is
//!
//! A component library for a browser leans on things Blitz does not have,
//! and unit leans on none of them:
//!
//! | A browser's library | Under Blitz | Here |
//! |---|---|---|
//! | closes a menu on Escape and on a press outside through `document::eval`, a JavaScript listener | `eval` is a no-op: a menu never closes | a backdrop under an open menu takes the press outside; keys reach the page's own handler first ([`Unit::key`]) |
//! | moves DOM focus with `MountedData::set_focus` from a task | Blitz polls tasks while it holds its document borrowed: a panic | the page keeps its own focus ([`use_focus`]), set by a press and by the renderer's Tab |
//! | presses a focused button on Return and Space | Blitz does not | the page does, where the renderer does not ([`Kind::Button`]) |
//! | acts on mouse events | a finger sends pointer and touch events, no mouse events | pointer events and `click`, which every renderer sends for every pointer |
//! | makes a menu when it opens, inside its button | Blitz lays out an absolutely positioned element on the first paint and not when a later frame inserts it | one overlay at the page's root, mounted from the first frame |
//! | measures an element to place a popup or read a slider | measuring needs the DOM, borrowed by Blitz | geometry computed from known sizes ([`menu::Sizes`]); a slider's track divided into stops |
//!
//! None of that breaks a browser: a backdrop, a page-kept focus and pointer
//! events behave the same on the web.
//!
//! **Accessibility** is ARIA on real elements: `<button>` where it can be,
//! `role` and `aria-checked`, `aria-selected`, `aria-expanded`,
//! `aria-valuenow`, `aria-modal`, labels by `aria-label` and
//! `aria-labelledby`. A browser reads them as it reads any page; Blitz
//! builds an AccessKit tree from the same attributes, which VoiceOver,
//! TalkBack and UI automation read.
//!
//! # Using it
//!
//! ```ignore
//! use unit::prelude::*;
//!
//! fn App() -> Element {
//!     let mut on = use_signal(|| false);
//!     rsx! {
//!         UnitRoot {
//!             Switch { on: on(), onchange: move |v| on.set(v), "Wrap lines" }
//!             Button { variant: Variant::Primary, onpress: move |_| {}, "Save" }
//!         }
//!     }
//! }
//! ```
//!
//! On the web and under dioxus-native's `launch`, that is all. A host that
//! reads keys itself — an editor taking them from winit — passes
//! `keys: Keys::Host` and calls [`Unit::key`] and [`Unit::text`] before it
//! uses a key, and gives keys to the renderer while [`Unit::typing`].

pub mod components;
pub mod dialog;
pub mod icon;
pub mod layer;
pub mod menu;
mod style;
pub mod theme;
mod unit;

pub use components::button::{Button, ToggleButton, Variant};
pub use components::choice::{Checkbox, Orientation, RadioGroup, Segmented, Switch};
pub use components::layout::{
    Align, Axis, Badge, Callout, Card, Divider, Field, Heading, Progress, Spacer, Stack, Text,
};
pub use components::list::{Accessory, Row, Section};
pub use components::menus::{Chips, Choice, ContextMenu, MenuButton, Select, left_top};
pub use components::modal::{Modal, Placement};
pub use components::navigation::{Disclosure, ListBox, Tabs};
pub use components::slider::Slider;
pub use components::text_field::{InputKind, TextArea, TextField};
pub use dialog::{Answer, Dialog};
pub use icon::{Icon, Ink};
pub use layer::{Toast, Tone};
pub use menu::{Entry, Item};
pub use style::CSS;
pub use theme::{Appearance, Density, Metrics, Palette, Theme};
pub use unit::{
    Clipboard, Focus, Insets, Keys, Kind, Unit, UnitRoot, next_id, use_focus, use_unit,
    use_unit_provider,
};

/// Everything a page uses, at once.
pub mod prelude {
    pub use crate::{
        Accessory, Align, Answer, Appearance, Axis, Badge, Button, Callout, Card, Checkbox, Chips,
        Choice, ContextMenu, Density, Dialog, Disclosure, Divider, Entry, Field, Heading, Icon,
        Ink, InputKind, Item, Keys, ListBox, MenuButton, Modal, Orientation, Placement, Progress,
        RadioGroup, Row, Section, Segmented, Select, Slider, Spacer, Stack, Switch, Tabs, Text,
        TextArea, TextField, Theme, Toast, ToggleButton, Tone, Unit, UnitRoot, Variant, use_unit,
    };
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use dioxus::prelude::*;

    use crate::prelude::*;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn a_page_renders_its_overlay_with_nothing_open() {
        fn Page() -> Element {
            rsx! {
                UnitRoot {
                    MenuButton { entries: vec![Item::new(1, "Bold").into()], onpick: |_| {}, "Format" }
                }
            }
        }
        let html = render(Page);
        assert!(
            html.contains("unit-layer") && html.contains("unit-backdrop"),
            "{html}"
        );
        // Every menu level is in the tree from the first frame, hidden.
        assert_eq!(
            html.matches("class=\"unit-menu\"").count(),
            crate::layer::LEVELS,
            "{html}"
        );
        assert!(html.contains("aria-haspopup=\"menu\""), "{html}");
        assert!(html.contains("aria-expanded=\"false\""), "{html}");
        // The theme is in the page.
        assert!(html.contains("--unit-accent"), "{html}");
    }

    #[test]
    fn every_control_says_what_it_is_and_its_state() {
        fn Page() -> Element {
            rsx! {
                UnitRoot {
                    Button { onpress: |_| {}, "Save" }
                    ToggleButton { pressed: true, onchange: |_| {}, "B" }
                    Checkbox { checked: true, onchange: |_| {}, "Spelling" }
                    Switch { on: false, onchange: |_| {}, "Wrap" }
                    RadioGroup { options: vec!["A".into(), "B".into()], selected: Some(1), onchange: |_| {} }
                    Segmented { options: vec!["Day".into(), "Week".into()], selected: Some(0), onchange: |_| {} }
                    Slider { value: 0.25, onchange: |_| {}, label: "Volume" }
                    Tabs { tabs: vec!["One".into(), "Two".into()], selected: 1, onchange: |_| {}, "two" }
                    ListBox { items: vec!["x".into(), "y".into()], selected: Some(0), onchange: |_| {} }
                    Progress { value: 0.5 }
                    TextField { value: "hi", oninput: |_| {}, label: "Name" }
                }
            }
        }
        let html = render(Page);
        for needle in [
            "<button type=\"button\" class=\"unit-button ",
            "aria-pressed=\"true\"",
            "role=\"checkbox\"",
            "role=\"switch\"",
            "role=\"radiogroup\"",
            "role=\"radio\"",
            "role=\"slider\"",
            "aria-valuenow=\"0.25\"",
            "aria-valuemax=\"1\"",
            "role=\"tablist\"",
            "role=\"tab\"",
            "role=\"tabpanel\"",
            "role=\"listbox\"",
            "role=\"option\"",
            "aria-selected=\"true\"",
            "role=\"progressbar\"",
            "aria-valuenow=\"50\"",
            "aria-label=\"Name\"",
            "role=\"dialog\"",
            "aria-modal=\"true\"",
        ] {
            assert!(html.contains(needle), "{needle} missing in {html}");
        }
        assert_eq!(html.matches("aria-checked=\"true\"").count(), 3, "{html}");
    }

    #[test]
    fn a_forced_appearance_and_density_are_attributes_of_the_root() {
        fn Page() -> Element {
            rsx! { UnitRoot { appearance: Appearance::Dark, density: Density::Touch, "x" } }
        }
        let html = render(Page);
        assert!(html.contains("data-appearance=\"dark\""), "{html}");
        assert!(html.contains("data-density=\"touch\""), "{html}");
    }

    #[test]
    fn an_icon_takes_its_colour_from_the_appearance() {
        fn Page() -> Element {
            rsx! {
                UnitRoot { appearance: Appearance::Dark,
                    Icon { d: crate::icon::paths::PLUS }
                    Icon { d: crate::icon::paths::TRASH, ink: Ink::Danger }
                }
            }
        }
        let html = render(Page);
        let dark = crate::Palette::DARK;
        assert!(
            html.contains(&format!("stroke=\"{}\"", dark.text)),
            "{html}"
        );
        assert!(
            html.contains(&format!("stroke=\"{}\"", dark.danger)),
            "{html}"
        );
        assert!(html.contains("aria-hidden=\"true\""), "{html}");
    }
}

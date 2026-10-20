# unit

Components for [Dioxus](https://dioxuslabs.com) that work the same on the web
and on dioxus-native (Blitz), on a desktop and on a phone, and that assistive
tech reads on each. It needs only `dioxus`, no renderer and no JavaScript.

```sh
cargo test --workspace                   # the library, and every component laid out by Blitz
cargo run -p unit-gallery                # every component, in a window drawn by Blitz
dx serve -p unit-gallery --platform web --no-default-features --features web
UNIT_APPEARANCE=dark UNIT_DENSITY=touch cargo run -p unit-gallery
```

The library is at the root, the gallery in `gallery/`. It began in
diaryx-org/leaf, as `crates/unit`, and grew out of leaf's `leaf-dioxus-ui`.
A consumer takes it by git:

```toml
unit = { git = "https://github.com/diaryx-org/unit", branch = "main" }
```

## What is in it

| | |
|---|---|
| Buttons | `Button` (default, primary, ghost, danger), `ToggleButton` |
| Choices | `Checkbox`, `Switch`, `RadioGroup`, `Segmented`, `Slider`, `Chips` |
| Text | `TextField` (text, search, email, URL, password, number), `TextArea` |
| Menus | `MenuButton`, `Select`, `ContextMenu`; items with shortcuts, ticks, separators and submenus |
| Navigation | `Tabs`, `ListBox` (arrows, Home, End, typeahead), `Disclosure` |
| Overlays | `Unit::open_dialog` (alerts, confirmations, a prompt), `Modal` and its `Sheet` placement, `Unit::toast` |
| Layout | `Stack`, `Spacer`, `Card`, `Divider`, `Field` (a form's row), `Heading`, `Text`, `Badge`, `Progress`, `Callout` |
| Theme | `Theme`: a light and a dark `Palette` and a pointer's and a finger's `Metrics`, written as CSS custom properties; `Appearance` and `Density` on the root |

Its component list and its theme of tokens take after
[Libero](https://github.com/tdymel/libero)'s; how it is built does not.

## Using it

```rust
use dioxus::prelude::*;
use unit::prelude::*;

fn App() -> Element {
    let mut wrap = use_signal(|| false);
    rsx! {
        UnitRoot {
            Switch { on: wrap(), onchange: move |v| wrap.set(v), "Wrap lines" }
            Button { variant: Variant::Primary, onpress: move |_| {}, "Save" }
        }
    }
}
```

`UnitRoot` puts the theme's stylesheet in the page, keeps the page's focus,
and draws the overlay that menus, dialogs and toasts appear in. A browser and
dioxus-native's `launch` need nothing more. A host that reads keys itself, as
an editor reading them from winit does, passes `keys: Keys::Host`, calls
`Unit::key` and `Unit::text` before it uses a key, and gives keys to the
renderer while `Unit::typing()`.

## Why it is built the way it is

A component library for a browser leans on things Blitz does not have:

| A browser's library | Under Blitz | Here |
|---|---|---|
| closes a menu through `document::eval` | `eval` is a no-op | a backdrop takes the press outside; keys reach the page's handler first |
| moves DOM focus with `set_focus` from a task | a panic: Blitz holds its document borrowed | the page keeps its own focus, set by a press and by Tab |
| lets the renderer press a focused button on Return | Blitz does not | the page does where the renderer does not |
| acts on mouse events | a finger sends pointer and touch events only | pointer events and `click` |
| makes a menu when it opens, inside its button | Blitz does not lay out an absolutely positioned element a later frame inserts | one overlay at the root, mounted from the first frame |
| measures an element to place a popup or read a slider | measuring needs the borrowed DOM | geometry computed from known sizes; a slider's track cut into stops |

Accessibility is ARIA on real elements: `<button>` where it can be, and
`role`, `aria-checked`, `aria-pressed`, `aria-selected`, `aria-expanded`,
`aria-valuenow`, `aria-modal`, `aria-label`, `aria-labelledby`. A browser
reads them as it reads any page; Blitz builds its AccessKit tree from the
same attributes, for VoiceOver, TalkBack and UI automation.

## Blitz

The gallery builds Blitz from diaryx-org's fork, its `unit` branch: the fork's
`leaf` branch and three changes the gallery's tests found it needed
(diaryx-org/blitz#1).

- **ARIA's states.** Blitz read roles but not states: a checkbox's tick, a
  toggle's pressed state, a slider's value, a dialog's modality.
- **A node removed under `display: none`** stayed in its layout parent's
  children, and showing the parent again laid out a freed node: the "layout
  child … is not in the slab" panic that kept `leaf-dioxus-ui`'s overlay
  always displayed.
- **A press** focused only text fields, so keys never reached a pressed list
  or tab row; and a click went to whatever the release landed on, so a press
  that closed a menu also clicked what lay under it.

The gallery goes back to the `leaf` branch once those are merged there.

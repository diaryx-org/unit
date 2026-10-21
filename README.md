# unit

Components for [Dioxus](https://dioxuslabs.com) that work the same on the web
and on dioxus-native (Blitz), on a desktop and on a phone, and that assistive
tech reads on each. It needs only `dioxus`, no renderer and no JavaScript.
Every component, working, is at <https://diaryx-org.github.io/unit/>: the
gallery built for the web from main.

```sh
cargo test --workspace                   # the library, and every component laid out by Blitz
cargo run -p unit-gallery                # every component, in a window drawn by Blitz
dx serve -p unit-gallery --platform web --no-default-features --features web
gallery/web/build.sh                     # the gallery as a static site, as Pages publishes it
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
| Buttons | `Button` (default, primary, ghost, danger), `ToggleButton`, each with an `icon` |
| Icons | `Icon`: a path on a 24-unit grid stroked in `currentColor`, the text's or an `Ink` of the theme's; `icon::paths` |
| Choices | `Checkbox`, `Switch`, `RadioGroup`, `Segmented`, `Slider`, `Chips` (each with a colour's dot and a count, if it has them) |
| Text | `TextField` (text, search, email, URL, password, number; with an `action` at its end, pressed by Return too), `TextArea`, `CodeField` (a one-time code: a cell for each digit, one field to assistive tech, a whole code pasted or filled in at once) |
| Menus | `MenuButton`, `Select`, `ContextMenu` (the secondary button, or a finger held); items with shortcuts, ticks, icons, separators, submenus, and destructive ones |
| Lists | `Section` (rows under a caption), `Row` (icon, title, subtitle, a note under it, value, a chevron or a tick, a control at the right, set in by a `depth`) |
| Navigation | `Tabs`, `ListBox` (arrows, Home, End, typeahead), `Disclosure`, `NavBar`, `TabBar`, `Tree` and `TreeRow` |
| Overlays | `Unit::open_dialog` (alerts, confirmations, a prompt), `Unit::open_actions` (an action sheet), `Modal` and its `Sheet` placement (Cancel and Done, dragged down to close, along the bottom or floating off the edges over a bar of the page's own), `Unit::toast` (until dismissed, or for as long as it `lasts`) |
| Gestures | `LongPress` and `use_hold`: a finger held still, timed by the host's sleep (`Unit::set_timer`) |
| Layout | `Stack`, `Spacer`, `Card`, `Divider`, `Field` (a form's row), `Heading`, `Text`, `Badge`, `Progress`, `Callout` |
| Theme | `Theme`: a light and a dark `Palette`, a pointer's and a finger's `Metrics`, and a text and a heading font, written as CSS custom properties; `Appearance`, `Density` and `Accent` on the root |

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
renderer while `Unit::typing()`. A toolbar over the host's editor gives
its buttons `refuses_keys: true`: pressed, they leave the keys with the
editor, as a Mac's toolbar buttons do. unit's styles set `box-sizing` on its
own elements only, so the host's under `UnitRoot` keep the box model they
chose.

`accent: Accent::System` on the root takes the accent from the system where
unit can read one, and the theme's anywhere else. On a Mac, with the `apple`
feature, that is the accent and highlight chosen in System Settings, and the
focus ring drawn from them, followed when the user changes them:

```toml
unit = { git = "https://github.com/diaryx-org/unit", branch = "main", features = ["apple"] }
```

On a phone, the host tells unit what the platform knows and Blitz does not
pass on: the safe area (`Unit::set_insets`), which sheets, bars, dialogs
and toasts keep clear of; the soft keyboard's height (`Unit::set_keyboard`),
which a sheet sits on and a toast above; the appearance, `Light` or `Dark`
rather than `System`; and a sleep (`Unit::set_timer`), which times a long
press while the finger is still down.

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

The gallery builds Blitz from diaryx-org's fork, its `leaf` branch, which
carries what the gallery's tests found Blitz needed (diaryx-org/blitz#1):

- **ARIA's states.** Blitz read roles but not states: a checkbox's tick, a
  toggle's pressed state, a slider's value, a dialog's modality.
- **A node removed under `display: none`** stayed in its layout parent's
  children, and showing the parent again laid out a freed node: the "layout
  child … is not in the slab" panic.
- **A press** focused only text fields, so keys never reached a pressed list
  or tab row; and a click went to whatever the release landed on, so a press
  that closed a menu also clicked what lay under it.

and blitz-shell carrying out assistive tech's click, so VoiceOver and an
agent's `AXPress` press a control in the gallery's window.

## Licence

unit is MIT or Apache-2.0, at your choice (`LICENSE-MIT`, `LICENSE-APACHE`).
Its icon paths (`icon::paths`) are [Lucide](https://lucide.dev)'s, under the
ISC licence in `LICENSE-LUCIDE`.

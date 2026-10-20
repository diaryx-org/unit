---
title: unit on iOS
part_of: '[Tasks](/docs/tasks/tasks.md)'
status: open
created: 2026-10-20
---
# unit on iOS

**Where.** The gallery. diaryx-org/blitz's `examples/ios-a11y` runs a Blitz
page on the iOS simulator with AccessKit's iOS adapter and the fork's
`patches/winit-uikit`, and checks it with XCUITest: the way to run the
gallery there.

**What.** unit is built for a finger: it acts on pointer events and
`click`, which Blitz sends for a finger as for the mouse (it sends a finger
no mouse events); its touch density draws 44-point targets; a menu keeps
inside a phone's width. The gallery's tests tap with a finger's pointer id,
and a 390-point page was looked at on the Mac. None of it has run on a
phone. What may differ there:

- A **drag** on a slider may pan the page instead: Blitz takes a finger's
  move as a pan unless something stops it, and unit's `touch-action` is not
  set.
- A **long press** opens no context menu: `ContextMenu` opens on
  `contextmenu`, which a finger does not send, and unit has no timers to
  time a long press with.
- The **soft keyboard** comes up for a real `<input>` only if Blitz on iOS
  asks for it when the field is focused.
- **VoiceOver** reads the AccessKit tree, and presses through blitz-shell's
  click (both seen in `examples/ios-a11y`); that each control's state
  (ticked, pressed, a value) is read out is unchecked.

**Done when.** The gallery runs on an iPhone and an iPad simulator, each
component is used by touch and with VoiceOver, and what does not work has a
fix or a task.

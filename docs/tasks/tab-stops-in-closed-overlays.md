---
title: Tab stops in closed overlays
part_of: '[Tasks](/docs/tasks/tasks.md)'
status: open
created: 2026-11-05
---
# Tab stops in closed overlays

**Where.** `Modal`, the dialog and the action sheet (`components/modal.rs`,
`layer.rs`), which stay in the page under `display: none` from the first
frame; and diaryx-org/blitz's `leaf` branch, whose `focus_next_node` takes any
`is_focussable` node and does not ask whether it is displayed.

**What.** Under Blitz, Tab goes on past the page's last control into the
controls of every closed overlay: a keyboard's focus lands on something drawn
nowhere. Found by the audit, as its `keyboard ... hidden:` cells. A browser
skips what is not displayed, so this is Blitz's alone.

**Done when.** Tab skips what is not displayed (a fix on the fork's `leaf`, or
upstream), and those cells pass.

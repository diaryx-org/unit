---
title: Overlays on a scrolled page
part_of: '[Tasks](/docs/tasks/tasks.md)'
status: open
created: 2026-10-20
---
# Overlays on a scrolled page

**Where.** The overlay (`layer.rs`: menus, the dialog, the action sheet,
toasts) and `Modal`, all `position: fixed`; Blitz's layout of a fixed box,
on diaryx-org/blitz's `leaf` branch.

**What.** In the gallery in a window, with the page scrolled down, the
overlay is laid out against the top of the document, not the window: the
scrim covers only the band of the window the document's first screen still
occupies, and a dialog or an action sheet opens off screen above. Seen on a
Mac, 2026-10-20, with Confirm… and Actions…; the headless tests never scroll
the page, so they do not see it. Blitz appears to place a fixed box as if it
were absolute to the root.

A page whose root does not scroll, and whose content scrolls in a box of its
own (as Diaryx's app does), does not meet it.

**Done when.** A fixed box is placed against the viewport in Blitz however
far the document is scrolled (a fix on the fork's `leaf`, or upstream), and
a gallery test opens a dialog and an action sheet on a scrolled page and
finds them on screen.

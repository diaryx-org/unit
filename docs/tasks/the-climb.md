---
title: The climb
part_of: '[Tasks](/docs/tasks/tasks.md)'
status: open
created: 2026-11-05
---
# The climb

**Where.** `gallery/tests/audit.rs` and `gallery/audit.baseline`, which the
README's "The audit" explains.

**What.** The audit is in place, and what it found is listed. What is left to
climb, in order:

1. **The floor.** Contrast, edges and focus rings: mostly the palette, so a
   change every consumer sees, for Adam to look at before it lands, with a
   `Behavioural-change:` trailer. Then target sizes and `Segmented`'s missing
   ring.
2. **What is known.** [Overlays on a scrolled
   page](/docs/tasks/overlays-on-a-scrolled-page.md), [Tab stops in closed
   overlays](/docs/tasks/tab-stops-in-closed-overlays.md), and `CodeField` past
   a 320-point page's edge.
3. **Adoption.** Why diaryx still draws its own controls, starting at
   `shell/sheets.rs`: each is a capability unit lacks, or a migration. diaryx
   is private, so this count runs in its checkout, not in unit's CI.
4. **The other renderers.** The audit reads Blitz alone. A browser needs a
   job with one, and [a phone](/docs/tasks/unit-on-ios.md) a simulator.
5. **The look.** Screenshots side by side, judged by a person. Blitz's
   `anyrender_vello_cpu` is in `Cargo.lock`; drawing with it headless is
   untried.

**Done when.** `gallery/audit.baseline` holds only what is there by design, and
adoption, a browser and a phone are each measured.

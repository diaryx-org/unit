---
title: A gooey look
part_of: '[Tasks](/docs/tasks/tasks.md)'
created: 2026-10-21
status: open
---
# A gooey look

**Where.** The branch `gooey-look` (5cb9a2c), taken out of `unit-styling`
again by 4cac29e: a `Look::Gooey` on `UnitRoot`, and a segmented control
that draws its choice through it.

**What it was.** `UnitRoot` puts an SVG filter in the page, a blur and then
an alpha edge (`feGaussianBlur` 4, `feColorMatrix` alpha ×20 −9), and a
segmented control draws its choice and a smaller drop through it, behind
its segments, at the choice's share of the width (the segments equal, a
grid of `1fr` columns). On a change the choice springs to the new segment
(`linear()`, 420 ms) and the drop follows (700 ms), stretching from it and
rejoining it. It drew on Blitz with diaryx-org/blitz's `leaf` (color
matrices and `--blitz-filter`, #10 and #11), smoothly, light and dark, at
both densities, with an SSR test and a Blitz layout test.

**Why it waits.** Tried in the gallery on 2026-10-21: smooth, but neither
familiar nor comparable to the libraries it was after, liquid-gooey's
Morph and Move or Apple's Liquid Glass. A drop chasing a pill is not what
those do: liquid-gooey's surface trails the moving element as one body
with a velocity stretch and a droplet tail, on springs it drives itself;
Liquid Glass melds glass shapes, refracting what is under them. Before
another try, a reference to match frame by frame, then the motion first.

**What is in place for it.** The renderer side stays: `vello-hybrid`
draws a blur followed by color matrices, and an SVG `<filter>` named by
`--blitz-filter` (`FORK.md` in diaryx-org/blitz). Glass that melds needs
more: the merged shape as a backdrop's mask, which vello_gpu does not
draw.

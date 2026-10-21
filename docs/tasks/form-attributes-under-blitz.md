---
title: Form attributes under Blitz
part_of: '[Tasks](/docs/tasks/tasks.md)'
status: open
created: 2026-10-21
---
# Form attributes under Blitz

**Where.** `TextField` and `TextArea` (`components/text_field.rs`), and
diaryx-org/blitz's `leaf` branch: `blitz-dom`'s text input and its
AccessKit tree.

**What.** The fields carry `name`, `autocomplete`, `required`, `minlength`
and `maxlength`, for the historica hub's account page, which runs in a
browser. A browser keeps to the length and fills a field from a password
manager. Blitz reads none of them, as of the fork's `a7eaa0d6`: a field
takes text past its `maxlength`, `required` reaches assistive tech only as
the `aria-required` unit writes beside it, and `autocomplete` has no
password manager to ask.

**Done when.** Blitz stops a field at its `maxlength` (a fix on the fork,
or upstream, or unit trimming the text in `oninput`), its AccessKit node
says a field is required, and a gallery test types past a `maxlength` and
finds the text cut at it.

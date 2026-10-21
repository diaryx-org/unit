//! Every component's styles. Colours and sizes are the theme's custom
//! properties (`theme.rs`) and nothing else, so a theme restyles them all.
//!
//! Written for what Blitz lays out as a browser does: flexbox, absolute
//! positions by `left`/`top`/`right`/`bottom`, attribute selectors, `:hover`
//! — not `inset`, `calc()`, transitions or animations. A menu's rows and
//! separators have fixed heights, the ones `menu::Sizes` computes its
//! geometry by, for each density.

pub const CSS: &str = r#"
.unit-root { font-family: var(--unit-font); font-size: var(--unit-font-size);
             color: var(--unit-text); background: var(--unit-bg); outline: none;
             line-height: 1.35; }
/* unit's own elements and what is in them, not the page's: a host's editor
   under the root lays out by the box model it chose. */
.unit-root [class*="unit-"], .unit-root [class*="unit-"] *,
.unit-root [class*="unit-"]::before, .unit-root [class*="unit-"]::after { box-sizing: border-box; }
.unit-focused { outline: 2px solid var(--unit-focus); outline-offset: 1px; }

/* The overlay */
.unit-layer { position: fixed; left: 0; top: 0; right: 0; bottom: 0; z-index: 100;
              pointer-events: none; white-space: nowrap; }
.unit-backdrop, .unit-menu, .unit-dialog, .unit-toast { pointer-events: auto; }
.unit-backdrop { position: absolute; left: 0; top: 0; right: 0; bottom: 0; }
.unit-backdrop.unit-scrim { background: var(--unit-scrim); }

/* Menus: rows and separators the heights menu::Sizes says */
.unit-menu { position: absolute; padding: 5px 0; font-size: 13px;
             background: var(--unit-surface); color: var(--unit-text);
             border: 1px solid var(--unit-border); border-radius: 7px;
             box-shadow: 0 6px 20px var(--unit-shadow); }
.unit-item { display: flex; height: 22px; line-height: 22px; margin: 0 5px; padding: 0 8px 0 0;
             border-radius: 4px; }
.unit-item.hot { background: var(--unit-accent); color: var(--unit-accent-text); }
.unit-item.off { color: var(--unit-dim); }
.unit-item.danger { color: var(--unit-danger); }
.unit-item.danger.hot { background: var(--unit-danger); color: var(--unit-accent-text); }
.unit-item .unit-svg { flex: none; align-self: center; margin-right: 6px; }
.unit-tick { width: 20px; flex: none; text-align: center; }
.unit-label { flex: 1; }
.unit-key { flex: none; padding-left: 18px; color: var(--unit-dim); }
.unit-item.hot .unit-key { color: var(--unit-accent-text); }
.unit-sep { height: 9px; padding: 4px 10px; }
.unit-rule { height: 1px; background: var(--unit-border); }
.unit-root[data-density="touch"] .unit-menu { padding: 6px 0; font-size: 17px; border-radius: 12px; }
.unit-root[data-density="touch"] .unit-item { height: 40px; line-height: 40px; border-radius: 8px; }
.unit-root[data-density="touch"] .unit-tick { width: 28px; }
.unit-root[data-density="touch"] .unit-sep { height: 13px; padding: 6px 12px; }

/* The overlay's dialog */
.unit-dialog-place { position: absolute; left: 0; top: 0; right: 0; bottom: 0;
                     flex-direction: column; align-items: center; padding: 72px 16px 16px; }
.unit-dialog { width: 380px; max-width: 100%; padding: 16px 18px 14px; white-space: normal;
               background: var(--unit-surface); color: var(--unit-text);
               border: 1px solid var(--unit-border); border-radius: var(--unit-radius-large);
               box-shadow: 0 12px 40px var(--unit-shadow); }
.unit-dialog-title { font-weight: 700; margin-bottom: 6px; }
.unit-dialog-message { margin-bottom: 10px; }
.unit-dialog .unit-input { margin: 4px 0 14px; }
.unit-dialog-buttons { display: flex; gap: 8px; justify-content: flex-end; margin-top: 12px; }
.unit-field { line-height: var(--unit-control); white-space: pre; overflow: hidden; }
.unit-field.all { background: var(--unit-selection); }
.unit-caret { display: inline-block; width: 1px; height: 16px; vertical-align: middle;
              background: var(--unit-text); }
.unit-field.all .unit-caret { width: 0; }
.unit-placeholder { color: var(--unit-dim); }

/* Toasts */
.unit-toasts { position: absolute; left: 0; right: 0; bottom: 16px; display: flex;
               flex-direction: column; align-items: center; gap: 8px; }
.unit-toast { display: flex; align-items: center; gap: 8px; padding: 6px 6px 6px 14px;
              max-width: 90%; white-space: normal;
              background: var(--unit-surface); color: var(--unit-text);
              border: 1px solid var(--unit-border); border-radius: var(--unit-radius-large);
              box-shadow: 0 6px 20px var(--unit-shadow); }
.unit-toast-message { flex: 1; }
.unit-toast.unit-success { border-color: var(--unit-success); }
.unit-toast.unit-warning { border-color: var(--unit-warning); }
.unit-toast.unit-danger { border-color: var(--unit-danger); }

/* Buttons */
.unit-button { display: inline-flex; align-items: center; justify-content: center; gap: 6px;
               height: var(--unit-control); padding: 0 12px; margin: 0;
               font-family: inherit; font-size: inherit; line-height: 1; white-space: nowrap;
               color: var(--unit-text); background: var(--unit-control);
               border: 1px solid var(--unit-border); border-radius: var(--unit-radius); }
.unit-button:hover { background: var(--unit-control-hover); }
.unit-button.unit-primary { background: var(--unit-accent); color: var(--unit-accent-text);
                            border-color: var(--unit-accent); }
.unit-button.unit-danger { background: var(--unit-danger); color: var(--unit-accent-text);
                           border-color: var(--unit-danger); }
.unit-button.unit-ghost { background: transparent; border-color: transparent; }
.unit-button.unit-ghost:hover { background: var(--unit-control-hover); }
.unit-button.unit-icon { width: var(--unit-control); padding: 0; }
.unit-svg { flex: none; }
.unit-button[disabled] { color: var(--unit-dim); }
.unit-toggle.on { background: var(--unit-selection); border-color: var(--unit-accent); }
.unit-select { justify-content: space-between; padding: 0 8px 0 10px; min-width: 120px; }
.unit-select-label { flex: 1; text-align: left; padding-right: 10px; }
.unit-select-arrows { color: var(--unit-dim); }

/* Checkbox, switch, radios, segments */
.unit-check, .unit-switch { display: flex; align-items: center; gap: 8px;
                            min-height: var(--unit-control); }
.unit-check-box { display: flex; align-items: center; justify-content: center; flex: none;
                  width: 16px; height: 16px; font-size: 12px; line-height: 1;
                  color: var(--unit-accent-text); background: var(--unit-control);
                  border: 1px solid var(--unit-border); border-radius: 4px; }
.unit-check.on .unit-check-box { background: var(--unit-accent); border-color: var(--unit-accent); }
.unit-check.off, .unit-switch.off, .unit-slider.off { color: var(--unit-dim); }
.unit-switch-label { flex: 1; }
.unit-switch-track { display: flex; flex: none; width: 32px; height: 18px; padding: 2px;
                     background: var(--unit-border); border-radius: 9px; }
.unit-switch.on .unit-switch-track { justify-content: flex-end; background: var(--unit-accent); }
.unit-switch-knob { width: 14px; height: 14px; border-radius: 7px; background: #ffffff;
                    box-shadow: 0 1px 2px var(--unit-shadow); }
.unit-radios { display: flex; flex-direction: column; gap: 4px; }
.unit-radios.unit-row { flex-direction: row; gap: 14px; }
.unit-radio { display: flex; align-items: center; gap: 8px; min-height: var(--unit-control); }
.unit-radio-ring { display: flex; align-items: center; justify-content: center; flex: none;
                   width: 16px; height: 16px; border-radius: 8px;
                   background: var(--unit-control); border: 1px solid var(--unit-border); }
.unit-radio.on .unit-radio-ring { background: var(--unit-accent); border-color: var(--unit-accent); }
.unit-radio-dot { width: 6px; height: 6px; border-radius: 3px; background: transparent; }
.unit-radio.on .unit-radio-dot { background: var(--unit-accent-text); }
.unit-root[data-density="touch"] .unit-check-box,
.unit-root[data-density="touch"] .unit-radio-ring { width: 24px; height: 24px; border-radius: 12px; }
.unit-root[data-density="touch"] .unit-check-box { border-radius: 6px; font-size: 16px; }
.unit-root[data-density="touch"] .unit-radio-dot { width: 10px; height: 10px; border-radius: 5px; }
.unit-root[data-density="touch"] .unit-switch-track { width: 51px; height: 31px; border-radius: 16px; }
.unit-root[data-density="touch"] .unit-switch-knob { width: 27px; height: 27px; border-radius: 14px; }
.unit-segmented { display: inline-flex; align-self: flex-start; height: var(--unit-control); overflow: hidden;
                  background: var(--unit-control); border: 1px solid var(--unit-border);
                  border-radius: var(--unit-radius); }
.unit-segment { display: flex; align-items: center; padding: 0 12px;
                border-left: 1px solid var(--unit-border); }
.unit-segment:first-child { border-left: none; }
.unit-segment.on { background: var(--unit-accent); color: var(--unit-accent-text); }

/* Slider: the thumb's and the stops' room inside its own box */
.unit-slider { position: relative; height: var(--unit-control); min-width: 120px; width: 100%; }
.unit-slider-rail { position: absolute; left: 8px; right: 8px; top: 0; bottom: 0; }
.unit-slider-line { position: absolute; left: 0; right: 0; top: 50%;
                    height: 4px; margin-top: -2px; border-radius: 2px;
                    background: var(--unit-border); }
.unit-slider-fill { position: absolute; left: 0; top: 50%; height: 4px; margin-top: -2px;
                    border-radius: 2px; background: var(--unit-accent); }
.unit-slider-thumb { position: absolute; top: 50%; width: 16px; height: 16px;
                     margin: -8px 0 0 -8px; border-radius: 8px; background: #ffffff;
                     border: 1px solid var(--unit-border); box-shadow: 0 1px 3px var(--unit-shadow); }
.unit-root[data-density="touch"] .unit-slider-thumb { width: 28px; height: 28px;
                                                      margin: -14px 0 0 -14px; border-radius: 14px; }
.unit-slider-stops { position: absolute; left: 8px; right: 8px; top: 0; bottom: 0; display: flex; }
.unit-slider-stop { flex-basis: 0; height: 100%; }

/* Text */
.unit-text-field { display: flex; flex-direction: column; gap: 4px; }
.unit-field-label { font-weight: 600; }
.unit-input { display: block; width: 100%; height: var(--unit-control); padding: 0 8px; margin: 0;
              font-family: inherit; font-size: inherit; color: var(--unit-text);
              background: var(--unit-control); border: 1px solid var(--unit-border);
              border-radius: var(--unit-radius); }
.unit-textarea { height: auto; padding: 6px 8px; }
.unit-heading { margin: 0 0 6px; font-weight: 700; }
.unit-text { margin: 0 0 6px; }
.unit-text.dim, .unit-note { color: var(--unit-dim); }
.unit-note { font-size: 0.92em; margin-top: 4px; }

/* Choosing among: chips, tabs, a list, a disclosure */
.unit-chips { display: flex; align-self: stretch; flex-wrap: wrap; gap: 6px; }
.unit-chip { padding: 3px 10px; margin: 0; font-family: inherit; font-size: inherit;
             color: var(--unit-text); background: var(--unit-control);
             border: 1px solid var(--unit-border); border-radius: 999px; white-space: nowrap; }
.unit-chip.on { background: var(--unit-accent); color: var(--unit-accent-text);
                border-color: var(--unit-accent); }
.unit-root[data-density="touch"] .unit-chip { padding: 8px 14px; }
.unit-tablist { display: flex; gap: 4px; border-bottom: 1px solid var(--unit-border); }
.unit-tab { display: flex; align-items: center; height: var(--unit-control); padding: 0 12px;
            color: var(--unit-dim); border-bottom: 2px solid transparent; }
.unit-tab.on { color: var(--unit-text); border-bottom-color: var(--unit-accent); }
.unit-tabpanel { padding-top: 12px; }
.unit-listbox { display: flex; flex-direction: column; padding: 4px; overflow: auto;
                background: var(--unit-control); border: 1px solid var(--unit-border);
                border-radius: var(--unit-radius); }
.unit-option { display: flex; align-items: center; min-height: var(--unit-control);
               padding: 0 8px; border-radius: 4px; }
.unit-option.on { background: var(--unit-accent); color: var(--unit-accent-text); }
.unit-disclosure-title { display: flex; align-items: center; gap: 6px; width: 100%;
                         height: var(--unit-control); padding: 0; margin: 0; text-align: left;
                         font-family: inherit; font-size: inherit; font-weight: 600;
                         color: var(--unit-text); background: transparent; border: none; }
.unit-disclosure-arrow { width: 12px; color: var(--unit-dim); }
.unit-disclosure-body { padding: 4px 0 8px 18px; }

/* Rows in groups */
.unit-section { display: flex; flex-direction: column; gap: 6px; }
.unit-section-title { padding: 0 14px; font-size: 0.85em; color: var(--unit-dim); }
.unit-section-footer { padding: 0 14px; font-size: 0.85em; color: var(--unit-dim); }
.unit-section-body { display: flex; flex-direction: column; overflow: hidden;
                     background: var(--unit-surface); border: 1px solid var(--unit-border);
                     border-radius: var(--unit-radius-large); }
.unit-section-body > * { border-top: 1px solid var(--unit-border); }
.unit-section-body > *:first-child { border-top: none; }
.unit-section-body > *:not(.unit-row) { padding: 6px 14px; }
.unit-row { display: flex; align-items: center; gap: 10px; min-height: var(--unit-control);
            padding: 6px 14px; }
.unit-row.press:hover { background: var(--unit-control-hover); }
.unit-row.danger { color: var(--unit-danger); }
.unit-row.off { color: var(--unit-dim); }
.unit-row-text { display: flex; flex-direction: column; flex: 1; min-width: 0; }
.unit-row-subtitle { font-size: 0.85em; color: var(--unit-dim); }
.unit-row-detail { flex: none; color: var(--unit-dim); }
.unit-row-trailing { display: flex; justify-content: flex-end; flex: 0 1 55%; }
.unit-row-blank { width: 16px; flex: none; }
.unit-text-field.unit-inline { flex: 1; }
.unit-input.unit-inline { padding: 0; text-align: right; background: transparent;
                          border-color: transparent; }

/* A modal of the page's own */
.unit-modal-place { position: fixed; left: 0; top: 0; right: 0; bottom: 0; z-index: 90;
                    flex-direction: column; align-items: center; justify-content: center;
                    padding: 16px; background: var(--unit-scrim); }
.unit-modal-place.unit-sheet-place { justify-content: flex-end; padding: 0; }
.unit-modal { max-width: 100%; max-height: 100%; overflow: auto; padding: 14px 16px;
              background: var(--unit-surface); color: var(--unit-text);
              border: 1px solid var(--unit-border); border-radius: var(--unit-radius-large);
              box-shadow: 0 12px 40px var(--unit-shadow); }
.unit-modal.unit-sheet { width: 100%; border-bottom-left-radius: 0; border-bottom-right-radius: 0;
                         padding-bottom: 28px; }
.unit-modal-head { display: flex; align-items: center; justify-content: space-between;
                   gap: 8px; margin-bottom: 8px; }
.unit-modal-head .unit-dialog-title { margin: 0; }
.unit-sheet .unit-modal-head, .unit-sheet-grabber { touch-action: none; }
.unit-sheet-grabber { display: flex; justify-content: center; padding: 0 0 8px; margin-top: -6px; }
.unit-sheet-grabber-bar { width: 36px; height: 5px; border-radius: 3px; background: var(--unit-border); }
.unit-modal-head.unit-phone-bar .unit-dialog-title { flex: 1; text-align: center; }
.unit-modal-side { display: flex; flex: 1 1 0; }
.unit-modal-side.unit-modal-end { justify-content: flex-end; }
.unit-modal-head.unit-phone-bar .unit-dialog-title { flex: 2 1 0; }
.unit-button.unit-done { font-weight: 700; color: var(--unit-accent); }

/* Layout and feedback */
.unit-stack { display: flex; }
.unit-spacer { flex: 1; }
.unit-card { display: flex; flex-direction: column; gap: var(--unit-gap);
             padding: 12px 14px; background: var(--unit-surface);
             border: 1px solid var(--unit-border); border-radius: var(--unit-radius-large); }
.unit-card-title { margin: 0; font-size: 1em; font-weight: 700; }
.unit-divider { height: 1px; background: var(--unit-border); margin: 4px 0; }
.unit-form-row { display: flex; align-items: flex-start; gap: var(--unit-gap); }
.unit-form-label { flex: none; width: 34%; padding-top: 4px; text-align: right; }
.unit-form-control { flex: 1; }
.unit-root[data-density="touch"] .unit-form-row { flex-direction: column; align-items: stretch; gap: 4px; }
.unit-root[data-density="touch"] .unit-form-label { width: auto; text-align: left; }
.unit-badge { display: inline-block; padding: 1px 7px; font-size: 0.85em; border-radius: 999px;
              background: var(--unit-control); border: 1px solid var(--unit-border); }
.unit-badge.unit-success { background: var(--unit-success); border-color: var(--unit-success); color: #ffffff; }
.unit-badge.unit-warning { background: var(--unit-warning); border-color: var(--unit-warning); color: #ffffff; }
.unit-badge.unit-danger { background: var(--unit-danger); border-color: var(--unit-danger); color: #ffffff; }
.unit-progress { height: 6px; overflow: hidden; border-radius: 3px; background: var(--unit-border); }
.unit-progress-fill { height: 100%; background: var(--unit-accent); }
.unit-callout { padding: 10px 12px; border-radius: var(--unit-radius);
                border: 1px solid var(--unit-border); background: var(--unit-surface); }
.unit-callout.unit-success { border-color: var(--unit-success); }
.unit-callout.unit-warning { border-color: var(--unit-warning); }
.unit-callout.unit-danger { border-color: var(--unit-danger); }
.unit-callout-title { font-weight: 700; margin-bottom: 4px; }
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::Sizes;

    /// A menu's geometry is computed from `Sizes`, so its rows must be drawn
    /// that tall.
    #[test]
    fn menu_rows_are_the_heights_their_geometry_assumes() {
        let r = Sizes::REGULAR;
        assert!(CSS.contains(&format!(
            ".unit-item {{ display: flex; height: {}px",
            r.item
        )));
        assert!(CSS.contains(&format!(".unit-sep {{ height: {}px", r.separator)));
        assert!(CSS.contains(&format!(
            ".unit-menu {{ position: absolute; padding: {}px 0",
            r.pad
        )));
        let t = Sizes::TOUCH;
        assert!(CSS.contains(&format!(".unit-item {{ height: {}px", t.item)));
        assert!(CSS.contains(&format!(".unit-sep {{ height: {}px", t.separator)));
        assert!(CSS.contains(&format!(".unit-menu {{ padding: {}px 0", t.pad)));
    }
}

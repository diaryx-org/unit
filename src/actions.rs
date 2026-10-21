//! An action sheet: a phone's menu, a column of actions rising from the
//! bottom of the page with Cancel apart under them, for what a long press or
//! a "⋯" offers. It is drawn by the overlay (`layer.rs`), in the tree from
//! the first frame like the overlay's menus and dialog, and like them takes
//! every key while it is open: Escape cancels it, ↑ and ↓ (or Tab) move a
//! highlight through the actions and Cancel, and Return or Space chooses the
//! one highlighted — so a sheet can be answered from a keyboard.

/// One action an [`ActionSheet`] offers: its `id` is what the sheet's pick
/// handler is given.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetAction {
    pub id: usize,
    pub label: String,
    /// An icon beside the label: a path for [`crate::Icon`].
    pub icon: Option<&'static str>,
    /// Destroys something: drawn in the danger colour.
    pub destructive: bool,
    pub enabled: bool,
}

impl SheetAction {
    pub fn new(id: usize, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            destructive: false,
            enabled: true,
        }
    }

    pub fn icon(mut self, d: &'static str) -> Self {
        self.icon = Some(d);
        self
    }

    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    pub fn enabled(mut self, yes: bool) -> Self {
        self.enabled = yes;
        self
    }
}

/// What an action sheet shows: a title and a message over its actions, and
/// the label of the button that cancels.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionSheet {
    pub title: Option<String>,
    pub message: Option<String>,
    pub actions: Vec<SheetAction>,
    pub cancel: String,
    /// The row the keys have highlighted: an action's index, or
    /// `actions.len()` for Cancel. None until a key moves it.
    pub hot: Option<usize>,
}

impl ActionSheet {
    pub fn new(actions: Vec<SheetAction>) -> Self {
        Self {
            title: None,
            message: None,
            actions,
            cancel: "Cancel".into(),
            hot: None,
        }
    }

    /// Move the highlight down (or up) to the next action that can be
    /// chosen, or to Cancel, round from the end to the start.
    pub fn step(&mut self, down: bool) {
        let rows = self.actions.len() + 1;
        let usable = |i: usize| i == self.actions.len() || self.actions[i].enabled;
        let mut i = match (self.hot, down) {
            (None, true) => rows - 1,
            (None, false) => 0,
            (Some(h), _) => h,
        };
        for _ in 0..rows {
            i = if down {
                (i + 1) % rows
            } else {
                (i + rows - 1) % rows
            };
            if usable(i) {
                self.hot = Some(i);
                return;
            }
        }
    }

    /// What Return chooses: `Some(Some(id))` for a highlighted action,
    /// `Some(None)` for Cancel, `None` with nothing highlighted.
    pub fn chosen(&self) -> Option<Option<usize>> {
        let h = self.hot?;
        match self.actions.get(h) {
            Some(a) if a.enabled => Some(Some(a.id)),
            Some(_) => None,
            None => Some(None),
        }
    }

    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    pub fn message(mut self, m: impl Into<String>) -> Self {
        self.message = Some(m.into());
        self
    }

    pub fn cancel(mut self, label: impl Into<String>) -> Self {
        self.cancel = label.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet() -> ActionSheet {
        ActionSheet::new(vec![
            SheetAction::new(7, "Save"),
            SheetAction::new(8, "Off").enabled(false),
            SheetAction::new(9, "Don't Save").destructive(),
        ])
    }

    #[test]
    fn the_keys_step_past_what_cannot_be_chosen_and_round_to_cancel() {
        let mut s = sheet();
        assert_eq!(s.chosen(), None);
        s.step(true);
        assert_eq!(s.chosen(), Some(Some(7)));
        s.step(true);
        assert_eq!(s.chosen(), Some(Some(9)));
        s.step(true);
        assert_eq!(s.chosen(), Some(None));
        s.step(true);
        assert_eq!(s.chosen(), Some(Some(7)));
        s.step(false);
        assert_eq!(s.chosen(), Some(None));
    }
}

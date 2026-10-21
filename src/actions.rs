//! An action sheet: a phone's menu, a column of actions rising from the
//! bottom of the page with Cancel apart under them, for what a long press or
//! a "⋯" offers. It is drawn by the overlay (`layer.rs`), in the tree from
//! the first frame like the overlay's menus and dialog, and like them takes
//! every key while it is open: Escape cancels it.

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
}

impl ActionSheet {
    pub fn new(actions: Vec<SheetAction>) -> Self {
        Self {
            title: None,
            message: None,
            actions,
            cancel: "Cancel".into(),
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

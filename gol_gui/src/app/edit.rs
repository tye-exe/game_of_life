use crate::lang;
use std::fmt::Display;

lang! {
    PREVIEW, "Preview";
    DRAW, "Draw"
}

#[derive(Default, PartialEq, Clone, Copy)]
pub(crate) enum EditState {
    #[default]
    Preview,
    Draw,
}

impl Display for EditState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            EditState::Preview => PREVIEW,
            EditState::Draw => DRAW,
        })
    }
}

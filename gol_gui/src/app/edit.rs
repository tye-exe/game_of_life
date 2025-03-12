use egui::Pos2;
use gol_lib::{Area, BoardDisplay, GlobalPosition, communication::UiPacket};

use crate::lang;
use std::fmt::Display;

use super::{Action, History};

lang! {
    PREVIEW, "Preview";
    DRAW, "Draw";
    SELECT, "Select"
}

#[derive(Default, PartialEq, Clone, Copy)]
pub(crate) enum EditState {
    #[default]
    Preview,
    Draw,
    Select,
}

impl Display for EditState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            EditState::Preview => PREVIEW,
            EditState::Draw => DRAW,
            EditState::Select => SELECT,
        })
    }
}

pub(crate) fn preview_interaction(
    x_offset: &mut f32,
    y_offset: &mut f32,
    display_area: &mut Area,
    cell_size: f32,
    to_send: &mut Vec<UiPacket>,
    interact: egui::Response,
) {
    // Scroll the display in response to user dragging mouse
    if interact.dragged() {
        let drag_delta = interact.drag_delta();
        *x_offset += drag_delta.x;
        *y_offset += drag_delta.y;
    }
}

pub(crate) fn draw_interaction(
    cell_size: f32,
    display_area: Area,
    display_cache: &BoardDisplay,
    x_offset: f32,
    y_offset: f32,
    history: &mut History,
    to_send: &mut Vec<UiPacket>,
    interact: egui::Response,
) {
    // Toggles the state of a cell when it is clicked.
    if let (true, Some(position)) = (interact.clicked(), interact.interact_pointer_pos()) {
        let x = position.x - x_offset;
        let y = position.y - y_offset;

        // Position of cell
        let local_x = (x / cell_size).trunc() as i32;
        let local_y = (y / cell_size).trunc() as i32;

        // Position of displayed board
        let offset_x = display_area.get_min().get_x();
        let offset_y = display_area.get_min().get_y();

        let position = GlobalPosition::new(local_x + offset_x, local_y + offset_y);

        let view_offset = display_area.get_min() - display_cache.get_area().get_min();
        let cell_state = display_cache
            .get_cell(GlobalPosition::new(local_x, local_y) + view_offset)
            .invert();

        history.add_action(Action::set(position, cell_state));

        to_send.push(UiPacket::Set {
            position,
            cell_state,
        });
    }
}

/// Updates the position of the selection whilst it is being drawn.
pub(crate) fn select_interaction(
    ctx: &egui::Context,
    interact: egui::Response,
    selection: &mut Selection,
    cell_size: f32,
    origin_min: GlobalPosition,
) {
    // If the selection is being dragged then expand the selection
    if let (true, Some(position)) = (interact.dragged(), ctx.pointer_interact_pos()) {
        let cell_x = (position.x / cell_size).trunc() as i32;
        let cell_y = (position.y / cell_size).trunc() as i32;

        let origin_x = origin_min.get_x();
        let origin_y = origin_min.get_y();

        let position = GlobalPosition::new(cell_x + origin_x, cell_y + origin_y);
        selection.drag_end = position;
    }
}

/// Holds the information regarding the area of the board that was selected by the user.
pub(crate) struct Selection {
    drag_start: GlobalPosition,
    drag_end: GlobalPosition,
}

impl Selection {
    /// Creates a new selection starting at the given global position.
    pub(crate) fn new(pos: GlobalPosition) -> Self {
        Self {
            drag_start: pos,
            drag_end: pos,
        }
    }

    /// Gets the positions that this selection should be drawn between.
    /// These positions **are not** sorted into minimum and maximum.
    pub(crate) fn get_draw_positions(
        &self,
        origin_min: GlobalPosition,
        cell_size: f32,
        x_board_offset: f32,
        y_board_offset: f32,
    ) -> (Pos2, Pos2) {
        (
            Self::from_pos(
                self.drag_start,
                origin_min,
                x_board_offset,
                y_board_offset,
                cell_size,
            ),
            Self::from_pos(
                self.drag_end,
                origin_min,
                x_board_offset,
                y_board_offset,
                cell_size,
            ),
        )
    }

    /// Converts a global position into a egui position.
    fn from_pos(
        position: GlobalPosition,
        origin_min: GlobalPosition,
        x_offset: f32,
        y_offset: f32,
        cell_size: f32,
    ) -> Pos2 {
        let origin_x = origin_min.get_x();
        let origin_y = origin_min.get_y();

        let x_diff = position.get_x() - origin_x;
        let y_diff = position.get_y() - origin_y;

        let x_cell_number = x_diff as f32 * cell_size;
        let y_cell_number = y_diff as f32 * cell_size;

        (x_cell_number + x_offset, y_cell_number + y_offset).into()
    }
}

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
    display_area: &mut Area,
    display_cache: &mut BoardDisplay,
    history: &mut History,
    to_send: &mut Vec<UiPacket>,
    interact: egui::Response,
) {
    // Toggles the state of a cell when it is clicked.
    if let (true, Some(position)) = (interact.clicked(), interact.interact_pointer_pos()) {
        // Position of cell
        let cell_x = (position.x / cell_size).trunc() as i32;
        let cell_y = (position.y / cell_size).trunc() as i32;

        // Position of displayed board
        let origin_x = display_area.get_min().get_x();
        let origin_y = display_area.get_min().get_y();

        let position = GlobalPosition::new(cell_x + origin_x, cell_y + origin_y);
        let cell_state = display_cache.get_cell((cell_x, cell_y)).invert();

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
) {
    // If a new drag is started then reset the origin
    if let (true, Some(pointer_position)) = (interact.drag_started(), ctx.pointer_interact_pos()) {
        selection.drag_start = pointer_position;
    }

    // If the selection is being dragged then expand the selection
    if let (true, Some(pointer_position)) = (interact.dragged(), ctx.pointer_interact_pos()) {
        selection.drag_end = pointer_position;
    }
}

/// Holds the information regarding the area of the board that was selected by the user.
pub(crate) struct Selection {
    drag_start: Pos2,
    drag_end: Pos2,
    origin: GlobalPosition,
    offset: GlobalPosition,
}

impl Selection {
    /// Creates a new selection with no offset from the current board position.
    pub(crate) fn new(drag_start: Pos2, drag_end: Pos2, origin: GlobalPosition) -> Self {
        Self {
            drag_start,
            drag_end,
            origin,
            offset: (0, 0).into(),
        }
    }

    /// Updates the current offset of the selection, if it is different from the current offset.
    pub(crate) fn update_offset(&mut self, current_min: GlobalPosition) {
        if current_min != self.origin + self.offset {
            self.offset = self.origin - current_min;
        }
    }

    /// Calculates the coordinates that the selection should be drawn between.
    /// These positions **are not** sorted into minimum and maximum.
    pub(crate) fn get_draw_positions(
        &self,
        cell_size: f32,
        x_board_offset: f32,
        y_board_offset: f32,
    ) -> (Pos2, Pos2) {
        let mut pos1 = self.drag_start;
        let mut pos2 = self.drag_end;

        // Apply offset based upon cell size & scroll position
        let x_offset = (self.offset.get_x() as f32 * cell_size) + x_board_offset;
        let y_offset = (self.offset.get_y() as f32 * cell_size) + y_board_offset;

        pos1.x += x_offset;
        pos2.x += x_offset;
        pos1.y += y_offset;
        pos2.y += y_offset;

        (pos1, pos2)
    }
}

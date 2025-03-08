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

impl EditState {}

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

        let mut modified_display = false;

        // While loops are used as display can be dragged further than one cell in one frame.
        while *x_offset % cell_size > 0.0 {
            display_area.translate_x(-1);
            *x_offset -= cell_size;
            modified_display = true;
        }

        while *x_offset % cell_size < 0.0 {
            display_area.translate_x(1);
            *x_offset += cell_size;
            modified_display = true;
        }

        while *y_offset % cell_size > 0.0 {
            display_area.translate_y(-1);
            *y_offset -= cell_size;
            modified_display = true;
        }

        while *y_offset % cell_size < 0.0 {
            display_area.translate_y(1);
            *y_offset += cell_size;
            modified_display = true;
        }

        if modified_display {
            let mut new_area = *display_area;

            new_area.modify_max((0, 1));
            new_area.modify_min((-1, 0));

            to_send.push(UiPacket::DisplayArea { new_area });
        }
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

pub(crate) fn select_interaction(
    ctx: &egui::Context,
    interact: egui::Response,
    position: &mut Option<(Pos2, Pos2)>,
) {
    let (drag_start, drag_end) = match position {
        // There is a secltion ongoing
        Some(position) => position,
        // Start a selection if a drag is started
        None => {
            if let (true, Some(pointer_position)) =
                (interact.drag_started(), ctx.pointer_interact_pos())
            {
                *position = Some((pointer_position, pointer_position));
            };
            return;
        }
    };

    // If a new drag is started then reset the origin
    if let (true, Some(pointer_position)) = (interact.drag_started(), ctx.pointer_interact_pos()) {
        *drag_start = pointer_position;
    }

    // If the selection is being dragged then expand the selection
    if let (true, Some(pointer_position)) = (interact.dragged(), ctx.pointer_interact_pos()) {
        *drag_end = pointer_position;
    }
}

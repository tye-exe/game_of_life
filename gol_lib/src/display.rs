//! Contains [`BoardDisplay`].
//! See its documentation for more information.

use std::{num::NonZeroUsize, sync::Arc};

use crate::Area;

use super::{cell::Cell, position::GlobalPosition};

/// Holds the board data for the ui to display.
///
/// This data type assumes that each sub-array has the same length.
/// The top array can be any length, regardless of the sub-array length.
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Clone))]
#[derive(Default)]
pub struct BoardDisplay {
    /// The generation of the board to be displayed.
    generation: u64,
    /// The area of the board that the display data represents.
    display_area: Area,
    /// The cell data for the area of the board to display.
    board: Arc<[Box<[Cell]>]>,
}

impl BoardDisplay {
    /// Constructs a new [`BoardDisplay`] with the given generation & the given board to display.
    ///
    /// # Example
    /// Simple way to create the correct board data type.
    /// ```
    /// # use gol_lib::{Cell, BoardDisplay, Area};
    /// # let generation = 0;
    /// let mut board_build = Vec::new();
    /// for _ in 0..4 {
    ///     let mut y_builder = Vec::new();
    ///     for y in 0..4 {
    ///         y_builder.push(Cell::Dead);
    ///     }
    ///     // Convert the vec into the correct type
    ///     let array: Box<[Cell]> = y_builder.into();
    ///     board_build.push(array);
    /// }
    ///
    /// let area = Area::new((0,0), (0,0));
    ///
    /// BoardDisplay::new(generation, area, board_build);
    /// ```
    pub fn new(generation: u64, display_area: Area, board: impl Into<Arc<[Box<[Cell]>]>>) -> Self {
        Self {
            generation,
            display_area,
            board: board.into(),
        }
    }

    /// Gets the amount of cells in the x axis.
    ///
    /// If the board is 0 sized then an amount of 10 will be returned.
    pub fn get_x(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.board.len()).unwrap_or(unsafe { NonZeroUsize::new_unchecked(10) })
    }

    /// Gets the amount of cells in the y axis.
    ///
    /// If the board is 0 sized then an amount of 10 will be returned.
    pub fn get_y(&self) -> NonZeroUsize {
        self.board
            .first()
            .and_then(|sub_array| NonZeroUsize::new(sub_array.len()))
            .unwrap_or(unsafe { NonZeroUsize::new_unchecked(10) })
    }

    /// Gets the cell at the given position **relative** to this [`BoardDisplay`].
    ///
    /// If the given position is outside the bounds of the display board then [`Cell::Dead`] will be returned.
    pub fn get_cell(&self, position: impl Into<GlobalPosition>) -> Cell {
        let position: GlobalPosition = position.into();

        self.board
            .get(position.get_x() as usize)
            .and_then(|sub_array| sub_array.get(position.get_y() as usize))
            .copied()
            .unwrap_or_default()
    }

    /// Gets the generation of the simulation that this [`BoardDisplay`] is generated from.
    pub fn get_generation(&self) -> u64 {
        self.generation
    }

    /// Gets the area that this [`BoardDisplay`] represents.
    pub fn get_area(&self) -> Area {
        self.display_area
    }
}

#[cfg(test)]
mod board_display_tests {
    use super::*;

    fn generate_board() -> BoardDisplay {
        let mut board_build = Vec::new();
        for _ in 0..5 {
            let mut y_builder = Vec::new();
            for y in 0..6 {
                y_builder.push(if y % 2 == 0 { Cell::Dead } else { Cell::Alive });
            }
            // Convert the vec into the correct type
            let array: Box<[Cell]> = y_builder.into();
            board_build.push(array);
        }

        let area = Area::new((0, 0), (0, 0));

        BoardDisplay::new(0, area, board_build)
    }

    /// The default value is as expected.
    #[test]
    fn default_is_correct() {
        let board_build: Vec<Box<[Cell]>> = Vec::new();
        let area = Area::new((0, 0), (0, 0));
        assert_eq!(
            BoardDisplay::default(),
            BoardDisplay::new(0, area, board_build)
        )
    }

    /// When the internal board is 0 sized a default x of 10 will be returned.
    #[test]
    fn default_x() {
        let get_x = BoardDisplay::default().get_x().get();
        assert_eq!(get_x, 10);
    }

    /// When the internal board is 0 sized a default y of 10 will be returned.
    #[test]
    fn default_y() {
        let get_y = BoardDisplay::default().get_y().get();
        assert_eq!(get_y, 10);
    }

    #[test]
    fn get_x() {
        let generate_board = generate_board();
        assert_eq!(generate_board.get_x().get(), 5);
    }

    #[test]
    fn get_y() {
        let generate_board = generate_board();
        assert_eq!(generate_board.get_y().get(), 6);
    }

    /// An out of bounds cell must return as dead.
    #[test]
    fn dead_out_of_bounds() {
        let cell = BoardDisplay::default().get_cell((2, 2));
        assert_eq!(cell, Cell::Dead)
    }

    /// Get cell must return the expected cell values.
    #[test]
    fn get_cell() {
        // Populate vector with dummy data
        let board_display = generate_board();

        assert_eq!(board_display.get_cell((1, 1)), Cell::Alive);
        assert_eq!(board_display.get_cell((3, 4)), Cell::Dead);
    }

    #[test]
    fn get_generation() {
        let mut board_display = BoardDisplay::default();
        assert_eq!(board_display.generation, 0);

        board_display.generation = 20;
        assert_eq!(board_display.generation, 20);
    }
}

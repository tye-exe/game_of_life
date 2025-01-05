use crate::{
    board_data::{SimulationBlueprint, SimulationSave},
    Area, Cell, GlobalPosition, SharedDisplay,
};

/// An implementation of [`Simulator`] can simulate Conways game of life.
///
/// Each implementation is guaranteed to correctly simulate Conways game of life, however the performance of any
/// implementation is not guaranteed.
pub trait Simulator: Send {
    /// Creates a new simulator.
    fn new(display: SharedDisplay) -> Self
    where
        Self: Sized;

    /// Advances the simulation by one tick.
    fn tick(&mut self);

    /// Updates the board being displayed by the ui.
    fn update_display(&mut self);

    /// Sets the display area sent to the ui to the given area.
    fn set_display_area(&mut self, new_area: Area);

    /// Sets the cell at the given position on the board.
    fn set(&mut self, position: GlobalPosition, cell: Cell);

    /// Gets the cell at the given position on the board.
    fn get(&self, position: GlobalPosition) -> Cell;

    /// Gets the current generation of simulation.
    fn get_generation(&self) -> u64;

    /// Sets the current generation of simulation.
    fn set_generation(&mut self, generation: u64);

    /// Sets all cells on the board to dead & sets the generation to 0.
    fn reset(&mut self);

    /// Gets the area taken up by the current board. The area for a board is a rectangle bounding the alive cells.
    fn get_board_area(&self) -> Area;

    /// Creates a save of the board in its current state.
    fn save_board(&self) -> SimulationSave {
        let board_area = self.get_board_area();

        let mut board_data = bitvec::vec::BitVec::new();
        for position in board_area.iterate_over() {
            board_data.push(self.get(position.into()).into());
        }

        SimulationSave::new(self.get_generation(), board_area, board_data)
    }

    /// Disgards the current state of the board & overwrites it with the given save.
    fn load_board(&mut self, board: SimulationSave) {
        let SimulationSave {
            generation,
            board_area,
            board_data,
        } = board;
        self.reset();

        self.set_generation(generation);
        for (position, cell) in board_area.iterate_over().zip(board_data.into_iter()) {
            self.set(position.into(), cell.into());
        }
    }

    /// Creates a save of the given area of the board.
    fn save_blueprint(&self, area: Area) -> SimulationBlueprint {
        let mut blueprint_data = bitvec::vec::BitVec::new();
        for position in area.iterate_over() {
            blueprint_data.push(self.get(position.into()).into());
        }

        SimulationBlueprint::new(area.x_difference(), area.y_difference(), blueprint_data)
    }

    /// Overwrites an area of the board with the blueprint. The given position is the "top-left" of the blueprint that
    /// will be loaded in.
    fn load_blueprint(&mut self, load_position: GlobalPosition, blueprint: SimulationBlueprint) {
        let SimulationBlueprint {
            x_size,
            y_size,
            blueprint_data,
        } = blueprint;

        let mut area = Area::new((0, 0), (x_size, y_size));
        area.translate_x(load_position.get_x());
        area.translate_y(load_position.get_y());

        for (position, cell) in area.iterate_over().zip(blueprint_data.into_iter()) {
            self.set(position.into(), cell.into());
        }
    }
}

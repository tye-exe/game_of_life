//! A simplistic implementation of [`Simulator`].
//! There is no consideration to performance; Only a Minimum Viable Product.

use std::{
    collections::{HashMap, HashSet},
    ops::AddAssign,
};

use gol_lib::{Area, BoardDisplay, Cell, GlobalPosition, SharedDisplay, Simulator};

/// Represents a board that the cells inhabit.
pub struct Board {
    /// The display data that is for the UI to render.
    display: SharedDisplay,
    /// The area of the board that the UI wants to render.
    display_size_buf: Area,

    /// The generation that this simulation is on.
    generation: u64,
    /// The board that the simulation will take place on
    board: HashSet<GlobalPosition>,
}

impl Simulator for Board {
    fn tick(&mut self) {
        let mut neighbours = HashMap::new();
        let mut to_die = HashSet::new();

        for position in &self.board {
            let position = *position;
            let mut surrounding = 0u8;
            surrounding += self.board.contains(&(position + (1, 1))) as u8;
            surrounding += self.board.contains(&(position + (1, 0))) as u8;
            surrounding += self.board.contains(&(position + (1, -1))) as u8;

            surrounding += self.board.contains(&(position + (0, 1))) as u8;
            surrounding += self.board.contains(&(position + (0, -1))) as u8;

            surrounding += self.board.contains(&(position + (-1, 1))) as u8;
            surrounding += self.board.contains(&(position + (-1, 0))) as u8;
            surrounding += self.board.contains(&(position + (-1, -1))) as u8;

            if surrounding == 0 {
                to_die.insert(position);
            }

            neighbours
                .entry(position + (1, 1))
                .or_insert(0u8)
                .add_assign(1);
            neighbours
                .entry(position + (1, 0))
                .or_insert(0u8)
                .add_assign(1);
            neighbours
                .entry(position + (1, -1))
                .or_insert(0u8)
                .add_assign(1);

            neighbours
                .entry(position + (0, 1))
                .or_insert(0u8)
                .add_assign(1);
            neighbours
                .entry(position + (0, -1))
                .or_insert(0u8)
                .add_assign(1);

            neighbours
                .entry(position + (-1, 1))
                .or_insert(0u8)
                .add_assign(1);
            neighbours
                .entry(position + (-1, 0))
                .or_insert(0u8)
                .add_assign(1);
            neighbours
                .entry(position + (-1, -1))
                .or_insert(0u8)
                .add_assign(1);
        }

        for position in to_die {
            self.board.remove(&position);
        }

        for (position, alive_neighbours) in neighbours {
            match alive_neighbours {
                // Under population
                0 | 1 => {
                    self.board.remove(&position);
                }
                // Nothing happens
                2 => {}
                // Cell if created if non-existing
                3 => {
                    self.board.insert(position);
                }
                // Over population
                _ => {
                    self.board.remove(&position);
                }
            }
        }

        self.generation += 1;
    }

    fn set(&mut self, position: GlobalPosition, cell: Cell) {
        match cell {
            Cell::Alive => {
                self.board.insert(position);
            }
            Cell::Dead => {
                self.board.remove(&position);
            }
        };
    }

    fn get(&self, position: GlobalPosition) -> Cell {
        match self.board.contains(&position) {
            true => Cell::Alive,
            false => Cell::Dead,
        }
    }

    fn update_display(&mut self) {
        // Attempts to acquire the lock on the display.
        // If a lock could not be acquired the method returns early.
        use std::sync::TryLockError;
        let mut display = match self.display.try_lock() {
            Ok(display) => display,
            Err(TryLockError::WouldBlock) => {
                return;
            }
            Err(TryLockError::Poisoned(_)) => {
                core::panic!("Ui panicked!");
            }
        };

        // If the ui has not taken the display return early.
        if display.is_some() {
            return;
        }

        // Get the state of the board within the specified size
        let mut board_build = Vec::new();

        let from = &self.display_size_buf.get_min();
        let to = &self.display_size_buf.get_max();
        for x in from.get_x()..to.get_x() {
            let mut y_builder = Vec::new();
            for y in from.get_y()..to.get_y() {
                y_builder.push(self.get((x, y).into()));
            }
            // Convert the vec into the correct type
            let array: Box<[Cell]> = y_builder.into();
            board_build.push(array);
        }

        // Updates the board to display.
        *display = Some(BoardDisplay::new(
            self.generation,
            self.display_size_buf,
            board_build,
        ));
    }

    fn new(display: SharedDisplay) -> Self {
        Self {
            board: Default::default(),
            display,
            display_size_buf: Default::default(),
            generation: 0,
        }
    }

    fn set_display_area(&mut self, new_area: Area) {
        self.display_size_buf = new_area;
    }

    fn get_generation(&self) -> u64 {
        self.generation
    }

    fn reset(&mut self) {
        self.board = HashSet::new();
        self.generation = 0;
    }

    fn get_board_area(&self) -> Area {
        let top_left = self
            .board
            .iter()
            .fold(GlobalPosition::new(0, 0), |mut corner, position| {
                if position.get_x() > corner.get_x() {
                    corner = GlobalPosition::new(position.get_x(), corner.get_y());
                }
                if position.get_y() > corner.get_y() {
                    corner = GlobalPosition::new(corner.get_x(), position.get_y());
                }

                corner
            });

        let bottom_right =
            self.board
                .iter()
                .fold(GlobalPosition::new(0, 0), |mut corner, position| {
                    if position.get_x() < corner.get_x() {
                        corner = GlobalPosition::new(position.get_x(), corner.get_y());
                    }
                    if position.get_y() < corner.get_y() {
                        corner = GlobalPosition::new(corner.get_x(), position.get_y());
                    }

                    corner
                });

        Area::new(top_left, bottom_right)
    }

    fn set_generation(&mut self, generation: u64) {
        self.generation = generation;
    }
}

#[cfg(test)]
mod tests {
    use bitvec::vec::BitVec;

    use gol_lib::persistence::{SimulationBlueprint, SimulationSave};

    use super::*;

    #[test]
    /// A cell will be dead unless it has been set to alive.
    fn dead_by_default() {
        let board = Board::new(Default::default());
        for x in -10..=10 {
            for y in -10..=10 {
                assert_eq!(board.get((x, y).into()), Cell::Dead)
            }
        }
    }

    #[test]
    /// Sets a cell to be alive.
    fn set_cell_alive() {
        let position = GlobalPosition::new(1, 1);
        let mut board = Board::new(Default::default());

        board.set(position, Cell::Alive);
        assert_eq!(board.get(position), Cell::Alive);
    }

    /// Returns an iterator over [`Cell`], which gives "[`Cell::Alive`], [`Cell::Dead`]" in that order, forever.
    fn generate_cell_iterator() -> std::iter::FromFn<impl FnMut() -> Option<Cell>> {
        let mut generated_cell = Cell::Dead;
        let cell_iter = std::iter::from_fn(move || {
            generated_cell = match generated_cell {
                Cell::Alive => Cell::Dead,
                Cell::Dead => Cell::Alive,
            };
            Some(generated_cell)
        });
        cell_iter
    }

    #[test]
    /// Sets a pattern of alive & dead cells.
    fn set_cell_pattern() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        // Populate board
        let mut cell_iter = generate_cell_iterator();
        for x in -10..=10 {
            for y in -10..=10 {
                let cell = cell_iter.next().unwrap();
                board.set((x, y).into(), cell)
            }
        }

        // Read board
        let mut cell_iter = generate_cell_iterator();
        for x in -10..=10 {
            for y in -10..=10 {
                let cell = cell_iter.next().unwrap();
                let get = board.get((x, y).into());
                assert_eq!(cell, get);
            }
        }
    }

    #[test]
    /// The derived display will correctly represent the board.
    fn generates_correct_display() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());
        let display_area = Area::new((-10, -10), (10, 10));

        // Populate board
        let mut cell_iter = generate_cell_iterator();
        for y in -10..=10 {
            for x in -10..=10 {
                board.set((x, y).into(), cell_iter.next().unwrap());
            }
        }

        // Display init
        board.set_display_area(display_area);
        board.update_display();

        let mut mutex_guard = display.lock().unwrap();
        let take = mutex_guard.take();
        assert!(take.is_some());

        // Generate expected result
        let expected_board = {
            use Cell::{Alive, Dead};
            let mut vec = Vec::new();

            for _ in 0..10 {
                let a: Box<[Cell]> = Box::new([
                    Alive, Dead, Alive, Dead, Alive, Dead, Alive, Dead, Alive, Dead, Alive, Dead,
                    Alive, Dead, Alive, Dead, Alive, Dead, Alive, Dead,
                ]);
                let b = Box::new([
                    Dead, Alive, Dead, Alive, Dead, Alive, Dead, Alive, Dead, Alive, Dead, Alive,
                    Dead, Alive, Dead, Alive, Dead, Alive, Dead, Alive,
                ]);
                vec.push(a);
                vec.push(b);
            }

            vec
        };

        let board_display = BoardDisplay::new(0, display_area, expected_board);
        assert_eq!(board_display, take.unwrap())
    }

    #[test]
    /// reset must remove all alive cells from board & set the generation to 0.
    fn reset() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        // Populate board
        for position in Area::new((-100, -100), (100, 100)).iterate_over() {
            board.set(position, Cell::Alive);
        }

        board.set_generation(100);

        board.reset();

        // Test reset
        for position in Area::new((-100, -100), (100, 100)).iterate_over() {
            assert_eq!(
                board.get(position),
                Cell::Dead,
                "Cell at {position:?} is alive. All cells must be dead after board reset"
            );
        }

        assert_eq!(
            board.get_generation(),
            0,
            "The board generation must be set to zero after a reset."
        );
    }

    #[test]
    /// Generation increases by one each time tick is called.
    fn generation_increases() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        assert_eq!(board.get_generation(), 0);

        for generation in 1..=100 {
            board.tick();
            assert_eq!(
                board.get_generation(),
                generation,
                "Calling tick must incrememnt the generation by one."
            );
        }
    }

    #[test]
    /// An alive cell with no neighbours will die
    fn alive_0_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        board.set((1, 1).into(), Cell::Alive);

        // Tick & test
        board.tick();
        assert_eq!(board.get((1, 1).into()), Cell::Dead);
    }

    #[test]
    /// A dead cell with no neighbours will stay dead
    fn dead_0_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        board.set((1, 1).into(), Cell::Dead);

        // Tick & test
        board.tick();
        assert_eq!(board.get((1, 1).into()), Cell::Dead);
    }

    #[test]
    /// An alive cell with one neighbour will die
    fn alive_1_neighbour() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for position in Area::new((0, 0), (2, 2)).iterate_over() {
            if position == (1, 1).into() {
                continue;
            }

            board.set((1, 1).into(), Cell::Alive);
            board.set(position, Cell::Alive);

            // Tick & test
            board.tick();
            assert_eq!(
                board.get((1, 1).into()),
                Cell::Dead,
                "Cell at (1, 1) must die from one neighbour at {position:?}"
            );
        }
    }

    #[test]
    /// A dead cell with one neighbour will stay dead
    fn dead_1_neighbour() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for position in Area::new((0, 0), (2, 2)).iterate_over() {
            if position == (1, 1).into() {
                continue;
            }

            board.set((1, 1).into(), Cell::Alive);
            board.set(position, Cell::Alive);

            // Tick & test
            board.tick();
            assert_eq!(
                board.get((1, 1).into()),
                Cell::Dead,
                "Cell at (1, 1) must stay dead with neighbour at: {position:?}"
            );
        }
    }

    #[test]
    /// An alive cell with two neighbours will stay alive
    fn alive_2_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for cell_a in Area::new((0, 0), (2, 2)).iterate_over() {
            if cell_a == (1, 1).into() {
                continue;
            }
            for cell_b in Area::new((0, 0), (2, 2)).iterate_over() {
                if cell_b == (1, 1).into() || cell_b == cell_a {
                    continue;
                }

                board.set((1, 1).into(), Cell::Alive);
                board.set(cell_a.into(), Cell::Alive);
                board.set(cell_b.into(), Cell::Alive);

                // Tick & test
                board.tick();
                assert_eq!(
                    board.get((1, 1).into()),
                    Cell::Alive,
                    "Cell at (1, 1) must live from neighbours at: {cell_a:?}, {cell_b:?}"
                );

                // Remove remenatns
                board.reset();
            }
        }
    }

    #[test]
    /// A dead cell with two neighbours will stay dead
    fn dead_2_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for cell_a in Area::new((0, 0), (2, 2)).iterate_over() {
            if cell_a == (1, 1).into() {
                continue;
            }
            for cell_b in Area::new((0, 0), (2, 2)).iterate_over() {
                if cell_b == (1, 1).into() || cell_b == cell_a {
                    continue;
                }

                board.set((1, 1).into(), Cell::Dead);
                board.set(cell_a.into(), Cell::Alive);
                board.set(cell_b.into(), Cell::Alive);

                // Tick & test
                board.tick();
                assert_eq!(
                    board.get((1, 1).into()),
                    Cell::Dead,
                    "Cell at (1, 1) must stay dead with neighbours at: {cell_a:?}, {cell_b:?}"
                );

                // Remove remenatns
                board.reset();
            }
        }
    }

    #[test]
    /// An alive cell with three neighbours will stay alive
    fn alive_3_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for cell_a in Area::new((0, 0), (2, 2)).iterate_over() {
            if cell_a == (1, 1).into() {
                continue;
            }
            for cell_b in Area::new((0, 0), (2, 2)).iterate_over() {
                if cell_b == (1, 1).into() || cell_b == cell_a {
                    continue;
                }
                for cell_c in Area::new((0, 0), (2, 2)).iterate_over() {
                    if cell_c == (1, 1).into() || cell_c == cell_a || cell_c == cell_b {
                        continue;
                    }

                    board.set((1, 1).into(), Cell::Alive);
                    board.set(cell_a.into(), Cell::Alive);
                    board.set(cell_b.into(), Cell::Alive);
                    board.set(cell_c.into(), Cell::Alive);

                    // Tick & test
                    board.tick();
                    assert_eq!(
                        board.get((1, 1).into()),
                        Cell::Alive,
                        "Cell at (1, 1) must live from neighbours at: {cell_a:?}, {cell_b:?}, {cell_c:?}"
                    );

                    // Remove remenatns
                    board.reset();
                }
            }
        }
    }

    #[test]
    /// A dead cell must become alive from three neighbouring cells.
    fn dead_3_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for cell_a in Area::new((0, 0), (2, 2)).iterate_over() {
            if cell_a == (1, 1).into() {
                continue;
            }
            for cell_b in Area::new((0, 0), (2, 2)).iterate_over() {
                if cell_b == (1, 1).into() || cell_b == cell_a {
                    continue;
                }
                for cell_c in Area::new((0, 0), (2, 2)).iterate_over() {
                    if cell_c == (1, 1).into() || cell_c == cell_a || cell_c == cell_b {
                        continue;
                    }

                    board.set((1, 1).into(), Cell::Dead);
                    board.set(cell_a.into(), Cell::Alive);
                    board.set(cell_b.into(), Cell::Alive);
                    board.set(cell_c.into(), Cell::Alive);

                    // Tick & test
                    board.tick();
                    assert_eq!(
                        board.get((1, 1).into()),
                        Cell::Alive,
                        "Cell at (1, 1) must be created from neighbours at: {cell_a:?}, {cell_b:?}, {cell_c:?}"
                    );

                    // Remove remenatns
                    board.reset();
                }
            }
        }
    }

    #[test]
    /// An alive cell with four neighbours will die
    fn alive_4_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for cell_a in Area::new((0, 0), (2, 2)).iterate_over() {
            if cell_a == (1, 1).into() {
                continue;
            }
            for cell_b in Area::new((0, 0), (2, 2)).iterate_over() {
                if cell_b == (1, 1).into() || cell_b == cell_a {
                    continue;
                }
                for cell_c in Area::new((0, 0), (2, 2)).iterate_over() {
                    if cell_c == (1, 1).into() || cell_c == cell_a || cell_c == cell_b {
                        continue;
                    }
                    for cell_d in Area::new((0, 0), (2, 2)).iterate_over() {
                        if cell_d == (1, 1).into()
                            || cell_d == cell_a
                            || cell_d == cell_b
                            || cell_d == cell_c
                        {
                            continue;
                        }

                        board.set((1, 1).into(), Cell::Alive);
                        board.set(cell_a.into(), Cell::Alive);
                        board.set(cell_b.into(), Cell::Alive);
                        board.set(cell_c.into(), Cell::Alive);
                        board.set(cell_d.into(), Cell::Alive);

                        // Tick & test
                        board.tick();
                        assert_eq!(
                            board.get((1, 1).into()),
                            Cell::Dead,
                            "Cell at (1, 1) must die from neighbours at: {cell_a:?}, {cell_b:?}, {cell_c:?}, {cell_d:?}"
                        );

                        // Remove remenatns
                        board.reset();
                    }
                }
            }
        }
    }

    #[test]
    /// An dead cell with four neighbours will die
    fn dead_4_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for cell_a in Area::new((0, 0), (2, 2)).iterate_over() {
            if cell_a == (1, 1).into() {
                continue;
            }
            for cell_b in Area::new((0, 0), (2, 2)).iterate_over() {
                if cell_b == (1, 1).into() || cell_b == cell_a {
                    continue;
                }
                for cell_c in Area::new((0, 0), (2, 2)).iterate_over() {
                    if cell_c == (1, 1).into() || cell_c == cell_a || cell_c == cell_b {
                        continue;
                    }
                    for cell_d in Area::new((0, 0), (2, 2)).iterate_over() {
                        if cell_d == (1, 1).into()
                            || cell_d == cell_a
                            || cell_d == cell_b
                            || cell_d == cell_c
                        {
                            continue;
                        }

                        board.set((1, 1).into(), Cell::Dead);
                        board.set(cell_a.into(), Cell::Alive);
                        board.set(cell_b.into(), Cell::Alive);
                        board.set(cell_c.into(), Cell::Alive);
                        board.set(cell_d.into(), Cell::Alive);

                        // Tick & test
                        board.tick();
                        assert_eq!(
                            board.get((1, 1).into()),
                            Cell::Dead,
                            "Cell at (1, 1) must stay dead with neighbours at: {cell_a:?}, {cell_b:?}, {cell_c:?}, {cell_d:?}"
                        );

                        // Remove remenatns
                        board.reset();
                    }
                }
            }
        }
    }

    #[test]
    /// An alive cell with five neighbours will die
    fn alive_5_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for cell_a in Area::new((0, 0), (2, 2)).iterate_over() {
            if cell_a == (1, 1).into() {
                continue;
            }
            for cell_b in Area::new((0, 0), (2, 2)).iterate_over() {
                if cell_b == (1, 1).into() || cell_b == cell_a {
                    continue;
                }
                for cell_c in Area::new((0, 0), (2, 2)).iterate_over() {
                    if cell_c == (1, 1).into() || cell_c == cell_a || cell_c == cell_b {
                        continue;
                    }

                    // Set alive by default
                    for alive in Area::new((0, 0), (2, 2)).iterate_over() {
                        board.set(alive.into(), Cell::Alive);
                    }

                    board.set(cell_a.into(), Cell::Dead);
                    board.set(cell_b.into(), Cell::Dead);
                    board.set(cell_c.into(), Cell::Dead);

                    // Tick & test
                    board.tick();
                    assert_eq!(
                        board.get((1, 1).into()),
                        Cell::Dead,
                        "Cell at (1, 1) must die from being surrounded by neighbours expect for cells: {cell_a:?}, {cell_b:?}, {cell_c:?}"
                    );

                    // Remove remenatns
                    board.reset();
                }
            }
        }
    }

    #[test]
    /// A dead cell with five neighbours stay dead
    fn dead_5_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for cell_a in Area::new((0, 0), (2, 2)).iterate_over() {
            if cell_a == (1, 1).into() {
                continue;
            }
            for cell_b in Area::new((0, 0), (2, 2)).iterate_over() {
                if cell_b == (1, 1).into() || cell_b == cell_a {
                    continue;
                }
                for cell_c in Area::new((0, 0), (2, 2)).iterate_over() {
                    if cell_c == (1, 1).into() || cell_c == cell_a || cell_c == cell_b {
                        continue;
                    }

                    // Set alive by default
                    for alive in Area::new((0, 0), (2, 2)).iterate_over() {
                        board.set(alive.into(), Cell::Alive);
                    }

                    board.set((1, 1).into(), Cell::Dead);
                    board.set(cell_a.into(), Cell::Dead);
                    board.set(cell_b.into(), Cell::Dead);
                    board.set(cell_c.into(), Cell::Dead);

                    // Tick & test
                    board.tick();
                    assert_eq!(
                        board.get((1, 1).into()),
                        Cell::Dead,
                        "Cell at (1, 1) must stay dead with neighbouring dead cells at: {cell_a:?}, {cell_b:?}, {cell_c:?}"
                    );

                    // Remove remenatns
                    board.reset();
                }
            }
        }
    }

    #[test]
    /// An alive cell with six neighbours will die
    fn alive_6_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for cell_a in Area::new((0, 0), (2, 2)).iterate_over() {
            if cell_a == (1, 1).into() {
                continue;
            }
            for cell_b in Area::new((0, 0), (2, 2)).iterate_over() {
                if cell_b == (1, 1).into() || cell_b == cell_a {
                    continue;
                }

                // Set alive by default
                for alive in Area::new((0, 0), (2, 2)).iterate_over() {
                    board.set(alive.into(), Cell::Alive);
                }

                board.set(cell_a.into(), Cell::Dead);
                board.set(cell_b.into(), Cell::Dead);

                // Tick & test
                board.tick();
                assert_eq!(
                    board.get((1, 1).into()),
                    Cell::Dead,
                    "Cell at (1, 1) must die from being surrounded by neighbours expect for cells: {cell_a:?}, {cell_b:?}"
                );

                // Remove remenatns
                board.reset();
            }
        }
    }

    #[test]
    /// A dead cell with six neighbours will stay dead
    fn dead_6_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for cell_a in Area::new((0, 0), (2, 2)).iterate_over() {
            if cell_a == (1, 1).into() {
                continue;
            }
            for cell_b in Area::new((0, 0), (2, 2)).iterate_over() {
                if cell_b == (1, 1).into() || cell_b == cell_a {
                    continue;
                }

                // Set alive by default
                for alive in Area::new((0, 0), (2, 2)).iterate_over() {
                    board.set(alive.into(), Cell::Alive);
                }

                board.set((1, 1).into(), Cell::Dead);
                board.set(cell_a.into(), Cell::Dead);
                board.set(cell_b.into(), Cell::Dead);

                // Tick & test
                board.tick();
                assert_eq!(
                    board.get((1, 1).into()),
                    Cell::Dead,
                    "Cell at (1, 1) must stay dead with dead neighbouring cells at: {cell_a:?}, {cell_b:?}"
                );

                // Remove remenatns
                board.reset();
            }
        }
    }

    #[test]
    /// An alive cell with seven neighbours will die
    fn alive_7_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for cell_a in Area::new((0, 0), (2, 2)).iterate_over() {
            if cell_a == (1, 1).into() {
                continue;
            }

            // Set alive by default
            for alive in Area::new((0, 0), (2, 2)).iterate_over() {
                board.set(alive.into(), Cell::Alive);
            }

            board.set(cell_a.into(), Cell::Dead);

            // Tick & test
            board.tick();
            assert_eq!(
                board.get((1, 1).into()),
                Cell::Dead,
                "Cell at (1, 1) must die from being surrounded by neighbours expect for cells: {cell_a:?}"
            );

            // Remove remenatns
            board.reset();
        }
    }

    #[test]
    /// A dead cell with seven neighbours will stay dead.
    fn dead_7_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        for cell_a in Area::new((0, 0), (2, 2)).iterate_over() {
            if cell_a == (1, 1).into() {
                continue;
            }

            // Set alive by default
            for alive in Area::new((0, 0), (2, 2)).iterate_over() {
                board.set(alive.into(), Cell::Alive);
            }

            board.set((1, 1).into(), Cell::Dead);
            board.set(cell_a.into(), Cell::Dead);

            // Tick & test
            board.tick();
            assert_eq!(
                board.get((1, 1).into()),
                Cell::Dead,
                "Cell at (1, 1) must stay dead with dead neighbouring cells at: {cell_a:?}"
            );

            // Remove remenatns
            board.reset();
        }
    }

    #[test]
    /// An alive cell with all neighbours will die
    fn alive_8_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        // Set alive by default
        for alive in Area::new((0, 0), (2, 2)).iterate_over() {
            board.set(alive.into(), Cell::Alive);
        }

        // Tick & test
        board.tick();
        assert_eq!(
            board.get((1, 1).into()),
            Cell::Dead,
            "Cell at (1, 1) must die from being fully surrounded by neighbours"
        );

        // Remove remenatns
        board.reset();
    }

    #[test]
    /// An alive cell with all neighbours will die
    fn dead_8_neighbours() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        // Set alive by default
        for alive in Area::new((0, 0), (2, 2)).iterate_over() {
            board.set(alive.into(), Cell::Alive);
        }

        board.set((1, 1).into(), Cell::Dead);

        // Tick & test
        board.tick();
        assert_eq!(
            board.get((1, 1).into()),
            Cell::Dead,
            "Cell at (1, 1) must die from being fully surrounded by neighbours"
        );

        // Remove remenatns
        board.reset();
    }

    #[test]
    /// Correctly simulates the "Block" pattern.
    fn block() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        // Create block pattern
        board.set((1, 1).into(), Cell::Alive);
        board.set((1, 2).into(), Cell::Alive);
        board.set((2, 1).into(), Cell::Alive);
        board.set((2, 2).into(), Cell::Alive);

        // Tick & test
        board.tick();
        for x in 0..4 {
            for y in 0..4 {
                let found = board.get((x, y).into());
                let expected = match (x, y) {
                    (1, 1) | (1, 2) | (2, 1) | (2, 2) => Cell::Alive,
                    _ => Cell::Dead,
                };
                assert_eq!(found, expected)
            }
        }

        // Once again for good measure
        board.tick();
        for x in 0..4 {
            for y in 0..4 {
                let found = board.get((x, y).into());
                let expected = match (x, y) {
                    (1, 1) | (1, 2) | (2, 1) | (2, 2) => Cell::Alive,
                    _ => Cell::Dead,
                };
                assert_eq!(found, expected)
            }
        }
    }

    #[test]
    /// Correctly loads empty board.
    fn load_board_empty() {
        // Generate board with alive cells.
        let mut board = Board::new(Default::default());
        for position in Area::new((-10, -10), (10, 10)).iterate_over() {
            board.set(position, Cell::Alive);
        }

        // Load empty board.
        let generation = 0;
        let area = Area::new((-4, -6), (4, 6));
        let board_data = BitVec::new();
        let simulation_save = SimulationSave::new(generation, area, board_data);
        board.load_board(simulation_save);

        assert_eq!(
            board.get((8, 8).into()),
            Cell::Dead,
            "Cells outside the new board area must be set to dead."
        );

        for position in area.iterate_over() {
            assert_eq!(
                board.get(position),
                Cell::Dead,
                "Cell at {position:?} must be dead as loaded board only contained dead cells."
            )
        }
    }

    #[test]
    /// Correctly loads full board.
    fn load_board_full() {
        let mut board = Board::new(Default::default());

        // Load full board.
        let generation = 0;
        let area = Area::new((-4, -6), (4, 6));
        let mut board_data = BitVec::new();
        for _ in area.iterate_over() {
            board_data.push(Cell::Alive.into());
        }

        let simulation_save = SimulationSave::new(generation, area, board_data);
        board.load_board(simulation_save);

        assert_eq!(
            board.get((8, 8).into()),
            Cell::Dead,
            "Cells outside the new board area must be set to dead."
        );

        for position in area.iterate_over() {
            assert_eq!(
                board.get(position),
                Cell::Alive,
                "Cell at {position:?} must be alive as loaded board only contained alive cells."
            )
        }
    }

    #[test]
    /// Correctly loads mixed board.
    fn load_board_mixed() {
        let mut board = Board::new(Default::default());

        // Load mixed board.
        let generation = 0;
        let area = Area::new((-4, -6), (4, 6));
        let mut board_data = BitVec::new();
        for (_, cell) in area.iterate_over().zip(generate_cell_iterator()) {
            board_data.push(cell.into());
        }

        let simulation_save = SimulationSave::new(generation, area, board_data);
        board.load_board(simulation_save);

        assert_eq!(
            board.get((8, 8).into()),
            Cell::Dead,
            "Cells outside the new board area must be set to dead."
        );

        for (position, cell) in area.iterate_over().zip(generate_cell_iterator()) {
            assert_eq!(
                board.get(position),
                cell,
                "Cell at {position:?} must be {cell:?} as loaded board had this cell in this state."
            )
        }
    }

    #[test]
    /// Correctly saves empty board.
    fn save_board_empty() {
        let board = Board::new(Default::default());

        let generation = 0;
        let board_area = Area::new((0, 0), (0, 0));
        let mut board_data = BitVec::new();
        for _ in board_area.iterate_over() {
            board_data.push(Cell::Dead.into());
        }

        let expected_save = SimulationSave::new(generation, board_area, board_data);
        let save_board = board.save_board();

        assert_eq!(save_board, expected_save);
    }

    #[test]
    /// Correctly saves full board area.
    fn save_board_full_area() {
        let mut board = Board::new(Default::default());
        let board_area = Area::new((-6, -6), (5, 5));

        for position in board_area.iterate_over() {
            board.set(position, Cell::Alive);
        }
        let save_board = board.save_board();

        let generation = 0;
        let mut board_data = BitVec::new();
        for _ in board_area.iterate_over() {
            board_data.push(Cell::Alive.into());
        }
        let expected_save = SimulationSave::new(generation, board_area, board_data);

        assert_eq!(save_board, expected_save);
    }

    #[test]
    /// Correctly saves mixed board area.
    fn save_board_mixed() {
        let mut board = Board::new(Default::default());
        let board_area = Area::new((-6, -6), (5, 5));

        for (position, cell) in board_area.iterate_over().zip(generate_cell_iterator()) {
            board.set(position, cell);
        }
        let save_board = board.save_board();

        let generation = 0;
        let mut board_data = BitVec::new();
        for (position, cell) in board_area.iterate_over().zip(generate_cell_iterator()) {
            // The last tile in each row is cut off due to it being empty.
            // This is intended.
            if position.get_x() == 5 {
                continue;
            }

            board_data.push(cell.into());
        }
        // Compensate for last tile being cut off in each row.
        let board_area = Area::new((-6, -6), (4, 5));
        let expected_save = SimulationSave::new(generation, board_area, board_data);

        assert_eq!(save_board, expected_save);
    }

    #[test]
    /// `set_generation()` correctly sets the generation.
    fn set_generation() {
        let mut board = Board::new(Default::default());
        board.set_generation(100);
        assert_eq!(board.get_generation(), 100);
    }

    #[test]
    /// Getting the board area of a filled area will return the filled area.
    fn get_board_area_full() {
        let mut board = Board::new(Default::default());

        let area = Area::new((-2, -2), (3, 4));
        for position in area.iterate_over() {
            board.set(position, Cell::Alive);
        }

        assert_eq!(board.get_board_area(), area)
    }

    #[test]
    /// The board area will included separate cells.
    fn get_board_area_partial() {
        let mut board = Board::new(Default::default());

        let area = Area::new((0, 0), (4, 4));
        for position in area.iterate_over() {
            board.set(position, Cell::Alive);
        }

        board.set((4, 6).into(), Cell::Alive);

        assert_eq!(board.get_board_area(), Area::new((0, 0), (4, 6)));
    }

    #[test]
    /// An empty blueprint will save correctly.
    fn save_empty_blueprint() {
        let board = Board::new(Default::default());
        let area = Area::new((-2, -2), (3, 3));

        let mut blueprint_data = BitVec::new();
        for _ in area.iterate_over() {
            blueprint_data.push(Cell::Dead.into());
        }

        let expected_blueprint =
            SimulationBlueprint::new(area.x_difference(), area.y_difference(), blueprint_data);

        let save_blueprint = board.save_blueprint(area);

        assert_eq!(expected_blueprint, save_blueprint);
    }

    #[test]
    /// An full blueprint will save correctly.
    fn save_full_blueprint() {
        let mut board = Board::new(Default::default());
        let area = Area::new((-2, -2), (3, 3));

        // Fill the expected blueprint and board with the dummy data.
        let mut blueprint_data = BitVec::new();
        for position in area.iterate_over() {
            board.set(position, Cell::Alive);

            blueprint_data.push(Cell::Alive.into());
        }

        let expected_blueprint =
            SimulationBlueprint::new(area.x_difference(), area.y_difference(), blueprint_data);

        let save_blueprint = board.save_blueprint(area);

        assert_eq!(expected_blueprint, save_blueprint);
    }

    #[test]
    /// A blueprint with mixed data will save correctly.
    fn save_mixed_blueprint() {
        let mut board = Board::new(Default::default());
        let area = Area::new((-2, -2), (3, 3));

        // Fills the expected blueprint and board with the dummy data.
        let mut blueprint_data = BitVec::new();
        for (position, cell) in area.iterate_over().zip(generate_cell_iterator()) {
            board.set(position, cell);

            blueprint_data.push(cell.into());
        }

        let expected_blueprint =
            SimulationBlueprint::new(area.x_difference(), area.y_difference(), blueprint_data);

        let save_blueprint = board.save_blueprint(area);

        assert_eq!(expected_blueprint, save_blueprint);
    }

    #[test]
    /// An empty blueprint must set the cells that it covers to dead.
    fn load_empty_blueprint() {
        let mut board = Board::new(Default::default());

        let board_area = Area::new((-2, -2), (5, 5));
        for position in board_area.iterate_over() {
            board.set(position, Cell::Alive);
        }

        // Construct the blueprint to load.
        let blueprint_area = Area::new((-1, 0), (2, 3));
        let mut blueprint_data = BitVec::new();

        for _ in blueprint_area.iterate_over() {
            blueprint_data.push(Cell::Dead.into());
        }

        let blueprint = SimulationBlueprint::new(
            blueprint_area.x_difference(),
            blueprint_area.y_difference(),
            blueprint_data,
        );

        // Blueprint has not applied yet
        for position in blueprint_area.iterate_over() {
            assert_eq!(
                board.get(position),
                Cell::Alive,
                "The cell at {position:?} must be alive as no blueprint has been loaded."
            );
        }

        board.load_blueprint(blueprint_area.get_min(), blueprint);

        // Test blueprint has applied.
        for position in blueprint_area.iterate_over() {
            assert_eq!(
                board.get(position),
                Cell::Dead,
                "The cell at {position:?} must be dead as the blueprint has been loaded."
            );
        }
    }

    #[test]
    /// An alive blueprint must set the cells that it covers to alive.
    fn load_full_blueprint() {
        let mut board = Board::new(Default::default());

        // Construct the blueprint to load.
        let blueprint_area = Area::new((-1, 0), (2, 3));
        let mut blueprint_data = BitVec::new();

        for _ in blueprint_area.iterate_over() {
            blueprint_data.push(Cell::Alive.into());
        }

        let blueprint = SimulationBlueprint::new(
            blueprint_area.x_difference(),
            blueprint_area.y_difference(),
            blueprint_data,
        );

        // Test dead
        for position in blueprint_area.iterate_over() {
            assert_eq!(
                board.get(position),
                Cell::Dead,
                "The cell at {position:?} must be dead as the blueprint has not been loaded."
            );
        }

        board.load_blueprint(blueprint_area.get_min(), blueprint);

        // Test alive
        for position in blueprint_area.iterate_over() {
            assert_eq!(
                board.get(position),
                Cell::Alive,
                "The cell at {position:?} must be alive as the blueprint has been loaded."
            );
        }
    }

    #[test]
    /// A blueprint of alive and dead cells must correct set the cells.
    fn load_mixed_blueprint() {
        let mut board = Board::new(Default::default());

        // Construct the blueprint to load.
        let blueprint_area = Area::new((-1, 0), (2, 3));
        let blueprint_data: BitVec = generate_cell_iterator()
            .take(blueprint_area.iterate_over().count())
            .map(|cell| -> bool { cell.into() })
            // .into()
            .collect();

        let blueprint = SimulationBlueprint::new(
            blueprint_area.x_difference(),
            blueprint_area.y_difference(),
            blueprint_data.clone(),
        );

        // Test dead
        for position in blueprint_area.iterate_over() {
            assert_eq!(
                board.get(position),
                Cell::Dead,
                "The cell at {position:?} must be dead as the blueprint has not been loaded."
            );
        }

        board.load_blueprint(blueprint_area.get_min(), blueprint);

        // Test after load
        for (position, cell) in blueprint_area.iterate_over().zip(blueprint_data) {
            assert_eq!(
                board.get(position),
                cell.into(),
                "The cell at {position:?} must be alive as the blueprint has been loaded."
            );
        }
    }

    #[test]
    /// A blueprint must not distrurb the surrounding cells.
    fn loaded_blueprint_surroundings() {
        let mut board = Board::new(Default::default());

        // Set area around and in blueprint to be alive.
        // This allows us to see the impact of the empty blueprint.
        let board_area = Area::new((-2, -2), (5, 5));
        for position in board_area.iterate_over() {
            board.set(position, Cell::Alive);
        }

        // Construct the blueprint to load.
        let blueprint_area = Area::new((-1, 0), (2, 3));
        let mut blueprint_data = BitVec::new();

        for _ in blueprint_area.iterate_over() {
            blueprint_data.push(Cell::Dead.into());
        }

        let blueprint = SimulationBlueprint::new(
            blueprint_area.x_difference(),
            blueprint_area.y_difference(),
            blueprint_data,
        );

        // Expected board
        let save_data: BitVec = board_area
            .iterate_over()
            .map(|position| -> bool {
                if blueprint_area.contains(position) {
                    Cell::Dead
                } else {
                    Cell::Alive
                }
                .into()
            })
            .collect();
        let expected_save = SimulationSave::new(0, board_area, save_data);

        // True board data
        board.load_blueprint(blueprint_area.get_min(), blueprint);
        let save_board = board.save_board();

        assert_eq!(
            save_board, expected_save,
            "The blueprint must only apply to the correct area."
        );
    }
}

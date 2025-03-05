use circular_buffer::CircularBuffer;
// use circular_buffer::CircularBuffer;
use gol_lib::{Cell, GlobalPosition, communication::UiPacket};

/// The number of user actions that can be undone.
/// Setting this value to a number under 2 may not work.
const UNDO_BUFFER: usize = 32;

/// Stores a buffer of user actions that can be undone or redone.
#[derive(Default)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct History {
    /// A FIFO Queue of interactions the user has made with the board.
    interactions: CircularBuffer<UNDO_BUFFER, Action>,
    /// The index into the interaction queue the user is at.
    undo_index: usize,
}

impl History {
    /// Adds an action to the history.
    ///
    /// If an action is added whilst a user is in an "undo", then any actions that have been undone will be discarded.
    /// The current action will be set as the most recent action, with any actions had not being undo remaining in history.
    pub(crate) fn add_action(&mut self, action: Action) {
        if self.undo_index != 0 {
            self.interactions
                .truncate_front(UNDO_BUFFER - self.undo_index);
        }

        self.interactions.push_front(action);
        self.undo_index = 0;
    }

    /// Returns a vector of [`UiPacket`]s that need to be sent to undo a previous action.
    /// Subsequent calls of this method will give packets for previous actions.
    ///
    /// If an empty vector is returned, then there are no more actions stored to undo.
    pub(crate) fn undo(&mut self) -> Vec<UiPacket> {
        if let Some(action) = self.interactions.get(self.undo_index) {
            self.undo_index += 1;
            return action.undo();
        }

        Vec::new()
    }

    /// Returns a vector of [`uiPacket`]s that need to be sent to redo an undone action.
    /// Subsequent calls to this method will give the packets for previous undo actions.
    ///
    /// If an empty vector is returned, then there are no more actions to redo.
    ///
    /// If [`Self::add_action()`] is called whilst there are actions to redo, then these actions will be discarded.
    pub(crate) fn redo(&mut self) -> Vec<UiPacket> {
        if self.undo_index == 0 {
            return Vec::new();
        }

        if let Some(action) = self.interactions.get(self.undo_index - 1) {
            if self.undo_index != 0 {
                self.undo_index -= 1;
            }
            return action.redo();
        }

        Vec::new()
    }

    /// Empties the action queue.
    pub(crate) fn clear(&mut self) {
        self.interactions.clear();
        self.undo_index = 0;
    }

    /// Returns true if there are actions that can be undone.
    pub(crate) fn can_undo(&self) -> bool {
        self.interactions.get(self.undo_index).is_some()
    }

    /// Returns true if there are actions can that can redone.
    pub(crate) fn can_redo(&self) -> bool {
        if self.undo_index == 0 {
            return false;
        }

        self.interactions.get(self.undo_index - 1).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a new [`History`] struct with a filled action buffer (see method for more details).
    fn fill_buffer() -> History {
        let mut history = History::default();

        for index in 0..UNDO_BUFFER {
            history.add_action(Action::SetAlive {
                position: (0, index as i32).into(),
            });
        }

        assert!(history.interactions.is_full());
        history
    }

    /// Undo returns the correct actions in the correct order.
    #[test]
    fn undo() {
        let mut history = fill_buffer();

        for index in (0..UNDO_BUFFER).rev() {
            let undo = history.undo();
            let ui_packet = undo.get(0).unwrap();

            let expected_packet = &UiPacket::Set {
                position: (0, index as i32).into(),
                cell_state: Cell::Dead,
            };

            assert_eq!(ui_packet, expected_packet);
        }

        assert!(history.undo().is_empty());
    }

    /// Redo returns the correct actions in the correct order.
    #[test]
    fn redo() {
        let mut history = fill_buffer();

        // Undo the entire buffer
        for _ in 0..UNDO_BUFFER {
            let _ = history.undo();
        }

        // Will start at 0 as the most recent undo is for 0
        for index in 0..UNDO_BUFFER {
            let redo = history.redo();
            let ui_packet = redo.get(0).unwrap();

            let expected_packet = &UiPacket::Set {
                position: (0, index as i32).into(),
                cell_state: Cell::Alive,
            };

            assert_eq!(ui_packet, expected_packet);
        }

        // All actions have been redone
        assert!(history.redo().is_empty());
    }

    /// Calling redo at the start of the queue must return an empty vec
    #[test]
    fn excessive_redo() {
        let mut history = fill_buffer();

        // One undo
        assert_eq!(
            history.undo().get(0).unwrap(),
            &UiPacket::Set {
                position: (0, UNDO_BUFFER as i32 - 1).into(),
                cell_state: Cell::Dead,
            }
        );

        // One redo
        assert_eq!(
            history.redo().get(0).unwrap(),
            &UiPacket::Set {
                position: (0, UNDO_BUFFER as i32 - 1).into(),
                cell_state: Cell::Alive,
            }
        );

        // Further redo's will return nothing
        assert_eq!(history.redo(), Vec::new());
    }

    /// Calling undo at the end of the queue must return an empty vec
    #[test]
    fn excessive_undo() {
        let mut history = fill_buffer();

        for _ in 0..UNDO_BUFFER {
            assert!(!history.undo().is_empty());
        }

        // Attempt calling more undo's when all have been done
        assert!(history.undo().is_empty());
        assert!(history.undo().is_empty());

        assert_eq!(
            history.undo_index, UNDO_BUFFER,
            "The undo index should be constrained to {UNDO_BUFFER}"
        );

        assert_eq!(
            history.redo(),
            vec![UiPacket::Set {
                position: (0, 0).into(),
                cell_state: Cell::Alive
            }]
        )
    }

    /// Editing from an undo should remove the previous actions and set this action to the first one.
    #[test]
    fn edit_from_undo() {
        let mut history = fill_buffer();

        assert_eq!(
            history.undo(),
            vec![UiPacket::Set {
                position: (0, UNDO_BUFFER as i32 - 1).into(),
                cell_state: Cell::Dead
            }]
        );
        assert_eq!(
            history.undo(),
            vec![UiPacket::Set {
                position: (0, UNDO_BUFFER as i32 - 2).into(),
                cell_state: Cell::Dead
            }]
        );

        // Perform an action
        history.add_action(Action::SetAlive {
            position: (1, 1).into(),
        });

        // At the start of queue so redo will have no effect
        assert_eq!(history.redo(), vec![]);

        // First undo is new action
        assert_eq!(
            history.undo(),
            vec![UiPacket::Set {
                position: (1, 1).into(),
                cell_state: Cell::Dead
            }]
        );
        // Second undo continues from original actions
        assert_eq!(
            history.undo(),
            vec![UiPacket::Set {
                position: (0, UNDO_BUFFER as i32 - 3).into(),
                cell_state: Cell::Dead
            }]
        );
    }

    /// [`History`] after clear must be identical to a new history.
    #[test]
    fn clear() {
        let mut history = fill_buffer();

        history.clear();

        assert_eq!(history, History::default());
    }

    /// Can undo must only be true when there are actions to undo.
    #[test]
    fn can_undo() {
        let mut history = fill_buffer();

        assert!(history.can_undo(), "There are actions that can be undone");

        for _ in 0..UNDO_BUFFER {
            let _ = history.undo();
        }

        assert!(!history.can_undo(), "There is no more actions to undo");
    }

    /// Can redo must only be true when there are arctions to redo.
    #[test]
    fn can_redo() {
        let mut history = fill_buffer();

        assert!(!history.can_redo(), "There are no actions to redo");

        let _ = history.undo();

        assert!(history.can_redo(), "There are actions that can be redone");
    }
}

/// The possible actions the user can perform to edit the board.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) enum Action {
    SetAlive { position: GlobalPosition },
    SetDead { position: GlobalPosition },
}

impl Action {
    /// Returns the correct [`Action`] for setting a [`Cell`] to the given state.
    pub fn set(position: GlobalPosition, cell_state: Cell) -> Self {
        match cell_state {
            Cell::Dead => Action::SetDead { position },
            Cell::Alive => Action::SetAlive { position },
        }
    }

    /// The packets that need to be sent to the board to undo an action.
    pub fn undo(&self) -> Vec<UiPacket> {
        match self {
            Action::SetAlive { position } => vec![UiPacket::Set {
                position: *position,
                cell_state: gol_lib::Cell::Dead,
            }],
            Action::SetDead { position } => vec![UiPacket::Set {
                position: *position,
                cell_state: gol_lib::Cell::Alive,
            }],
        }
    }

    /// The packets that need to be sent to the board to redo an action.
    pub fn redo(&self) -> Vec<UiPacket> {
        match self {
            Action::SetAlive { position } => vec![UiPacket::Set {
                position: *position,
                cell_state: gol_lib::Cell::Alive,
            }],
            Action::SetDead { position } => vec![UiPacket::Set {
                position: *position,
                cell_state: gol_lib::Cell::Dead,
            }],
        }
    }
}

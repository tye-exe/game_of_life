use egui::{InputState, Key, KeyboardShortcut, Modifiers};
use egui_keybind::{Bind, Shortcut};
use enum_iterator::Sequence;

/// The keybind identifiers that this application is listening for.
#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq, Clone, Copy, Sequence)]
pub(crate) enum Keybind {
    SettingsMenu,
    StartSimulation,
    StopSimulation,
    LoadBoard,
    LoadBlueprint,
    SaveBoard,
    SaveBlueprint,
}

/// A combination of a keybind identifier and the input data needed to trigger the keybind.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct KeybindData {
    /// Keybind identifier
    keybind: Keybind,
    /// Trigger
    shortcut: Shortcut,
}

/// Holds the collection of [`Keybind`]s and their corresponding [`Shortcut`]s.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct KeybindHolder {
    /// The keybinds this application is listening for.
    ///
    /// It is guaranteed that each keybind will have one and exactly one entry in this vector.
    keybinds: Vec<KeybindData>,
}

impl PartialEq for KeybindData {
    fn eq(&self, other: &Self) -> bool {
        self.keybind == other.keybind
    }
}

impl PartialEq<Keybind> for KeybindData {
    fn eq(&self, other: &Keybind) -> bool {
        self.keybind == *other
    }
}

impl PartialEq<Keybind> for &KeybindData {
    fn eq(&self, other: &Keybind) -> bool {
        self.keybind == *other
    }
}

impl PartialEq<Keybind> for &mut KeybindData {
    fn eq(&self, other: &Keybind) -> bool {
        self.keybind == *other
    }
}

impl Keybind {
    pub(crate) fn get_default(self) -> KeybindData {
        let shortcut = match self {
            Keybind::SettingsMenu => {
                Shortcut::new(Some(KeyboardShortcut::new(Modifiers::CTRL, Key::P)), None)
            }
            Keybind::StartSimulation => {
                Shortcut::new(Some(KeyboardShortcut::new(Modifiers::NONE, Key::E)), None)
            }
            Keybind::StopSimulation => {
                Shortcut::new(Some(KeyboardShortcut::new(Modifiers::CTRL, Key::E)), None)
            }
            Keybind::LoadBoard => {
                Shortcut::new(Some(KeyboardShortcut::new(Modifiers::CTRL, Key::O)), None)
            }
            Keybind::LoadBlueprint => Shortcut::new(
                Some(KeyboardShortcut::new(
                    Modifiers::CTRL | Modifiers::SHIFT,
                    Key::O,
                )),
                None,
            ),
            Keybind::SaveBoard => {
                Shortcut::new(Some(KeyboardShortcut::new(Modifiers::CTRL, Key::S)), None)
            }
            Keybind::SaveBlueprint => Shortcut::new(
                Some(KeyboardShortcut::new(
                    Modifiers::CTRL | Modifiers::SHIFT,
                    Key::S,
                )),
                None,
            ),
        };

        KeybindData {
            keybind: self,
            shortcut,
        }
    }
}

impl Default for KeybindHolder {
    fn default() -> Self {
        Self {
            keybinds: enum_iterator::all::<Keybind>()
                .map(|keybind| keybind.get_default())
                .collect(),
        }
    }
}

impl KeybindHolder {
    /// Returns an instance of a [`Shortcut`] that will trigger the given [`Keybind`].
    pub(crate) fn get_shortcut(&self, keybind: Keybind) -> Shortcut {
        self.keybinds
            .iter()
            .filter_map(|data| {
                if data != keybind {
                    return None;
                }

                Some(data.shortcut)
            })
            .next()
            .expect("All keybinds are guaranteed to be in the vector.")
    }

    /// Returns a mutable reference to the [`Shortcut`] that the given [`Keybind`] will be triggered by.
    pub(crate) fn get_shortcut_mut(&mut self, keybind: Keybind) -> &mut Shortcut {
        self.keybinds
            .iter_mut()
            .filter_map(|data| {
                if data != keybind {
                    return None;
                }

                Some(&mut data.shortcut)
            })
            .next()
            .expect("All keybinds are guaranteed to be in the vector.")
    }

    /// Resets the [`Shortcut`] of the given [`Keybind`] to its default value.
    pub(crate) fn reset(&mut self, keybind: Keybind) {
        self.keybinds
            .iter_mut()
            .filter(|data| *data == keybind)
            .for_each(|data| *data = keybind.get_default());
    }

    /// Returns an iterator of [`Keybind`]s that have been triggered by the given [`InputState`].
    ///
    /// If a keybind has been pressed, then it consumes the input keys. This prevents any other matches.
    /// Due to this keybinds are checked via the most complex keybinds (ones with the most keys) first.
    pub(crate) fn pressed(
        &mut self,
        input_state: &mut InputState,
    ) -> impl Iterator<Item = Keybind> {
        // Ensure that the keybinds are in the correct order.
        self.sort();

        self.keybinds
            .iter()
            .filter_map(|data| data.shortcut.pressed(input_state).then_some(data.keybind))
    }

    /// Sorts the internal keybind data structure, such that more complex keybinds are towards the front of the vector.
    fn sort(&mut self) {
        // Not unstable so that keybind order remains consistent.
        self.keybinds.sort_by_key(|data| {
            let mut weight = u8::MAX;

            let keyboard_shortcut = match data.shortcut.keyboard() {
                Some(var) => var,
                None => return weight,
            };

            if keyboard_shortcut.modifiers.alt {
                weight -= 1;
            }
            if keyboard_shortcut.modifiers.shift {
                weight -= 1;
            }
            // Use command attribute rather than "ctrl" or "mac_cmd" to eliminate for platform independence.
            if keyboard_shortcut.modifiers.command {
                weight -= 1;
            }

            weight
        });
    }
}

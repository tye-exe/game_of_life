use crate::GlobalPosition;

/// Contains the data for the two opposite corners of a rectangle.
/// One corner will have the minimum x and minimum y values, the other will have the maximum x and maximum y values.
#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, Hash)]
#[cfg_attr(any(test, debug_assertions), derive(Debug))]
pub struct Area {
    /// The min x & the min y position.
    min: GlobalPosition,
    /// The max x & the max y position.
    max: GlobalPosition,
}

impl Default for Area {
    /// Constructs a new [`Area`], with zero size.
    fn default() -> Self {
        Self::new((0, 0), (0, 0))
    }
}

impl Area {
    /// Constructs a new [`Area`] covering from the small x & y to the large x & y.
    /// The positions passed into this method will be sorted into the minimum and maximum corners.
    ///
    /// # Examples
    /// Using i32 tuples to create [`GlobalPosition`]s:
    /// ```
    /// # use gol_lib::Area;
    /// let area = Area::new((1, 4), (10, -6));
    /// // Notice how they are sorted into the max & min corners.
    /// assert_eq!(area.get_min(), (1, -6).into());
    /// assert_eq!(area.get_max(), (10, 4).into());
    /// ```
    pub fn new(pos1: impl Into<GlobalPosition>, pos2: impl Into<GlobalPosition>) -> Self {
        let pos1 = pos1.into();
        let pos2 = pos2.into();

        // Construct from with the smallest x & y
        let min = GlobalPosition {
            x: pos1.get_x().min(pos2.get_x()),
            y: pos1.get_y().min(pos2.get_y()),
        };
        // Construct to with the biggest x & y
        let max = GlobalPosition {
            x: pos1.get_x().max(pos2.get_x()),
            y: pos1.get_y().max(pos2.get_y()),
        };

        Self { min, max }
    }

    /// Gets the minimum x & minimum y of the area.
    pub fn get_min(&self) -> GlobalPosition {
        self.min
    }

    /// Gets the maximum x & biggest y of the area.
    pub fn get_max(&self) -> GlobalPosition {
        self.max
    }

    /// A range from the minimum x to the maximum x (inclusive).
    pub fn x_range(&self) -> std::ops::RangeInclusive<i32> {
        self.get_min().get_x()..=self.get_max().get_x()
    }

    /// A range from the minimum y to the maximum y (inclusive).
    pub fn y_range(&self) -> std::ops::RangeInclusive<i32> {
        self.get_min().get_y()..=self.get_max().get_y()
    }

    /// Gets size of this area in the x axis.
    #[doc(alias = "x_size")]
    pub fn x_difference(&self) -> i32 {
        self.max.x - self.min.x
    }

    /// Gets the size of this area in the y axis.
    #[doc(alias = "y_size")]
    pub fn y_difference(&self) -> i32 {
        self.max.y - self.min.y
    }

    /// Returns an iterator that iterates over all the x & y positions within this area as [`GlobalPosition`]s.
    ///
    /// # Examples
    /// ```rust
    /// # use gol_lib::Area;
    /// let area = Area::new((1, 1), (2, 2));
    /// let mut iterate_over = area.iterate_over();
    ///
    /// // A (i32, i32) can be converted into a GlobalPosition with .into()
    /// assert_eq!(iterate_over.next().unwrap(), (1, 1).into());
    /// assert_eq!(iterate_over.next().unwrap(), (2, 1).into());
    /// assert_eq!(iterate_over.next().unwrap(), (1, 2).into());
    /// assert_eq!(iterate_over.next().unwrap(), (2, 2).into());
    /// assert!(iterate_over.next().is_none());
    /// ```
    ///
    /// When the difference between min x/y & max x/y is 0, the tiles at that axis will still be iterated over.
    /// ```rust
    /// # use gol_lib::Area;
    /// let area = Area::new((1, 1), (1, 1));
    /// let mut iterate_over = area.iterate_over();
    ///
    /// assert_eq!(iterate_over.next().unwrap(), (1, 1).into());
    /// assert!(iterate_over.next().is_none());
    /// ```
    pub fn iterate_over(&self) -> impl Iterator<Item = GlobalPosition> + use<> {
        let GlobalPosition { x: min_x, y: min_y } = self.get_min();
        let GlobalPosition { x: max_x, y: max_y } = self.get_max();

        let mut x_pos = min_x - 1;
        let mut y_pos = min_y;
        std::iter::from_fn(move || {
            x_pos += 1;

            if x_pos > max_x {
                x_pos = min_x;
                y_pos += 1;
            }

            if y_pos > max_y {
                return None;
            }

            Some(GlobalPosition::new(x_pos, y_pos))
        })
    }

    /// Moves the area in the x axis by the given value.
    pub fn translate_x(&mut self, move_by: i32) {
        self.min.x += move_by;
        self.max.x += move_by;
    }

    /// Moves the area in the y axis by the given value.
    pub fn translate_y(&mut self, move_by: i32) {
        self.min.y += move_by;
        self.max.y += move_by;
    }

    /// Modifies the area via increasing/decreasing the maximum x position by the given amount.
    ///
    /// If the modified x would be lower than the minimum x, it will instead be set to the minimum x value.
    pub fn modify_x(&mut self, x_change: i32) {
        self.max.x = self.min.x.max(self.max.x + x_change);
    }

    /// Modifies the area via increasing/decreasing the maximum y position by the given amount.
    ///
    /// If the modified y would be lower than the minimum y, it will instead be set to the minimum y value.
    pub fn modify_y(&mut self, y_change: i32) {
        self.max.y = self.min.y.max(self.max.y + y_change)
    }
}

#[cfg(test)]
pub(crate) mod area_tests {
    use super::*;

    #[test]
    /// Tests that the fields within the area struct are correctly sorted into the smallest x & y and into the
    /// largest x & y respectively.
    fn from_lower_to_higher() {
        let area = Area::new((10, 5), (5, 10));

        assert_eq!(area.get_min(), (5, 5).into());
        assert_eq!(area.get_max(), (10, 10).into());
    }

    #[test]
    /// The iterate over method will increase x then y.
    fn iterate_over_positive() {
        let area = Area::new((2, 2), (4, 4));

        let mut iterate_over = area.iterate_over();
        assert_eq!(iterate_over.next().unwrap(), (2, 2).into());
        assert_eq!(iterate_over.next().unwrap(), (3, 2).into());
        assert_eq!(iterate_over.next().unwrap(), (4, 2).into());
        assert_eq!(iterate_over.next().unwrap(), (2, 3).into());
        assert_eq!(iterate_over.next().unwrap(), (3, 3).into());
        assert_eq!(iterate_over.next().unwrap(), (4, 3).into());
        assert_eq!(iterate_over.next().unwrap(), (2, 4).into());
        assert_eq!(iterate_over.next().unwrap(), (3, 4).into());
        assert_eq!(iterate_over.next().unwrap(), (4, 4).into());
        assert!(iterate_over.next().is_none());
    }

    #[test]
    /// The iterate over method can handle mixed bounds.
    fn iterate_over_mixed() {
        let area = Area::new((-1, -2), (1, 0));

        let mut iterate_over = area.iterate_over();
        assert_eq!(iterate_over.next().unwrap(), (-1, -2).into());
        assert_eq!(iterate_over.next().unwrap(), (0, -2).into());
        assert_eq!(iterate_over.next().unwrap(), (1, -2).into());
        assert_eq!(iterate_over.next().unwrap(), (-1, -1).into());
        assert_eq!(iterate_over.next().unwrap(), (0, -1).into());
        assert_eq!(iterate_over.next().unwrap(), (1, -1).into());
        assert_eq!(iterate_over.next().unwrap(), (-1, 0).into());
        assert_eq!(iterate_over.next().unwrap(), (0, 0).into());
        assert_eq!(iterate_over.next().unwrap(), (1, 0).into());
        assert!(iterate_over.next().is_none());
    }

    #[test]
    /// The smallestet area (zero difference between min & max) still represents one tile.
    fn area_cannot_be_zero_sized() {
        // Test positive area.
        let positive_area = Area::new((0, 0), (0, 0));
        let mut iterate_over = positive_area.iterate_over();
        assert_eq!(iterate_over.next().unwrap(), (0, 0).into());
        assert!(iterate_over.next().is_none());

        // Test negative area.
        let negative_area = Area::new((-1, -1), (-1, -1));
        let mut iterate_over = negative_area.iterate_over();
        assert_eq!(iterate_over.next().unwrap(), (-1, -1).into());
        assert!(iterate_over.next().is_none());
    }

    #[test]
    /// Modifying the area caps at a x & y difference of 0.
    /// You cannot have a negative difference.
    fn modify_keeps_invariant() {
        let mut area = Area::new((1, 1), (4, 4));

        area.modify_x(-10);
        assert_eq!(area, Area::new((1, 1), (1, 4)));

        area.modify_y(-10);
        assert_eq!(area, Area::new((1, 1), (1, 1)));
    }

    #[test]
    /// Modify will expand the corresponding maximum position.
    fn modify_expands_area() {
        let mut area = Area::new((1, 1), (4, 4));

        area.modify_x(10);
        assert_eq!(area, Area::new((1, 1), (14, 4)));

        area.modify_y(10);
        assert_eq!(area, Area::new((1, 1), (14, 14)));
    }
}

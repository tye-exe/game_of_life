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
    pub fn x_difference(&self) -> u32 {
        debug_assert!(
            (self.max.x - self.min.x) >= 0,
            "The x difference of any area must always be positive."
        );
        (self.max.x - self.min.x) as u32
    }

    /// Gets the size of this area in the y axis.
    #[doc(alias = "y_size")]
    pub fn y_difference(&self) -> u32 {
        debug_assert!(
            (self.max.y - self.min.y) >= 0,
            "The y difference of any area must always be positive."
        );
        (self.max.y - self.min.y) as u32
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

    /// Modifies the minimum x & y by the given values, with the first representing x and the latter y.
    /// If x or y value would exceed the maximum value after the change, it will be clamped to the value of maximums values.
    pub fn modify_min(&mut self, min_change: (i32, i32)) {
        self.min.x += min_change.0;
        self.min.y += min_change.1;

        // Ensure that the min cannot be larger than the max.
        self.min.x = self.min.x.min(self.max.x);
        self.min.y = self.min.y.min(self.max.y);
    }

    /// Modifies the maximum x & y by the given values, with the first representing x and the latter y.
    /// If x or y would decrease bellow the minimum value after the change, it will be clamped to the value of the minimum values.
    pub fn modify_max(&mut self, max_change: (i32, i32)) {
        self.max.x += max_change.0;
        self.max.y += max_change.1;

        // Ensure that max cannot be smaller than min.
        self.max.x = self.max.x.max(self.min.x);
        self.max.y = self.max.y.max(self.min.y);
    }

    /// Returns true if the given position is a location inside of this area.
    ///
    /// All positions produced via [`Self::iterate_over`] will return true for this method.
    pub fn contains(&self, position: impl Into<GlobalPosition>) -> bool {
        let position: GlobalPosition = position.into();

        position.get_x() >= self.get_min().get_x()
            && position.get_x() <= self.get_max().get_x()
            && position.get_y() >= self.get_min().get_y()
            && position.get_y() <= self.get_max().get_y()
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
    /// The smallest area (zero difference between min & max) still represents one tile.
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
    /// Modifying the minimum behaves as expected.
    fn modify_min() {
        let mut area = Area::new((0, 0), (10, 10));

        // Positive modification
        area.modify_min((1, 1));
        assert_eq!(area, Area::new((1, 1), (10, 10)));

        // Negative modification
        area.modify_min((-2, -2));
        assert_eq!(area, Area::new((-1, -1), (10, 10)));
    }

    #[test]
    /// Minimum values cannot be changed to larger than the maximum values.
    fn modify_min_excessive() {
        let mut area = Area::new((0, 0), (10, 10));
        area.modify_min((19, 19));
        assert_eq!(
            area,
            Area::new((10, 10), (10, 10)),
            "The minimum value cannot be larger than the maximum value."
        );
    }

    #[test]
    /// Modifying the maximum behaves as expected.
    fn modify_max() {
        let mut area = Area::new((0, 0), (10, 10));

        area.modify_max((1, 1));
        assert_eq!(area, Area::new((0, 0), (11, 11)));

        area.modify_max((-2, -2));
        assert_eq!(area, Area::new((0, 0), (9, 9)));
    }

    #[test]
    /// Maximum values cannot be changed to larger than the minimum values.
    fn modify_max_excessive() {
        let mut area = Area::new((0, 0), (10, 10));
        area.modify_max((-19, -19));
        assert_eq!(
            area,
            Area::new((0, 0), (0, 0)),
            "The maximum value cannot be smaller than the minimum value."
        );
    }

    #[test]
    /// A position in the middle area of an area will be contained by it.
    fn contains_in_middle() {
        let area = Area::new((0, 0), (10, 10));
        assert!(
            area.contains((5, 5)),
            "This position is within the area bounds"
        );
    }

    #[test]
    /// The positions on the edges of an area will be contained by it.
    fn contains_borders() {
        let area = Area::new((0, 0), (10, 10));
        let x_max = (10, 5);
        let x_min = (0, 5);
        let y_max = (5, 10);
        let y_min = (5, 0);

        assert!(area.contains(x_max), "{x_max:?} is within the area bounds");
        assert!(area.contains(x_min), "{x_max:?} is within the area bounds");
        assert!(area.contains(y_max), "{x_max:?} is within the area bounds");
        assert!(area.contains(y_min), "{x_max:?} is within the area bounds");
    }

    #[test]
    /// The positions outside the area will not be contained by it.
    fn contains_out_of_area() {
        let area = Area::new((0, 0), (10, 10));
        let x_max_over = (11, 5);
        let x_min_over = (-1, 5);
        let y_max_over = (5, 11);
        let y_min_over = (5, -1);

        assert!(
            !area.contains(x_max_over),
            "{x_max_over:?} is not within the area bounds"
        );
        assert!(
            !area.contains(x_min_over),
            "{x_max_over:?} is not within the area bounds"
        );
        assert!(
            !area.contains(y_max_over),
            "{x_max_over:?} is not within the area bounds"
        );
        assert!(
            !area.contains(y_min_over),
            "{x_max_over:?} is not within the area bounds"
        );
    }

    #[test]
    /// The entire area iterated over must be contained by the area.
    fn contains_all_from_iter() {
        let area = Area::new((-3, -3), (6, 6));

        for position in area.iterate_over() {
            assert!(
                area.contains(position),
                "Area must contain '{position:?}', as it is iterated over."
            );
        }
    }
}

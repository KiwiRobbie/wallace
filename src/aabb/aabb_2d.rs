#[derive(Debug, Clone, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Aabb2D {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Aabb2D {
    pub fn contains(&self, p: &Point2D) -> bool {
        return self.min_x < p.x && self.min_y < p.y && self.max_x > p.x && self.max_y > p.y;
    }

    pub fn to_array(&self) -> [[f32; 2]; 2] {
        return [[self.min_x, self.min_y], [self.max_x, self.max_y]];
    }

    pub fn area(&self) -> f32 {
        return (self.max_x - self.min_x) * (self.max_y - self.min_y);
    }

    pub fn cmp(&self, other: &Self) -> Aabb2CmpResult {
        let delta_min_x = self.min_x - other.min_x;
        let delta_min_y = self.min_y - other.min_y;
        let delta_max_x = self.max_x - other.max_x;
        let delta_max_y = self.max_y - other.max_y;

        if delta_min_x <= 0.0 && delta_min_y <= 0.0 && delta_max_x >= 0.0 && delta_max_y >= 0.0 {
            return Aabb2CmpResult::Superset;
        }
        if delta_min_x >= 0.0 && delta_min_y >= 0.0 && delta_max_x <= 0.0 && delta_max_y <= 0.0 {
            return Aabb2CmpResult::Subset;
        }
        Aabb2CmpResult::None
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        return (self.min_x < other.max_x || other.min_x < self.max_x)
            && (self.min_y < other.max_y || other.min_y < self.max_y);
    }

    pub fn cut(&self, other: &Self) -> Option<(Aabb2D, Aabb2D)> {
        let self_array = self.to_array();
        let mut other_array = other.to_array();

        for fixed_axis in 0..2 {
            for dir in 0..2 {
                //   ^
                //   | Fixed axis
                //  _____
                //-|-----|---> cut axis
                // |Other|

                let cut_axis = 1 - fixed_axis;

                let fixed_value = self_array[dir][fixed_axis];

                let cut_start_value = self_array[0][cut_axis];
                let cut_end_value = self_array[1][cut_axis];

                // Fixed coordinate of cut inside rectangle
                if other_array[0][fixed_axis] < fixed_value
                    && fixed_value < other_array[1][fixed_axis]
                {
                    // Cut cut ends after rectangle start and starts before rectangle end
                    if other_array[0][cut_axis] < cut_end_value
                        && cut_start_value < other_array[1][cut_axis]
                    {
                        // Cut line does cut through rectangle
                        // println!(
                        //     "cutting on axis={} dir={} value={}",
                        //     fixed_axis, dir, fixed_value
                        // );

                        // new is chosen to be the portion after,
                        // will be disjoint from self so we don't need to check in future.
                        let mut new_array = other_array.clone();
                        new_array[1 - dir][fixed_axis] = fixed_value;

                        // other is portion before cut,
                        // may still overlap with self
                        other_array[dir][fixed_axis] = fixed_value;

                        return Some((other_array.into(), new_array.into()));
                    }
                }
            }
        }
        return None;
    }

    pub fn union(&self, other: &Self) -> Vec<Self> {
        match self.cmp(other) {
            Aabb2CmpResult::Superset => return vec![self.clone()],
            Aabb2CmpResult::Subset => return vec![other.clone()],
            Aabb2CmpResult::None => {}
        };

        // check for possible clean and reorder
        let (a, b) = if {
            let a_array = self.to_array();

            let count: usize = (0usize..4usize)
                .map(|i| {
                    let x_index = i.rem_euclid(2);
                    let y_index = i.div_euclid(2);

                    let x = a_array[x_index][0];
                    let y = a_array[y_index][1];

                    other.contains(&Point2D { x, y }) as usize
                })
                .sum();
            count == 2
        } {
            (other, self)
        } else {
            (self, other)
        };

        // Try to cut b with edges of a
        if let Some((b, c)) = a.cut(b) {
            // Check if result rectangle before the cut is no longer needed
            match a.cmp(&b) {
                Aabb2CmpResult::Superset => return vec![a.clone(), c],
                Aabb2CmpResult::Subset => return vec![b, c],
                Aabb2CmpResult::None => {}
            };

            // Since neither a/b was superset, cutting b again will produce one external piece, and one subset of a
            if let Some((_, b)) = a.cut(&b) {
                return vec![a.clone(), b, c];
            }
        }

        // Rectangles must have being disjoint
        return vec![a.clone(), b.clone()];
    }
}

pub enum Aabb2CmpResult {
    Superset,
    Subset,
    None,
}

impl From<[[f32; 2]; 2]> for Aabb2D {
    fn from(value: [[f32; 2]; 2]) -> Self {
        return Aabb2D {
            min_x: value[0][0],
            min_y: value[0][1],
            max_x: value[1][0],
            max_y: value[1][1],
        };
    }
}

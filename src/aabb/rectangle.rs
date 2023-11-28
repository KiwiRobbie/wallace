#[derive(Debug, Clone, PartialEq)]
pub struct Rectangle {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Rectangle {
    pub fn contains(&self, p: &Point) -> bool {
        return self.min_x < p.x && self.min_y < p.y && self.max_x > p.x && self.max_y > p.y;
    }

    pub fn to_array(&self) -> [[f32; 2]; 2] {
        return [[self.min_x, self.min_y], [self.max_x, self.max_y]];
    }

    pub fn area(&self) -> f32 {
        return (self.max_x - self.min_x) * (self.max_y - self.min_y);
    }
}

impl From<[[f32; 2]; 2]> for Rectangle {
    fn from(value: [[f32; 2]; 2]) -> Self {
        return Rectangle {
            min_x: value[0][0],
            min_y: value[0][1],
            max_x: value[1][0],
            max_y: value[1][1],
        };
    }
}

pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub fn superset(a: &Rectangle, b: &Rectangle) -> SupersetResult {
    let delta_min_x = a.min_x - b.min_x;
    let delta_min_y = a.min_y - b.min_y;
    let delta_max_x = a.max_x - b.max_x;
    let delta_max_y = a.max_y - b.max_y;

    if delta_min_x <= 0.0 && delta_min_y <= 0.0 && delta_max_x >= 0.0 && delta_max_y >= 0.0 {
        return SupersetResult::A;
    }
    if delta_min_x >= 0.0 && delta_min_y >= 0.0 && delta_max_x <= 0.0 && delta_max_y <= 0.0 {
        return SupersetResult::B;
    }
    SupersetResult::None
}

pub enum SupersetResult {
    A,
    B,
    None,
}

pub fn union(a: &Rectangle, b: &Rectangle) -> Vec<Rectangle> {
    match superset(a, b) {
        SupersetResult::A => return vec![a.clone()],
        SupersetResult::B => return vec![b.clone()],
        SupersetResult::None => {}
    };

    // check for possible clean and reorder
    let (a, b) = if {
        let a_array = a.to_array();

        let count: usize = (0usize..4usize)
            .map(|i| {
                let x_index = i.rem_euclid(2);
                let y_index = i.div_euclid(2);

                let x = a_array[x_index][0];
                let y = a_array[y_index][1];

                b.contains(&Point { x, y }) as usize
            })
            .sum();
        count == 2
    } {
        (b, a)
    } else {
        (a, b)
    };

    // Try to cut b with edges of a
    if let Some((b, c)) = cut(a, b) {
        // Check if result rectangle before the cut is no longer needed
        match superset(a, &b) {
            SupersetResult::A => return vec![a.clone(), c],
            SupersetResult::B => return vec![b, c],
            SupersetResult::None => {}
        };

        // Since neither a/b was superset, cutting b again will produce one external piece, and one subset of a
        if let Some((_, b)) = cut(a, &b) {
            return vec![a.clone(), b, c];
        }
    }

    // Rectangles must have being disjoint
    return vec![a.clone(), b.clone()];
}

pub fn cut(a: &Rectangle, b: &Rectangle) -> Option<(Rectangle, Rectangle)> {
    let a_array = a.to_array();
    let mut b_array = b.to_array();

    for fixed_axis in 0..2 {
        for dir in 0..2 {
            //   ^
            //   | Fixed axis
            //  _____
            //-|-----|---> cut axis
            // |  B  |

            let cut_axis = 1 - fixed_axis;

            let fixed_value = a_array[dir][fixed_axis];

            let cut_start_value = a_array[0][cut_axis];
            let cut_end_value = a_array[1][cut_axis];

            // Fixed coordinate of cut inside rectangle
            if b_array[0][fixed_axis] < fixed_value && fixed_value < b_array[1][fixed_axis] {
                // Cut cut ends after rectangle start and starts before rectangle end
                if b_array[0][cut_axis] < cut_end_value && cut_start_value < b_array[1][cut_axis] {
                    // Cut line does cut through rectangle
                    // println!(
                    //     "cutting on axis={} dir={} value={}",
                    //     fixed_axis, dir, fixed_value
                    // );

                    // c is chosen to be the portion after,
                    // will be disjoint from a so we don't need to check in future.
                    let mut c_array = b_array.clone();
                    c_array[1 - dir][fixed_axis] = fixed_value;

                    // b is portion before cut,
                    // may still overlap with a
                    b_array[dir][fixed_axis] = fixed_value;

                    return Some((b_array.into(), c_array.into()));
                }
            }
        }
    }
    return None;
}

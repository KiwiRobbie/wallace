#[derive(Debug, Clone, PartialEq)]
struct Rectangle {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

fn main() {
    {
        // Strict Superset
        let a = Rectangle {
            min_x: -1.0,
            max_x: 1.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        let b = Rectangle {
            min_x: -2.0,
            max_x: 2.0,
            min_y: -2.0,
            max_y: 2.0,
        };
        assert_eq!(vec![b.clone()], union(&a, &b));
        assert_eq!(vec![b.clone()], union(&b, &a));
    }

    {
        // Superset
        let a = Rectangle {
            min_x: -1.0,
            max_x: 1.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        assert_eq!(vec![a.clone()], union(&a, &a));
    }

    {
        // Disjoint
        let a = Rectangle {
            min_x: -1.0,
            max_x: 1.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        let b = Rectangle {
            min_x: 2.0,
            max_x: 4.0,
            min_y: -1.0,
            max_y: 1.0,
        };
        assert_eq!(vec![a.clone(), b.clone()], union(&a, &b));
        assert_eq!(vec![b.clone(), a.clone()], union(&b, &a));
    }

    {
        // Disjoint Touching Corner
        let a = Rectangle {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        let b = Rectangle {
            min_x: 1.0,
            min_y: 1.0,
            max_x: 2.0,
            max_y: 2.0,
        };
        assert_eq!(vec![a.clone(), b.clone()], union(&a, &b));
        assert_eq!(vec![b.clone(), a.clone()], union(&b, &a));
    }
    {
        // Disjoint Touching Edge
        let a = Rectangle {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        let b = Rectangle {
            min_x: 1.0,
            min_y: 0.0,
            max_x: 2.0,
            max_y: 1.0,
        };
        assert_eq!(vec![a.clone(), b.clone()], union(&a, &b));
        assert_eq!(vec![b.clone(), a.clone()], union(&b, &a));
    }

    {
        // Complex Corner
        let a = Rectangle {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 2.0,
            max_y: 2.0,
        };
        let b = Rectangle {
            min_x: 1.0,
            min_y: 1.0,
            max_x: 3.0,
            max_y: 3.0,
        };

        const EXPECTED_AREA: f32 = 7.0f32;

        {
            let result = union(&a, &b);
            let area: f32 = result.iter().map(|rect| rect.area()).sum();
            assert_eq!(EXPECTED_AREA, area);
            assert_eq!(3, result.len());
        }
        {
            let result = union(&b, &a);
            let area: f32 = result.iter().map(|rect| rect.area()).sum();
            assert_eq!(EXPECTED_AREA, area);
            assert_eq!(3, result.len());
        }
    }

    {
        // Complex Edge
        let a = Rectangle {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 2.0,
            max_y: 2.0,
        };
        let b = Rectangle {
            min_x: -1.0,
            min_y: 1.0,
            max_x: 3.0,
            max_y: 3.0,
        };

        const EXPECTED_AREA: f32 = 10.0f32;
        {
            let result = union(&a, &b);
            let area: f32 = result.iter().map(|rect| rect.area()).sum();
            assert_eq!(area, EXPECTED_AREA);
            assert_eq!(result.len(), 2);
        }
        {
            const EXPECTED_AREA: f32 = 10.0f32;

            let result = union(&a, &b);
            let area: f32 = result.iter().map(|rect| rect.area()).sum();
            assert_eq!(area, EXPECTED_AREA);
            assert_eq!(result.len(), 2);
        }
    }

    println!("All tests passed!")
}

impl Rectangle {
    fn contains(&self, p: &Point) -> bool {
        return self.min_x < p.x && self.min_y < p.y && self.max_x > p.x && self.max_y > p.y;
    }

    fn to_array(&self) -> [[f32; 2]; 2] {
        return [[self.min_x, self.min_y], [self.max_x, self.max_y]];
    }

    fn area(&self) -> f32 {
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

struct Point {
    x: f32,
    y: f32,
}

fn superset(a: &Rectangle, b: &Rectangle) -> SupersetResult {
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

enum SupersetResult {
    A,
    B,
    None,
}

fn union(a: &Rectangle, b: &Rectangle) -> Vec<Rectangle> {
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

fn cut(a: &Rectangle, b: &Rectangle) -> Option<(Rectangle, Rectangle)> {
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

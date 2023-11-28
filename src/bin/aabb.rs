#[derive(Debug, Clone, PartialEq)]
struct Rect {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

fn main() {
    // test superset
    // {
    //     println!("\nSUPERSET");
    //     let a = Rect {
    //         min_x: -1.0,
    //         max_x: 1.0,
    //         min_y: -1.0,
    //         max_y: 1.0,
    //     };
    //     let b = Rect {
    //         min_x: -2.0,
    //         max_x: 2.0,
    //         min_y: -2.0,
    //         max_y: 2.0,
    //     };
    //     assert_eq!(vec![b.clone()], union(&a, &b));
    //     assert_eq!(vec![b.clone()], union(&b, &a));
    // }

    // {
    //     println!("\nDISJOINT");
    //     let a = Rect {
    //         min_x: -1.0,
    //         max_x: 1.0,
    //         min_y: -1.0,
    //         max_y: 1.0,
    //     };
    //     let b = Rect {
    //         min_x: 2.0,
    //         max_x: 4.0,
    //         min_y: -1.0,
    //         max_y: 1.0,
    //     };
    //     assert_eq!(vec![a.clone(), b.clone()], union(&a, &b));
    //     assert_eq!(vec![b.clone(), a.clone()], union(&b, &a));
    // }

    {
        println!("\nCOMPLEX");
        let a = Rect {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 2.0,
            max_y: 2.0,
        };
        let b = Rect {
            min_x: 1.0,
            min_y: 1.0,
            max_x: 3.0,
            max_y: 3.0,
        };

        const EXPECTED_AREA: f32 = 10.0f32;

        let result = union(&a, &b);
        dbg!(&result);
        let area: f32 = result.iter().map(|rect| rect.area()).sum();
        assert_eq!(EXPECTED_AREA, area);
    }

    // {
    //     println!("\nCOMPLEX");
    //     let b = Rect {
    //         min_x: 0.0,
    //         min_y: -2.0,
    //         max_x: 2.0,
    //         max_y: 2.0,
    //     };
    //     let a = Rect {
    //         min_x: 1.0,
    //         min_y: -1.0,
    //         max_x: 3.0,
    //         max_y: 1.0,
    //     };

    //     const EXPECTED_AREA: f32 = 10.0f32;

    //     let result = union(&a, &b);
    //     dbg!(&result);
    //     let area: f32 = result.iter().map(|rect| rect.area()).sum();
    //     assert_eq!(EXPECTED_AREA, area);
    // }
}

impl Rect {
    fn contains(&self, p: &Point) -> bool {
        return self.min_x < p.x && self.min_y < p.y && self.max_x > p.x && self.max_y > p.y;
    }

    fn contains_strict(&self, p: &Point) -> bool {
        return self.min_x <= p.x && self.min_y <= p.y && self.max_x >= p.x && self.max_y >= p.y;
    }

    fn to_array(&self) -> [[f32; 2]; 2] {
        return [[self.min_x, self.min_y], [self.max_x, self.max_y]];
    }

    fn area(&self) -> f32 {
        return (self.max_x - self.min_x) * (self.max_y - self.min_y);
    }
}

impl From<[[f32; 2]; 2]> for Rect {
    fn from(value: [[f32; 2]; 2]) -> Self {
        return Rect {
            min_x: value[0][0],
            min_y: value[0][1],
            max_x: value[1][1],
            max_y: value[1][1],
        };
    }
}

struct Point {
    x: f32,
    y: f32,
}

// fn disjoint(a: &Rect, b: &Rect) -> bool {
//     {
//         // non-overlapping x-axis
//         if a.max_x < b.min_x || a.min_x > b.max_x {
//             return true;
//         }

//         // non-overlapping y-axis
//         if a.max_y < b.min_y || a.min_y > b.max_y {
//             return true;
//         }
//     }
//     return false;
// }

fn superset(a: &Rect, b: &Rect) -> SupersetResult {
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

fn union(a: &Rect, b: &Rect) -> Vec<Rect> {
    // if disjoint(a, b) {
    //     return vec![a.clone(), b.clone()];
    // }

    println!("Testing superset");
    match superset(a, b) {
        SupersetResult::A => return vec![a.clone()],
        SupersetResult::B => return vec![b.clone()],
        SupersetResult::None => {}
    };

    println!("Testing first split");
    if let Some((b, c)) = split(a, b) {
        println!("Testing split superset");
        match superset(a, &b) {
            SupersetResult::A => return vec![a.clone(), c],
            SupersetResult::B => return vec![b, c],
            SupersetResult::None => {}
        };

        println!("Testing second split");
        if let Some((_, b)) = split(a, &b) {
            return vec![a.clone(), b, c];
        }
    }

    println!("Disjoint");
    return vec![a.clone(), b.clone()];
}

fn split(a: &Rect, b: &Rect) -> Option<(Rect, Rect)> {
    let a_array = a.to_array();
    let b_array = b.to_array();

    let contains = [0usize, 1usize, 2usize, 3usize].map(|i| {
        let x_index = i.rem_euclid(2);
        let y_index = i.div_euclid(2);

        let x = a_array[x_index][0];
        let y = a_array[y_index][1];

        b.contains(&Point { x, y })
    });
    dbg!(contains);

    for axis in 0..2 {
        for dir in 0..2 {
            // dir = 0 negative
            // dir = 1 positive

            let o_x = dir * (1 - axis);
            let o_y = dir * (axis);

            let d_x = axis;
            let d_y = 1 - axis;

            let start_x = o_x;
            let start_y = o_y;
            let end_x = o_x + d_x;
            let end_y = o_y + d_y;

            let start = start_x + 2 * start_y;
            let end = end_x + 2 * end_y;

            let intersection = contains[start] ^ contains[end];

            if intersection {
                let value = a_array[dir][axis];
                println!("splitting on axis={} dir={} t={}", axis, dir, value);

                let mut b_array = b_array.clone();
                let mut c_array = b_array.clone();

                b_array[dir][axis] = value;
                c_array[1 - dir][axis] = value;

                dbg!(b_array);
                dbg!(c_array);

                return Some((b_array.into(), c_array.into()));
            }
        }
    }
    return None;
}

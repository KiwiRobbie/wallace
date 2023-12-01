use azalea::core::aabb::AABB;

use super::aabb_2d::Aabb2D;

#[derive(Debug, Clone, PartialEq)]
pub struct Point3D(pub [f32; 2]);

#[derive(Debug, Clone, PartialEq)]
pub struct Aabb3D(pub [f32; 6]);

impl Aabb3D {
    pub fn min_x(&self) -> f32 {
        self.0[0]
    }
    pub fn min_y(&self) -> f32 {
        self.0[1]
    }
    pub fn min_z(&self) -> f32 {
        self.0[2]
    }

    pub fn max_x(&self) -> f32 {
        self.0[0 + 3]
    }
    pub fn max_y(&self) -> f32 {
        self.0[1 + 3]
    }
    pub fn max_z(&self) -> f32 {
        self.0[2 + 3]
    }

    pub const FULL_BLOCK: Self = Aabb3D([0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);

    pub fn strict_contains(&self, p: &Point3D) -> bool {
        for axis in 0..3 {
            if self.0[axis] >= p.0[axis] || self.0[axis + 3] <= p.0[axis] {
                return false;
            }
        }
        true
    }

    pub fn volume(&self) -> f32 {
        return (0..3).map(|axis| self.0[axis + 3] - self.0[axis]).product();
    }

    pub fn superset(&self, other: &Self) -> SupersetResult {
        let deltas = [0, 1, 2, 3, 4, 5].map(|index| self.0[index] - other.0[index]);

        if deltas[0..3].iter().all(|v| v <= &0.0f32) && deltas[3..6].iter().all(|v| v >= &0.0f32) {
            dbg!("a");
            return SupersetResult::A;
        }
        if deltas[0..3].iter().all(|v| v >= &0.0f32) && deltas[3..6].iter().all(|v| v <= &0.0f32) {
            dbg!("b");
            return SupersetResult::B;
        }
        SupersetResult::None
    }

    pub fn cut(&self, other: &Self) -> Option<(Aabb3D, Aabb3D)> {
        let self_array = self.0;
        let mut other_array = other.0;

        for fixed_axis in 0..3 {
            for dir in 0..2 {
                //   ^
                //   | Fixed axis
                //  _____
                //-|-----|---> cut axis
                // |Other|

                let cut_u_axis = (fixed_axis + 1usize).rem_euclid(3usize);
                let cut_v_axis = (fixed_axis + 2usize).rem_euclid(3usize);

                let fixed_value = self_array[fixed_axis + 3 * dir];

                // Fixed coordinate of cut inside rectangle
                if other_array[fixed_axis] < fixed_value
                    && fixed_value < other_array[fixed_axis + 3]
                {
                    let cut_surface = super::aabb_2d::Aabb2D {
                        min_x: self_array[cut_u_axis],
                        min_y: self_array[cut_v_axis],
                        max_x: self_array[cut_u_axis + 3],
                        max_y: self_array[cut_v_axis + 3],
                    };

                    let target_surface = super::aabb_2d::Aabb2D {
                        min_x: other_array[cut_u_axis],
                        min_y: other_array[cut_v_axis],
                        max_x: other_array[cut_u_axis + 3],
                        max_y: other_array[cut_v_axis + 3],
                    };

                    // Check that cut overlaps with target
                    if cut_surface.overlaps(&target_surface) {
                        // Cut line does cut through rectangle
                        // println!(
                        //     "cutting on axis={} dir={} value={}",
                        //     fixed_axis, dir, fixed_value
                        // );

                        // new is chosen to be the portion after,
                        // will be disjoint from self so we don't need to check in future.
                        let mut new_array = other_array.clone();
                        new_array[fixed_axis + 3 * (1 - dir)] = fixed_value;

                        // other is portion before cut,
                        // may still overlap with self
                        other_array[fixed_axis + 3 * dir] = fixed_value;

                        return Some((Aabb3D(other_array), Aabb3D(new_array)));
                    }
                }
            }
        }
        return None;
    }

    pub fn union(&self, other: &Self) -> Vec<Self> {
        match self.superset(other) {
            SupersetResult::A => return vec![self.clone()],
            SupersetResult::B => return vec![other.clone()],
            SupersetResult::None => {}
        };

        // check for possible clean and reorder
        // let (a, b) = if {
        //     let a_array = self.0;

        //     let count: usize = (0usize..4usize)
        //         .map(|i| {
        //             let x_index = i.rem_euclid(2);
        //             let y_index = i.div_euclid(2);

        //             let x = a_array[x_index][0];
        //             let y = a_array[y_index][1];

        //             other.strict_contains(&Point3D { x, y }) as usize
        //         })
        //         .sum();
        //     count == 2
        // } {
        //     (other, self)
        // } else {
        //     (self, other)
        // };
        let (a, b) = (self, other);
        dbg!((&a, &b));

        // Try to cut b with edges of a
        if let Some((b, c)) = a.cut(b) {
            dbg!((&b, &c));

            // Check if result rectangle before the cut is no longer needed
            match a.superset(&b) {
                SupersetResult::A => return vec![a.clone(), c],
                SupersetResult::B => return vec![b, c],
                SupersetResult::None => {}
            };

            // Since neither a/b was superset, cutting b again will produce one external piece, and one subset of a
            if let Some((_, b)) = a.cut(&b) {
                return vec![a.clone(), b, c];
            }
        }

        // Rectangles must have being disjoint
        return vec![a.clone(), b.clone()];
    }

    pub fn surface_projection(&self, axis: usize) -> Aabb2D {
        let u = (axis + 1).rem_euclid(3);
        let v = (axis + 2).rem_euclid(3);
        Aabb2D {
            min_x: self.0[u],
            min_y: self.0[v],
            max_x: self.0[u + 3],
            max_y: self.0[v + 3],
        }
    }

    // pub fn cmp_faces(&self, other: &Self) -> [ContactType; 6] {
    //     let mut result = [ContactType::None; 6];
    //     for direction in 0..3 {
    //         let self_surface = self.surface_projection(axis);
    //         let other_surface = other.surface_projection(axis);

    //         match self_surface.cmp(&other_surface) {
    //             super::aabb_2d::Aabb2CmpResult::Superset => {

    //             }
    //             super::aabb_2d::Aabb2CmpResult::Subset => {}
    //             super::aabb_2d::Aabb2CmpResult::Mixed => {}
    //             super::aabb_2d::Aabb2CmpResult::None => {}
    //         };
    //     }
    //     return result;
    // }
}

impl From<AABB> for Aabb3D {
    fn from(value: AABB) -> Self {
        Self([
            value.min_x as f32,
            value.min_y as f32,
            value.min_z as f32,
            value.max_x as f32,
            value.max_y as f32,
            value.max_z as f32,
        ])
    }
}

pub enum Axis {
    X,
    Y,
    Z,
    NegX,
    NegY,
    NegZ,
}

#[derive(Debug, Clone, Copy)]
pub enum ContactType {
    Superset,
    Subset,
    Mixed,
    None,
}

pub enum SupersetResult {
    A,
    B,
    None,
}

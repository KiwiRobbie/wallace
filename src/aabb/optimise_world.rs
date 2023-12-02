use bevy::math::{IVec3, UVec2, UVec3, Vec2};
use itertools::Itertools;

use super::aabb_2d::Aabb2D;
use super::aabb_3d::Aabb3D;

pub const CHUNK_WIDTH: usize = 16;
pub const SUB_CHUNK_HEIGHT: usize = 16;

pub const SUB_CHUNK_SIZE: IVec3 = IVec3 {
    x: CHUNK_WIDTH as i32,
    y: SUB_CHUNK_HEIGHT as i32,
    z: CHUNK_WIDTH as i32,
};

// Index order: data[z][x][y]
pub struct SubChunkNavMesh {
    pub location: IVec3,
    pub floor: Box<[NavMeshLayer]>,
    pub ceiling: Box<[NavMeshLayer]>,
}

#[derive(Debug)]
pub enum NavMeshLayerType {
    Floor,
    Ceiling,
}

#[derive(Debug)]
pub struct NavMeshLayer {
    pub height: f32,
    pub nodes: Vec<NavMeshNode>,
    pub blocks: Box<[[Vec<usize>; CHUNK_WIDTH]; CHUNK_WIDTH]>,
}

impl NavMeshLayer {
    fn insert(&mut self, aabb: Aabb2D, pos: UVec2) {
        let node = NavMeshNode {
            aabb,
            pos,
            adjacent: vec![],
        };
        self.blocks[pos.y as usize][pos.x as usize].push(self.nodes.len());
        self.nodes.push(node);
    }

    fn cut(&mut self, pos: UVec2, cutting_surface: Aabb2D) {
        println!("PASS");
        // TODO: MAKE NOT AWFUL THIS IS A WAR CRIME
        // REMEMBER TO FIX AFTER self.blocks

        let mut dirty = vec![];
        std::mem::swap(&mut self.blocks[pos.y as usize][pos.x as usize], &mut dirty);
        let mut split = vec![];

        for node in dirty.iter().map(|index| self.nodes[*index].clone()) {
            split.extend(node.aabb.subtract(&cutting_surface).into_iter());
        }

        let split = split.into_iter();
        let mut dirty = dirty.into_iter();

        for new in split {
            if let Some(dirty_index) = dirty.next() {
                self.nodes[dirty_index] = NavMeshNode {
                    aabb: new,
                    adjacent: vec![],
                    pos,
                };
                self.blocks[pos.y as usize][pos.x as usize].push(dirty_index);
            } else {
                self.blocks[pos.y as usize][pos.x as usize].push(self.nodes.len());
                self.nodes.push(NavMeshNode {
                    aabb: new,
                    adjacent: vec![],
                    pos,
                });
            }
        }
        for dirty_index in dirty.sorted().rev() {
            for block_index in self.blocks.iter_mut().flatten().flatten() {
                if *block_index > dirty_index {
                    *block_index -= 1;
                }
            }
            self.nodes.remove(dirty_index);
        }
        // for (y, row) in self.blocks.iter().enumerate() {
        //     for (x, blocks) in row.iter().enumerate() {
        //         for block in blocks {
        //             assert_eq!(
        //                 self.nodes[*block].pos,
        //                 UVec2 {
        //                     x: x as u32,
        //                     y: y as u32
        //                 }
        //             );
        //         }
        //     }
        // }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum OptionalMany<T> {
    None,
    Single(T),
    Multiple(Box<[T]>),
}

impl<'a, T> OptionalMany<T> {
    pub const NONE: Self = Self::None;

    pub fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        Into::<&'a [T]>::into(self).iter()
    }
}

impl<T: PartialEq> OptionalMany<T> {
    pub fn is_some(&self) -> bool {
        self != &Self::None
    }
}

impl<T> Default for OptionalMany<T> {
    fn default() -> Self {
        Self::None
    }
}

impl<T, U: From<T>> From<Vec<T>> for OptionalMany<U> {
    fn from(value: Vec<T>) -> Self {
        match value.len() {
            0 => Self::None,
            1 => Self::Single(value.into_iter().next().unwrap().into()),
            _ => Self::Multiple(
                value
                    .into_iter()
                    .map(|item| item.into())
                    .collect::<Box<[U]>>(),
            ),
        }
    }
}

impl<'a, T> Into<&'a [T]> for &'a OptionalMany<T> {
    fn into(self) -> &'a [T] {
        match self {
            OptionalMany::None => &[],
            OptionalMany::Single(single) => std::slice::from_ref(single),
            OptionalMany::Multiple(multiple) => multiple.as_ref(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavMeshNode {
    pub aabb: Aabb2D,
    pub pos: UVec2,
    pub adjacent: Vec<NavMeshAdjacent>,
}

#[derive(Debug, Clone)]
enum NavMeshAdjacent {
    Superset {
        index: usize,
        axis: u8,
    },
    Subset {
        index: usize,
        axis: u8,
    },
    Overlapping {
        min: f32,
        max: f32,
        index: usize,
        axis: u8,
    },
}

pub struct SubChunk {
    pub location: IVec3,
    aabbs: Box<[[[OptionalMany<Aabb3D>; SUB_CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH]>,
    block_collision_mask: Box<[[u16; CHUNK_WIDTH]; CHUNK_WIDTH]>,
    block_floor_mask: Box<[[u16; CHUNK_WIDTH]; CHUNK_WIDTH]>,
    full_block_mask: Box<[[u16; CHUNK_WIDTH]; CHUNK_WIDTH]>,
}

impl SubChunk {
    pub fn new(
        location: IVec3,
        aabbs: Box<[[[OptionalMany<Aabb3D>; SUB_CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH]>,
    ) -> Self {
        let mut collision_blocks = Box::new([[0; CHUNK_WIDTH]; CHUNK_WIDTH]);
        let mut full_blocks = Box::new([[0; CHUNK_WIDTH]; CHUNK_WIDTH]);

        for (z, plane) in aabbs.iter().enumerate() {
            for (x, column) in plane.iter().enumerate() {
                let mut column_full_blocks = 0;
                let mut column_collision_blocks = 0;
                for (y, block) in column.iter().enumerate() {
                    if block.is_some() {
                        column_collision_blocks |= 1 << y;
                        for aabb in block.iter() {
                            if aabb.max_y() > 1.0 {
                                column_collision_blocks |= 2 << y; // TODO: Handle chunk boundaries
                            }
                        }
                    }
                    if block == &OptionalMany::Single(Aabb3D::FULL_BLOCK) {
                        column_full_blocks = 1 << y;
                    }
                }
                collision_blocks[z][x] = column_collision_blocks;
                full_blocks[z][x] = column_full_blocks;
            }
        }

        let mut chunk = Self {
            location,
            aabbs,
            block_floor_mask: collision_blocks.clone(),
            block_collision_mask: collision_blocks,
            full_block_mask: full_blocks,
        };
        chunk.apply_full_block_occlusion();
        chunk
    }

    pub fn iter_floor(&self) -> impl Iterator<Item = (UVec3, &[Aabb3D])> {
        [(0..CHUNK_WIDTH), (0..CHUNK_WIDTH), (0..SUB_CHUNK_HEIGHT)]
            .into_iter()
            .multi_cartesian_product()
            .flat_map(|value| {
                let z = value[0];
                let y = value[1];
                let x = value[2];

                if self.block_floor_mask[z][x] >> y & 1 == 1 {
                    Some((
                        UVec3 {
                            x: x as u32,
                            y: y as u32,
                            z: z as u32,
                        },
                        Into::<&[_]>::into(&self.aabbs[z][x][y]),
                    ))
                } else {
                    None
                }
            })
    }

    pub fn iter_ceiling(&self) -> impl Iterator<Item = (UVec3, &[Aabb3D])> {
        // TODO: ADD MASKS
        self.iter_collisions()
    }

    pub fn iter_collisions(&self) -> impl Iterator<Item = (UVec3, &[Aabb3D])> {
        [(0..CHUNK_WIDTH), (0..CHUNK_WIDTH), (0..SUB_CHUNK_HEIGHT)]
            .into_iter()
            .multi_cartesian_product()
            .map(|value| {
                let z = value[0];
                let x = value[1];
                let y = value[2];
                (
                    UVec3 {
                        x: x as u32,
                        y: y as u32,
                        z: z as u32,
                    },
                    Into::<&[_]>::into(&self.aabbs[z][x][y]),
                )
            })
    }

    pub fn build_nav_mesh(&self) -> SubChunkNavMesh {
        let mut ceiling: Vec<NavMeshLayer> = vec![];
        let mut floor: Vec<NavMeshLayer> = vec![];

        for (pos, block) in self.iter_floor() {
            for aabb in block.iter() {
                Self::insert_aabb_into_layers(&mut floor, aabb, pos, NavMeshLayerType::Floor);
            }
        }
        for (pos, block) in self.iter_ceiling() {
            for aabb in block.iter() {
                Self::insert_aabb_into_layers(&mut ceiling, aabb, pos, NavMeshLayerType::Ceiling);
            }
        }

        self.remove_overlap_floor(&mut floor);
        self.cut_floor(&mut floor);

        SubChunkNavMesh {
            location: self.location,
            floor: floor.into(),
            ceiling: ceiling.into(),
        }
    }

    fn apply_full_block_occlusion(&mut self) {
        for z in 0..CHUNK_WIDTH {
            for x in 0..CHUNK_WIDTH {
                self.block_floor_mask[z][x] = !(self.full_block_mask[z][x] >> 1)
                    & !(self.full_block_mask[z][x] >> 2)
                    & self.block_collision_mask[z][x];
            }
        }
    }

    fn remove_overlap_floor(&self, floor: &mut Vec<NavMeshLayer>) {
        // remove needless overlap for inflated full blocks first
        // doesn't work

        for layer in floor {
            let mut updated_aabbs = vec![];
            for node in layer.nodes.iter() {
                if node.aabb
                    == (Aabb2D {
                        min_x: 0.3,
                        max_x: 1.3,
                        min_y: 0.3,
                        max_y: 1.3,
                    })
                {
                    let mut updated_aabb = node.aabb.clone();
                    let mut self_array = updated_aabb.to_array();

                    for dir in 0..=1 {
                        for axis in 0..=1 {
                            let target_x = ((1 - axis) * (2 * dir - 1)) as usize;
                            let target_z = (axis * (2 * dir - 1)) as usize;

                            for aabb in layer.blocks[target_z][target_x].clone() {
                                let other_array = &layer.nodes[aabb].aabb.to_array();

                                if (dir == 0 && other_array[1][axis] >= 1.0)
                                    || (dir == 1 && other_array[0][axis] <= 0.0)
                                {
                                    if other_array[0][1 - axis] == self_array[0][1 - axis]
                                        && other_array[1][1 - axis] == self_array[1][1 - axis]
                                    {
                                        self_array[dir][axis] =
                                            self_array[dir][axis].clamp(0.0, 1.0);
                                        updated_aabb = self_array.into();
                                    }
                                }
                            }
                        }
                    }
                    updated_aabbs.push(updated_aabb);
                }
            }
            for (node, aabb) in layer.nodes.iter_mut().zip(updated_aabbs.into_iter()) {
                node.aabb = aabb;
            }
        }

        // reduce other sources of overlapping
    }

    fn aabb_2d_subtract_list(aabb: &Aabb2D, others: &[Aabb2D]) -> Vec<Aabb2D> {
        let mut result = vec![];
        let mut cutting_stack = vec![aabb.clone()];
        'next_aabb: while let Some(aabb) = cutting_stack.pop() {
            for other in others {
                let cut = aabb.subtract(&other);
                if cut.len() > 1 || Some(&aabb) != cut.first() {
                    cutting_stack.extend(cut.into_iter());
                    continue 'next_aabb;
                }
            }
            result.push(aabb);
        }
        result
    }

    fn cut_floor(&self, floor: &mut Vec<NavMeshLayer>) {
        for layer in floor.iter_mut() {
            let height = layer.height;
            let cut_indices = (height as usize)..(((height + 1.8).ceil() + 0.1) as usize);

            let mut new_nodes: Vec<(UVec2, Aabb2D)> = vec![];
            for block in layer.blocks.iter_mut().flatten() {
                block.clear();
            }

            for node in layer.nodes.drain(0..layer.nodes.len()) {
                let mut cutting_stack = vec![node.aabb.clone()];
                'next_aabb: while let Some(aabb) = cutting_stack.pop() {
                    for cut_layer in cut_indices.clone() {
                        for dz in [-1isize, 0isize, 1isize].into_iter() {
                            let sample_z = node.pos.y as isize + dz;
                            if !(0isize..CHUNK_WIDTH as isize).contains(&sample_z) {
                                continue;
                            }
                            for dx in [-1isize, 0isize, 1isize].into_iter() {
                                let sample_x = node.pos.x as isize + dx;
                                if !(0isize..CHUNK_WIDTH as isize).contains(&sample_x) {
                                    continue;
                                }
                                for cutting_aabb in self.aabbs[sample_z as usize][sample_x as usize]
                                    [cut_layer]
                                    .iter()
                                {
                                    if cut_layer as f32 + cutting_aabb.min_y() - 1.8 < height
                                        && height < cut_layer as f32 + cutting_aabb.max_y()
                                    {
                                        let cut = aabb.subtract(
                                            &cutting_aabb
                                                .surface_projection(1)
                                                .translate(Vec2 {
                                                    x: dx as f32,
                                                    y: dz as f32,
                                                })
                                                .inflate(Vec2::splat(0.3)),
                                        );

                                        if cut.len() > 1 || Some(&aabb) != cut.first() {
                                            cutting_stack.extend(cut.into_iter());
                                            continue 'next_aabb;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    new_nodes.push((
                        UVec2 {
                            x: node.pos.x,
                            y: node.pos.y,
                        },
                        aabb.clone(),
                    ));
                }
            }

            for (pos, aabb) in new_nodes.into_iter() {
                layer.blocks[pos.y as usize][pos.x as usize].push(layer.nodes.len());
                layer.nodes.push(NavMeshNode {
                    aabb,
                    pos,
                    adjacent: vec![],
                });
            }
        }
    }

    /// Occlude self using another sub chunk
    fn _apply_other_occlusion(&mut self, _other: &Self) {
        todo!();
    }

    fn insert_aabb_into_layers(
        layers: &mut Vec<NavMeshLayer>,
        aabb: &Aabb3D,
        block_location: UVec3,
        layer_type: NavMeshLayerType,
    ) {
        let height = block_location.y as f32
            + match layer_type {
                NavMeshLayerType::Ceiling => aabb.min_y(),
                NavMeshLayerType::Floor => aabb.max_y(),
            };

        let surface = aabb.surface_projection(1).inflate(Vec2::splat(0.3));

        // TODO: Optimise by using initial bounds for search (can't change by more that 1.5 blocks)
        match layers.binary_search_by(|layer| layer.height.partial_cmp(&height).unwrap()) {
            Err(index) => {
                let mut layer = NavMeshLayer {
                    height,
                    nodes: vec![],
                    blocks: Default::default(),
                };
                layer.insert(
                    surface.clone(),
                    UVec2 {
                        x: block_location.x,
                        y: block_location.z,
                    },
                );
                layers.insert(index, layer);
            }
            Ok(index) => layers[index].insert(
                surface.clone(),
                UVec2 {
                    x: block_location.x,
                    y: block_location.z,
                },
            ),
        }
    }
}

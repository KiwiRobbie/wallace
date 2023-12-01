use bevy::math::{IVec2, IVec3, UVec2, UVec3, Vec2, Vec3};
use bevy_rapier3d::na::ComplexField;
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

    fn cut_floor(&self, floor: &mut Vec<NavMeshLayer>) {
        for layer in floor {
            let height = layer.height;
            let cut_indices = (height as usize)..(((height + 1.8).ceil() + 0.1) as usize);

            for cut_layer in cut_indices.clone() {
                for z in 0..CHUNK_WIDTH {
                    for x in 0..CHUNK_WIDTH {
                        for dz in [-1isize, 0isize, 1isize].into_iter() {
                            let sample_z = z as isize + dz;
                            if !(0isize..CHUNK_WIDTH as isize).contains(&sample_z) {
                                continue;
                            }

                            for dx in [-1isize, 0isize, 1isize].into_iter() {
                                let sample_x = x as isize + dx;
                                if !(0isize..CHUNK_WIDTH as isize).contains(&sample_x) {
                                    continue;
                                }
                                for aabb in self.aabbs[sample_z as usize][sample_x as usize]
                                    [cut_layer]
                                    .iter()
                                {
                                    if cut_layer as f32 + aabb.min_y() - 1.8 < height
                                        && height < cut_layer as f32 + aabb.max_y()
                                    {
                                        layer.cut(
                                            UVec2 {
                                                x: x as u32,
                                                y: z as u32,
                                            },
                                            aabb.surface_projection(1)
                                                .translate(Vec2 {
                                                    x: dx as f32,
                                                    y: dz as f32,
                                                })
                                                .inflate(Vec2::splat(0.3)),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Occlude self using another sub chunk
    fn apply_other_occlusion(&mut self, other: &Self) {
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

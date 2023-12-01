use bevy::math::{IVec2, IVec3, UVec3};
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
    pub layers: Box<[NavMeshLayer]>,
}

pub enum NavMeshLayerType {
    Floor,
    Ceiling,
}
struct NavMeshLayer {
    pub layer_type: NavMeshLayerType,
    pub height: f32,
    pub nodes: Vec<NavMeshNode>,
    pub blocks: Box<[[OptionalMany<usize>; CHUNK_WIDTH]; CHUNK_WIDTH]>,
}

impl NavMeshLayer {
    fn insert(&mut self, node: Aabb3D, pos: IVec2) {}
}

#[derive(Debug, PartialEq, Clone)]
pub enum OptionalMany<T> {
    None,
    Single(T),
    Multiple(Box<[T]>),
}

impl<T> OptionalMany<T> {
    pub const NONE: Self = Self::None;
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

struct NavMeshNode {
    aabb: Aabb2D,
    adjacent: Box<[NavMeshAdjacent]>,
}

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
    block_top_mask: Box<[[u16; CHUNK_WIDTH]; CHUNK_WIDTH]>,
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
                        column_collision_blocks |= 1 << y
                    }
                    if block == &OptionalMany::Single(Aabb3D::FULL_BLOCK) {
                        column_full_blocks |= 1 << y;
                    }
                }
                collision_blocks[z][x] = column_collision_blocks;
                full_blocks[z][x] = column_full_blocks;
            }
        }

        let mut chunk = Self {
            location,
            aabbs,
            block_top_mask: collision_blocks.clone(),
            block_collision_mask: collision_blocks,
            full_block_mask: full_blocks,
        };
        chunk.apply_full_block_occlusion();
        chunk
    }

    pub fn iter_top(&self) -> impl Iterator<Item = (UVec3, &[Aabb3D])> {
        [(0..CHUNK_WIDTH), (0..CHUNK_WIDTH), (0..SUB_CHUNK_HEIGHT)]
            .into_iter()
            .multi_cartesian_product()
            .flat_map(|value| {
                let z = value[0];
                let y = value[1];
                let x = value[2];

                if self.block_top_mask[z][x] >> y & 1 == 1 {
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

    pub fn iter_collisions(&self) -> impl Iterator<Item = (UVec3, &[Aabb3D])> {
        [(0..CHUNK_WIDTH), (0..CHUNK_WIDTH), (0..SUB_CHUNK_HEIGHT)]
            .into_iter()
            .multi_cartesian_product()
            .map(|value| {
                let z = value[0];
                let y = value[1];
                let x = value[2];
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

    pub fn build_nav_mesh(&self) -> () {
        let mut layers: Vec<NavMeshLayer> = vec![];

        for (z, plane) in self.aabbs.iter().enumerate() {
            for (x, column) in plane.iter().enumerate() {
                for (y, block) in column.iter().enumerate() {
                    match block {
                        OptionalMany::None => {}
                        OptionalMany::Single(aabb) => {}
                        OptionalMany::Multiple(aabbs) => {}
                    }
                }
            }
        }
    }

    fn apply_full_block_occlusion(&mut self) {
        for z in 0..CHUNK_WIDTH {
            for x in 0..CHUNK_WIDTH {
                self.block_top_mask[z][x] = !(self.full_block_mask[z][x] >> 1)
                    & !(self.full_block_mask[z][x] >> 2)
                    & self.block_collision_mask[z][x];
            }
        }
    }

    /// Occlude self using another sub chunk
    fn apply_other_occlusion(&mut self, other: &Self) {
        todo!();
    }

    fn insert_aabb_into_layers(
        layers: &mut Vec<NavMeshLayer>,
        aabb: Aabb3D,
        block_location: IVec3,
    ) {
        let height = block_location.y as f32 + aabb.max_y();
        let surface = aabb.surface_projection(1);

        // TODO: Optimise by using initial bounds for search (can't change by more that 1.5 blocks)
        match layers.binary_search_by(|layer| layer.height.partial_cmp(&height).unwrap()) {
            Err(index) => layers.insert(
                index,
                NavMeshLayer {
                    layer_type: NavMeshLayerType::Floor,
                    height: height,
                    nodes: vec![],
                    blocks: Default::default(),
                },
            ),
            Ok(index) => layers[index].insert(
                aabb,
                IVec2 {
                    x: block_location.x,
                    y: block_location.z,
                },
            ),
        }
    }
}

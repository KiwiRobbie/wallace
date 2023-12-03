use std::f32::consts::E;

use bevy::math::{IVec2, IVec3, UVec2, UVec3, Vec2};

use smallvec::{smallvec, SmallVec};

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
    pub blocks: Box<[[SmallVec<[usize; 1]>; CHUNK_WIDTH]; CHUNK_WIDTH]>,
}

impl NavMeshLayer {
    fn insert(&mut self, aabb: Aabb2D, pos: UVec2) {
        let node = NavMeshNode {
            aabb,
            pos,
            _adjacent: smallvec![],
        };
        self.blocks[pos.y as usize][pos.x as usize].push(self.nodes.len());
        self.nodes.push(node);
    }
}

#[derive(Debug, Clone)]
pub struct NavMeshNode {
    pub aabb: Aabb2D,
    pub pos: UVec2,
    pub _adjacent: SmallVec<[NavMeshAdjacent; 0]>,
}

#[derive(Debug, Clone)]
pub enum NavMeshAdjacent {
    _Superset {
        index: usize,
        axis: u8,
    },
    _Subset {
        index: usize,
        axis: u8,
    },
    _Overlapping {
        min: f32,
        max: f32,
        index: usize,
        axis: u8,
    },
}

pub struct SubChunk {
    pub location: IVec3,
    aabbs: Vec<(UVec3, Aabb3D)>,
    blocks: Box<[[[SmallVec<[usize; 1]>; SUB_CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH]>,
    block_collision_mask: Box<[[u16; CHUNK_WIDTH]; CHUNK_WIDTH]>,
    block_floor_mask: Box<[[u16; CHUNK_WIDTH]; CHUNK_WIDTH]>,
    full_block_mask: Box<[[u16; CHUNK_WIDTH]; CHUNK_WIDTH]>,
}

impl SubChunk {
    pub fn new(
        location: IVec3,
        source: Box<[[[SmallVec<[Aabb3D; 1]>; SUB_CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH]>,
    ) -> Self {
        let mut aabbs = vec![];
        let mut blocks: Box<
            [[[SmallVec<[usize; 1]>; SUB_CHUNK_HEIGHT]; CHUNK_WIDTH]; CHUNK_WIDTH],
        > = Default::default();

        let mut collision_blocks = Box::new([[0; CHUNK_WIDTH]; CHUNK_WIDTH]);
        let mut full_blocks = Box::new([[0; CHUNK_WIDTH]; CHUNK_WIDTH]);

        for (z, plane) in source.into_iter().enumerate() {
            for (x, column) in plane.into_iter().enumerate() {
                let mut column_full_blocks = 0;
                let mut column_collision_blocks = 0;
                for (y, block) in column.into_iter().enumerate() {
                    if !block.is_empty() {
                        column_collision_blocks |= 1 << y;
                        if block[0] == Aabb3D::FULL_BLOCK && block.len() == 1 {
                            column_full_blocks = 1 << y;
                        }
                        for aabb in block.into_iter() {
                            if aabb.max_y() > 1.0 {
                                column_collision_blocks |= 2 << y; // TODO: Handle chunk boundaries
                                blocks[z][x]
                                    .get_mut(y + 1)
                                    .and_then(|a| Some(a.push(aabbs.len())));
                            }
                            blocks[z][x][y].push(aabbs.len());
                            aabbs.push((
                                UVec3 {
                                    x: x as u32,
                                    y: y as u32,
                                    z: z as u32,
                                },
                                aabb,
                            ));
                        }
                    }
                }
                collision_blocks[z][x] = column_collision_blocks;
                full_blocks[z][x] = column_full_blocks;
            }
        }

        let mut chunk = Self {
            location,
            aabbs,
            blocks,
            block_floor_mask: collision_blocks.clone(),
            block_collision_mask: collision_blocks,
            full_block_mask: full_blocks,
        };
        chunk.apply_full_block_occlusion();
        chunk
    }

    pub fn iter_floor(&self) -> impl Iterator<Item = (UVec3, &Aabb3D)> {
        self.aabbs.iter().flat_map(|(pos, aabb)| {
            let (x, y, z) = (pos.x as usize, pos.y as usize, pos.z as usize);
            if self.block_floor_mask[z][x] >> y & 1 == 1 {
                Some((*pos, aabb))
            } else {
                None
            }
        })
    }

    pub fn iter_ceiling(&self) -> impl Iterator<Item = (UVec3, &Aabb3D)> {
        // TODO: ADD MASKS
        self.iter_collisions()
    }

    pub fn iter_collisions(&self) -> impl Iterator<Item = (UVec3, &Aabb3D)> {
        self.aabbs.iter().map(|(pos, aabb)| (*pos, aabb))
    }

    pub fn build_nav_mesh(&self) -> SubChunkNavMesh {
        let mut ceiling: Vec<NavMeshLayer> = vec![];
        let mut floor: Vec<NavMeshLayer> = vec![];

        for (pos, aabb) in self.iter_floor() {
            Self::insert_aabb_into_layers(&mut floor, aabb, pos, NavMeshLayerType::Floor);
        }
        for (pos, aabb) in self.iter_ceiling() {
            Self::insert_aabb_into_layers(&mut ceiling, aabb, pos, NavMeshLayerType::Ceiling);
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

    pub fn apply_greedy_meshing(&mut self) {
        fn check_mask(mask: &[[u16; CHUNK_WIDTH]; CHUNK_WIDTH], pos: &[usize; 3]) -> bool {
            return mask[pos[2]][pos[0]] & (1 << pos[1]) != 0;
        }
        fn set_mask(mask: &mut [[u16; CHUNK_WIDTH]; CHUNK_WIDTH], pos: &[usize; 3], value: bool) {
            mask[pos[2]][pos[0]] |= (value as u16) << pos[1];
        }

        let mut visited = Box::new([[0u16; CHUNK_WIDTH]; CHUNK_WIDTH]);

        // First apply to full blocks
        for start_index in 0..CHUNK_WIDTH * CHUNK_WIDTH * SUB_CHUNK_HEIGHT {
            let start_y = start_index.rem_euclid(SUB_CHUNK_HEIGHT);
            let start_x = start_index
                .div_euclid(SUB_CHUNK_HEIGHT)
                .rem_euclid(CHUNK_WIDTH);
            let start_z = start_index
                .div_euclid(SUB_CHUNK_HEIGHT * CHUNK_WIDTH)
                .rem_euclid(CHUNK_WIDTH);
            let start = [start_x, start_y, start_z];

            // Skip to first valid, unvisited coordinate
            if check_mask(&visited, &start) || !check_mask(&self.full_block_mask, &start) {
                continue;
            }

            // End point exclusive
            let mut end = [start_x + 1, start_y + 1, start_z + 1];

            // For each axis find maximum expansion
            for axis in 0usize..3usize {
                let u_axis: usize = (axis + 1).rem_euclid(3);
                let v_axis: usize = (axis + 2).rem_euclid(3);

                loop {
                    let mut valid = true;
                    'check_expansion: for u in start[u_axis]..end[u_axis] {
                        for v in start[u_axis]..end[u_axis] {
                            let mut pos = end.clone();
                            pos[u_axis] = u;
                            pos[v_axis] = v;

                            if check_mask(&visited, &start)
                                || !check_mask(&self.full_block_mask, &start)
                            {
                                valid = false;
                                break 'check_expansion;
                            } else {
                                set_mask(&mut visited, &pos, true);
                            }
                        }
                    }
                    if valid {
                        // Update visited mask with new blocks
                        for u in start[u_axis]..end[u_axis] {
                            for v in start[u_axis]..end[u_axis] {
                                let mut pos = end.clone();
                                pos[u_axis] = u;
                                pos[v_axis] = v;
                                set_mask(&mut visited, &pos, true);
                            }
                        }
                        end[axis] += 1;
                    } else {
                        break;
                    }
                }
            }
            dbg!(start, end);
        }
    }

    fn remove_overlap_floor(&self, floor: &mut Vec<NavMeshLayer>) {
        // remove needless overlap for inflated full blocks first
        // doesn't work

        // for layer in floor {
        //     let mut updated_aabbs = vec![];
        //     for node in layer.nodes.iter() {
        //         if node.aabb
        //             == (Aabb2D {
        //                 min_x: 0.3,
        //                 max_x: 1.3,
        //                 min_y: 0.3,
        //                 max_y: 1.3,
        //             })
        //         {
        //             let mut updated_aabb = node.aabb.clone();
        //             let mut self_array = updated_aabb.to_array();

        //             for dir in 0..=1 {
        //                 for axis in 0..=1 {
        //                     let target_x = ((1 - axis) * (2 * dir - 1)) as usize;
        //                     let target_z = (axis * (2 * dir - 1)) as usize;

        //                     for aabb in layer.blocks[target_z][target_x].clone() {
        //                         let other_array = &layer.nodes[aabb].aabb.to_array();

        //                         if (dir == 0 && other_array[1][axis] >= 1.0)
        //                             || (dir == 1 && other_array[0][axis] <= 0.0)
        //                         {
        //                             if other_array[0][1 - axis] == self_array[0][1 - axis]
        //                                 && other_array[1][1 - axis] == self_array[1][1 - axis]
        //                             {
        //                                 self_array[dir][axis] =
        //                                     self_array[dir][axis].clamp(0.0, 1.0);
        //                                 updated_aabb = self_array.into();
        //                             }
        //                         }
        //                     }
        //                 }
        //             }
        //             updated_aabbs.push(updated_aabb);
        //         }
        //     }
        //     for (node, aabb) in layer.nodes.iter_mut().zip(updated_aabbs.into_iter()) {
        //         node.aabb = aabb;
        //     }
        // }

        // reduce other sources of overlapping
    }

    fn cut_floor(&self, floor: &mut Vec<NavMeshLayer>) {
        for layer in floor.iter_mut() {
            let height = layer.height;
            let cut_indices = (height.max(0.0) as usize)
                ..(((height + 1.8).ceil() + 0.1) as usize).min(SUB_CHUNK_HEIGHT - 1);

            let mut new_nodes: Vec<(UVec2, Aabb2D)> = vec![];
            for block in layer.blocks.iter_mut().flatten() {
                block.clear();
            }

            for node in layer.nodes.drain(0..layer.nodes.len()) {
                let mut cutting_stack = vec![node.aabb.clone()];
                'next_aabb: while let Some(aabb) = cutting_stack.pop() {
                    for cut_layer in cut_indices.clone() {
                        for sample_z in (node.pos.y as isize - 1).max(0isize)
                            ..=(node.pos.y as isize + 1).min(CHUNK_WIDTH as isize - 1)
                        {
                            for sample_x in (node.pos.x as isize - 1).max(0isize)
                                ..=(node.pos.x as isize + 1).min(CHUNK_WIDTH as isize - 1)
                            {
                                for (cutting_aabb_pos, cutting_aabb) in self.blocks
                                    [sample_z as usize][sample_x as usize][cut_layer]
                                    .iter()
                                    .map(|index| &self.aabbs[*index])
                                {
                                    let cutting_aabb_offset = IVec2 {
                                        x: cutting_aabb_pos.x as i32,
                                        y: cutting_aabb_pos.z as i32,
                                    } - node.pos.as_ivec2();

                                    if cut_layer as f32 + cutting_aabb.min_y() - 1.8 < height
                                        && height < cut_layer as f32 + cutting_aabb.max_y()
                                    {
                                        let cut = aabb.subtract(
                                            &cutting_aabb
                                                .surface_projection(1)
                                                .translate(cutting_aabb_offset.as_vec2())
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
                    _adjacent: smallvec![],
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

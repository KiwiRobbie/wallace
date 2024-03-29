use azalea::core::aabb::AABB;
use bevy::{math::vec3, pbr::ExtendedMaterial, prelude::*, render::mesh::Indices, utils::HashMap};
use tokio::sync::mpsc::{Receiver, Sender};
use wallace::{
    aabb::{
        debug_aabb_material::DebugAabbMaterial,
        debug_surface_material::DebugSurfaceMaterial,
        optimise_world::{SubChunk, SubChunkNavMesh, SUB_CHUNK_SIZE},
    },
    tools::mesh_builder::MeshBuilder,
};

#[derive(Resource)]
pub struct BotDebugChannels {
    pub tx: Sender<OutboundDebugVisEvent>,
    pub rx: Receiver<InboundDebugVisEvent>,
}

pub struct DebugVisPlugin;

impl Plugin for DebugVisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, debug_vis_system);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::hsl(0.074475f32, 0.15f32, 0.8f32),
            illuminance: 100000f32,
            shadows_enabled: true,
            ..Default::default()
        },

        transform: Transform::IDENTITY.looking_to(vec3(-0.25, -1.0, -0.5), Vec3::Y),
        ..Default::default()
    },));

    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.5, 0.75, 1.0),
        brightness: 0.6,
    });
}
#[derive(Component)]
struct CollisionVisMarker;

struct PlayerPath {
    pub bot: bool,
    pub path: Vec<Vec3>,
}

fn debug_vis_system(
    mut commands: Commands,
    mut bot_channels: ResMut<BotDebugChannels>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut debug_surface_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, DebugSurfaceMaterial>>,
    >,
    mut debug_aabb_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, DebugAabbMaterial>>>,
    mut player_paths: Local<HashMap<[u8; 16], PlayerPath>>,
    mut gizmos: Gizmos,
    q_collision_vis: Query<Entity, With<CollisionVisMarker>>,
) {
    while let Ok(event) = bot_channels.rx.try_recv() {
        match event {
            InboundDebugVisEvent::PlayerPosition { uuid, pos, bot } => {
                player_paths
                    .entry(uuid)
                    .or_insert(PlayerPath { bot, path: vec![] })
                    .path
                    .push(Vec3 {
                        x: pos.0 as f32,
                        y: pos.1 as f32,
                        z: pos.2 as f32,
                    });
            }
            InboundDebugVisEvent::Clear => {
                for entity in q_collision_vis.iter() {
                    commands.entity(entity).despawn();
                }
                player_paths.clear();
            }
            InboundDebugVisEvent::AddCollisions { blocks } => {
                let mut collider_mesh_builder = MeshBuilder::new();
                let mut nav_mesh_builder = MeshBuilder::new();

                for DebugBlock { x, y, z, aabbs } in blocks {
                    for aabb in aabbs {
                        collider_mesh_builder.add_mesh(
                            &shape::Box {
                                min_x: aabb.min_x as f32,
                                min_y: aabb.min_y as f32,
                                min_z: aabb.min_z as f32,
                                max_x: aabb.max_x as f32,
                                max_y: aabb.max_y as f32,
                                max_z: aabb.max_z as f32,
                            }
                            .into(),
                            Transform::from_translation(Vec3 {
                                x: x as f32,
                                y: y as f32,
                                z: z as f32,
                            }),
                        );

                        nav_mesh_builder.add_mesh(
                            &shape::Box {
                                min_x: aabb.min_x as f32 - 0.3f32,
                                min_y: aabb.min_y as f32 - 1.8f32,
                                min_z: aabb.min_z as f32 - 0.3f32,
                                max_x: aabb.max_x as f32 + 0.3f32,
                                max_y: aabb.max_y as f32 + 0.0f32,
                                max_z: aabb.max_z as f32 + 0.3f32,
                            }
                            .into(),
                            Transform::from_translation(Vec3 {
                                x: x as f32,
                                y: y as f32,
                                z: z as f32,
                            }),
                        );
                    }
                }

                commands.spawn((
                    CollisionVisMarker,
                    PbrBundle {
                        mesh: meshes.add(collider_mesh_builder.build()),
                        material: materials.add(Color::WHITE.into()),
                        ..default()
                    },
                ));

                commands.spawn((
                    CollisionVisMarker,
                    PbrBundle {
                        mesh: meshes.add(nav_mesh_builder.build()),
                        material: materials.add(StandardMaterial {
                            base_color: Color::Rgba {
                                red: 0.8,
                                green: 0.2,
                                blue: 0.2,
                                alpha: 0.25,
                            },
                            alpha_mode: AlphaMode::Blend,
                            ..Default::default()
                        }),

                        ..default()
                    },
                ));
            }
            InboundDebugVisEvent::NavMesh { sub_chunk_nav } => {
                let mut positions: Vec<[f32; 3]> = vec![];
                let mut indices: Vec<u32> = vec![];

                let mut index: u32 = 0u32;

                for layer in sub_chunk_nav.floor.iter() {
                    let y = layer.height;
                    for node in layer.nodes.iter() {
                        let aabb = &node.aabb;
                        let pos = Vec3 {
                            x: node.pos.x as f32,
                            y,
                            z: node.pos.y as f32,
                        };

                        positions.push(
                            (pos + Vec3 {
                                x: aabb.min_x,
                                y: 0.0,
                                z: aabb.min_y,
                            })
                            .to_array(),
                        );
                        positions.push(
                            (pos + Vec3 {
                                x: aabb.min_x,
                                y: 0.0,
                                z: aabb.max_y,
                            })
                            .to_array(),
                        );
                        positions.push(
                            (pos + Vec3 {
                                x: aabb.max_x,
                                y: 0.0,
                                z: aabb.max_y,
                            })
                            .to_array(),
                        );
                        positions.push(
                            (pos + Vec3 {
                                x: aabb.max_x,
                                y: 0.0,
                                z: aabb.min_y,
                            })
                            .to_array(),
                        );

                        indices.extend([index, index + 1, index + 2, index + 2, index + 3, index]);
                        index += 4;
                    }
                }
                let mut chunk_entity = commands.spawn(());

                let mut mesh =
                    Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_NORMAL,
                    vec![[0.0, 1.0, 0.0]; positions.len()],
                );
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

                mesh.set_indices(Some(Indices::U32(indices)));

                let build_collider = mesh.count_vertices() > 0;
                if build_collider {
                    if let Some(collider) = bevy_rapier3d::prelude::Collider::from_bevy_mesh(
                        &mesh,
                        &bevy_rapier3d::prelude::ComputedColliderShape::TriMesh,
                    ) {
                        chunk_entity.insert(collider);
                    }
                }

                let material = ExtendedMaterial {
                    base: StandardMaterial::from(Color::rgb(1.0, 1.0, 1.0)),
                    extension: DebugSurfaceMaterial { quantize_steps: 10 },
                };

                chunk_entity.insert((
                    CollisionVisMarker,
                    MaterialMeshBundle {
                        mesh: meshes.add(mesh),
                        material: debug_surface_materials.add(material),
                        transform: Transform::from_translation(
                            (sub_chunk_nav.location * SUB_CHUNK_SIZE).as_vec3(),
                        ),
                        ..default()
                    },
                ));
            }
            InboundDebugVisEvent::SubChunk { sub_chunk } => {
                let mut collider_mesh_builder = MeshBuilder::new();

                for (pos, aabb) in sub_chunk.iter_collisions() {
                    collider_mesh_builder.add_mesh(
                        &shape::Box {
                            min_x: aabb.min_x() as f32,
                            min_y: aabb.min_y() as f32,
                            min_z: aabb.min_z() as f32,
                            max_x: aabb.max_x() as f32,
                            max_y: aabb.max_y() as f32,
                            max_z: aabb.max_z() as f32,
                        }
                        .into(),
                        Transform::from_translation(pos.as_vec3()),
                    );
                }

                commands.spawn((
                    CollisionVisMarker,
                    MaterialMeshBundle {
                        mesh: meshes.add(collider_mesh_builder.build()),
                        material: debug_aabb_materials.add(ExtendedMaterial {
                            base: Color::WHITE.into(),
                            extension: DebugAabbMaterial { quantize_steps: 0 },
                        }),
                        transform: Transform::from_translation(
                            (sub_chunk.location * SUB_CHUNK_SIZE).as_vec3(),
                        ),
                        ..default()
                    },
                ));
            }
        }
    }

    for player in player_paths.values() {
        gizmos.linestrip(
            player.path.iter().map(|p| p.clone()),
            if player.bot { Color::GREEN } else { Color::RED },
        );
    }
}

pub struct DebugBlock {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub aabbs: Vec<AABB>,
}

pub enum InboundDebugVisEvent {
    Clear,
    AddCollisions {
        blocks: Vec<DebugBlock>,
    },
    PlayerPosition {
        uuid: [u8; 16],
        pos: (f64, f64, f64),
        bot: bool,
    },
    SubChunk {
        sub_chunk: SubChunk,
    },
    NavMesh {
        sub_chunk_nav: SubChunkNavMesh,
    },
}
pub enum OutboundDebugVisEvent {}

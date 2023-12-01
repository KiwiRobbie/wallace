use std::f32::consts::PI;

use azalea::core::aabb::AABB;
use bevy::{math::vec3, prelude::*, utils::HashMap};
use tokio::sync::mpsc::{Receiver, Sender};
use wallace::{
    aabb::optimise_world::{
        SubChunk, SubChunkNavMesh, CHUNK_WIDTH, SUB_CHUNK_HEIGHT, SUB_CHUNK_SIZE,
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

fn debug_vis_system(
    mut commands: Commands,
    mut bot_channels: ResMut<BotDebugChannels>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut bot_paths: Local<HashMap<[u8; 16], Vec<Vec3>>>,
    mut gizmos: Gizmos,
    q_collision_vis: Query<Entity, With<CollisionVisMarker>>,
) {
    while let Ok(event) = bot_channels.rx.try_recv() {
        match event {
            InboundDebugVisEvent::PlayerPosition { uuid, pos } => {
                bot_paths.entry(uuid).or_insert(vec![]).push(Vec3 {
                    x: pos.0 as f32,
                    y: pos.1 as f32,
                    z: pos.2 as f32,
                });
            }
            InboundDebugVisEvent::Clear => {
                for entity in q_collision_vis.iter() {
                    commands.entity(entity).despawn();
                }
                bot_paths.clear();
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
                let mut collider_mesh_builder = MeshBuilder::new();

                for layer in sub_chunk_nav.floor.iter() {
                    let y = layer.height;
                    for node in layer.nodes.iter() {
                        let aabb = &node.aabb;
                        let (x, z) = (node.pos.x, node.pos.y);

                        let transform = Transform::from_translation(Vec3 {
                            x: x as f32 + 0.5,
                            y,
                            z: z as f32 + 0.5,
                        });

                        let size = Vec2 {
                            x: aabb.max_x - aabb.min_x,
                            y: aabb.max_y - aabb.min_y,
                        };

                        collider_mesh_builder.add_mesh(
                            &shape::Quad { size, flip: false }.into(),
                            transform
                                * Transform::from_rotation(
                                    Quat::from_rotation_x(-PI / 2.0)
                                        * Quat::from_rotation_z(-PI / 2.0),
                                ),
                        );
                    }
                }

                let mesh = collider_mesh_builder.build();
                let build_collider = mesh.count_vertices() > 0;

                let mut chunk_entity = commands.spawn(());

                if build_collider {
                    if let Some(collider) = bevy_rapier3d::prelude::Collider::from_bevy_mesh(
                        &mesh,
                        &bevy_rapier3d::prelude::ComputedColliderShape::TriMesh,
                    ) {
                        chunk_entity.insert(collider);
                    }
                }

                chunk_entity.insert((
                    CollisionVisMarker,
                    PbrBundle {
                        mesh: meshes.add(mesh),
                        material: materials.add(Color::rgb(0.4, 0.6, 0.4).into()),
                        transform: Transform::from_translation(
                            (sub_chunk_nav.location * SUB_CHUNK_SIZE).as_vec3(),
                        ),
                        ..default()
                    },
                ));
            }
            InboundDebugVisEvent::SubChunk { sub_chunk } => {
                // println!("[VIS] Received sub chunk");
                // let mut collider_mesh_builder = MeshBuilder::new();
                // for (UVec3 { x, y, z }, aabbs) in sub_chunk.iter_floor() {
                //     for aabb in aabbs {
                //         let transform = Transform::from_translation(Vec3 {
                //             x: x as f32 + 0.5,
                //             y: y as f32 + aabb.max_y(),
                //             z: z as f32 + 0.5,
                //         });

                //         let size = Vec2 {
                //             x: aabb.max_x() - aabb.min_x(),
                //             y: aabb.max_z() - aabb.min_z(),
                //         };

                //         collider_mesh_builder.add_mesh(
                //             &shape::Quad { size, flip: false }.into(),
                //             transform * Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)),
                //         );
                //     }
                // }

                // let mesh = collider_mesh_builder.build();
                // let build_collider = mesh.count_vertices() > 0;

                // let mut chunk_entity = commands.spawn(());

                // if build_collider {
                //     if let Some(collider) = bevy_rapier3d::prelude::Collider::from_bevy_mesh(
                //         &mesh,
                //         &bevy_rapier3d::prelude::ComputedColliderShape::TriMesh,
                //     ) {
                //         chunk_entity.insert(collider);
                //     }
                // }

                // chunk_entity.insert((
                //     CollisionVisMarker,
                //     PbrBundle {
                //         mesh: meshes.add(mesh),
                //         material: materials.add(Color::rgb(0.2, 0.8, 0.2).into()),
                //         transform: Transform::from_translation(
                //             (sub_chunk.location * SUB_CHUNK_SIZE).as_vec3(),
                //         ),
                //         ..default()
                //     },
                // ));

                // // commands.spawn((
                // //     CollisionVisMarker,
                // //     PbrBundle {
                // //         mesh: meshes.add(nav_mesh_builder.build()),
                // //         material: materials.add(StandardMaterial {
                // //             base_color: Color::Rgba {
                // //                 red: 0.8,
                // //                 green: 0.2,
                // //                 blue: 0.2,
                // //                 alpha: 0.25,
                // //             },
                // //             alpha_mode: AlphaMode::Blend,
                // //             ..Default::default()
                // //         }),

                // //         ..default()
                // //     },
                // // ));
            }
        }
    }

    for path in bot_paths.values() {
        gizmos.linestrip(path.iter().map(|p| p.clone()), Color::GREEN);
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
    },
    SubChunk {
        sub_chunk: SubChunk,
    },
    NavMesh {
        sub_chunk_nav: SubChunkNavMesh,
    },
}
pub enum OutboundDebugVisEvent {}

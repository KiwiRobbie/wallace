use azalea::core::aabb::AABB;
use bevy::{math::vec3, prelude::*, utils::HashMap};
use bevy_rapier3d::na::coordinates::X;
use tokio::sync::mpsc::{Receiver, Sender};
use wallace::tools::mesh_builder::MeshBuilder;

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
            InboundDebugVisEvent::ClearCollision => {
                for entity in q_collision_vis.iter() {
                    commands.entity(entity).despawn();
                }
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
                                min_y: aabb.min_y as f32 - 0.8f32,
                                min_z: aabb.min_z as f32 - 0.3f32,
                                max_x: aabb.max_x as f32 + 0.3f32,
                                max_y: aabb.max_y as f32 + 1.0f32,
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
    ClearCollision,
    AddCollisions {
        blocks: Vec<DebugBlock>,
    },
    PlayerPosition {
        uuid: [u8; 16],
        pos: (f64, f64, f64),
    },
}
pub enum OutboundDebugVisEvent {}

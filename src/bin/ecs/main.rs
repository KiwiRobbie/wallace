use azalea_physics::collision::BlockWithShape;

use azalea::{
    app::{Plugin, Update},
    chat::{ChatPacket, ChatReceivedEvent, SendChatEvent},
    core::aabb::AABB,
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        query::{Added, With},
        system::{Commands, Query, Res, ResMut},
    },
    entity::{metadata::Player, EntityUuid, LocalEntity, Position},
    pathfinder::{
        goals::BlockPosGoal,
        moves::{self},
        ComputePath, ExecutingPath, GotoEvent, StopPathfindingEvent,
    },
    prelude::*,
    world::{InstanceContainer, InstanceName, MinecraftEntityId},
    BlockPos, PlayerInfo,
};
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{channel, Receiver, Sender};

const OWNER: [u8; 16] = [
    0xaa, 0xf3, 0x72, 0x32, 0x31, 0x93, 0x43, 0x6b, 0xaa, 0x5b, 0x2b, 0x2b, 0x2e, 0xd0, 0xd1,
    0x4d, // Single player UUID
         // 128, 128, 216, 237, 206, 100, 50, 223, 138, 37, 96, 105, 111, 220, 59, 88, // Server UUID
];

mod vis;
use vis::{
    BotDebugChannels, DebugBlock, DebugVisPlugin, InboundDebugVisEvent, OutboundDebugVisEvent,
};

fn main() {
    let (vis_tx, bot_rx) = channel(100);
    let (bot_tx, vis_rx) = channel(100);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let account = Account::offline("CGO55CREGY");
            ClientBuilder::new()
                .add_plugins(ChatControlPlugin {
                    owner: OWNER,
                    debug: Mutex::new(Some(DebugVisChannels {
                        tx: bot_tx,
                        rx: bot_rx,
                    })),
                })
                .start(account, "127.0.0.1")
                // .start(account, "192.168.1.28")
                .await
                .unwrap();
        });
    });
    bevy::app::App::new()
        .insert_resource(BotDebugChannels {
            tx: vis_tx,
            rx: vis_rx,
        })
        .add_plugins((
            bevy::prelude::DefaultPlugins,
            DebugVisPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            wallace::camera_plugin::SwitchingCameraPlugin,
        ))
        .run();
}

#[derive(Resource)]
struct DebugVisChannels {
    tx: Sender<InboundDebugVisEvent>,
    rx: Receiver<OutboundDebugVisEvent>,
}

struct ChatControlPlugin {
    owner: [u8; 16],
    debug: Mutex<Option<DebugVisChannels>>,
}

#[derive(Resource)]
struct OwnerUuid([u8; 16]);

#[derive(Component)]
struct OwnerMarker;

impl Plugin for ChatControlPlugin {
    fn build(&self, app: &mut azalea::app::App) {
        let debug = self.debug.lock().unwrap().take().unwrap();

        app.insert_resource(debug)
            .add_systems(
                Update,
                (
                    login_system,
                    debug_position,
                    follow_system,
                    chat_follow_system,
                    update_owner_system,
                ),
            )
            .insert_resource(OwnerUuid(self.owner));
    }
}

fn update_owner_system(
    mut commands: Commands,
    owner: Res<OwnerUuid>,
    q_owner: Query<&Player, With<OwnerMarker>>,
    q_players: Query<(Entity, &EntityUuid), With<Player>>,
) {
    if q_owner.get_single().is_ok() {
    } else {
        for (entity, uuid) in q_players.iter() {
            if uuid.as_bytes() == &owner.0 {
                commands.entity(entity).insert(OwnerMarker);
            }
        }
    }
}

#[derive(Component)]
struct BotMarker;

fn login_system(
    mut commands: Commands,
    query: Query<Entity, (Added<MinecraftEntityId>, With<LocalEntity>)>,
    mut chat_events: EventWriter<SendChatEvent>,
) {
    for entity_id in &query {
        commands.entity(entity_id).insert(BotMarker);
        // chat_events.send(SendChatEvent {
        //     entity: entity_id,
        //     content: "/login N333LE744WV5RBNW".to_string(),
        // });
    }
}

#[derive(Component)]
struct FollowTargetMarker;

fn filter_auth_chat_content(
    msg: &ChatReceivedEvent,
    owner_uuid: &OwnerUuid,
) -> Option<(Entity, String)> {
    if let ChatPacket::Player(packet) = &msg.packet {
        if packet.sender.as_bytes() == &owner_uuid.0 {
            return Some((msg.entity, packet.content().to_ansi()));
        }
    }
    None
}

fn debug_position(
    q_player: Query<(&EntityUuid, &Position), With<BotMarker>>,
    debug_vis: ResMut<DebugVisChannels>,
) {
    for (uuid, pos) in q_player.iter() {
        debug_vis
            .tx
            .blocking_send(InboundDebugVisEvent::PlayerPosition {
                uuid: uuid.as_bytes().clone(),
                pos: (pos.x, pos.y, pos.z),
            })
            .unwrap();
    }
}

fn chat_follow_system(
    mut commands: Commands,
    mut chat_events: EventReader<ChatReceivedEvent>,
    owner_uuid: Res<OwnerUuid>,
    q_owner: Query<(Entity, &Position), With<OwnerMarker>>,
    q_position: Query<&Position>,
    mut ev_goto: EventWriter<GotoEvent>,
    mut ev_stop: EventWriter<StopPathfindingEvent>,
    q_following: Query<Entity, With<FollowTargetMarker>>,
    q_instance_name: Query<&InstanceName>,
    instance_container: Res<InstanceContainer>,
    debug_vis: ResMut<DebugVisChannels>,
) {
    for (client, content) in chat_events
        .read()
        .flat_map(|msg| filter_auth_chat_content(msg, owner_uuid.as_ref()))
    {
        let mut cmd = content.as_str().split_ascii_whitespace().peekable();
        match cmd.next() {
            Some("come") if cmd.peek().is_none() => {
                if let Ok((_, pos)) = q_owner.get_single() {
                    ev_goto.send(GotoEvent {
                        entity: client,
                        goal: Arc::new(BlockPosGoal(pos.into())),
                        successors_fn: moves::default_move,
                    })
                }
            }
            Some("follow") if cmd.peek().is_none() => {
                if let Ok((owner_entity, _)) = q_owner.get_single() {
                    commands.entity(owner_entity).insert(FollowTargetMarker);
                }
            }
            Some("dbg") => match cmd.next() {
                Some("clear") if cmd.peek().is_none() => {
                    debug_vis
                        .tx
                        .blocking_send(InboundDebugVisEvent::ClearCollision)
                        .unwrap();
                }
                Some("shape") => {
                    let radius = cmd.next().and_then(|r| r.parse::<i32>().ok()).unwrap_or(2);
                    if cmd.peek().is_none() {
                        let client_position: BlockPos = q_position
                            .get(client)
                            .expect("Couldn't get client position")
                            .clone()
                            .into();

                        let world_name = q_instance_name
                            .get(client)
                            .expect("Couldn't get world name");
                        let world_lock = instance_container
                            .get(&world_name)
                            .expect("Couldn't get instance");

                        let world = world_lock.read();

                        let mut blocks = vec![];

                        for i in -radius..radius {
                            for j in -radius..radius {
                                for k in -radius..radius {
                                    let block_pos = client_position + BlockPos { x: i, y: j, z: k };
                                    let block = world.get_block_state(&block_pos);
                                    if let Some(block) = block {
                                        let block_shape = block.shape().to_aabbs();
                                        blocks.push(DebugBlock {
                                            x: block_pos.x,
                                            y: block_pos.y,
                                            z: block_pos.z,
                                            aabbs: block_shape,
                                        })
                                    }
                                }
                            }
                        }

                        debug_vis
                            .tx
                            .blocking_send(InboundDebugVisEvent::AddCollisions { blocks })
                            .unwrap();
                    }
                }

                _ => {}
            },
            Some("stop") if cmd.peek().is_none() => {
                for entity in q_following.iter() {
                    commands.entity(entity).remove::<FollowTargetMarker>();
                }

                ev_stop.send(StopPathfindingEvent {
                    entity: client,
                    force: false,
                });
            }
            _ => {}
        }
    }
}

fn follow_system(
    q_computing: Query<Entity, With<ComputePath>>,
    q_pathfinding: Query<Entity, With<ExecutingPath>>,
    q_target: Query<&Position, With<FollowTargetMarker>>,
    q_bot: Query<Entity, With<BotMarker>>,
    // mut ev_stop: EventWriter<StopPathfindingEvent>,
    mut ev_goto: EventWriter<GotoEvent>,
) {
    let computing = q_computing.iter().len();
    let pathfinding = q_pathfinding.iter().len();

    if computing == 0 {
        if let Ok(pos) = q_target.get_single() {
            for bot_entity in q_bot.iter() {
                ev_goto.send(GotoEvent {
                    entity: bot_entity,
                    goal: Arc::new(BlockPosGoal(pos.into())),
                    successors_fn: moves::default_move,
                })
            }
        }
    }
    // if computing > 1 {
    //     for entity in q_computing.iter().chain(q_pathfinding.iter()) {
    //         ev_stop.send(StopPathfindingEvent {
    //             entity,
    //             force: true,
    //         })
    //     }
    // }
    // print!(
    //     "{}{} computing, pathfinding {}",
    //     cursor::Goto(1, 1),
    //     computing,
    //     pathfinding,
    // );
}

// async fn handle(mut bot: Client, event: Event, state: State) -> anyhow::Result<()> {
//     match event {
//         Event::Login => {
//             // bot.chat("/register N333LE744WV5RBNW N333LE744WV5RBNW");
//             bot.chat("/login N333LE744WV5RBNW");
//         }

//         Event::Chat(m) => match m {
//             ChatPacket::Player(m) => {
//                 if m.sender.as_bytes() == &OWNER {
//                     let msg = m.content();
//                     println!("{}", msg);
//                     if let Some(target) = m.content().to_ansi().strip_prefix("goto ") {
//                         if let Some((x, z)) = target.split_once(" ") {
//                             if let (Ok(x), Ok(z)) = (x.parse(), z.parse()) {
//                                 bot.goto(XZGoal { x, z })
//                             }
//                         }
//                     } else {
//                         println!("Malformed goto!");
//                     }

//                     if m.content().to_ansi() == "come" {
//                         println!("coming!");
//                         let sender_uuid = m.sender;
//                         if let Some(sender_entity) = bot
//                             .entity_by::<With<Player>, (&GameProfileComponent,)>(
//                                 |(profile,): &(&GameProfileComponent,)| profile.uuid == sender_uuid,
//                             )
//                         {
//                             let goal = bot.entity_component::<Position>(sender_entity).into();
//                             bot.goto(BlockPosGoal(goal))
//                         } else {
//                             println!("Couldn't find player {:?}", sender_uuid);
//                         }
//                     }

//                     if m.content().to_ansi() == "find bed" {
//                         println!("finding bed!");

//                         let world = bot.world();
//                         let world = world.read();

//                         let beds: [BlockState; 16] = [
//                             Block::WhiteBed.into(),
//                             Block::OrangeBed.into(),
//                             Block::MagentaBed.into(),
//                             Block::LightBlueBed.into(),
//                             Block::YellowBed.into(),
//                             Block::LimeBed.into(),
//                             Block::PinkBed.into(),
//                             Block::GrayBed.into(),
//                             Block::LightGrayBed.into(),
//                             Block::CyanBed.into(),
//                             Block::PurpleBed.into(),
//                             Block::BlueBed.into(),
//                             Block::BrownBed.into(),
//                             Block::GreenBed.into(),
//                             Block::RedBed.into(),
//                             Block::BlackBed.into(),
//                         ];

//                         let bed_block_states: BlockStates = BlockStates { set: beds.into() };
//                         if let Some(bed_pos) = world.find_block(bot.position(), &bed_block_states) {
//                             bot.goto(BlockPosGoal(bed_pos));
//                             bot.chat(format!("Found bed at {bed_pos:?}").as_str());
//                         }
//                     }

//                     if m.content().to_ansi() == "sleep" {
//                         println!("sleeping!");

//                         let world = bot.world();
//                         let world = world.read();

//                         let beds: [BlockState; 16] = [
//                             Block::WhiteBed.into(),
//                             Block::OrangeBed.into(),
//                             Block::MagentaBed.into(),
//                             Block::LightBlueBed.into(),
//                             Block::YellowBed.into(),
//                             Block::LimeBed.into(),
//                             Block::PinkBed.into(),
//                             Block::GrayBed.into(),
//                             Block::LightGrayBed.into(),
//                             Block::CyanBed.into(),
//                             Block::PurpleBed.into(),
//                             Block::BlueBed.into(),
//                             Block::BrownBed.into(),
//                             Block::GreenBed.into(),
//                             Block::RedBed.into(),
//                             Block::BlackBed.into(),
//                         ];

//                         let bed_block_states: BlockStates = BlockStates { set: beds.into() };
//                         if let Some(bed_pos) = world.find_block(bot.position(), &bed_block_states) {
//                             bot.look_at(
//                                 bed_pos.to_vec3_floored()
//                                     + Vec3 {
//                                         x: 0.5,
//                                         y: 0.5,
//                                         z: 0.5,
//                                     },
//                             );
//                             bot.block_interact(bed_pos);
//                         }
//                     }

//                     if m.content().to_ansi() == "follow" {
//                         *state.following.lock() = true;
//                     }
//                     if m.content().to_ansi() == "stop" {
//                         *state.following.lock() = false;
//                         bot.stop_pathfinding();
//                     }
//                 }
//             }
//             _ => {}
//         },
//         Event::Tick => {
//             let mut following = state.following.lock();
//             if *following {
//                 let computing = {
//                     let mut ecs = bot.ecs.lock();
//                     ecs.query::<(Entity, &mut ComputePath)>().iter(&ecs).len() > 0
//                 };

//                 if !computing {
//                     if let Some(sender_entity) = bot
//                         .entity_by::<With<Player>, (&GameProfileComponent,)>(
//                             |(profile,): &(&GameProfileComponent,)| {
//                                 profile.uuid.as_bytes() == &OWNER
//                             },
//                         )
//                     {
//                         println!("updating path");
//                         let goal = bot.entity_component::<Position>(sender_entity).into();
//                         bot.goto(BlockPosGoal(goal));
//                     } else {
//                         println!("Couldn't find player!");
//                         *following = false;
//                     }
//                 }
//             }
//         }
//         _ => {}
//     }

//     Ok(())
// }

//! A bot that logs chat messages sent in the server to the console.

use std::sync::Arc;

use azalea::{
    blocks::{BlockState, BlockStates},
    chat::ChatPacket,
    ecs::{entity::Entity, query::With, system::Query},
    entity::{metadata::Player, EntityUuid, Position},
    pathfinder::{
        goals::{BlockPosGoal, XZGoal},
        ComputePath,
    },
    prelude::*,
    registry::tags::blocks::AZALEA_GROWS_ON,
    GameProfileComponent,
};
use azalea::{registry::Block, Vec3};
use parking_lot::Mutex;
use uuid::Uuid;
#[tokio::main]
async fn main() {
    let account = Account::offline("CGO55CREGY");
    loop {
        let e = ClientBuilder::new()
            .set_handler(handle)
            .start(account.clone(), "192.168.1.28")
            .await;
        eprintln!("{e:?}");
    }
}

#[derive(Default, Clone, Component)]
pub struct State {
    following: Arc<Mutex<bool>>,
}
const OWNER: [u8; 16] = [
    128, 128, 216, 237, 206, 100, 50, 223, 138, 37, 96, 105, 111, 220, 59, 88,
];

async fn handle(mut bot: Client, event: Event, state: State) -> anyhow::Result<()> {
    match event {
        Event::Login => {
            // bot.chat("/register N333LE744WV5RBNW N333LE744WV5RBNW");
            bot.chat("/login N333LE744WV5RBNW");
        }

        Event::Chat(m) => match m {
            ChatPacket::Player(m) => {
                if m.sender.as_bytes() == &OWNER {
                    let msg = m.content();
                    println!("{}", msg);
                    if let Some(target) = m.content().to_ansi().strip_prefix("goto ") {
                        if let Some((x, z)) = target.split_once(" ") {
                            if let (Ok(x), Ok(z)) = (x.parse(), z.parse()) {
                                bot.goto(XZGoal { x, z })
                            }
                        }
                    } else {
                        println!("Malformed goto!");
                    }

                    if m.content().to_ansi() == "come" {
                        println!("coming!");
                        let sender_uuid = m.sender;
                        if let Some(sender_entity) = bot
                            .entity_by::<With<Player>, (&GameProfileComponent,)>(
                                |(profile,): &(&GameProfileComponent,)| profile.uuid == sender_uuid,
                            )
                        {
                            let goal = bot.entity_component::<Position>(sender_entity).into();
                            bot.goto(BlockPosGoal(goal))
                        } else {
                            println!("Couldn't find player {:?}", sender_uuid);
                        }
                    }

                    if m.content().to_ansi() == "find bed" {
                        println!("finding bed!");

                        let world = bot.world();
                        let world = world.read();

                        let beds: [BlockState; 16] = [
                            Block::WhiteBed.into(),
                            Block::OrangeBed.into(),
                            Block::MagentaBed.into(),
                            Block::LightBlueBed.into(),
                            Block::YellowBed.into(),
                            Block::LimeBed.into(),
                            Block::PinkBed.into(),
                            Block::GrayBed.into(),
                            Block::LightGrayBed.into(),
                            Block::CyanBed.into(),
                            Block::PurpleBed.into(),
                            Block::BlueBed.into(),
                            Block::BrownBed.into(),
                            Block::GreenBed.into(),
                            Block::RedBed.into(),
                            Block::BlackBed.into(),
                        ];

                        let bed_block_states: BlockStates = BlockStates { set: beds.into() };
                        if let Some(bed_pos) = world.find_block(bot.position(), &bed_block_states) {
                            bot.goto(BlockPosGoal(bed_pos));
                            bot.chat(format!("Found bed at {bed_pos:?}").as_str());
                        }
                    }

                    if m.content().to_ansi() == "sleep" {
                        println!("sleeping!");

                        let world = bot.world();
                        let world = world.read();

                        let beds: [BlockState; 16] = [
                            Block::WhiteBed.into(),
                            Block::OrangeBed.into(),
                            Block::MagentaBed.into(),
                            Block::LightBlueBed.into(),
                            Block::YellowBed.into(),
                            Block::LimeBed.into(),
                            Block::PinkBed.into(),
                            Block::GrayBed.into(),
                            Block::LightGrayBed.into(),
                            Block::CyanBed.into(),
                            Block::PurpleBed.into(),
                            Block::BlueBed.into(),
                            Block::BrownBed.into(),
                            Block::GreenBed.into(),
                            Block::RedBed.into(),
                            Block::BlackBed.into(),
                        ];

                        let bed_block_states: BlockStates = BlockStates { set: beds.into() };
                        if let Some(bed_pos) = world.find_block(bot.position(), &bed_block_states) {
                            bot.look_at(
                                bed_pos.to_vec3_floored()
                                    + Vec3 {
                                        x: 0.5,
                                        y: 0.5,
                                        z: 0.5,
                                    },
                            );
                            bot.block_interact(bed_pos);
                        }
                    }

                    if m.content().to_ansi() == "follow" {
                        *state.following.lock() = true;
                    }
                    if m.content().to_ansi() == "stop" {
                        *state.following.lock() = false;
                        bot.stop_pathfinding();
                    }
                }
            }
            _ => {}
        },
        Event::Tick => {
            let mut following = state.following.lock();
            if *following {
                let computing = {
                    let mut ecs = bot.ecs.lock();
                    ecs.query::<(Entity, &mut ComputePath)>().iter(&ecs).len() > 0
                };

                if !computing {
                    if let Some(sender_entity) = bot
                        .entity_by::<With<Player>, (&GameProfileComponent,)>(
                            |(profile,): &(&GameProfileComponent,)| {
                                profile.uuid.as_bytes() == &OWNER
                            },
                        )
                    {
                        println!("updating path");
                        let goal = bot.entity_component::<Position>(sender_entity).into();
                        bot.goto(BlockPosGoal(goal));
                    } else {
                        println!("Couldn't find player!");
                        *following = false;
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

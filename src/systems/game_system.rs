use amethyst::core::ecs::shred::SetupHandler;
use amethyst::core::ecs::{DispatcherBuilder, Entity, ReadStorage, WriteStorage};
use amethyst::core::math::Vector3;
use amethyst::core::{SystemBundle, Transform};
use amethyst::ecs::{System, SystemData};
use amethyst::error::Error as AmethystError;
use amethyst::prelude::*;
use amethyst::renderer::palette::Srgba;
use amethyst::renderer::resources::Tint;
use amethyst::renderer::SpriteRender;
use crossbeam::channel;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use log::debug;

use crate::events::{GameRxEvent, GameTxEvent, InputEvent};
use crate::sprite_loader::Sprites;
use crate::sprite_loader::PIXEL_DIMENSION as ACTUAL_PIXEL_DIMENSION;
use crate::systems::game_system::BoardPixel::Filled;
use crate::systems::KnownSystems;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;

const PIXEL_DIMENSION: f32 = 50.;

struct UpdatedState {
    board_changed: bool,
    events: Vec<GameTxEvent>,
}

impl UpdatedState {
    fn rx_event(board_changed: bool, event: GameRxEvent) -> UpdatedState {
        UpdatedState {
            board_changed,
            events: vec![GameTxEvent::RxEvent(event)],
        }
    }
}

pub struct TetrisGameSystem {
    piece: Option<(usize, usize)>,
    board_state: [[BoardPixel; BOARD_HEIGHT]; BOARD_WIDTH],
    board_entities: [[Entity; BOARD_HEIGHT]; BOARD_WIDTH],
    in_rx: Receiver<GameRxEvent>,
    out_tx: Sender<GameTxEvent>,
}

impl TetrisGameSystem {
    fn receive(&mut self, event: GameRxEvent) -> UpdatedState {
        debug!("simulation received event: {:?}", event);

        match event {
            GameRxEvent::Input(input) => self.handle_input(input),
            GameRxEvent::Tick => self.tick(),
            GameRxEvent::AddRows(count) => self.add_rows(count),
        }
    }

    fn handle_input(&mut self, event: InputEvent) -> UpdatedState {
        if let Some((mut x, mut y)) = self.piece {
            match event {
                InputEvent::Left => {
                    x -= 1;
                    self.board_state[x][y] = Filled(PieceColor::LightBlue);
                }
                InputEvent::Right => {
                    x += 1;
                    self.board_state[x][y] = Filled(PieceColor::DarkBlue);
                }
                InputEvent::RotateClockwise => {
                    y += 1;
                    self.board_state[x][y] = Filled(PieceColor::Orange);
                }
                InputEvent::DropSoft => {
                    y -= 1;
                    self.board_state[x][y] = Filled(PieceColor::Green);
                }
                _ => (), // noop
            };
            self.piece = Some((x, y));

            UpdatedState::rx_event(true, GameRxEvent::Input(event))
        } else {
            UpdatedState {
                board_changed: false,
                events: vec![],
            }
        }
    }

    fn tick(&self) -> UpdatedState {
        UpdatedState::rx_event(true, GameRxEvent::Tick)
    }

    fn add_rows(&self, count: usize) -> UpdatedState {
        UpdatedState::rx_event(true, GameRxEvent::AddRows(count))
    }
}

impl<'s> System<'s> for TetrisGameSystem {
    // #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'s, Tint>,
        // TODO, we're not using using this, why did we need to import it?
        ReadStorage<'s, SpriteRender>,
    );

    fn run(&mut self, (mut tint_storage, _): Self::SystemData) {
        let mut any_board_changes = false;
        while let Ok(event) = self.in_rx.try_recv() {
            let UpdatedState {
                board_changed,
                events,
            } = self.receive(event);
            any_board_changes |= board_changed;

            // forward along all the events we found
            events.into_iter().for_each(|e| {
                self.out_tx
                    .send(e)
                    .expect("We should always be able to send this")
            });
        }

        if any_board_changes {
            for x in 0..BOARD_WIDTH {
                for y in 0..BOARD_HEIGHT {
                    let entity = self.board_entities[x][y];
                    let tint_color = self.board_state[x][y].into();

                    let tint = tint_storage
                        .get_mut(entity)
                        .expect("We should always have this entity");

                    tint.0 = tint_color;
                }
            }
        }
    }
}

pub struct GameChannels {
    pub player_in: Sender<GameRxEvent>,
    pub player_out: Receiver<GameTxEvent>,
    pub opponent_in: Sender<GameRxEvent>,
    pub opponent_out: Receiver<GameTxEvent>,
}

impl SetupHandler<GameChannels> for GameChannels {
    fn setup(_world: &mut World) {
        panic!("We should never initialize this, we should always build it in our Bundle")
    }
}

#[derive(Default)]
pub struct GameSystemBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for GameSystemBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), AmethystError> {
        let (player_in_tx, player_in_rx) = channel::unbounded();
        let (player_out_tx, player_out_rx) = channel::unbounded();

        let margin = PIXEL_DIMENSION / 2. + 20.;

        builder.add(
            TetrisGameSystemDesc {
                position: (margin, margin),
                in_rx: player_in_rx,
                out_tx: player_out_tx,
            }
            .build(world),
            "game_system_player",
            &[KnownSystems::SpriteLoader.into()],
        );

        let (opponent_in_tx, opponent_in_rx) = channel::unbounded();
        let (opponent_out_tx, opponent_out_rx) = channel::unbounded();

        builder.add(
            TetrisGameSystemDesc {
                position: ((PIXEL_DIMENSION * BOARD_WIDTH as f32) + margin * 2., margin),
                in_rx: opponent_in_rx,
                out_tx: opponent_out_tx,
            }
            .build(world),
            "game_system_opponent",
            &[KnownSystems::SpriteLoader.into()],
        );

        let channels = GameChannels {
            player_in: player_in_tx,
            player_out: player_out_rx,
            opponent_in: opponent_in_tx,
            opponent_out: opponent_out_rx,
        };
        world.insert(channels);

        Ok(())
    }
}

struct TetrisGameSystemDesc {
    position: (f32, f32),
    in_rx: Receiver<GameRxEvent>,
    out_tx: Sender<GameTxEvent>,
}

impl<'a, 'b> SystemDesc<'a, 'b, TetrisGameSystem> for TetrisGameSystemDesc {
    fn build(self, world: &mut World) -> TetrisGameSystem {
        // setup data we need to initialize, but not actually run
        <ReadStorage<'a, SpriteRender> as SystemData>::setup(&mut *world);

        <TetrisGameSystem as System<'_>>::SystemData::setup(world);

        let pixel_sprite = world.read_resource::<Sprites>().pixel_sprite.clone();

        let dummy_entity = world.create_entity().entity;

        let (offset_x, offset_y) = self.position;

        debug!("loading at {}, {}", offset_x, offset_y);

        let mut board_entities = [[dummy_entity; BOARD_HEIGHT]; BOARD_WIDTH];
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                let mut transform = Transform::default();
                transform.set_translation_xyz(
                    offset_x + (x as f32 * PIXEL_DIMENSION),
                    offset_y + (y as f32 * PIXEL_DIMENSION),
                    0.,
                );
                let scale = PIXEL_DIMENSION / ACTUAL_PIXEL_DIMENSION;
                transform.set_scale(Vector3::new(scale, scale, scale));

                let entity = world
                    .create_entity()
                    .with(pixel_sprite.clone())
                    .with(transform)
                    .with(Tint(BoardPixel::Empty.into()))
                    .build();

                board_entities[x][y] = entity;
            }
        }

        TetrisGameSystem {
            piece: Some((5, 5)),
            board_state: [[BoardPixel::Empty; BOARD_HEIGHT]; BOARD_WIDTH],
            board_entities,
            in_rx: self.in_rx,
            out_tx: self.out_tx,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum BoardPixel {
    Filled(PieceColor),
    Empty,
}

impl Into<Srgba> for BoardPixel {
    fn into(self) -> Srgba<f32> {
        match self {
            BoardPixel::Filled(PieceColor::LightBlue) => {
                Srgba::new(0. / 255., 230. / 255., 254. / 255., 1.0)
            }
            BoardPixel::Filled(PieceColor::DarkBlue) => {
                Srgba::new(24. / 255., 1. / 255., 255. / 255., 1.0)
            }
            BoardPixel::Filled(PieceColor::Orange) => {
                Srgba::new(255. / 255., 115. / 255., 8. / 255., 1.0)
            }
            BoardPixel::Filled(PieceColor::Yellow) => {
                Srgba::new(255. / 255., 222. / 255., 0. / 255., 1.0)
            }
            BoardPixel::Filled(PieceColor::Green) => {
                Srgba::new(102. / 255., 253. / 255., 0. / 255., 1.0)
            }
            BoardPixel::Filled(PieceColor::Red) => {
                Srgba::new(254. / 255., 16. / 255., 60. / 255., 1.0)
            }
            BoardPixel::Filled(PieceColor::Magenta) => {
                Srgba::new(184. / 255., 2. / 255., 253. / 255., 1.0)
            }
            BoardPixel::Empty => Srgba::new(0.05, 0.05, 0.05, 1.0),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum PieceColor {
    LightBlue,
    DarkBlue,
    Orange,
    Yellow,
    Green,
    Red,
    Magenta,
}

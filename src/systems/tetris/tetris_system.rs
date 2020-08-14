use amethyst::core::ecs::{Entity, ReadStorage, WriteStorage};
use amethyst::core::math::Vector3;
use amethyst::core::Transform;
use amethyst::ecs::{System, SystemData};
use amethyst::prelude::*;
use amethyst::renderer::palette::Srgba;
use amethyst::renderer::resources::Tint;
use amethyst::renderer::SpriteRender;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use log::debug;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::events::{GameRxEvent, GameTxEvent, InputEvent};
use crate::sprite_loader::Sprites;
use crate::sprite_loader::PIXEL_DIMENSION as ACTUAL_PIXEL_DIMENSION;
use crate::systems::tetris::{BoardPixel, Piece, PieceColor, Rotation, Tetrimino};

const RENDER_BOUNDING_BOX: bool = true;

const STAGING_HEIGHT: usize = 10;

const VISIBLE_WIDTH: usize = 10;
const VISIBLE_HEIGHT: usize = 20;

pub const BOARD_WIDTH: usize = VISIBLE_WIDTH;
const BOARD_HEIGHT: usize = VISIBLE_HEIGHT + STAGING_HEIGHT;

pub const PIXEL_DIMENSION: f32 = 50.;

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

    fn empty() -> UpdatedState {
        UpdatedState {
            board_changed: false,
            events: vec![],
        }
    }
}

pub struct TetrisGameSystem {
    running: bool,
    piece: Option<Piece>,
    board_state: [[BoardPixel; BOARD_HEIGHT]; BOARD_WIDTH],
    board_entities: [[Entity; VISIBLE_HEIGHT]; VISIBLE_WIDTH],
    piece_bag: Vec<Tetrimino>,
    rng: StdRng,
    in_rx: Receiver<GameRxEvent>,
    out_tx: Sender<GameTxEvent>,
}

impl TetrisGameSystem {
    fn receive(&mut self, event: GameRxEvent) -> bool {
        let UpdatedState {
            board_changed,
            events,
        } = self.handle_event(event);
        // forward along all the events we found
        events.into_iter().for_each(|e| {
            //debug!("Forwarding event: {:?}", e);

            self.out_tx
                .send(e)
                .expect("We should always be able to send this")
        });

        board_changed
    }

    fn handle_event(&mut self, event: GameRxEvent) -> UpdatedState {
        //debug!("Received event: {:?}", event);

        match event {
            GameRxEvent::Start(seed) => {
                self.running = true;
                // clear our board
                self.board_state = [[BoardPixel::Empty; BOARD_HEIGHT]; BOARD_WIDTH];
                self.rng = StdRng::seed_from_u64(seed);

                UpdatedState::rx_event(true, GameRxEvent::Start(seed))
            }
            GameRxEvent::Input(input) => {
                if self.running {
                    self.handle_input(input)
                } else {
                    UpdatedState::empty()
                }
            }
            GameRxEvent::Tick(id) => {
                if self.running {
                    self.tick(id)
                } else {
                    UpdatedState::empty()
                }
            }
            GameRxEvent::AddRows(count) => {
                if self.running {
                    self.add_rows(count)
                } else {
                    UpdatedState::empty()
                }
            }
        }
    }

    fn handle_input(&mut self, event: InputEvent) -> UpdatedState {
        if let Some(mut piece) = self.piece.clone() {
            let [valid_change, lock_piece] = match event {
                InputEvent::Left => {
                    piece.offset.0 -= 1;
                    [!self.check_collision(&piece), false]
                }
                InputEvent::Right => {
                    piece.offset.0 += 1;
                    [!self.check_collision(&piece), false]
                }
                InputEvent::RotateClockwise => {
                    let mut valid_rotated_piece = None;
                    for rotated_piece in piece.rotate(Rotation::Clockwise) {
                        if !self.check_collision(&rotated_piece) {
                            valid_rotated_piece = Some(rotated_piece);
                            break;
                        }
                    }

                    match valid_rotated_piece {
                        Some(rotated_piece) => {
                            piece = rotated_piece;
                            [true, false]
                        }
                        None => [false; 2],
                    }
                }
                InputEvent::DropSoft => {
                    piece.offset.1 -= 1;

                    // if we collided with something move the piece back and lock it
                    let lock = self.check_collision(&piece);
                    if lock {
                        piece.offset.1 += 1;
                    }

                    // this move is always valid
                    [true, lock]
                }
                _ => [false; 2],
            };

            if valid_change {
                if lock_piece {
                    self.lock_piece(piece);
                } else {
                    self.piece = Some(piece);
                }
                UpdatedState::rx_event(true, GameRxEvent::Input(event))
            } else {
                UpdatedState {
                    board_changed: false,
                    events: vec![],
                }
            }
        } else {
            UpdatedState {
                board_changed: false,
                events: vec![],
            }
        }

        // if let Some((mut x, mut y)) = self.piece {

        //     self.piece = Some((x, y));
        //
        //     UpdatedState::rx_event(true, GameRxEvent::Input(event))
        // } else {
        //     UpdatedState {
        //         board_changed: false,
        //         events: vec![],
        //     }
        // }
    }

    fn tick(&mut self, id: u64) -> UpdatedState {
        if let Some(mut piece) = self.piece.clone() {
            piece.offset.1 -= 1;

            // if we collided with something move the piece back and lock it
            let lock = self.check_collision(&piece);
            if lock {
                piece.offset.1 += 1;
                self.lock_piece(piece);

                self.piece = None;
            } else {
                self.piece = Some(piece)
            }
        } else {
            if self.piece_bag.is_empty() {
                // put all the pieces in the bag
                self.piece_bag = vec![
                    Tetrimino::I,
                    Tetrimino::J,
                    Tetrimino::L,
                    Tetrimino::O,
                    Tetrimino::S,
                    Tetrimino::T,
                    Tetrimino::Z,
                ];
            }
            let next_index = self.rng.gen_range(0, self.piece_bag.len());
            let next_tetrimino = self.piece_bag.remove(next_index);

            self.piece = Some(Piece::new(next_tetrimino, (5, 20)));
        }

        UpdatedState::rx_event(true, GameRxEvent::Tick(id))
    }

    // fn soft_drop(&mut self, mut piece: Piece) -> bool {
    //     piece.offset.1 -= 1;
    //
    //     // if we collided with something move the piece back and lock it
    //     let lock = self.check_collision(&piece);
    //     if lock {
    //         piece.offset.1 += 1;
    //     }
    //
    //     lock
    // }

    fn lock_piece(&mut self, piece: Piece) {
        for x in 0..piece.bounding_box.len() {
            let board_x = x as isize + piece.offset.0;
            for y in 0..piece.bounding_box[x].len() {
                if piece.bounding_box[x][y] {
                    let board_y = y as isize + piece.offset.1;

                    self.set_board_pixel(
                        board_x,
                        board_y,
                        BoardPixel::Filled(piece.tetrimino.color()),
                    );
                }
            }
        }

        // check for filled rows
        'check_row: for y in (0..piece.bounding_box.len()).rev() {
            let board_y = y as isize + piece.offset.1;

            // todo do we need this check?
            if board_y >= 0 {
                let board_y = board_y as usize;

                for x in 0..BOARD_WIDTH {
                    if self.board_state[x][board_y] == BoardPixel::Empty {
                        // we found an empty pixel so skip this row
                        continue 'check_row;
                    }
                }

                for x in 0..BOARD_WIDTH {
                    for y in board_y..BOARD_HEIGHT - 1 {
                        self.board_state[x][y] = self.board_state[x][y + 1]
                    }
                    self.board_state[x][BOARD_HEIGHT - 1] = BoardPixel::Empty;
                }
            }
        }

        self.piece = None;
    }

    fn add_rows(&self, count: usize) -> UpdatedState {
        UpdatedState::rx_event(true, GameRxEvent::AddRows(count))
    }

    fn check_collision(&self, piece: &Piece) -> bool {
        for x in 0..piece.bounding_box.len() {
            for y in 0..piece.bounding_box[x].len() {
                if piece.bounding_box[x][y] {
                    let board_x = x as isize + piece.offset.0;
                    let board_y = y as isize + piece.offset.1;

                    if let BoardPixel::Filled(_) = self.board_pixel(board_x, board_y) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn board_entity(&self, x: isize, y: isize) -> Option<Entity> {
        if x >= 0 && x < VISIBLE_WIDTH as isize && y >= 0 && y < VISIBLE_HEIGHT as isize {
            Some(self.board_entities[x as usize][y as usize])
        } else {
            None
        }
    }

    fn set_board_pixel(&mut self, x: isize, y: isize, pixel: BoardPixel) {
        if Self::on_board(x, y) {
            self.board_state[x as usize][y as usize] = pixel;
        }
    }

    fn board_pixel(&self, x: isize, y: isize) -> BoardPixel {
        if Self::on_board(x, y) {
            self.board_state[x as usize][y as usize]
        } else {
            BoardPixel::Filled(PieceColor::Gray)
        }
    }

    fn on_board(x: isize, y: isize) -> bool {
        x >= 0 && x < BOARD_WIDTH as isize && y >= 0 && y < BOARD_HEIGHT as isize
    }
}

impl<'s> System<'s> for TetrisGameSystem {
    // #[allow(clippy::type_complexity)]
    type SystemData = WriteStorage<'s, Tint>;

    fn run(&mut self, mut tint_storage: Self::SystemData) {
        let mut any_board_changes = false;
        while let Ok(event) = self.in_rx.try_recv() {
            any_board_changes |= self.receive(event);
        }

        if any_board_changes {
            for x in 0..VISIBLE_WIDTH {
                for y in 0..VISIBLE_HEIGHT {
                    let entity = self.board_entities[x][y];
                    let tint_color = self.board_state[x][y].into();

                    let tint = tint_storage
                        .get_mut(entity)
                        .expect("We should always have this entity");

                    tint.0 = tint_color;
                }
            }

            // render our piece
            if let Some(Piece {
                offset,
                ref bounding_box,
                tetrimino,
                orientation: _,
            }) = self.piece
            {
                for x in 0..bounding_box.len() {
                    for y in 0..bounding_box[x].len() {
                        if bounding_box[x][y] {
                            let board_x = x as isize + offset.0;
                            let board_y = y as isize + offset.1;

                            // make sure we're inside the board
                            if let Some(entity) = self.board_entity(board_x, board_y) {
                                let tint_color: Srgba = tetrimino.color().into();

                                let tint = tint_storage
                                    .get_mut(entity)
                                    .expect("We should always have this entity");

                                tint.0 = tint_color;
                            }
                        } else if RENDER_BOUNDING_BOX {
                            // for debugging, lighten the bounding box

                            let board_x = x as isize + offset.0;
                            let board_y = y as isize + offset.1;

                            // make sure we're inside the board
                            if let Some(entity) = self.board_entity(board_x, board_y) {
                                let tint = tint_storage
                                    .get_mut(entity)
                                    .expect("We should always have this entity");

                                tint.0.red *= 1.5;
                                tint.0.green *= 1.5;
                                tint.0.blue *= 1.5;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct TetrisGameSystemDesc {
    pub position: (f32, f32),
    pub in_rx: Receiver<GameRxEvent>,
    pub out_tx: Sender<GameTxEvent>,
}

impl<'a, 'b> SystemDesc<'a, 'b, TetrisGameSystem> for TetrisGameSystemDesc {
    fn build(self, world: &mut World) -> TetrisGameSystem {
        // setup data we need to initialize, but not to actually run
        <ReadStorage<'a, SpriteRender> as SystemData>::setup(&mut *world);

        <TetrisGameSystem as System<'_>>::SystemData::setup(world);

        let pixel_sprite = world.read_resource::<Sprites>().pixel_sprite.clone();

        let dummy_entity = world.create_entity().entity;

        let (x_offset, y_offset) = self.position;

        debug!("loading at {}, {}", x_offset, y_offset);

        let board_state = [[BoardPixel::Empty; BOARD_HEIGHT]; BOARD_WIDTH];
        let mut board_entities = [[dummy_entity; VISIBLE_HEIGHT]; VISIBLE_WIDTH];
        // // build our side borders
        // for &x in &[0, BOARD_WIDTH - 1] {
        //     for y in 0..BOARD_HEIGHT {
        //         board_state[x][y] = BoardPixel::Filled(PieceColor::Gray)
        //     }
        // }
        // // build our bottom border
        // for x in 1..BOARD_WIDTH - 1 {
        //     board_state[x][0] = BoardPixel::Filled(PieceColor::Gray)
        // }

        for x in 0..VISIBLE_WIDTH {
            for y in 0..VISIBLE_HEIGHT {
                board_entities[x][y] = create_board_entity(
                    x,
                    y,
                    x_offset,
                    y_offset,
                    board_state[x][y],
                    &pixel_sprite,
                    world,
                );
            }
        }

        TetrisGameSystem {
            running: false,
            piece: None,
            board_state,
            board_entities,
            piece_bag: vec![],
            rng: StdRng::seed_from_u64(0),
            in_rx: self.in_rx,
            out_tx: self.out_tx,
        }
    }
}

fn create_board_entity(
    x: usize,
    y: usize,
    offset_x: f32,
    offset_y: f32,
    board_pixel: BoardPixel,
    pixel_sprite: &SpriteRender,
    world: &mut World,
) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(
        offset_x + (x as f32 * PIXEL_DIMENSION),
        offset_y + (y as f32 * PIXEL_DIMENSION),
        0.,
    );
    let scale = PIXEL_DIMENSION / ACTUAL_PIXEL_DIMENSION;
    transform.set_scale(Vector3::new(scale, scale, scale));

    world
        .create_entity()
        .with(pixel_sprite.clone())
        .with(transform)
        .with(Tint(board_pixel.into()))
        .build()
}

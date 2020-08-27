use amethyst::core::ecs::{Entity, ReadStorage, WriteStorage};
use amethyst::core::math::Vector3;
use amethyst::core::Transform;
use amethyst::ecs::{System, SystemData};
use amethyst::prelude::*;
use amethyst::renderer::palette::Hsla;
use amethyst::renderer::palette::Srgba;
use amethyst::renderer::resources::Tint;
use amethyst::renderer::SpriteRender;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use log::debug;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::events::{TetrisIn, TetrisOut, UserInput};
use crate::sprite_loader::Sprites;
use crate::sprite_loader::PIXEL_DIMENSION as ACTUAL_PIXEL_DIMENSION;

use crate::systems::tetris::board::Board;
use crate::systems::tetris::{
    BoardPixel, Piece, PixelColor, Rotation, Tetrimino, BOARD_WIDTH, PIXEL_DIMENSION,
    PREVIEW_HEIGHT, PREVIEW_WIDTH, RENDERED_BOARD_HEIGHT, RENDERED_BOARD_WIDTH,
    RENDERED_PREVIEW_HEIGHT, VISIBLE_HEIGHT, VISIBLE_WIDTH,
};
use crate::ExpectSender;

const RENDER_BOUNDING_BOX: bool = false;

struct UpdatedState {
    board_changed: bool,
    events: Vec<TetrisOut>,
}

impl UpdatedState {
    fn input(board_changed: bool, event: TetrisIn) -> UpdatedState {
        UpdatedState {
            board_changed,
            events: vec![TetrisOut::ValidIn(event)],
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
    held_piece: Option<Tetrimino>,
    /// tracks whether we've held a piece this drop
    held_this_drop: bool,
    board: Board,
    board_entities: [[Entity; VISIBLE_HEIGHT]; VISIBLE_WIDTH],
    next_preview_entities: [[Entity; PREVIEW_HEIGHT]; PREVIEW_WIDTH],
    hold_preview_entities: [[Entity; PREVIEW_HEIGHT]; PREVIEW_WIDTH],
    piece_bag: Vec<Tetrimino>,
    rng: StdRng,
    in_rx: Receiver<TetrisIn>,
    out_tx: Sender<TetrisOut>,
    config: TetrisRenderingConfig,
}

pub struct TetrisRenderingConfig {
    pub show_ghost: bool,
    pub show_next: bool,
    pub show_hold: bool,
}

impl Default for TetrisRenderingConfig {
    fn default() -> Self {
        TetrisRenderingConfig {
            show_ghost: true,
            show_next: true,
            show_hold: true,
        }
    }
}

impl TetrisGameSystem {
    fn receive(&mut self, event: TetrisIn) -> bool {
        let UpdatedState {
            board_changed,
            events,
        } = self.handle_event(event);
        // forward along all the events we found
        events.into_iter().for_each(|e| {
            //debug!("Forwarding event: {:?}", e);

            self.out_tx.send_expect(e)
        });

        board_changed
    }

    fn handle_event(&mut self, event: TetrisIn) -> UpdatedState {
        match event {
            TetrisIn::Start(seed) => {
                self.running = true;
                // clear our board
                self.board = Board::new();
                self.rng = StdRng::seed_from_u64(seed);

                UpdatedState::input(true, TetrisIn::Start(seed))
            }
            TetrisIn::User(input) => {
                if self.running {
                    self.handle_input(input)
                } else {
                    UpdatedState::empty()
                }
            }
            TetrisIn::Tick => {
                if self.running {
                    self.tick()
                } else {
                    UpdatedState::empty()
                }
            }
            TetrisIn::AddRows(count) => {
                if self.running {
                    self.add_rows_event(count)
                } else {
                    UpdatedState::empty()
                }
            }
        }
    }

    fn handle_input(&mut self, event: UserInput) -> UpdatedState {
        if let Some(mut piece) = self.piece.clone() {
            let valid_change = match event {
                UserInput::Left => {
                    piece.offset.0 -= 1;

                    if !self.board.check_collision(&piece) {
                        self.piece = Some(piece);

                        true
                    } else {
                        false
                    }
                }
                UserInput::Right => {
                    piece.offset.0 += 1;

                    if !self.board.check_collision(&piece) {
                        self.piece = Some(piece);

                        true
                    } else {
                        false
                    }
                }
                UserInput::RotateClockwise => {
                    match piece.rotate(Rotation::Clockwise, &self.board) {
                        Some(rotated_piece) => {
                            self.piece = Some(rotated_piece);

                            true
                        }
                        None => false,
                    }
                }
                UserInput::DropSoft => {
                    piece.offset.1 -= 1;

                    // if we collided with something move the piece back and lock it
                    if self.board.check_collision(&piece) {
                        piece.offset.1 += 1;
                        self.lock_piece(piece);
                    } else {
                        self.piece = Some(piece)
                    }

                    // this move is always valid
                    true
                }
                UserInput::DropHard => {
                    piece = self.drop_hard_piece(piece);
                    // this move always locks the piece
                    self.lock_piece(piece);

                    // this move is always valid
                    true
                }
                UserInput::Hold => {
                    if !self.held_this_drop {
                        self.piece = self.held_piece.map(spawn_piece);
                        self.held_piece = Some(piece.tetrimino);

                        self.held_this_drop = true;

                        true
                    } else {
                        false
                    }
                }
            };

            if valid_change {
                UpdatedState::input(true, TetrisIn::User(event))
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
    }

    fn tick(&mut self) -> UpdatedState {
        if let Some(mut piece) = self.piece.clone() {
            piece.offset.1 -= 1;

            // if we collided with something move the piece back and lock it
            let lock = self.board.check_collision(&piece);
            if lock {
                piece.offset.1 += 1;
                self.lock_piece(piece);

                self.piece = None;
            } else {
                self.piece = Some(piece)
            }
        } else {
            let next_tetrimino = self.pop_next_piece();

            let new_piece = spawn_piece(next_tetrimino);
            if self.board.check_collision(&new_piece) {
                self.running = false;
                self.out_tx.send_expect(TetrisOut::Lose);
            } else {
                self.piece = Some(new_piece);
            }
        }

        UpdatedState::input(true, TetrisIn::Tick)
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
        self.out_tx.send_expect(TetrisOut::LockedPiece);

        let mut cleared_lines = 0;

        for (x, y) in piece.filled_pixels() {
            let board_x = x as isize + piece.offset.0;
            let board_y = y as isize + piece.offset.1;
            self.board.set(
                board_x,
                board_y,
                BoardPixel::Filled(piece.tetrimino.color()),
            );
        }

        // check for filled rows
        'check_row: for y in (0..piece.bounding_box.len()).rev() {
            let board_y = y as isize + piece.offset.1;

            // todo do we need this check?
            if board_y >= 0 {
                let board_y = board_y as usize;

                for x in 0..BOARD_WIDTH {
                    if self.board.get(x as isize, board_y as isize) == BoardPixel::Empty {
                        // we found an empty pixel so skip this row
                        continue 'check_row;
                    }
                }

                self.board.clear_row(board_y);
                cleared_lines += 1;
            }
        }

        self.piece = None;
        self.held_this_drop = false;

        if cleared_lines > 0 {
            // todo do we need to make sure this event is ordered in any way?
            // we cleared some rows
            self.out_tx
                .send_expect(TetrisOut::RemovedRows(cleared_lines));
        }

        //self.add_pending_rows();
    }

    fn add_rows_event(&mut self, count: usize) -> UpdatedState {
        assert_ne!(count, 0);

        self.board.fill_rows(count, &mut self.rng);

        UpdatedState::input(true, TetrisIn::AddRows(count))
    }

    fn drop_hard_piece(&self, mut piece: Piece) -> Piece {
        piece.offset.1 -= 1;

        while !self.board.check_collision(&piece) {
            piece.offset.1 -= 1;
        }

        piece.offset.1 += 1;

        piece
    }

    fn peek_next_piece(&mut self) -> Tetrimino {
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
            // shuffle the bag
            self.piece_bag.shuffle(&mut self.rng);
        }

        *self.piece_bag.last().expect("Our bag is never empty here")
    }

    fn pop_next_piece(&mut self) -> Tetrimino {
        let next = self.peek_next_piece();
        self.piece_bag.pop();

        next
    }

    fn board_entity(&self, x: isize, y: isize) -> Option<Entity> {
        if x >= 0 && x < VISIBLE_WIDTH as isize && y >= 0 && y < VISIBLE_HEIGHT as isize {
            Some(self.board_entities[x as usize][y as usize])
        } else {
            None
        }
    }

    fn render_piece(
        &self,
        Piece {
            offset,
            bounding_box,
            tetrimino: _,
            orientation: _,
        }: &Piece,
        color: Srgba,
        tint_storage: &mut WriteStorage<'_, Tint>,
    ) {
        render_piece(
            *offset,
            bounding_box,
            color,
            |x, y| self.board_entity(x, y),
            tint_storage,
        );
    }
}

fn spawn_piece(tetrimino: Tetrimino) -> Piece {
    let offset = match tetrimino {
        Tetrimino::I => (3, 21 - 3),
        Tetrimino::O => (4, 22 - 2),
        _ => (3, 22 - 3),
    };

    Piece::new(tetrimino, offset)
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
                    let tint_color = self.board.get(x as isize, y as isize).into();

                    let tint = tint_storage
                        .get_mut(entity)
                        .expect("We should always have this entity");

                    tint.0 = tint_color;
                }
            }

            // render our piece
            if let Some(ref piece) = self.piece {
                // render a ghost
                if self.config.show_ghost {
                    let ghost = self.drop_hard_piece(piece.clone());
                    let mut color: Hsla = Into::<Srgba>::into(ghost.tetrimino.color()).into();
                    color.saturation *= 0.3;
                    color.lightness *= 0.2;

                    self.render_piece(&ghost, color.into(), &mut tint_storage);
                }

                self.render_piece(piece, piece.tetrimino.color().into(), &mut tint_storage);

                if RENDER_BOUNDING_BOX {
                    let Piece {
                        offset,
                        ref bounding_box,
                        tetrimino: _,
                        orientation: _,
                    } = piece;
                    for x in 0..bounding_box.len() {
                        for y in 0..bounding_box[x].len() {
                            // for debugging, lighten the bounding box
                            if !bounding_box[x][y] {
                                let board_x = x as isize + offset.0;
                                let board_y = y as isize + offset.1;

                                // get the bounding box and make sure we're inside the board
                                if let Some(entity) = self.board_entity(board_x, board_y) {
                                    let tint = tint_storage
                                        .get_mut(entity)
                                        .expect("We should always have this entity");

                                    let mut color: Hsla = tint.0.into();
                                    color.lightness *= 4.;

                                    tint.0 = color.into();
                                }
                            }
                        }
                    }
                }
            }

            // render our next piece
            if self.config.show_next {
                render_preview(
                    self.peek_next_piece(),
                    &self.next_preview_entities,
                    &mut tint_storage,
                );
            }

            if self.config.show_hold {
                if let Some(held_piece) = self.held_piece {
                    render_preview(held_piece, &self.hold_preview_entities, &mut tint_storage);
                }
            }
        }
    }
}

fn render_preview(
    piece: Tetrimino,
    entities: &[[Entity; PREVIEW_HEIGHT]; PREVIEW_WIDTH],
    tint_storage: &mut WriteStorage<'_, Tint>,
) {
    entities.iter().flatten().for_each(|entity| {
        // clear our existing pixels
        let tint = tint_storage
            .get_mut(*entity)
            .expect("We should always have this entity");

        tint.0 = PixelColor::Gray.into();
    });

    // center our piece
    let offset = match piece {
        Tetrimino::O => (1, 1),
        _ => (0, 0),
    };
    render_piece(
        offset,
        &piece.bounding_box(),
        piece.color().into(),
        |x, y| {
            // we know our piece will always be on the "board"
            Some(entities[x as usize][y as usize])
        },
        tint_storage,
    );
}

fn render_piece<FnEntity>(
    offset: (isize, isize),
    bounding_box: &[Vec<bool>],
    color: Srgba,
    mut board_entities: FnEntity,
    tint_storage: &mut WriteStorage<'_, Tint>,
) where
    FnEntity: FnMut(isize, isize) -> Option<Entity>,
{
    for x in 0..bounding_box.len() {
        for y in 0..bounding_box[x].len() {
            if bounding_box[x][y] {
                let board_x = x as isize + offset.0;
                let board_y = y as isize + offset.1;

                if let Some(entity) = board_entities(board_x, board_y) {
                    let tint = tint_storage
                        .get_mut(entity)
                        .expect("We should always have this entity");

                    tint.0 = color;
                }
            }
        }
    }
}

pub struct TetrisGameSystemDesc {
    pub position: (f32, f32),
    pub in_rx: Receiver<TetrisIn>,
    pub out_tx: Sender<TetrisOut>,
    pub config: TetrisRenderingConfig,
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

        let board_state = Board::new();
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
                    board_state.get(x as isize, y as isize).into(),
                    &pixel_sprite,
                    world,
                );
            }
        }

        let mut next_preview_entities = [[dummy_entity; PREVIEW_HEIGHT]; PREVIEW_WIDTH];
        let mut hold_preview_entities = [[dummy_entity; PREVIEW_HEIGHT]; PREVIEW_WIDTH];

        let hold_y_offset =
            RENDERED_BOARD_HEIGHT - PIXEL_DIMENSION - RENDERED_PREVIEW_HEIGHT + y_offset;
        let next_preview_color = if self.config.show_next {
            PixelColor::Gray.into()
        } else {
            Srgba::new(0., 0., 0., 0.)
        };
        let hold_preview_color = if self.config.show_next {
            PixelColor::Gray.into()
        } else {
            Srgba::new(0., 0., 0., 0.)
        };
        for x in 0..PREVIEW_WIDTH {
            for y in 0..PREVIEW_HEIGHT {
                next_preview_entities[x][y] = create_board_entity(
                    x,
                    y,
                    RENDERED_BOARD_WIDTH + PIXEL_DIMENSION + x_offset,
                    hold_y_offset,
                    next_preview_color,
                    &pixel_sprite,
                    world,
                );
                hold_preview_entities[x][y] = create_board_entity(
                    x,
                    y,
                    RENDERED_BOARD_WIDTH + PIXEL_DIMENSION + x_offset,
                    hold_y_offset - PIXEL_DIMENSION - RENDERED_PREVIEW_HEIGHT,
                    hold_preview_color,
                    &pixel_sprite,
                    world,
                );
            }
        }

        TetrisGameSystem {
            running: false,
            piece: None,
            held_piece: None,
            held_this_drop: false,
            board: board_state,
            board_entities,
            next_preview_entities,
            hold_preview_entities,
            piece_bag: vec![],
            rng: StdRng::seed_from_u64(0),
            in_rx: self.in_rx,
            out_tx: self.out_tx,
            config: self.config,
        }
    }
}

fn create_board_entity(
    x: usize,
    y: usize,
    offset_x: f32,
    offset_y: f32,
    pixel_color: Srgba,
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
        .with(Tint(pixel_color))
        .build()
}

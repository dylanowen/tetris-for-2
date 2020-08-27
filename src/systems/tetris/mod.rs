use amethyst::renderer::palette::Srgba;
use lazy_static::lazy_static;

pub use piece::*;

mod board;
mod piece;

pub mod tetris_system;

pub const VISIBLE_WIDTH: usize = 10;
pub const VISIBLE_HEIGHT: usize = 20;

const PREVIEW_WIDTH: usize = 4;
const PREVIEW_HEIGHT: usize = 4;

pub const PIXEL_DIMENSION: f32 = 50.;

const RENDERED_BOARD_WIDTH: f32 = PIXEL_DIMENSION * VISIBLE_WIDTH as f32;
const RENDERED_BOARD_HEIGHT: f32 = PIXEL_DIMENSION * VISIBLE_HEIGHT as f32;

const RENDERED_PREVIEW_WIDTH: f32 = PIXEL_DIMENSION * PREVIEW_WIDTH as f32;
const RENDERED_PREVIEW_HEIGHT: f32 = PIXEL_DIMENSION * PREVIEW_HEIGHT as f32;

pub const RENDERED_WIDTH: f32 = RENDERED_BOARD_WIDTH + PIXEL_DIMENSION + RENDERED_PREVIEW_WIDTH;
//pub const RENDERED_HEIGHT: f32 = RENDERED_BOARD_HEIGHT;

const BOARD_WIDTH: usize = VISIBLE_WIDTH;
const BOARD_HEIGHT: usize = VISIBLE_HEIGHT * 2;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tetrimino {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl Tetrimino {
    fn bounding_box(&self) -> Vec<Vec<bool>> {
        // generate the shape box in the game world coordinates with (x, y) and + y going up
        fn normalize_box(natural_box: Vec<Vec<bool>>) -> Vec<Vec<bool>> {
            let mut game_coordinates_box = natural_box.clone();
            for y in 0..natural_box.len() {
                for x in 0..natural_box[y].len() {
                    game_coordinates_box[x][natural_box.len() - 1 - y] = natural_box[y][x];
                }
            }

            game_coordinates_box
        }

        lazy_static! {
            // write out the shapes in a natural way (how they look)
            static ref I_SHAPE: Vec<Vec<bool>> = normalize_box(vec![
                vec![false; 4],
                vec![true; 4],
                vec![false; 4],
                vec![false; 4],
            ]);
            static ref J_SHAPE: Vec<Vec<bool>> = normalize_box(vec![
                vec![true, false, false],
                vec![true, true, true],
                vec![false, false, false],
            ]);
            static ref L_SHAPE: Vec<Vec<bool>> = normalize_box(vec![
                vec![false, false, true],
                vec![true, true, true],
                vec![false, false, false],
            ]);
            static ref O_SHAPE: Vec<Vec<bool>> = normalize_box(vec![vec![true; 2], vec![true; 2]]);
            static ref S_SHAPE: Vec<Vec<bool>> = normalize_box(vec![
                vec![false, true, true],
                vec![true, true, false],
                vec![false, false, false],
            ]);
            static ref T_SHAPE: Vec<Vec<bool>> = normalize_box(vec![
                vec![false, true, false],
                vec![true, true, true],
                vec![false, false, false],
            ]);
            static ref Z_SHAPE: Vec<Vec<bool>> = normalize_box(vec![
                vec![true, true, false],
                vec![false, true, true],
                vec![false, false, false],
            ]);
        }

        match self {
            Tetrimino::I => I_SHAPE.clone(),
            Tetrimino::J => J_SHAPE.clone(),
            Tetrimino::L => L_SHAPE.clone(),
            Tetrimino::O => O_SHAPE.clone(),
            Tetrimino::S => S_SHAPE.clone(),
            Tetrimino::T => T_SHAPE.clone(),
            Tetrimino::Z => Z_SHAPE.clone(),
        }
    }

    pub fn color(&self) -> PixelColor {
        match self {
            Tetrimino::I => PixelColor::LightBlue,
            Tetrimino::J => PixelColor::DarkBlue,
            Tetrimino::L => PixelColor::Orange,
            Tetrimino::O => PixelColor::Yellow,
            Tetrimino::S => PixelColor::Green,
            Tetrimino::T => PixelColor::Magenta,
            Tetrimino::Z => PixelColor::Red,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BoardPixel {
    Filled(PixelColor),
    Empty,
}

impl Into<Srgba> for BoardPixel {
    fn into(self) -> Srgba<f32> {
        match self {
            BoardPixel::Filled(piece) => piece.into(),
            BoardPixel::Empty => Srgba::new(0.05, 0.05, 0.05, 1.0),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PixelColor {
    LightBlue,
    DarkBlue,
    Orange,
    Yellow,
    Green,
    Red,
    Magenta,
    Gray,
}

impl Into<Srgba> for PixelColor {
    fn into(self) -> Srgba<f32> {
        #[allow(clippy::eq_op)]
        match self {
            PixelColor::LightBlue => Srgba::new(0. / 255., 230. / 255., 254. / 255., 1.0),
            PixelColor::DarkBlue => Srgba::new(24. / 255., 1. / 255., 255. / 255., 1.0),
            PixelColor::Orange => Srgba::new(255. / 255., 115. / 255., 8. / 255., 1.0),
            PixelColor::Yellow => Srgba::new(255. / 255., 222. / 255., 0. / 255., 1.0),
            PixelColor::Green => Srgba::new(102. / 255., 253. / 255., 0. / 255., 1.0),
            PixelColor::Red => Srgba::new(254. / 255., 16. / 255., 60. / 255., 1.0),
            PixelColor::Magenta => Srgba::new(184. / 255., 2. / 255., 253. / 255., 1.0),
            PixelColor::Gray => Srgba::new(50. / 255., 50. / 255., 50. / 255., 1.0),
        }
    }
}

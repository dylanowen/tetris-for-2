use amethyst::renderer::palette::Srgba;
use lazy_static::lazy_static;

pub mod tetris_system;

#[derive(Clone, Debug)]
pub struct Piece {
    pub offset: (isize, isize),
    pub bounding_box: Vec<Vec<bool>>,
    pub tetrimino: Tetrimino,
    pub orientation: Orientation,
}

impl Piece {
    pub fn new(tetrimino: Tetrimino, offset: (isize, isize)) -> Self {
        Piece {
            offset,
            bounding_box: tetrimino.bounding_box(),
            tetrimino,
            orientation: Orientation::North,
        }
    }

    fn filled_pixels(&self) -> impl Iterator<Item = (usize, usize, bool)> + '_ {
        self.bounding_box
            .iter()
            .enumerate()
            .map(|(x, column)| column.iter().enumerate().map(move |(y, set)| (x, y, *set)))
            .flatten()
    }

    pub fn rotate(&self, rotation: Rotation) -> PieceRotationIter {
        let next_orientation = match (self.orientation, rotation) {
            (Orientation::North, Rotation::Clockwise) => Orientation::East,
            (Orientation::North, Rotation::CounterClockwise) => Orientation::West,
            (Orientation::East, Rotation::Clockwise) => Orientation::South,
            (Orientation::East, Rotation::CounterClockwise) => Orientation::North,
            (Orientation::South, Rotation::Clockwise) => Orientation::West,
            (Orientation::South, Rotation::CounterClockwise) => Orientation::East,
            (Orientation::West, Rotation::Clockwise) => Orientation::North,
            (Orientation::West, Rotation::CounterClockwise) => Orientation::South,
        };

        PieceRotationIter {
            piece: self,
            index: 0,
            rotation,
            next_orientation,
            rotation_points: self.rotation_points(rotation),
        }
    }

    fn rotation_points(&self, rotation: Rotation) -> [(isize, isize); 5] {
        match self.tetrimino {
            Tetrimino::J | Tetrimino::L | Tetrimino::S | Tetrimino::T | Tetrimino::Z => {
                match (self.orientation, rotation) {
                    (Orientation::North, Rotation::Clockwise) => {
                        [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)]
                    }
                    (Orientation::North, Rotation::CounterClockwise) => {
                        [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)]
                    }
                    (Orientation::East, Rotation::Clockwise) => {
                        [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)]
                    }
                    (Orientation::East, Rotation::CounterClockwise) => {
                        [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)]
                    }
                    (Orientation::South, Rotation::Clockwise) => {
                        [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)]
                    }
                    (Orientation::South, Rotation::CounterClockwise) => {
                        [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)]
                    }
                    (Orientation::West, Rotation::Clockwise) => {
                        [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)]
                    }
                    (Orientation::West, Rotation::CounterClockwise) => {
                        [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)]
                    }
                }
            }
            Tetrimino::I => match (self.orientation, rotation) {
                (Orientation::North, Rotation::Clockwise) => {
                    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)]
                }
                (Orientation::North, Rotation::CounterClockwise) => {
                    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)]
                }
                (Orientation::East, Rotation::Clockwise) => {
                    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)]
                }
                (Orientation::East, Rotation::CounterClockwise) => {
                    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)]
                }
                (Orientation::South, Rotation::Clockwise) => {
                    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)]
                }
                (Orientation::South, Rotation::CounterClockwise) => {
                    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)]
                }
                (Orientation::West, Rotation::Clockwise) => {
                    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)]
                }
                (Orientation::West, Rotation::CounterClockwise) => {
                    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)]
                }
            },
            Tetrimino::O => [(0, 0); 5],
        }
    }
}

pub struct PieceRotationIter<'p> {
    piece: &'p Piece,
    index: usize,
    rotation: Rotation,
    next_orientation: Orientation,
    rotation_points: [(isize, isize); 5],
}

impl<'p> Iterator for PieceRotationIter<'p> {
    type Item = Piece;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.rotation_points.len() && self.piece.tetrimino != Tetrimino::O {
            let (previous_x, previous_y) = self.piece.offset;
            let (x_offset, y_offset) = self.rotation_points[self.index];
            self.index += 1;

            let bb = &self.piece.bounding_box;
            let rotated_box = match self.rotation {
                Rotation::Clockwise => match self.piece.tetrimino {
                    Tetrimino::I => vec![
                        vec![bb[3][0], bb[2][0], bb[1][0], bb[0][0]],
                        vec![bb[3][1], bb[2][1], bb[1][1], bb[0][1]],
                        vec![bb[3][2], bb[2][2], bb[1][2], bb[0][2]],
                        vec![bb[3][3], bb[2][3], bb[1][3], bb[0][3]],
                    ],
                    Tetrimino::J | Tetrimino::L | Tetrimino::S | Tetrimino::T | Tetrimino::Z => {
                        vec![
                            vec![bb[2][0], bb[1][0], bb[0][0]],
                            vec![bb[2][1], bb[1][1], bb[0][1]],
                            vec![bb[2][2], bb[1][2], bb[0][2]],
                        ]
                    }
                    Tetrimino::O => self.piece.bounding_box.clone(),
                },
                Rotation::CounterClockwise => todo!(),
            };

            let piece = Piece {
                offset: (previous_x - x_offset, previous_y - y_offset),
                bounding_box: rotated_box,
                tetrimino: self.piece.tetrimino,
                orientation: self.next_orientation,
            };

            Some(piece)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.rotation_points.len() - self.index;
        (remaining, Some(remaining))
    }
}

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
pub enum Rotation {
    Clockwise,
    CounterClockwise,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
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
    LightGray,
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
            PixelColor::LightGray => Srgba::new(50. / 255., 50. / 255., 50. / 255., 1.0),
        }
    }
}

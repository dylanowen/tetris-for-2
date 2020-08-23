use crate::systems::tetris::board::Board;
use crate::systems::tetris::Tetrimino;

#[derive(Clone, Debug, Eq, PartialEq)]
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

    pub fn rotate(&self, rotation: Rotation, board: &Board) -> Option<Piece> {
        self.iter_rotate(rotation)
            .filter_map(|rotated_piece| {
                if !board.check_collision(&rotated_piece) {
                    Some(rotated_piece)
                } else {
                    None
                }
            })
            .next()
    }

    fn iter_rotate(&self, rotation: Rotation) -> PieceRotationIter {
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
                offset: (previous_x + x_offset, previous_y + y_offset),
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

#[cfg(test)]
mod tests {
    use crate::systems::tetris::{BoardPixel, PixelColor};

    use super::*;

    #[test]
    fn rotate_clockwise_i_north() {
        test_levels(
            Piece::new(Tetrimino::I, (1, 1)),
            [(3, 1), (1, 4), (4, 1), (1, 1)],
        );
    }

    #[test]
    fn rotate_clockwise_t_north() {
        test_levels(
            Piece::new(Tetrimino::T, (2, 2)),
            [(4, 3), (3, 3), (2, 3), (3, 2)],
        );
    }

    fn test_levels(piece: Piece, filled_squares: [(isize, isize); 4]) {
        let mut board = Board::new();
        test_rotation(piece.clone(), Rotation::Clockwise, 0, &board);
        for (i, &(x, y)) in filled_squares.iter().enumerate() {
            board.set(x, y, BoardPixel::Filled(PixelColor::Gray));
            test_rotation(piece.clone(), Rotation::Clockwise, i + 1, &board);
        }
    }

    fn test_rotation(piece: Piece, rotation: Rotation, expected_level: usize, board: &Board) {
        let rotated = piece.rotate(rotation, &board).unwrap();

        assert_eq!(
            rotated,
            piece.iter_rotate(rotation).nth(expected_level).unwrap(),
            "expected level: {} found level: {}",
            expected_level + 1,
            piece
                .iter_rotate(rotation)
                .enumerate()
                .find(|(_, piece_level)| rotated == *piece_level)
                .unwrap()
                .0
                + 1
        )
    }
}

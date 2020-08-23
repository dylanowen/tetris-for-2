use crate::systems::tetris::{BoardPixel, Piece, PixelColor, BOARD_HEIGHT, BOARD_WIDTH};
use rand::rngs::StdRng;
use rand::Rng;

pub struct Board {
    pixels: [[BoardPixel; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Board {
    pub fn new() -> Board {
        Board {
            pixels: [[BoardPixel::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
        }
    }

    pub fn check_collision(&self, piece: &Piece) -> bool {
        for x in 0..piece.bounding_box.len() {
            for y in 0..piece.bounding_box[x].len() {
                if piece.bounding_box[x][y] {
                    let board_x = x as isize + piece.offset.0;
                    let board_y = y as isize + piece.offset.1;

                    if let BoardPixel::Filled(_) = self.get(board_x, board_y) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn clear_row(&mut self, row_y: usize) {
        for y in row_y..BOARD_HEIGHT - 1 {
            self.pixels[y] = self.pixels[y + 1];
        }

        self.pixels[BOARD_HEIGHT - 1] = [BoardPixel::Empty; BOARD_WIDTH];
    }

    pub fn fill_rows(&mut self, rows: usize, rng: &mut StdRng) {
        assert_ne!(rows, 0);

        // move our rows up
        for y in (rows..BOARD_HEIGHT).rev() {
            self.pixels[y] = self.pixels[y - rows]
        }

        // add in our filled rows
        for y in 0..rows {
            self.pixels[y] = [BoardPixel::Filled(PixelColor::Gray); BOARD_WIDTH];

            let empty_space = rng.gen_range(0, BOARD_WIDTH);
            self.pixels[y][empty_space] = BoardPixel::Empty;
        }
    }

    pub fn get(&self, x: isize, y: isize) -> BoardPixel {
        if Self::on_board(x, y) {
            //*Index::index(Index::index(self, x as usize), y as usize)
            self.pixels[y as usize][x as usize]
        } else {
            BoardPixel::Filled(PixelColor::Gray)
        }
    }

    pub fn set(&mut self, x: isize, y: isize, pixel: BoardPixel) {
        if Self::on_board(x, y) {
            self.pixels[y as usize][x as usize] = pixel;
        }
    }

    fn on_board(x: isize, y: isize) -> bool {
        x >= 0 && x < BOARD_WIDTH as isize && y >= 0 && y < BOARD_HEIGHT as isize
    }
}

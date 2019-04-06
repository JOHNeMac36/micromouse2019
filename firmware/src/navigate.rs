//use arrayvec::ArrayVec;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand::Rng;

use crate::plan::Move;
use crate::plan::MoveOptions;

use ignore_result::Ignore;
use core::fmt::Write;
use crate::uart::Uart;
use crate::uart::Command;

pub trait Navigate: Command {
    fn navigate(&mut self, move_options: MoveOptions) -> [Option<Move>; 2];
}

pub struct Inteligate {
    maze: [[Cell; 16]; 16],
    current_cell: Cell,
    current_direction: CardinalDirection,
}

#[derive(Copy,Clone)]
pub struct Cell {
    row: usize,
    col: usize,
    weight: u16,
}

pub enum CardinalDirection {
    North,
    East,
    South,
    West
}

pub enum MoveDirection {
    Forward,
    Left,
    Backward,
    Right,
}

impl Cell {
    pub fn new(row: usize, col: usize, weight: u16) -> Cell {
        Cell{row, col, weight}
    }

    pub fn invalid() -> Cell {
        Cell::new(0xff, 0xff, 0xffff)
    }

    pub fn best_move(forward_cell: Cell, left_cell: Cell, right_cell: Cell, backward_cell: Cell, move_options: MoveOptions) -> MoveDirection {
        let fw = forward_cell.weight;
        let lw = left_cell.weight;
        let bw = backward_cell.weight;
        let rw = right_cell.weight;

        if move_options.forward && fw <= lw && fw <= bw && fw <= rw {
            MoveDirection::Forward
        }
        else if move_options.left && lw <= fw && lw <= bw && lw <= rw {
            MoveDirection::Left
        }
        else if move_options.right && rw <= fw && rw <= lw && rw <= bw {
            MoveDirection::Right
        }
        else {
            MoveDirection::Backward
        }
    }

    pub fn calc_new_weight(self, forward_cell: Cell, left_cell: Cell, right_cell: Cell, move_options: MoveOptions) -> u16 {
        let cw = self.weight;
        let fw = forward_cell.weight;
        let lw = left_cell.weight;
        let rw = right_cell.weight;

        // If path of dead-end path, 0xffff, else increment weight by 1
        match (move_options.forward, move_options.left, move_options.right, fw, lw, rw) {
            (true , true , true , 0xffff, 0xffff, 0xffff) => 0xffff,
            (true , true , false, 0xffff, 0xffff, _     ) => 0xffff,
            (true , false, true , 0xffff, _     , 0xffff) => 0xffff,
            (false, true , true , _     , 0xffff, 0xffff) => 0xffff,
            (false, false, true , _     , _     , 0xffff) => 0xffff,
            (false, true , false, _     , 0xffff, _     ) => 0xffff,
            (true , false, false, 0xffff, _     , _     ) => 0xffff,
            (false, false, false, _     , _     , _     ) => 0xffff,
            _ => cw + 1,
        }
    }
}

pub fn isqrt(num: u16) -> u16 {
    let mut x = num;
    if x == 0 || x == 1 {
        x
    }
    else {
        let mut result = 1;
        while result <= x {
            x = x + 1;
            result = x * x;
        }
        x - 1
    }
}

impl Inteligate {
    pub fn new() -> Inteligate {
        let mut maze: [[Cell; 16]; 16] = [[Cell::invalid(); 16]; 16];
        for row in 0..15 {
            for col in 0..15 {
                let distance: u16 = isqrt((row - 4) * (row - 4) + (col - 4) * (col - 4));
                maze[row as usize][col as usize] = Cell::new(row as usize, col as usize, distance);
            }
        }
        let current_cell:Cell = Cell::new(0,0,maze[0][0].weight);
        let current_direction = CardinalDirection::North;
        Inteligate{maze, current_cell, current_direction}
    }
}

impl Navigate for Inteligate {
    fn navigate(&mut self, move_options: MoveOptions) -> [Option<Move>; 2] {
        self.current_cell.weight += 1;
        let cr = self.current_cell.row;
        let cc = self.current_cell.col;
        let current_direction = &self.current_direction;
        if cr == 8 && cc == 8 {
            // found center!
            [None, None]
        }
        else {
            let (forward_cell, left_cell, right_cell, backward_cell): (Cell, Cell, Cell, Cell) = match current_direction {
                CardinalDirection::North => {
                    (
                        self.maze.get(cr+1).map_or(Cell::invalid(), |row| row.get(cc+0).map_or(Cell::invalid(), |cell| cell.clone())),
                        self.maze.get(cr+0).map_or(Cell::invalid(), |row| row.get(cc+1).map_or(Cell::invalid(), |cell| cell.clone())),
                        self.maze.get(cr+0).map_or(Cell::invalid(), |row| row.get(cc-1).map_or(Cell::invalid(), |cell| cell.clone())),
                        self.maze.get(cr-1).map_or(Cell::invalid(), |row| row.get(cc+0).map_or(Cell::invalid(), |cell| cell.clone())),
                        )
                },
                CardinalDirection::East => {
                    (
                        self.maze.get(cr+0).map_or(Cell::invalid(), |row| row.get(cc+1).map_or(Cell::invalid(), |cell| cell.clone())),
                        self.maze.get(cr+1).map_or(Cell::invalid(), |row| row.get(cc+0).map_or(Cell::invalid(), |cell| cell.clone())),
                        self.maze.get(cr-1).map_or(Cell::invalid(), |row| row.get(cc+0).map_or(Cell::invalid(), |cell| cell.clone())),
                        self.maze.get(cr+0).map_or(Cell::invalid(), |row| row.get(cc-1).map_or(Cell::invalid(), |cell| cell.clone())),
                        )
                },
                CardinalDirection::South => {
                    (
                        self.maze.get(cr-1).map_or(Cell::invalid(), |row| row.get(cc+0).map_or(Cell::invalid(), |cell| cell.clone())),
                        self.maze.get(cr+0).map_or(Cell::invalid(), |row| row.get(cc-1).map_or(Cell::invalid(), |cell| cell.clone())),
                        self.maze.get(cr+0).map_or(Cell::invalid(), |row| row.get(cc+1).map_or(Cell::invalid(), |cell| cell.clone())),
                        self.maze.get(cr+1).map_or(Cell::invalid(), |row| row.get(cc+0).map_or(Cell::invalid(), |cell| cell.clone())),
                        )
                },
                CardinalDirection::West => {
                    (
                        self.maze.get(cr+0).map_or(Cell::invalid(), |row| row.get(cc-1).map_or(Cell::invalid(), |cell| cell.clone())),
                        self.maze.get(cr-1).map_or(Cell::invalid(), |row| row.get(cc+0).map_or(Cell::invalid(), |cell| cell.clone())),
                        self.maze.get(cr+1).map_or(Cell::invalid(), |row| row.get(cc+0).map_or(Cell::invalid(), |cell| cell.clone())),
                        self.maze.get(cr+0).map_or(Cell::invalid(), |row| row.get(cc+1).map_or(Cell::invalid(), |cell| cell.clone())),
                        )
                },
            };

            let next_move = Cell::best_move(forward_cell, right_cell, backward_cell, left_cell, move_options);

            self.maze[cr][cc].weight = self.current_cell.calc_new_weight(forward_cell, right_cell, left_cell, move_options);

            match next_move {
                MoveDirection::Forward => {
                    self.current_cell = forward_cell;
                    [Some(Move::Forward), None]
                },
                MoveDirection::Left => {
                    self.current_cell = left_cell;
                    [Some(Move::TurnLeft), Some(Move::Forward)]
                },
                MoveDirection::Backward => {
                    self.current_cell = backward_cell;
                    [Some(Move::TurnAround), Some(Move::Forward)]
                },
                MoveDirection::Right => {
                    self.current_cell = right_cell;
                    [Some(Move::TurnRight), Some(Move::Forward)]
                },
            }
        }
    }
}

impl Command for Inteligate {
    fn keyword_command(&self) -> &str {
        "nav"
    }
    fn handle_command<'b, I: Iterator<Item = &'b str>>(
        &mut self,
        uart: &mut Uart,
        mut args: I,
        ){
        match args.next() {
            Some("~") => {
                for row in 0..15 {
                    for col in 0..15{
                        writeln!(uart, "{0: ^02}", self.maze[row][col].weight).ignore();
                    }
                    writeln!(uart, "\n").ignore();
                }
            },
            None => writeln!(uart, "Missing navigate control command!").ignore(),
            _ => writeln!(uart, "Invalid navigate control command!").ignore(),
        }
    }
}

pub struct RandomNavigate {
    rng: SmallRng,
}


impl RandomNavigate {
    pub fn new(seed: [u8; 16]) -> RandomNavigate {
        RandomNavigate {
            rng: SmallRng::from_seed(seed),
        }
    }
}

//impl Navigate for RandomNavigate {
//    fn navigate(&mut self, move_options: MoveOptions) -> [Option<Move>; 2] {
//        match (move_options.left, move_options.forward, move_options.right) {
//            (true, true, true) => {
//                match self.rng.gen_range(0, 3) {
//                    0 => [Some(Move::TurnLeft), Some(Move::Forward)],
//                    1 => [Some(Move::TurnRight), Some(Move::Forward)],
//                    _ => [Some(Move::Forward), None],
//                }
//            }
//
//            (true, false, true) => {
//                match self.rng.gen_range(0, 2) {
//                    0 => [Some(Move::TurnLeft), Some(Move::Forward)],
//                    _ => [Some(Move::TurnRight), Some(Move::Forward)],
//                }
//            }
//
//            (false, true, true) => {
//                match self.rng.gen_range(0, 2) {
//                    0 => [Some(Move::TurnRight), Some(Move::Forward)],
//                    _ => [Some(Move::Forward), None],
//                }
//            }
//
//            (true, true, false) => {
//                match self.rng.gen_range(0, 2) {
//                    0 => [Some(Move::TurnLeft), Some(Move::Forward)],
//                    _ => [Some(Move::Forward), None],
//                }
//            }
//
//            (false, true, false) => {
//                [Some(Move::Forward), None]
//            }
//
//            (true, false, false) => {
//                [Some(Move::TurnLeft), Some(Move::Forward)]
//            }
//
//            (false, false, true) => {
//                [Some(Move::TurnRight), Some(Move::Forward)]
//            }
//
//            (false, false, false) => {
//                [Some(Move::TurnAround), Some(Move::Forward)]
//            }
//        }
//    }
//}
//

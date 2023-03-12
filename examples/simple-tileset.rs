use std::io::stdin;
use std::marker::PhantomData;

use rand::thread_rng;
use termion::color::{Fg, Green, Magenta, Red, Reset};
use wave_collapse::tile2d::*;
use wave_collapse::wave_function::{WaveShape, WaveSolver};
use wave_collapse::*;

fn main() {
    // *************************** Settings *********************************
    let log_steps = true;
    let wait_for_user = false;
    let color = true;
    let tile_size = Size2D::new(50, 16); // 50, 16
    let cutoff_behaviour = CutoffBehaviour::Ignored;
    type WrappingMode = wrapping_mode::Wrapping;
    let tiles = tiles_all();
    // *************************** Settings *********************************

    let shape = TileMap2D::new(tile_size, Size2D::square(3), &tiles);

    if log_steps {
        println!("Initial Position");
        print_tile_map(&shape, false, color);
    }

    let solver = TileSolver::<WrappingMode>::new(cutoff_behaviour);

    let mut rng = thread_rng();
    let mut result_iter = collapse_wave(shape, &solver, &mut rng);

    if log_steps {
        for (n, shape) in &mut result_iter.enumerate() {
            if wait_for_user {
                let mut buf = String::new();
                let _ = stdin().read_line(&mut buf);
            }
            println!("Iteration {n}");
            print_tile_map(&shape, wait_for_user, color);
        }
        println!();
    }

    println!("Result: ");
    match result_iter.calc_result() {
        Ok(shape) => print_tile_map(&shape, false, color),
        Err(error) => eprintln!("Failed to collapse wave: {error:?}"),
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Tile2D {
    left: bool,
    right: bool,
    top: bool,
    bot: bool,
}

impl Tile2D {
    fn get_char_at(&self, x: u32, y: u32) -> char {
        match (x, y) {
            (1, 1) => {
                if self.left || self.right || self.bot || self.top {
                    '+'
                } else {
                    ' '
                }
            }
            (0, 1) => {
                if self.left {
                    '-'
                } else {
                    ' '
                }
            }
            (2, 1) => {
                if self.right {
                    '-'
                } else {
                    ' '
                }
            }
            (1, 0) => {
                if self.top {
                    '|'
                } else {
                    ' '
                }
            }
            (1, 2) => {
                if self.bot {
                    '|'
                } else {
                    ' '
                }
            }
            _ => ' ',
        }
    }
}

#[allow(dead_code)]
enum CutoffBehaviour {
    Wall,
    Passage,
    Ignored,
}

impl CutoffBehaviour {
    fn cutoff(&self, passage: bool) -> bool {
        match self {
            CutoffBehaviour::Wall => !passage,
            CutoffBehaviour::Passage => passage,
            CutoffBehaviour::Ignored => true,
        }
    }
}

pub struct TileSolver<WrappingMode> {
    cutoff_behaviour: CutoffBehaviour,
    _wrapping_mode: PhantomData<WrappingMode>,
}

impl<WrappingMode> Default for TileSolver<WrappingMode> {
    fn default() -> Self {
        Self {
            cutoff_behaviour: CutoffBehaviour::Ignored,
            _wrapping_mode: Default::default(),
        }
    }
}

impl<WrappingMode> TileSolver<WrappingMode> {
    fn new(cutoff_behaviour: CutoffBehaviour) -> Self {
        Self {
            cutoff_behaviour,
            _wrapping_mode: Default::default(),
        }
    }

    fn is_tile_valid(&self, tile: &Tile2D, kernel: &Kernel2D<WrappingMode, Tile2D>) -> bool {
        assert!(kernel.radius_x == 1 && kernel.radius_y == 1);

        let left_node = kernel.get(-1, 0);
        let right_node = kernel.get(1, 0);
        let top_node = kernel.get(0, -1);
        let bot_node = kernel.get(0, 1);

        let left_valid = left_node
            .map(|node| {
                node.possible_values()
                    .iter()
                    .any(|other_tile| tile.left == other_tile.right)
            })
            .unwrap_or(self.cutoff_behaviour.cutoff(tile.left));

        let right_valid = right_node
            .map(|node| {
                node.possible_values()
                    .iter()
                    .any(|other_tile| tile.right == other_tile.left)
            })
            .unwrap_or(self.cutoff_behaviour.cutoff(tile.right));

        let top_valid = top_node
            .map(|node| {
                node.possible_values()
                    .iter()
                    .any(|other_tile| tile.top == other_tile.bot)
            })
            .unwrap_or(self.cutoff_behaviour.cutoff(tile.top));

        let bot_valid = bot_node
            .map(|node| {
                node.possible_values()
                    .iter()
                    .any(|other_tile| tile.bot == other_tile.top)
            })
            .unwrap_or(self.cutoff_behaviour.cutoff(tile.bot));

        left_valid && right_valid && top_valid && bot_valid
    }
}

impl<WrappingMode> WaveSolver<Tile2D, Kernel2D<WrappingMode, Tile2D>> for TileSolver<WrappingMode> {
    fn is_valid(&self, tile: &Tile2D, kernel: &Kernel2D<WrappingMode, Tile2D>) -> bool {
        self.is_tile_valid(tile, kernel)
    }
}

fn tiles() -> Vec<Tile2D> {
    vec![
        Tile2D {
            left: true,
            right: true,
            top: false,
            bot: false,
        },
        Tile2D {
            left: true,
            right: true,
            top: true,
            bot: false,
        },
        Tile2D {
            left: false,
            right: true,
            top: true,
            bot: false,
        },
        Tile2D {
            left: false,
            right: false,
            top: false,
            bot: false,
        },
        Tile2D {
            left: false,
            right: false,
            top: true,
            bot: true,
        },
        Tile2D {
            left: false,
            right: true,
            top: false,
            bot: true,
        },
        Tile2D {
            left: true,
            right: true,
            top: false,
            bot: true,
        },
    ]
}

fn tiles_all() -> Vec<Tile2D> {
    let mut result = Vec::new();

    let b = [true, false];
    for left in b {
        for right in b {
            for top in b {
                for bot in b {
                    result.push(Tile2D {
                        left,
                        right,
                        top,
                        bot,
                    })
                }
            }
        }
    }
    assert!(result.len() == 16);
    result
}

fn print_tile_map(tile_map: &TileMap2D<Tile2D>, user_step: bool, use_color: bool) {
    let size = tile_map.size();

    // println!("{}", "-".repeat(size.width as usize * 2 + 3));

    /*
    let set_collapsed_color = |id: Index2D| {
        if !use_color {
            return;
        }
        if use_color && Some(id) == tile_map.get_last_collapsed_id() {
            print!("{}", Fg(Green));
        } else {
            print!("{}", Fg(Reset));
        }
    };
    let set_error_color = |id: Index2D| {
        if !use_color {
            return;
        }
        if Some(id) == tile_map.get_last_collapsed_id() {
            print!("{}", Fg(Magenta));
        } else {
            print!("{}", Fg(Red));
        };
    };
    let reset_color = || {
        if !use_color {
            return;
        }
        print!("{}", Fg(Reset));
    };
    */

    for y in 0..size.height {
        for sub_y in 0..3 {
            for x in 0..size.width {
                let node = tile_map.get_node(&(x, y)).unwrap();
                for sub_x in 0..3 {
                    // TODO add color
                    if node.is_overspecified() {
                        print!("X");
                    } else if let Some(tile) = node.collapsed() {
                        print!("{}", tile.get_char_at(sub_x, sub_y));
                    } else if user_step {
                        if sub_x == 1 && sub_y == 1 {
                            print!("{:2X}", node.entropy());
                        } else if sub_x == 2 || sub_y != 1 {
                            print!("O");
                        }
                    } else {
                        print!(" ");
                    }
                }
            }
            println!();
        }
    }
}

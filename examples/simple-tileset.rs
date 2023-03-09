use std::io::stdin;
use std::marker::PhantomData;
use std::thread;
use std::time::Duration;

use rand::thread_rng;
use wave_collapse::gen_iter_return_result::GenIterReturnResult;
use wave_collapse::tile2d::*;
use wave_collapse::*;

fn main() {
    // *************************** Settings *********************************
    let log_steps = true;
    let wait_for_user = true;
    let tile_size = Size2D::new(10, 10); // 100, 48
    let cutoff_behaviour = CutoffBehaviour::Ignored;
    type WrappingMode = wrapping_mode::Cutoff;
    // *************************** Settings *********************************

    let tiles = tiles();
    let shape = TileMap2D::new(tile_size, Size2D::square(3), &tiles);

    if log_steps {
        println!("Initial Position");
        print_tile_map(&shape, false);
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
            print_tile_map(&shape, wait_for_user);
        }
        println!();
    }

    println!("Result: ");
    match result_iter.calc_result() {
        Ok(shape) => print_tile_map(&shape, false),
        Err(error) => eprintln!("Failed to collapse wave: {error:?}"),
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Tile2D {
    value: String,
    left: bool,
    right: bool,
    top: bool,
    bot: bool,
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
            value: "_".to_string(),
            left: true,
            right: true,
            top: false,
            bot: false,
        },
        Tile2D {
            value: "A".to_string(),
            left: true,
            right: true,
            top: true,
            bot: false,
        },
        Tile2D {
            value: "L".to_string(),
            left: false,
            right: true,
            top: true,
            bot: false,
        },
        Tile2D {
            value: "□".to_string(),
            left: false,
            right: false,
            top: false,
            bot: false,
        },
        Tile2D {
            value: "|".to_string(),
            left: false,
            right: false,
            top: true,
            bot: true,
        },
        Tile2D {
            value: "P".to_string(),
            left: false,
            right: true,
            top: false,
            bot: true,
        },
        Tile2D {
            value: "T".to_string(),
            left: true,
            right: true,
            top: false,
            bot: true,
        },
    ]
}

fn print_tile_map(tile_map: &TileMap2D<Tile2D>, user_step: bool) {
    let size = tile_map.size();

    println!("{}", "-".repeat(size.width as usize * 2 + 3));

    for y in 0..size.height {
        print!("| ");
        for x in 0..size.width {
            let node = tile_map.get_node(&(x, y));
            match node {
                Some(node) => {
                    if !user_step {
                        match node.entropy() {
                            0 => print!("x"),
                            1 => print!("{}", node.possible_values().first().unwrap().value),
                            _ => print!(" "),
                        }
                    } else {
                        if node.is_overspecified() {
                            print!("x");
                        } else if node.is_collapsed() {
                            print!("{}", node.collapsed().unwrap().value)
                        } else {
                            print!("{}", node.entropy());
                        }
                    }
                }
                None => print!(" "),
            };
            print!(" ")
        }
        println!("|");
    }
    println!("{}", "-".repeat(size.width as usize * 2 + 3));
}

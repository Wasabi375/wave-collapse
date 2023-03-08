use wave_collapse::gen_iter_return_result::GenIterReturnResult;
use wave_collapse::tile2d::*;
use wave_collapse::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Tile2D {
    value: String,
    left: bool,
    right: bool,
    top: bool,
    bot: bool,
}

pub struct TileSolver;

impl TileSolver {
    fn is_tile_valid(&self, _tile: &Tile2D, kernel: &Kernel2D<Tile2D>) -> bool {
        assert!(kernel.radius_x == 1 && kernel.radius_y == 1);

        let _left = kernel.get(-1, 0);
        let _right = kernel.get(1, 0);
        let _top = kernel.get(0, -1);
        let _bot = kernel.get(0, 1);

        todo!()

        /*
        tile.left == left.map(|t| t.right).unwrap_or(false)
            && tile.right == right.map(|t| t.left).unwrap_or(false)
            && tile.top == top.map(|t| t.bot).unwrap_or(false)
            && tile.bot == bot.map(|t| t.top).unwrap_or(false)
        */
    }
}

impl WaveSolver<Tile2D, Kernel2D<Tile2D>> for TileSolver {
    fn is_valid(&self, tile: &Tile2D, kernel: &Kernel2D<Tile2D>) -> bool {
        self.is_tile_valid(tile, kernel)
    }
}

fn main() {
    let tiles: Vec<Tile2D> = vec![
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
            value: " ".to_string(),
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
    ];

    let shape = TileMap2D::new(Size2D::square(10), Size2D::square(3), &tiles);

    match collapse_wave(shape, &TileSolver).calc_result() {
        Ok(_shape) => {
            todo!("handle resulting shape")
        }
        Err(error) => {
            eprint!("Failed to run \"simple tilset\" with error {:?}", error)
        }
    };
}

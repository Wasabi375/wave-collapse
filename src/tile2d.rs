use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use vecgrid::Vecgrid;

use crate::node::{Node, NodeIdIter};
use crate::wave_function::{WaveKernel, WaveShape};

use gen_iter::gen_iter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size2D {
    pub width: u32,
    pub height: u32,
}

pub type Index2D = (u32, u32);

impl Size2D {
    pub fn new(width: u32, height: u32) -> Size2D {
        Size2D { width, height }
    }

    pub fn square(size: u32) -> Size2D {
        Self::new(size, size)
    }
}

pub struct TileMap2D<NodeValue: Clone> {
    size: Size2D,
    kernel_size: Size2D,

    last_collapsed: RefCell<Option<Index2D>>,

    nodes: Vecgrid<Node<Index2D, NodeValue>>,
}

impl<NodeValue: Clone> TileMap2D<NodeValue> {
    /// Create a new [TileMap2D]. `kernel_size` must be uneven in both widht and height. `possible_values` must not be empty.
    pub fn new(size: Size2D, kernel_size: Size2D, possible_values: &[NodeValue]) -> Self {
        assert!(kernel_size.width % 2 == 1, "Kernel width must be uneven");
        assert!(kernel_size.height % 2 == 1, "Kernel height must be uneven");
        assert!(!possible_values.is_empty(), "At least one value required!");

        let mut data = Vec::new();
        for y in 0..size.height {
            for x in 0..size.width {
                data.push(Node::new((x, y), possible_values));
            }
        }

        TileMap2D {
            size,
            kernel_size,
            last_collapsed: RefCell::new(None),
            nodes: Vecgrid::from_column_major(data, size.width as usize, size.height as usize)
                .expect("data size should be valid"),
        }
    }

    pub fn get_collapsed(&self) -> Option<Vecgrid<NodeValue>> {
        let nodes = self
            .nodes
            .elements_column_major_iter()
            .map(|node| node.collapsed());

        if nodes.clone().any(|it| it.is_none()) {
            None
        } else {
            let nodes: Vec<NodeValue> = Vec::from_iter(nodes.map(|it| it.unwrap()));
            Some(
                Vecgrid::from_column_major(
                    nodes,
                    self.size.width as usize,
                    self.size.height as usize,
                )
                .expect("dimensions should match with source vecgrid"),
            )
        }
    }

    pub fn size(&self) -> &Size2D {
        &self.size
    }

    pub fn kernel_size(&self) -> &Size2D {
        &self.kernel_size
    }
}

impl<NodeValue> WaveShape<Index2D, NodeValue> for TileMap2D<NodeValue>
where
    NodeValue: Clone,
{
    fn get_node(&self, id: &Index2D) -> Option<&Node<Index2D, NodeValue>> {
        self.nodes.get(id.0 as usize, id.1 as usize)
    }

    fn iter_node_ids(&self) -> NodeIdIter<Index2D> {
        let vec: Vec<_> = gen_iter!({
            for y in 0..self.size.height {
                for x in 0..self.size.width {
                    yield (x, y);
                }
            }
        })
        .collect();

        vec.into_iter()
    }

    fn set_last_collapsed_id(&self, node_id: Index2D) {
        let _ = self.last_collapsed.borrow_mut().insert(node_id);
    }

    fn get_last_collapsed_id(&self) -> Option<Index2D> {
        *self.last_collapsed.borrow()
    }
}

pub mod wrapping_mode {
    pub struct Wrapping;
    pub struct Cutoff;
}

pub struct Kernel2D<WrappingMode, NodeValueDescription: Clone> {
    tile_map: Rc<TileMap2D<NodeValueDescription>>,
    node_id: Index2D,
    pub radius_x: i64,
    pub radius_y: i64,
    _wrapping_mode: PhantomData<WrappingMode>,
}

impl<WrappingMode, NodeValueDescription: Clone> Kernel2D<WrappingMode, NodeValueDescription> {
    fn new(
        shape: Rc<TileMap2D<NodeValueDescription>>,
        node: &Node<Index2D, NodeValueDescription>,
    ) -> Self {
        let radius_y = ((shape.kernel_size.height - 1) / 2) as i64;
        let radius_x = ((shape.kernel_size.width - 1) / 2) as i64;

        Kernel2D {
            tile_map: shape,
            node_id: node.id,
            radius_x,
            radius_y,
            _wrapping_mode: PhantomData::default(),
        }
    }

    pub fn get(&self, x: i64, y: i64) -> Option<&Node<Index2D, NodeValueDescription>> {
        if x.abs() > self.radius_x || y.abs() > self.radius_y {
            return None;
        }

        let index = (
            (self.node_id.0 as i64 + x) as u32,
            (self.node_id.1 as i64 + y) as u32,
        );

        self.tile_map.get_node(&index)
    }
}

impl<NodeValueDescription: Clone>
    WaveKernel<Index2D, NodeValueDescription, TileMap2D<NodeValueDescription>>
    for Kernel2D<wrapping_mode::Cutoff, NodeValueDescription>
{
    fn new(
        shape: Rc<TileMap2D<NodeValueDescription>>,
        node: &Node<Index2D, NodeValueDescription>,
    ) -> Self {
        Kernel2D::new(shape, node)
    }

    fn iter_node_ids(&self) -> NodeIdIter<Index2D> {
        use std::cmp::{max, min};

        let x_min = max(self.node_id.0 as i64 - self.radius_x, 0);
        let x_max = min(
            self.node_id.0 as i64 + self.radius_x,
            self.tile_map.size.width as i64 - 1,
        );
        let y_min = max(self.node_id.1 as i64 - self.radius_y, 0);
        let y_max = min(
            self.node_id.1 as i64 + self.radius_y,
            self.tile_map.size.height as i64 - 1,
        );

        let vec: Vec<_> = gen_iter!({
            for y in y_min..=y_max {
                for x in x_min..=x_max {
                    yield (x as u32, y as u32);
                }
            }
        })
        .collect();
        vec.into_iter()
    }
}

impl<NodeValueDescription: Clone>
    WaveKernel<Index2D, NodeValueDescription, TileMap2D<NodeValueDescription>>
    for Kernel2D<wrapping_mode::Wrapping, NodeValueDescription>
{
    fn new(
        shape: Rc<TileMap2D<NodeValueDescription>>,
        node: &Node<Index2D, NodeValueDescription>,
    ) -> Self {
        Kernel2D::new(shape, node)
    }

    fn iter_node_ids(&self) -> NodeIdIter<Index2D> {
        let x_min = self.node_id.0 as i64 - self.radius_x;
        let x_max = self.node_id.0 as i64 + self.radius_x;
        let y_min = self.node_id.1 as i64 - self.radius_y;
        let y_max = self.node_id.1 as i64 + self.radius_y;

        let vec: Vec<_> = gen_iter!({
            for y in y_min..=y_max {
                for x in x_min..=x_max {
                    yield (
                        x.rem_euclid(self.tile_map.size.width as i64) as u32,
                        y.rem_euclid(self.tile_map.size.height as i64) as u32,
                    );
                }
            }
        })
        .collect();
        vec.into_iter()
    }
}

use vecgrid::Vecgrid;

use crate::{Node, WaveShape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size2D {
    pub width: u32,
    pub height: u32,
}

type Index2D = (u32, u32);

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

    pub fn get(&self, index: Index2D) -> Option<&Node<Index2D, NodeValue>> {
        self.nodes.get(index.0 as usize, index.1 as usize)
    }
}

impl<'a, NodeValue> WaveShape<'a, Index2D, NodeValue, Kernel2D<'a, NodeValue>>
    for TileMap2D<NodeValue>
where
    NodeValue: Clone + 'a,
{
    fn create_kernel(&'a self, node: &Node<Index2D, NodeValue>) -> Kernel2D<'a, NodeValue> {
        let radius_x = ((self.kernel_size.width - 1) / 2) as i64;
        let radius_y = ((self.kernel_size.height - 1) / 2) as i64;

        Kernel2D {
            tile_map: &self,
            node_id: node.id,
            radius_x,
            radius_y,
        }
    }

    fn iter_nodes(&'a mut self) -> impl Iterator<Item = &mut Node<Index2D, NodeValue>> {
        self.nodes.elements_row_major_iter_mut()
    }
}

pub struct Kernel2D<'a, NodeValue: Clone> {
    tile_map: &'a TileMap2D<NodeValue>,
    node_id: Index2D,
    pub radius_x: i64,
    pub radius_y: i64,
}

impl<NodeValue: Clone> Kernel2D<'_, NodeValue> {
    pub fn get(&self, x: i64, y: i64) -> Option<&Node<Index2D, NodeValue>> {
        if x.abs() > self.radius_x || y.abs() > self.radius_y {
            return None;
        }

        let index = (
            (self.node_id.0 as i64 + x) as u32,
            (self.node_id.1 as i64 + y) as u32,
        );

        self.tile_map.get(index)
    }
}

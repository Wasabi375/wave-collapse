#![feature(associated_type_defaults)]
#![feature(generators, generator_trait)]

pub mod binary_heap_set;
pub mod gen_iter_return_result;
pub mod node;
pub mod tile2d;

use binary_heap_set::BinaryHeapSet;
use node::{Node, NodeIdIter, NodeIter};

use rand::{seq::SliceRandom, Rng};
use std::{cmp::Reverse, fmt::Debug, hash::Hash, ops::Generator, rc::Rc};

use gen_iter::{gen_iter_return, GenIterReturn};

use thiserror::Error;
pub type Result<T> = std::result::Result<T, WaveCollapseError>;

/// This represents a set of rules that define how to colapse a given wave function.
pub trait WaveSolver<NodeValue, Kernel> {
    /// This function should return true, if a `value` is valid within a kernel
    fn is_valid(&self, value: &NodeValue, kernel: &Kernel) -> bool;
}

/// A wave shape defines the dimension/size/shape of the wave function. It also provides functions
/// to create a kernel for a given node and iterate all nodes.
pub trait WaveShape<NodeId, NodeValue: Clone> {
    /// returns an `Iterator` over all ids of the nodes in the wave function.
    fn iter_node_ids(&self) -> NodeIdIter<NodeId>;

    /// returns an `Iterator` over all nodes in the wave function.
    fn iter_nodes(&self) -> NodeIter<'_, NodeId, NodeValue, Self> {
        NodeIter::new(self.iter_node_ids(), self)
    }

    fn get_node(&self, id: &NodeId) -> Option<&Node<NodeId, NodeValue>>;

    fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node<NodeId, NodeValue>>;

    /// returns `true` if all nodes in the [WaveShape] are collapsed. The default implementation uses
    /// ```no_run
    /// # use wave_collapse::WaveShape;
    /// # use wave_collapse::tile2d::{TileMap2D, Size2D};
    /// # let tiles: Vec<u32> = vec![];
    /// # let shape = TileMap2D::new(Size2D::square(10), Size2D::square(3), &tiles);
    /// shape.iter_nodes().all(|node| node.is_collapsed());
    /// ```
    fn is_collapsed(&self) -> bool {
        self.iter_nodes().all(|node| node.is_collapsed())
    }

    /// returns `true` if any node in the [WaveShape] is overspecified, meaning that there are no valid
    /// values for it left.
    /// ```no_run
    /// # use wave_collapse::WaveShape;
    /// # use wave_collapse::tile2d::{TileMap2D, Size2D};
    /// # let tiles: Vec<u32> = vec![];
    /// # let shape = TileMap2D::new(Size2D::square(10), Size2D::square(3), &tiles);
    /// shape.iter_nodes().all(|node| node.is_overspecified());
    /// ```
    fn is_overspecified(&self) -> bool {
        self.iter_nodes().any(|node| node.is_overspecified())
    }
}

pub trait WaveKernel<
    NodeId,
    NodeValueDescription: Clone,
    Shape: WaveShape<NodeId, NodeValueDescription>,
>
{
    /// Creates a kernel for the given `node` and 'shape'. A kernel needs to contain all nodes that can
    /// influcence the current nodes valid states.
    fn new(shape: Rc<Shape>, node: &Node<NodeId, NodeValueDescription>) -> Self;

    /// returns an `Iterator` over all ids of the nodes in the wave function.
    fn iter_node_ids(&self) -> NodeIdIter<NodeId>;

    /// returns an `Iterator` over all nodes in the wave function.
    fn iter_nodes(&self) -> NodeIter<NodeId, NodeValueDescription, Self> {
        NodeIter::new(self.iter_node_ids(), self)
    }
}

pub fn collapse_wave<'solver, Shape, NodeId, NodeValue, Kernel, Solver>(
    shape: Shape,
    solver: &'solver Solver,
) -> GenIterReturn<impl Generator<Yield = Rc<Shape>, Return = Result<Rc<Shape>>> + '_>
where
    NodeId: Copy + Eq + Hash + Debug,
    NodeValue: Clone + PartialEq + Debug,
    Shape: WaveShape<NodeId, NodeValue> + 'solver,
    Kernel: WaveKernel<NodeId, NodeValue, Shape>,
    Solver: WaveSolver<NodeValue, Kernel>,
{
    let result_iter = gen_iter_return!(move {

        // TODO: let user pass in their own Rng
        //      This should be useful for debugging, etc
        let mut rng = rand::thread_rng();

        let shape = Rc::new(shape);

        if shape.iter_nodes().count() == 0 {
            return Err(WaveCollapseError::EmptyInput);
        }

        loop {
            if shape.is_collapsed() {
                return Ok(shape.clone());
            }
            if shape.is_overspecified() {
                return Err(WaveCollapseError::InvalidSuperposition);
            }

            // find node with the least possible values, that is not collapsed
            // FIXME: this should be a function of shape, which can be optimized using binaryheap, etc,
            //      however we need to know when/how to upate the heap when a node changes
            // FIXME: choose a random node if there are multiple
            let first_node = shape
                .iter_nodes()
                .filter(|n| !n.is_collapsed())
                .min_by_key(|n| n.possibilities())
                .expect(
                    "This should only fail if all nodes are collapsed, but we checked that above",
                );


            // randomly choose a value from and assign it to the first node
            collapse_node(first_node, &mut rng);

            let mut open_list = BinaryHeapSet::new();
            open_list.push(Reverse(first_node));

            while !open_list.is_empty() {
                let node = open_list.pop().expect("open list is not empty").0;

                let kernel = Kernel::new(shape.clone(), node);

                let mut values = node.possible_values.borrow_mut();
                let possibilities_before = values.len();
                if !node.is_collapsed() {
                    values.retain(|v| solver.is_valid(v, &kernel));
                }

                if node.is_collapsed() || possibilities_before != values.len() {

                    drop(values);

                    for node in kernel
                        .iter_node_ids()
                        .filter(|id| *id != node.id)
                        .map(|id|shape.get_node(&id).expect(&format!("NodeIdIter is always valid. Id: {:?}", id)))
                        .filter(|node| !node.is_collapsed()) {
                        open_list.push(Reverse(node));
                    };


                }
            }

            // yield the current state of the calculation. That way we can inspect every iteration easily.
            // also this might be interesting for animation or debugging
            yield shape.clone();
        }
    });

    result_iter
}

fn collapse_node<NodeId, NodeValue>(node: &Node<NodeId, NodeValue>, rng: &mut impl Rng)
where
    NodeId: Debug,
    NodeValue: Clone + PartialEq + Debug,
{
    let mut node_values = node.possible_values.borrow_mut();
    let collapsed_value = node_values
        .choose(rng)
        .expect("This should never be None, because the current shape is not overspecified.")
        .clone();
    node_values.clear();
    node_values.push(collapsed_value);

    *node.is_collapsed.borrow_mut() = true;

    #[cfg(debug_assertions)]
    {
        drop(node_values);
        println!("{:?}", node);
    }
}

trait IntoWaveCollapseErrorResult<T> {
    fn err_into(self) -> Result<T>;
}

impl<T, E> IntoWaveCollapseErrorResult<T> for std::result::Result<T, E>
where
    E: Into<WaveCollapseError>,
{
    fn err_into(self) -> Result<T> {
        self.map_err(|e| e.into())
    }
}

#[derive(Error, Debug)]
pub enum WaveCollapseError {
    #[error("failed to collapse wave function")]
    InvalidSuperposition,
    #[error("unknown error")]
    Other,
    #[error("not implemented")]
    NotImplemented,
    #[error("input is empty")]
    EmptyInput,
    #[error("iteration failed, this should never happen")]
    IterationError,
}

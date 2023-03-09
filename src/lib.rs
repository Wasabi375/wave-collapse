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
    /// shape.iter_nodes().any(|node| node.is_overspecified());
    /// ```
    fn is_overspecified(&self) -> bool {
        self.iter_nodes().any(|node| node.is_overspecified())
    }

    /// returns a random node where the [Node] has the lowest possible entropy and is not collapsed or
    /// overspecified.
    /// If no node is found [None] is returend.
    fn choose_random_with_lowest_entropy(
        &self,
        rng: &mut impl Rng,
    ) -> Option<&Node<NodeId, NodeValue>> {
        let mut bucket = Vec::new();
        let mut entropy = u32::MAX;
        for node in self.iter_nodes() {
            if node.is_collapsed() || node.is_overspecified() {
                continue;
            }

            let node_entropy = node.entropy();
            #[allow(clippy::comparison_chain)]
            if node_entropy < entropy {
                entropy = node_entropy;
                bucket.clear();
                bucket.push(node);
            } else if node_entropy == entropy {
                bucket.push(node);
            }
        }

        bucket.choose(rng).copied()
    }
}

/// A wave kernel is a structure that represents all nodes that can affect the [Node] that is
/// used to create the kernel, e.g. in a tile map that would be all nodes sorounding the center node.
pub trait WaveKernel<
    NodeId,
    NodeValueDescription: Clone,
    Shape: WaveShape<NodeId, NodeValueDescription>,
>
{
    /// Creates a kernel for the given [Node] and [WaveShape]. A kernel needs to contain all nodes that can
    /// influcence the current nodes valid states.
    fn new(shape: Rc<Shape>, node: &Node<NodeId, NodeValueDescription>) -> Self;

    /// returns an [Iterator] over all ids of the [Node]s in the [WaveKernel].
    fn iter_node_ids(&self) -> NodeIdIter<NodeId>;

    /// returns an [Iterator] over all nodes in the [WaveKernel].
    fn iter_nodes(&self) -> NodeIter<NodeId, NodeValueDescription, Self> {
        NodeIter::new(self.iter_node_ids(), self)
    }
}

/// collapses the `shape` so that each [Node] in the [WaveShape] has only value.
///
/// The result is an [Iterator]. Each iteration collapses a single [Node] in the [WaveShape] and
/// yields the itermediate [WaveShape].
/// When the [Iterator] yields [None], the result value in the [GenIterReturn] is the [Result]
/// of the collapse. In the success case the collapsed [WaveShape] or a [WaveCollapseError]
/// if something went wrong.
///
/// * `result.next()`: a reference to the [WaveShape]. The state of this [WaveShape] is not
///         stable and will change with each iteration. Each iteration returns a reference to
///         the same [WaveShape].
/// * `result.calculate_result()`: Automatically advances the [Iterator] until it yields [None]
///         and than returns the [Result] of the wave function collapse. See [gen_iter_return_result::GenIterReturnResult]
///
/// # Example
/// ```no_run
/// let mut rng = todo!();
/// use wave_collapse::{WaveShape, collapse_wave, WaveSolver, WaveKernel};
/// use wave_collapse::tile2d::{TileMap2D, Size2D, Kernel2D};
/// let tiles: Vec<u32> = vec![];
/// let shape = TileMap2D::new(Size2D::square(10), Size2D::square(3), &tiles);
/// let solver: WaveSolver<u32, Kernel2D> = todo!();
///
/// let result = collapse_wave(shape, &solver, &mut rng);
///
/// for (n, shape) in &mut result_iter.enumerate() {
///     // print_tile_map(&shape);
/// }
/// match result_iter.calc_result() {
///     Ok(shape) => todo!(), // print_tile_map(&shape)
///     Err(error) => eprintln!("Failed to collapse wave: {error:?}"),
/// }
/// ```
pub fn collapse_wave<'solver, Shape, NodeId, NodeValue, Kernel, Solver>(
    shape: Shape,
    solver: &'solver Solver,
    rng: &'solver mut impl Rng,
) -> GenIterReturn<impl Generator<Yield = Rc<Shape>, Return = Result<Rc<Shape>>> + 'solver>
where
    NodeId: Copy + Eq + Hash + Debug,
    NodeValue: Clone + PartialEq + Debug,
    Shape: WaveShape<NodeId, NodeValue> + 'solver,
    Kernel: WaveKernel<NodeId, NodeValue, Shape>,
    Solver: WaveSolver<NodeValue, Kernel>,
{
    let result_iter = gen_iter_return!(move {

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

            let first_node = shape.choose_random_with_lowest_entropy(rng)
                .expect("This should never be none, because shape is not collapsed or overspecified");


            // randomly choose a value from and assign it to the first node
            collapse_node(first_node, rng);

            let mut open_list = BinaryHeapSet::new();
            open_list.push(Reverse(first_node));

            while let Some(node) = open_list.pop() {
                let node = node.0;

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
                        .map(|id|shape.get_node(&id)
                            .unwrap_or_else(|| panic!("NodeIdIter is always valid. Id: {id:?}")))
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

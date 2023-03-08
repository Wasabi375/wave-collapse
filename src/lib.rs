#![feature(associated_type_defaults)]
#![feature(generators, generator_trait)]
// TODO temp
#![allow(unused_variables)]
#![allow(unreachable_code)]

pub mod gen_iter_return_result;
pub mod tile2d;

use std::{
    cell::{Ref, RefCell},
    fmt::Debug,
    marker::PhantomData,
    ops::Generator,
    rc::Rc,
};

use gen_iter::{gen_iter_return, GenIterReturn};
use thiserror::Error;
type Result<T> = std::result::Result<T, WaveCollapseError>;

type NodeIdIter<T> = std::vec::IntoIter<T>;

/// This represents a set of rules that define how to colapse a given wave function.
pub trait WaveSolver<NodeValue, Kernel> {
    /// This function should return true, if a `NodeValue` is valid within a kernel
    fn is_valid(&self, tile: &NodeValue, kernel: &Kernel) -> bool;
}

/// A wave shape defines the dimension/size/shape of the wave function. It also provides functions
/// to create a kernel for a given node and iterate all nodes.
pub trait WaveShape<NodeId, NodeValue: Clone> {
    /// returns an `Iterator` over all ids of the nodes in the wave function.
    fn iter_node_ids(&self) -> NodeIdIter<NodeId>;

    /// returns an `Iterator` over all nodes in the wave function.
    fn iter_nodes(&self) -> NodeIter<'_, NodeId, NodeValue, Self> {
        NodeIter::new(self.iter_node_ids(), &self)
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
        self.iter_nodes().all(|node| node.is_overspecified())
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
        NodeIter::new(self.iter_node_ids(), &self)
    }
}

/// This describes a single node within the wave function. It contains all possible values this node can
/// be collopsed into.
#[derive(Clone)]
pub struct Node<Id, NodeValueDescription> {
    /// a unique id within a wave shape
    id: Id,

    /// all possible values this node can be collopsed into.
    possible_values: RefCell<Vec<NodeValueDescription>>,
}

impl<Id, NodeValueDescription: Clone> Node<Id, NodeValueDescription> {
    pub fn new<Values>(id: Id, possible_values: Values) -> Self
    where
        Values: Into<Vec<NodeValueDescription>>,
    {
        Node {
            id,
            possible_values: RefCell::new(possible_values.into()),
        }
    }

    pub fn collapsed(&self) -> Option<NodeValueDescription> {
        if self.is_collapsed() {
            Some(self.possible_values.borrow()[0].clone())
        } else {
            None
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.possible_values.borrow().len() == 1
    }

    /// returns `true` if this node is overspecified, meaning that there are no valid
    /// values for it left.
    pub fn is_overspecified(&self) -> bool {
        self.possible_values.borrow().len() == 0
    }

    pub fn possible_values(&self) -> Ref<'_, [NodeValueDescription]> {
        Ref::map(self.possible_values.borrow(), |v| v.as_slice())
    }

    pub fn possibilities(&self) -> u32 {
        self.possible_values.borrow().len() as u32
    }
}

pub struct NodeIter<'a, NodeId, NodeValueDescription, Shape: ?Sized> {
    shape: &'a Shape,
    iterator: NodeIdIter<NodeId>,
    _id_phantom: PhantomData<NodeId>,
    _value_phantom: PhantomData<NodeValueDescription>,
}

impl<'a, NodeId, NodeValueDescription, Shape> NodeIter<'a, NodeId, NodeValueDescription, Shape>
where
    Shape: ?Sized,
{
    fn new(iterator: NodeIdIter<NodeId>, shape: &'a Shape) -> Self {
        NodeIter {
            shape,
            iterator,
            _id_phantom: PhantomData::default(),
            _value_phantom: PhantomData::default(),
        }
    }
}

impl<'a, NodeId, NodeValueDescription, Shape> Iterator
    for NodeIter<'a, NodeId, NodeValueDescription, Shape>
where
    NodeId: 'a,
    NodeValueDescription: Clone + 'a,
    Shape: WaveShape<NodeId, NodeValueDescription> + ?Sized,
{
    type Item = &'a Node<NodeId, NodeValueDescription>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|id| {
            self.shape
                .get_node(&id)
                .expect("A valid node iterator only returns valid node ids")
        })
    }
}

pub fn collapse_wave<Shape, NodeId, Value, Values, Kernel, Solver>(
    shape: Shape,
    _possible_values: &Values, // TODO I think this is not necessary
    solver: &Solver,
) -> GenIterReturn<impl Generator<Yield = Rc<Shape>, Return = Result<Rc<Shape>>>>
where
    NodeId: Copy,
    Value: Clone,
    Values: IntoIterator<Item = Value>,
    Shape: WaveShape<NodeId, Value>,
    Kernel: WaveKernel<NodeId, Value, Shape>,
    Solver: WaveSolver<Value, Kernel>,
{
    gen_iter_return!(move {

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
            let first_node = shape
                .iter_nodes()
                .filter(|n| !n.is_collapsed())
                .min_by_key(|n| n.possibilities())
                .expect(
                    "This should only fail if all nodes are collapsed, but we checked that above",
                );

            // randomly choose a value from and assign it to the first node

            // expand node
            let first_node_kernel = Kernel::new(shape.clone(), &first_node);

            // yield the current state of the calculation. That way we can inspect every iteration easily.
            // also this might be interesting for animation or debugging
            yield shape.clone();
        }
    })
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

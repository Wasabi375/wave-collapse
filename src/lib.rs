#![feature(associated_type_defaults)]
#![feature(generators, generator_trait)]

pub mod gen_iter_return_result;
pub mod tile2d;

use rand::{seq::SliceRandom, Rng};
use std::{
    cell::{Ref, RefCell},
    cmp::Reverse,
    collections::BinaryHeap,
    fmt::Debug,
    marker::PhantomData,
    ops::Generator,
    rc::Rc,
};

use gen_iter::{gen_iter_return, GenIterReturn};
use thiserror::Error;
pub type Result<T> = std::result::Result<T, WaveCollapseError>;

type NodeIdIter<T> = std::vec::IntoIter<T>;

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
        NodeIter::new(self.iter_node_ids(), self)
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
}

impl<Id, NodeValueDescription> Node<Id, NodeValueDescription> {
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

impl<Id, NodeValueDescription> Eq for Node<Id, NodeValueDescription> where Id: Eq {}

impl<Id, NodeValueDescription> PartialEq for Node<Id, NodeValueDescription>
where
    Id: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<Id, NodeValueDescription> PartialOrd for Node<Id, NodeValueDescription>
where
    Id: PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            return Some(std::cmp::Ordering::Equal);
        }
        self.possibilities().partial_cmp(&other.possibilities())
    }
}

impl<Id, NodeValueDescription> Ord for Node<Id, NodeValueDescription>
where
    Id: Eq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self == other {
            return std::cmp::Ordering::Equal;
        }
        self.possibilities().cmp(&other.possibilities())
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

pub fn collapse_wave<'solver, Shape, NodeId, NodeValue, Kernel, Solver>(
    shape: Shape,
    solver: &'solver Solver,
) -> GenIterReturn<impl Generator<Yield = Rc<Shape>, Return = Result<Rc<Shape>>> + '_>
where
    NodeId: Copy + Eq,
    NodeValue: Clone + PartialEq,
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
            let first_node = shape
                .iter_nodes()
                .filter(|n| !n.is_collapsed())
                .min_by_key(|n| n.possibilities())
                .expect(
                    "This should only fail if all nodes are collapsed, but we checked that above",
                );


            // randomly choose a value from and assign it to the first node
            collapse_node(first_node, &mut rng);

            let mut open_list = BinaryHeap::new();
            open_list.push(Reverse(first_node));

            while !open_list.is_empty() {
                let node = open_list.pop().expect("open list is not empty").0;

                let kernel = Kernel::new(shape.clone(), node);

                let mut values = node.possible_values.borrow_mut();
                let possibilities_before = values.len();
                values.retain(|v| solver.is_valid(v, &kernel));

                if possibilities_before != values.len() {

                    for node in kernel
                        .iter_node_ids()
                        .filter(|id| *id != node.id)
                        .map(|id|shape.get_node(&id).expect("NodeIdIter is always valid")) {
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
    NodeValue: Clone + PartialEq,
{
    let mut node_values = node.possible_values.borrow_mut();
    let collapsed_value = node_values
        .choose(rng)
        .expect("This should never be None, because the current shape is not overspecified.")
        .clone();
    node_values.clear();
    node_values.push(collapsed_value);
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

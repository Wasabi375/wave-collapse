#![allow(incomplete_features)]
#![feature(return_position_impl_trait_in_trait)]
// TODO temp
// #![allow(unused_variables)]
// #![allow(dead_code)]

pub mod tile2d;

/// This represents a set of rules that define how to colapse a given wave function.
pub trait WaveSolver<NodeValue, Kernel> {
    /// This function should return true, if a `NodeValue` is valid within a kernel
    fn is_valid(&self, tile: &NodeValue, kernel: &Kernel) -> bool;
}

/// A wave shape defines the dimension/size/shape of the wave function. It also provides functions
/// to create a kernel for a given node and iterate all nodes.
pub trait WaveShape<'a, NodeId: 'a, NodeValue: 'a + Clone, Kernel: 'a> {
    /// Creates a kernel for the given `node`. A kernel needs to contain all nodes that can
    /// influcence the current nodes valid states.
    fn create_kernel(&'a self, node: &Node<NodeId, NodeValue>) -> Kernel;

    /// returns an `Iterator` over all nodes in the wave function.
    fn iter_nodes(&'a mut self) -> impl Iterator<Item = &mut Node<NodeId, NodeValue>>;
}

/// This describes a single node within the wave function. It contains all possible values this node can
/// be collopsed into.
#[derive(Clone)]
pub struct Node<Id, NodeValueDescription: Clone> {
    /// a unique id within a wave shape
    id: Id,

    /// all possible values this node can be collopsed into.
    possible_values: Vec<NodeValueDescription>,
}

impl<Id, NodeValueDescription: Clone> Node<Id, NodeValueDescription> {
    pub fn new<Values>(id: Id, possible_values: Values) -> Self
    where
        Values: Into<Vec<NodeValueDescription>>,
    {
        Node {
            id,
            possible_values: possible_values.into(),
        }
    }

    pub fn collapsed(&self) -> Option<NodeValueDescription> {
        if self.is_collapsed() {
            Some(self.possible_values[0].clone())
        } else {
            None
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.possible_values.len() == 1
    }

    pub fn possible_values(&self) -> &[NodeValueDescription] {
        &self.possible_values
    }
}

pub fn collapse_wave<'a, Shape, NodeId, Value, Values, Kernel, Solver>(
    _shape: &'a mut Shape,
    _possible_values: &Values,
    _solver: &Solver,
) -> Result<()>
where
    NodeId: 'a + Copy,
    Value: Clone + 'a,
    Values: IntoIterator<Item = Value>,
    Kernel: 'a,
    Shape: WaveShape<'a, NodeId, Value, Kernel>,
    Solver: WaveSolver<Value, Kernel>,
{
    Err(WaveCollapseError::NotImplemented)
}

use thiserror::Error;
type Result<T> = std::result::Result<T, WaveCollapseError>;

#[derive(Error, Debug)]
pub enum WaveCollapseError {
    #[error("failed to collapse wave function")]
    InvalidSuperposition,
    #[error("unknown error")]
    Other,
    #[error("not implemented")]
    NotImplemented,
}

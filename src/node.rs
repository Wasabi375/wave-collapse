use std::{
    cell::{Ref, RefCell},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
};

use crate::WaveShape;

/// This describes a single node within the wave function. It contains all possible values this node can
/// be collopsed into.
#[derive(Clone)]
pub struct Node<Id, NodeValueDescription> {
    /// a unique id within a wave shape
    pub id: Id,

    /// all possible values this node can be collopsed into.
    pub(super) possible_values: RefCell<Vec<NodeValueDescription>>,

    /// denotes whether or not this cell is collapsed or not.
    pub(super) is_collapsed: RefCell<bool>,
}

impl<Id, NodeValue> Debug for Node<Id, NodeValue>
where
    Id: Debug,
    NodeValue: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("possible_values", &self.possible_values())
            .finish()
    }
}

impl<Id, NodeValueDescription: Clone> Node<Id, NodeValueDescription> {
    pub fn new<Values>(id: Id, possible_values: Values) -> Self
    where
        Values: Into<Vec<NodeValueDescription>>,
    {
        Node {
            id,
            possible_values: RefCell::new(possible_values.into()),
            is_collapsed: RefCell::new(false),
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
        *self.is_collapsed.borrow()
    }

    /// returns `true` if this node is overspecified, meaning that there are no valid
    /// values for it left.
    pub fn is_overspecified(&self) -> bool {
        self.possible_values.borrow().len() == 0
    }

    pub fn possible_values(&self) -> Ref<'_, [NodeValueDescription]> {
        Ref::map(self.possible_values.borrow(), |v| v.as_slice())
    }

    pub fn entropy(&self) -> u32 {
        self.possible_values.borrow().len() as u32
    }
}

impl<Id, NodeValueDescription> Hash for Node<Id, NodeValueDescription>
where
    Id: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
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
        if *self == *other {
            return Some(std::cmp::Ordering::Equal);
        }
        self.entropy().partial_cmp(&other.entropy())
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
        self.entropy().cmp(&other.entropy())
    }
}

/// The internally used iterator type when iterating node ids.
pub type NodeIdIter<T> = std::vec::IntoIter<T>;

/// An iterator over nodes, that uses [NodeIdIter] and [crate::WaveShape] to
/// iterate nodes.
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
    pub fn new(iterator: NodeIdIter<NodeId>, shape: &'a Shape) -> Self {
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

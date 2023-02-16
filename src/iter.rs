use crate::{Interval, IntervalTreeNode, NodeContent};

pub enum NodeContentIter<'a, I: Interval<K>, const K: usize> {
    Subtree(IntervalTreeIterator<'a, I, K>),
    Leaf(std::slice::Iter<'a, I>),
}

impl<'a, I: Interval<K>, const K: usize> Iterator for NodeContentIter<'a, I, K> {
    type Item = &'a I;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            NodeContentIter::Subtree(n) => n.next(),
            NodeContentIter::Leaf(i) => i.next(),
        }
    }
}

impl<'a, I: Interval<K>, const K: usize> NodeContentIter<'a, I, K> {
    pub(crate) fn new(node_content: &'a NodeContent<I, K>) -> Self {
        match node_content {
            NodeContent::Subtree(i) => NodeContentIter::Subtree(i.iter()),
            NodeContent::Leaf(v) => NodeContentIter::Leaf(v.iter()),
        }
    }
}

impl<'a, I: Interval<K>, const K: usize> IntervalTreeIterator<'a, I, K> {
    pub(crate) fn new(curr_node: &'a IntervalTreeNode<I, K>) -> Self {
        let it = if let Some(lt) = &curr_node.lt_nodes {
            CurrentIterationState::Left(IntervalTreeIterator::new(lt))
        } else {
            CurrentIterationState::Center(NodeContentIter::new(&curr_node.center))
        };
        IntervalTreeIterator {
            curr_node,
            it: Box::new(it),
        }
    }
}

pub struct IntervalTreeIterator<'a, I: Interval<K>, const K: usize> {
    curr_node: &'a IntervalTreeNode<I, K>,
    it: Box<CurrentIterationState<'a, I, K>>,
}

enum CurrentIterationState<'a, I: Interval<K>, const K: usize> {
    Left(IntervalTreeIterator<'a, I, K>),
    Center(NodeContentIter<'a, I, K>),
    Right(IntervalTreeIterator<'a, I, K>),
    None,
}

// This could trivially be implemented by returning a range_search with an infinitely sized
// interval...
impl<'a, I: Interval<K>, const K: usize> Iterator for IntervalTreeIterator<'a, I, K> {
    type Item = &'a I;

    fn next(&mut self) -> Option<Self::Item> {
        if let CurrentIterationState::Left(it) = &mut *self.it {
            if let lt @ Some(_) = it.next() {
                return lt;
            }
            *self.it = CurrentIterationState::Center(NodeContentIter::new(&self.curr_node.center));
        }

        if let CurrentIterationState::Center(it) = &mut *self.it {
            if let lt @ Some(_) = it.next() {
                return lt;
            }
            *self.it = match &self.curr_node.gt_nodes {
                None => CurrentIterationState::None,
                Some(gt_nodes) => CurrentIterationState::Right(IntervalTreeIterator::new(gt_nodes)),
            };
        }

        match &mut *self.it {
            CurrentIterationState::Right(it) => it.next(),
            CurrentIterationState::None => None,
            _ => unreachable!(),
        }
    }
}

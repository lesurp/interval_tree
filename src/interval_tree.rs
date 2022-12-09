use std::cmp::Ordering;

use crate::iter::IntervalTreeIterator;
use num_traits::{Num, NumOps, One};
use std::cmp::PartialOrd;

pub trait Point<const K: usize> {
    type Scalar: NumOps + Num + PartialOrd + Copy;
    fn value(&self, k: usize) -> Self::Scalar;

    #[inline(always)]
    fn dimension() -> usize {
        K
    }
}

impl<const K: usize, P: Point<K>> Interval<K> for P {
    type Scalar = <P as Point<K>>::Scalar;
    fn min(&self, k: usize) -> Self::Scalar {
        self.value(k)
    }

    fn max(&self, k: usize) -> Self::Scalar {
        self.value(k)
    }
}

pub trait Interval<const K: usize> {
    type Scalar: NumOps + Num + PartialOrd + Copy;

    fn min(&self, k: usize) -> Self::Scalar;
    fn max(&self, k: usize) -> Self::Scalar;
    fn center(&self, k: usize) -> Self::Scalar {
        (self.min(k) + self.max(k)) / (Self::Scalar::one() + Self::Scalar::one())
    }

    #[inline(always)]
    fn overlaps<I: Interval<K, Scalar = Self::Scalar>>(&self, o: &I) -> bool {
        (0..Self::dimension()).all(|k| self.overlaps_at_k(o, k))
    }

    #[inline(always)]
    fn overlaps_at_k<I: Interval<K, Scalar = Self::Scalar>>(&self, o: &I, k: usize) -> bool {
        debug_assert!(k < K);
        self.min(k) <= o.max(k) && o.min(k) <= self.max(k)
    }

    #[inline(always)]
    fn dimension() -> usize {
        K
    }

    fn cmp_at_k(&self, k: usize, s: Self::Scalar) -> std::cmp::Ordering {
        debug_assert!(k < K);
        if self.min(k) > s {
            std::cmp::Ordering::Greater
        } else if self.max(k) < s {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

pub enum NodeContent<I: Interval<K>, const K: usize> {
    Subtree(Box<IntervalTreeNode<I, K>>),
    Leaf(Vec<I>),
}

pub struct IntervalTreeNode<I: Interval<K>, const K: usize> {
    pub(crate) center: NodeContent<I, K>,
    pub(crate) center_val: I::Scalar,
    pub(crate) k: usize,
    pub(crate) lt_nodes: Option<Box<IntervalTreeNode<I, K>>>,
    pub(crate) gt_nodes: Option<Box<IntervalTreeNode<I, K>>>,
}

impl<const K: usize, I: Interval<K>> IntervalTreeNode<I, K> {
    pub fn range_search<II: Interval<K, Scalar = I::Scalar>>(&self, x: &II) -> Vec<&I> {
        let same_level = match x.cmp_at_k(self.k, self.center_val) {
            Ordering::Less => self
                .lt_nodes
                .as_ref()
                .map_or(Vec::new(), |n| n.range_search(x)),
            Ordering::Greater => self
                .gt_nodes
                .as_ref()
                .map_or(Vec::new(), |n| n.range_search(x)),
            Ordering::Equal => self
                .lt_nodes
                .as_ref()
                .map_or(Vec::new(), |n| n.range_search(x))
                .into_iter()
                .chain(
                    self.gt_nodes
                        .as_ref()
                        .map_or(Vec::new(), |n| n.range_search(x))
                        .into_iter(),
                )
                .collect(),
        };

        match &self.center {
            NodeContent::Subtree(n) => n.range_search(x),
            NodeContent::Leaf(intervals) => intervals.iter().filter(|i| i.overlaps(x)).collect(),
        }
        .into_iter()
        .chain(same_level.into_iter())
        .collect()
    }

    pub fn from_intervals(is: Vec<I>) -> IntervalTreeNode<I, K> {
        IntervalTreeNode::from_intervals_rec(is, 0)
    }

    fn from_intervals_rec(mut intervals: Vec<I>, k: usize) -> IntervalTreeNode<I, K> {
        intervals.sort_by(|a, b| a.center(k).partial_cmp(&b.center(k)).unwrap());
        let median = intervals[intervals.len() / 2].center(k);

        let mut lt_nodes = Vec::new();
        let mut center = Vec::new();
        let mut gt_nodes = Vec::new();
        for i in intervals {
            if i.max(k) < median {
                lt_nodes.push(i);
            } else if i.min(k) > median {
                gt_nodes.push(i);
            } else {
                center.push(i);
            }
        }

        let lt_nodes = if lt_nodes.is_empty() {
            None
        } else {
            Some(Box::new(IntervalTreeNode::from_intervals_rec(lt_nodes, k)))
        };

        let gt_nodes = if gt_nodes.is_empty() {
            None
        } else {
            Some(Box::new(IntervalTreeNode::from_intervals_rec(gt_nodes, k)))
        };

        let center = if k + 1 < I::dimension() {
            NodeContent::Subtree(Box::new(IntervalTreeNode::from_intervals_rec(
                center,
                k + 1,
            )))
        } else {
            NodeContent::Leaf(center)
        };

        IntervalTreeNode {
            center_val: median,
            k,
            center,
            lt_nodes,
            gt_nodes,
        }
    }

    pub fn iter(&self) -> IntervalTreeIterator<I, K> {
        IntervalTreeIterator::new(self)
    }
}

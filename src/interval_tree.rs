use std::cmp::Ordering;

use crate::iter::IntervalTreeIterator;
use num_traits::{NumAssign, NumOps, One};
use std::cmp::PartialOrd;

pub trait Scalar: NumOps + NumAssign + PartialOrd + Copy + std::iter::Product {}
// Can be replaced with auto_traits eventually
impl<S: NumOps + NumAssign + PartialOrd + Copy + std::iter::Product> Scalar for S {}

pub trait Point<const K: usize> {
    type Scalar: Scalar;
    fn value(&self, k: usize) -> Self::Scalar;

    #[inline(always)]
    fn dimension() -> usize {
        K
    }
}

impl<const K: usize, P: Point<K>> Interval<K> for P {
    type Scalar = <P as Point<K>>::Scalar;
    fn min_at(&self, k: usize) -> Self::Scalar {
        self.value(k)
    }

    fn max_at(&self, k: usize) -> Self::Scalar {
        self.value(k)
    }
}

// To avoid adding an Ord bound on Scalar...
fn max<S: Scalar>(a: S, b: S) -> S {
    if a >= b {
        a
    } else {
        b
    }
}

fn min<S: Scalar>(a: S, b: S) -> S {
    if a <= b {
        a
    } else {
        b
    }
}

pub trait Interval<const K: usize> {
    type Scalar: Scalar;

    fn min_at(&self, k: usize) -> Self::Scalar;
    fn max_at(&self, k: usize) -> Self::Scalar;
    fn avg_at(&self, k: usize) -> Self::Scalar {
        // TODO: there *has* to be a better way creating a 2...
        (self.min_at(k) + self.max_at(k)) / (Self::Scalar::one() + Self::Scalar::one())
    }

    #[inline(always)]
    fn overlaps<I: Interval<K, Scalar = Self::Scalar>>(&self, o: &I) -> bool {
        (0..Self::dimension()).all(|k| self.overlaps_at(k, o))
    }

    #[inline(always)]
    fn overlaps_at<I: Interval<K, Scalar = Self::Scalar>>(&self, k: usize, o: &I) -> bool {
        debug_assert!(k < K);
        self.min_at(k) <= o.max_at(k) && o.min_at(k) <= self.max_at(k)
    }

    #[inline(always)]
    fn dimension() -> usize {
        K
    }

    fn try_overlapping_volume<I: Interval<K, Scalar = Self::Scalar>>(
        &self,
        o: &I,
    ) -> Option<Self::Scalar> {
        let mut v = Self::Scalar::one();
        for k in 0..Self::dimension() {
            if self.overlaps_at(k, o) {
                v *= min(self.max_at(k), o.max_at(k)) - max(self.min_at(k), o.min_at(k));
            } else {
                return None;
            }
        }
        Some(v)
    }

    fn overlapping_volume<I: Interval<K, Scalar = Self::Scalar>>(&self, o: &I) -> Self::Scalar {
        (0..Self::dimension())
            .map(|k| min(self.max_at(k), o.max_at(k)) - max(self.min_at(k), o.min_at(k)))
            .product()
    }

    fn cmp_at(&self, k: usize, s: Self::Scalar) -> std::cmp::Ordering {
        debug_assert!(k < K);
        if self.min_at(k) > s {
            std::cmp::Ordering::Greater
        } else if self.max_at(k) < s {
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
        let same_level = match x.cmp_at(self.k, self.center_val) {
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
        intervals.sort_by(|a, b| a.avg_at(k).partial_cmp(&b.avg_at(k)).unwrap());
        let median = intervals[intervals.len() / 2].avg_at(k);

        let mut lt_nodes = Vec::new();
        let mut center = Vec::new();
        let mut gt_nodes = Vec::new();
        for i in intervals {
            if i.max_at(k) < median {
                lt_nodes.push(i);
            } else if i.min_at(k) > median {
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

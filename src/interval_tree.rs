use std::cmp::Ordering;

use crate::iter::IntervalTreeIterator;
use num_traits::{NumAssign, NumOps, One};
use std::cmp::PartialOrd;

/// Most scalar types should already implement these (refer to num_traits for details)
pub trait Scalar: NumOps + NumAssign + PartialOrd + Copy + std::iter::Product {}
// Can be replaced with auto_traits eventually
impl<S: NumOps + NumAssign + PartialOrd + Copy + std::iter::Product> Scalar for S {}

/// This trait has to be implemented when searching intervals containing specific values.
/// Note that implementors of Point automatically implement Interval, meaning nothing else is
/// required for computing Interval's in the tree containing a given Point.
/// K: The dimension should be 0 for dynamically sized Point's, and the dimension() function should
/// then be overriden to return the proper dimension. Failure to override the method will panic at
/// runtime.
pub trait Point<const K: usize> {
    /// The scalar used for indexing, comparing etc.
    type Scalar: Scalar;

    /// This should return the value of the 0-index element of our point.
    fn value(&self, k: usize) -> Self::Scalar;

    /// For compile-time known dimensions, returns the dimension.
    /// For dynamically-sized objects, K should be set to 0 and this function overriden.
    fn dimension(&self) -> usize {
        assert!(K != 0, "0-sized tree should only be used for dynamically sized trees. In that case, the dimension() function should be overriden to provide the actual number of dimensions.");
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

/// This trait defines what our intervals are. Note that the dimension of the interval should be
/// greater than 0.
/// K: The dimension should be 0 for dynamically sized Point's, and the dimension() function should
/// then be overriden to return the proper dimension. Failure to override the method will panic at
/// runtime.
pub trait Interval<const K: usize> {
    type Scalar: Scalar;

    /// The minimum value of self on the kth dimension (0-indexed).

    fn min_at(&self, k: usize) -> Self::Scalar;
    /// The maximum value of self on the kth dimension (0-indexed).
    fn max_at(&self, k: usize) -> Self::Scalar;

    /// The average between min_at and max_at
    fn avg_at(&self, k: usize) -> Self::Scalar {
        // TODO: there *has* to be a better way creating a 2...
        (self.min_at(k) + self.max_at(k)) / (Self::Scalar::one() + Self::Scalar::one())
    }

    /// Returns whether self overlaps with the given interval
    fn overlaps<I: Interval<K, Scalar = Self::Scalar>>(&self, o: &I) -> bool {
        (0..self.dimension()).all(|k| self.overlaps_at(k, o))
    }

    /// Returns whether self overlaps with the given interval at the specified dimension k
    /// (0-indexed)
    fn overlaps_at<I: Interval<K, Scalar = Self::Scalar>>(&self, k: usize, o: &I) -> bool {
        debug_assert!(k < K);
        self.min_at(k) <= o.max_at(k) && o.min_at(k) <= self.max_at(k)
    }

    /// For compile-time known dimensions, returns the dimension.
    /// For dynamically-sized objects, K should be set to 0 and this function overriden.
    fn dimension(&self) -> usize {
        assert!(K != 0, "0-sized tree should only be used for dynamically sized trees. In that case, the dimension() function should be overriden to provide the actual number of dimensions.");
        K
    }

    /// Returns the volume of the overlapping space between two intervals, if they overlap. Returns
    /// None if they do not.
    fn try_overlapping_volume<I: Interval<K, Scalar = Self::Scalar>>(
        &self,
        o: &I,
    ) -> Option<Self::Scalar> {
        let mut v = Self::Scalar::one();
        for k in 0..self.dimension() {
            if self.overlaps_at(k, o) {
                v *= min(self.max_at(k), o.max_at(k)) - max(self.min_at(k), o.min_at(k));
            } else {
                return None;
            }
        }
        Some(v)
    }

    /// Returns the volume of the overlapping space between two intervals, assuming they overlap.
    /// The they do not, the result returned is undefined.
    fn overlapping_volume<I: Interval<K, Scalar = Self::Scalar>>(&self, o: &I) -> Self::Scalar {
        (0..self.dimension())
            .map(|k| min(self.max_at(k), o.max_at(k)) - max(self.min_at(k), o.min_at(k)))
            .product()
    }

    /// Manual implementation of a comparison function. This allows !Ord types (e.g. floats) to be
    /// used with this library without having to resort to NonNanFloat or equivalents.
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

/// Should not be matched on.
/// Internal representation of the tree, based on whether there are further dimensions to process.
pub enum NodeContent<I: Interval<K>, const K: usize> {
    Subtree(Box<IntervalTreeNode<I, K>>),
    Leaf(Vec<I>),
}

/// Implementation detail.
/// Contains sub-trees, which either contain the raw intervals, or the nodes of trees at dimension
/// k+1.
/// Also contain the left and right nodes, which contain intervals with lower or greater values
/// respectively for dimension k.
pub struct IntervalTreeNode<I: Interval<K>, const K: usize> {
    pub(crate) center: NodeContent<I, K>,
    pub(crate) center_val: I::Scalar,
    pub(crate) k: usize,
    pub(crate) lt_nodes: Option<Box<IntervalTreeNode<I, K>>>,
    pub(crate) gt_nodes: Option<Box<IntervalTreeNode<I, K>>>,
}

impl<const K: usize, I: Interval<K>> IntervalTreeNode<I, K> {
    /// Given an interval, returns all the Interval's in the tree overlapping with it.
    /// Note that the bound on the input is relaxed - only the dimension type needs to be the same.
    /// This means two Interval types with the same dimensions can be used in the tree and during
    /// the search.
    /// TODO: make a "safe" overload to avoid confusing differents elements
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

    /// Creates an IntervalTreeNode given a collection of intervals.
    /// TODO: make generics, does not have to be a Vec
    pub fn from_intervals(intervals: Vec<I>) -> IntervalTreeNode<I, K> {
        IntervalTreeNode::from_intervals_rec(intervals, 0)
    }

    fn from_intervals_rec(mut intervals: Vec<I>, k: usize) -> IntervalTreeNode<I, K> {
        assert!(
            !intervals.is_empty(),
            "Input intervals should not be empty!"
        );
        intervals.sort_by(|a, b| a.avg_at(k).partial_cmp(&b.avg_at(k)).unwrap());
        let median = intervals[intervals.len() / 2].avg_at(k);

        let mut lt_nodes = Vec::new();
        let mut center = Vec::new();
        let mut gt_nodes = Vec::new();

        let mut dimension = None;
        for i in intervals {
            dimension = match dimension {
                Some(d) => {
                    assert!(
                        i.dimension() == d,
                        "Intervals need to have the same dimension when transformed into a tree!"
                    );
                    dimension
                }
                None => Some(i.dimension()),
            };

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

        let center = if k + 1 < dimension.unwrap() {
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

    /// Returns an interator over all intervals in the tree.
    pub fn iter(&self) -> IntervalTreeIterator<I, K> {
        IntervalTreeIterator::new(self)
    }
}

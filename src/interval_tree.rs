use std::cmp::Ordering;

use num_traits::{Num, NumOps, One};
use std::cmp::PartialOrd;

pub trait Point<const K: usize>: Copy {
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

pub trait Interval<const K: usize>: Clone {
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
    center: NodeContent<I, K>,
    center_val: I::Scalar,
    k: usize,
    lt_nodes: Option<Box<IntervalTreeNode<I, K>>>,
    gt_nodes: Option<Box<IntervalTreeNode<I, K>>>,
}

impl<const K: usize, I: Interval<K>> IntervalTreeNode<I, K> {
    pub fn range_search<II: Interval<K, Scalar = I::Scalar>>(&self, x: &II) -> Vec<I> {
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
            NodeContent::Leaf(intervals) => intervals
                .iter()
                .filter(|i| i.overlaps(x))
                .cloned()
                .collect(),
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
}

#[cfg(test)]
mod tests {

    use super::{Interval, IntervalTreeNode, NodeContent, Point};
    use std::borrow::Borrow;

    #[derive(Clone, Debug)]
    pub struct Rectangle {
        xmin: f64,
        xmax: f64,

        ymin: f64,
        ymax: f64,
    }

    impl Interval<2> for Rectangle {
        type Scalar = f64;
        fn min(&self, k: usize) -> f64 {
            match k {
                0 => self.xmin,
                1 => self.ymin,
                _ => unreachable!(),
            }
        }

        fn max(&self, k: usize) -> f64 {
            match k {
                0 => self.xmax,
                1 => self.ymax,
                _ => unreachable!(),
            }
        }
    }

    impl Rectangle {
        fn new(xmin: f64, xmax: f64, ymin: f64, ymax: f64) -> Self {
            Rectangle {
                xmin,
                xmax,
                ymin,
                ymax,
            }
        }
    }

    impl Point<2> for (f64, f64) {
        type Scalar = f64;
        fn value(&self, k: usize) -> f64 {
            match k {
                0 => self.0,
                1 => self.1,
                _ => unreachable!(),
            }
        }
    }

    fn basic_tree() -> IntervalTreeNode<Rectangle, 2> {
        let intervals = vec![
            Rectangle::new(2.0, 3.0, 5.0, 6.0),
            Rectangle::new(3.0, 7.0, 1.0, 3.0),
            Rectangle::new(0.0, 4.0, -3.0, 2.0),
            Rectangle::new(-5.0, 1.0, 2.0, 4.0),
            Rectangle::new(-3.0, 2.0, -4.0, 2.0),
        ];

        IntervalTreeNode::from_intervals(intervals)
    }

    fn assert_approx<R1: Borrow<Rectangle>, R2: Borrow<Rectangle>>(a: R1, b: R2) {
        let eps = 1e-5;
        let a = a.borrow();
        let b = b.borrow();
        let xl = (a.xmin - b.xmin).abs();
        let xu = (a.xmax - b.xmax).abs();
        let yl = (a.ymin - b.ymin).abs();
        let yu = (a.ymax - b.ymax).abs();

        assert!(
            xl < eps && xu < eps && yl < eps && yu < eps,
            "Rectangles {a:?} and {b:?} are equal"
        );
    }

    #[test]
    fn test_tree_creation() {
        let tree = basic_tree();

        {
            let lt_nodes = tree.lt_nodes.unwrap();
            assert!(lt_nodes.lt_nodes.is_none());
            assert!(lt_nodes.gt_nodes.is_none());
            let lt_nodes_center = match lt_nodes.center {
                NodeContent::Subtree(n) => n,
                _ => unreachable!(),
            };
            assert!(lt_nodes_center.lt_nodes.is_none());
            assert!(lt_nodes_center.gt_nodes.is_none());
            let lt_nodes_intervals = match lt_nodes_center.center {
                NodeContent::Leaf(intervals) => intervals,
                _ => unreachable!(),
            };
            assert_eq!(lt_nodes_intervals.len(), 1);
            assert_approx(&lt_nodes_intervals[0], Rectangle::new(-5.0, 1.0, 2.0, 4.0));
        }

        {
            let gt_nodes = tree.gt_nodes.unwrap();
            assert!(gt_nodes.lt_nodes.is_none());
            assert!(gt_nodes.gt_nodes.is_none());
            let gt_nodes_center = match gt_nodes.center {
                NodeContent::Subtree(n) => n,
                _ => unreachable!(),
            };
            assert!(gt_nodes_center.lt_nodes.is_none());
            assert!(gt_nodes_center.gt_nodes.is_none());
            let gt_nodes_intervals = match gt_nodes_center.center {
                NodeContent::Leaf(intervals) => intervals,
                _ => unreachable!(),
            };
            assert_eq!(gt_nodes_intervals.len(), 1);
            assert_approx(&gt_nodes_intervals[0], Rectangle::new(3.0, 7.0, 1.0, 3.0));
        }

        {
            let root_center = match tree.center {
                NodeContent::Subtree(n) => n,
                _ => unreachable!(),
            };
            assert!(root_center.lt_nodes.is_none());
            let root_leaves = match root_center.center {
                NodeContent::Leaf(intervals) => intervals,
                _ => unreachable!(),
            };
            assert_eq!(root_leaves.len(), 2);

            let more_leaves = match root_center.gt_nodes.unwrap().center {
                NodeContent::Leaf(intervals) => intervals,
                _ => unreachable!(),
            };
            assert_eq!(more_leaves.len(), 1);
            assert_approx(&more_leaves[0], Rectangle::new(2.0, 3.0, 5.0, 6.0));
        }
    }

    #[test]
    fn test_tree_querying_point() {
        let point = (1.0, 2.0);
        let tree = basic_tree();
        let mut intervals = tree.range_search(&point);
        assert_eq!(intervals.len(), 3);
        intervals.sort_by(|a, b| a.center(0).partial_cmp(&b.center(0)).unwrap());
        assert_approx(&intervals[0], Rectangle::new(-5.0, 1.0, 2.0, 4.0));
        assert_approx(&intervals[1], Rectangle::new(-3.0, 2.0, -4.0, 2.0));
        assert_approx(&intervals[2], Rectangle::new(0.0, 4.0, -3.0, 2.0));
        assert!(intervals[0].overlaps(&point));
        assert!(intervals[1].overlaps(&point));
        assert!(intervals[2].overlaps(&point));
    }

    #[test]
    fn test_tree_querying_rect() {
        let rect = Rectangle::new(1.0, 4.0, 2.5, 6.0);
        let tree = basic_tree();
        let mut intervals = tree.range_search(&rect);
        assert_eq!(intervals.len(), 3);
        intervals.sort_by(|a, b| a.center(0).partial_cmp(&b.center(0)).unwrap());
        assert_approx(&intervals[0], Rectangle::new(-5.0, 1.0, 2.0, 4.0));
        assert_approx(&intervals[1], Rectangle::new(2.0, 3.0, 5.0, 6.0));
        assert_approx(&intervals[2], Rectangle::new(3.0, 7.0, 1.0, 3.0));
        assert!(intervals[0].overlaps(&rect));
        assert!(intervals[1].overlaps(&rect));
        assert!(intervals[2].overlaps(&rect));
    }
}

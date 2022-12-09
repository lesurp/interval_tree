use std::cmp::Ordering;

use num_traits::{Num, NumOps, One};
use std::cmp::PartialOrd;

pub trait Point {
    type Scalar: NumOps + Num + PartialOrd;
    fn value(&self, k: usize) -> Self::Scalar;
}

pub trait Interval: Clone {
    //type Scalar: NumOps + Num + PartialOrd;
    type Point: Point;

    fn min(&self, k: usize) -> <Self::Point as Point>::Scalar;
    fn max(&self, k: usize) -> <Self::Point as Point>::Scalar;
    fn center(&self, k: usize) -> <Self::Point as Point>::Scalar {
        (self.min(k) + self.max(k))
            / (<Self::Point as Point>::Scalar::one() + <Self::Point as Point>::Scalar::one())
    }

    fn is_in(&self, p: &Self::Point) -> bool;
    fn dimension() -> usize;
}

pub enum NodeContent<I: Interval> {
    Subtree(Box<IntervalTreeNode<I>>),
    Leaf(Vec<I>),
}

pub struct IntervalTreeNode<I: Interval> {
    center: NodeContent<I>,
    center_val: <I::Point as Point>::Scalar,
    k: usize,
    lt_nodes: Option<Box<IntervalTreeNode<I>>>,
    gt_nodes: Option<Box<IntervalTreeNode<I>>>,
}

impl<I: Interval> IntervalTreeNode<I> {
    pub fn range_search(&self, x: &I::Point) -> Vec<I> {
        let same_level = match x.value(self.k).partial_cmp(&self.center_val).unwrap() {
            Ordering::Less => self
                .lt_nodes
                .as_ref()
                .map_or(Vec::new(), |n| n.range_search(x)),
            Ordering::Greater => self
                .gt_nodes
                .as_ref()
                .map_or(Vec::new(), |n| n.range_search(x)),
            Ordering::Equal => Vec::new(),
        };

        match &self.center {
            NodeContent::Subtree(n) => n.range_search(x),
            NodeContent::Leaf(intervals) => {
                intervals.iter().filter(|i| i.is_in(x)).cloned().collect()
            }
        }
        .into_iter()
        .chain(same_level.into_iter())
        .collect()
    }

    pub fn from_intervals(is: Vec<I>) -> IntervalTreeNode<I> {
        IntervalTreeNode::from_intervals_rec(is, 0)
    }

    fn from_intervals_rec(mut intervals: Vec<I>, k: usize) -> IntervalTreeNode<I> {
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

    impl Interval for Rectangle {
        type Point = (f64, f64);

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

        fn dimension() -> usize {
            2
        }

        fn is_in(&self, p: &Self::Point) -> bool {
            p.0 >= self.xmin && p.0 <= self.xmax && p.1 >= self.ymin && p.1 <= self.ymax
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

    impl Point for (f64, f64) {
        type Scalar = f64;
        fn value(&self, k: usize) -> f64 {
            match k {
                0 => self.0,
                1 => self.1,
                _ => unreachable!(),
            }
        }
    }

    fn basic_tree() -> IntervalTreeNode<Rectangle> {
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
    fn test_tree_querying() {
        let point = (1.0, 2.0);
        let tree = basic_tree();
        let mut intervals = tree.range_search(&point);
        assert_eq!(intervals.len(), 3);
        intervals.sort_by(|a, b| a.center(0).partial_cmp(&b.center(0)).unwrap());
        assert_approx(&intervals[0], Rectangle::new(-5.0, 1.0, 2.0, 4.0));
        assert_approx(&intervals[1], Rectangle::new(-3.0, 2.0, -4.0, 2.0));
        assert_approx(&intervals[2], Rectangle::new(0.0, 4.0, -3.0, 2.0));
    }
}

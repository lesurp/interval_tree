use crate::{Interval, IntervalTreeNode, NodeContent, Point};
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
    fn min_at(&self, k: usize) -> f64 {
        match k {
            0 => self.xmin,
            1 => self.ymin,
            _ => unreachable!(),
        }
    }

    fn max_at(&self, k: usize) -> f64 {
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

fn basic_tree_rectangles() -> Vec<Rectangle> {
    vec![
        Rectangle::new(2.0, 3.0, 5.0, 6.0),
        Rectangle::new(3.0, 7.0, 1.0, 3.0),
        Rectangle::new(0.0, 4.0, -3.0, 2.0),
        Rectangle::new(-5.0, 1.0, 2.0, 4.0),
        Rectangle::new(-3.0, 2.0, -4.0, 2.0),
    ]
}

fn basic_tree() -> IntervalTreeNode<Rectangle, 2> {
    IntervalTreeNode::from_intervals(basic_tree_rectangles())
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
    intervals.sort_by(|a, b| a.avg_at(0).partial_cmp(&b.avg_at(0)).unwrap());
    assert_approx(intervals[0], Rectangle::new(-5.0, 1.0, 2.0, 4.0));
    assert_approx(intervals[1], Rectangle::new(-3.0, 2.0, -4.0, 2.0));
    assert_approx(intervals[2], Rectangle::new(0.0, 4.0, -3.0, 2.0));
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
    intervals.sort_by(|a, b| a.avg_at(0).partial_cmp(&b.avg_at(0)).unwrap());
    assert_approx(intervals[0], Rectangle::new(-5.0, 1.0, 2.0, 4.0));
    assert_approx(intervals[1], Rectangle::new(2.0, 3.0, 5.0, 6.0));
    assert_approx(intervals[2], Rectangle::new(3.0, 7.0, 1.0, 3.0));
    assert!(intervals[0].overlaps(&rect));
    assert!(intervals[1].overlaps(&rect));
    assert!(intervals[2].overlaps(&rect));
}

#[test]
fn test_tree_iter() {
    let tree = basic_tree();
    let mut intervals = basic_tree_rectangles();
    intervals.sort_by(|a, b| a.avg_at(0).partial_cmp(&b.avg_at(0)).unwrap());

    let collected_interval = tree.iter().collect::<Vec<_>>();
    assert_eq!(collected_interval.len(), intervals.len());
    for (collected, given) in collected_interval.iter().zip(intervals.iter()) {
        assert_approx(*collected, given);
    }
}

#[test]
fn test_tree_measure_volume() {
    let rect = Rectangle::new(1.0, 4.0, 2.5, 6.0);
    let tree = basic_tree();
    let mut intervals = tree.range_search(&rect);
    assert_eq!(intervals.len(), 3);
    intervals.sort_by(|a, b| a.avg_at(0).partial_cmp(&b.avg_at(0)).unwrap());
    println!("{:?}", intervals[0]);
    let areas_assuming_overlap = intervals
        .iter()
        .map(|i| i.overlapping_volume(&rect))
        .collect::<Vec<_>>();
    let areas_from_try = intervals
        .iter()
        .map(|i| i.try_overlapping_volume(&rect).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(areas_from_try[0], 0.0);
    assert_eq!(areas_from_try[1], 1.0);
    assert_eq!(areas_from_try[2], 0.5);
    assert_eq!(areas_from_try, areas_assuming_overlap);
}

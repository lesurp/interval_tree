use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use interval_tree::IntervalTreeNode;
use interval_tree::*;
use rand::Rng;

pub struct Point2d(f64, f64);

#[derive(Clone, Debug)]
pub struct Rectangle {
    xmin: f64,
    xmax: f64,

    ymin: f64,
    ymax: f64,
}

impl Interval for Rectangle {
    type Point = Point2d;

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

impl Point for Point2d {
    type Scalar = f64;
    fn value(&self, k: usize) -> f64 {
        match k {
            0 => self.0,
            1 => self.1,
            _ => unreachable!(),
        }
    }
}

fn random_point() -> Point2d {
    let mut rng = rand::thread_rng();
    let dist = rand::distributions::Uniform::new(-100.0, 100.0);
    Point2d(rng.sample(dist), rng.sample(dist))
}

fn random_tree(n: u64) -> IntervalTreeNode<Rectangle> {
    let mut rng = rand::thread_rng();
    let dist = rand::distributions::Uniform::new(-100.0f64, 100.0);
    let intervals = (0..n)
        .map(|_| {
            let xa = rng.sample(dist);
            let xb = rng.sample(dist);
            let ya = rng.sample(dist);
            let yb = rng.sample(dist);

            Rectangle::new(xa.min(xb), xa.max(xb), ya.min(yb), ya.max(yb))
        })
        .collect();

    IntervalTreeNode::from_intervals(intervals)
}

fn access_n(c: &mut Criterion) {
    let mut group = c.benchmark_group("access_n");
    for size in (0..9).map(|n| 10u64.pow(n)) {
        group.throughput(Throughput::Elements(size));
        let tree = random_tree(size);
        let point = random_point();
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| tree.range_search(&point));
        });
    }
    group.finish();
}

criterion_group!(benches, access_n);
criterion_main!(benches);

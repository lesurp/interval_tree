use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use interval_tree::IntervalTreeNode;
use interval_tree::*;
use rand::distributions::Uniform;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct Point2d(f64, f64);

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

impl Point<2> for Point2d {
    type Scalar = f64;
    fn value(&self, k: usize) -> f64 {
        match k {
            0 => self.0,
            1 => self.1,
            _ => unreachable!(),
        }
    }
}

struct RectRng {
    rng: StdRng,
    range_dist: Uniform<f64>,
}

impl RectRng {
    pub fn new() -> RectRng {
        RectRng {
            rng: StdRng::seed_from_u64(0),
            range_dist: rand::distributions::Uniform::new(-100.0, 100.0),
        }
    }

    fn random_point(&mut self) -> Point2d {
        Point2d(
            self.rng.sample(self.range_dist),
            self.rng.sample(self.range_dist),
        )
    }

    fn random_rect(&mut self) -> Rectangle {
        let xa = self.rng.sample(self.range_dist);
        let xb = self.rng.sample(self.range_dist);
        let ya = self.rng.sample(self.range_dist);
        let yb = self.rng.sample(self.range_dist);

        Rectangle::new(xa.min(xb), xa.max(xb), ya.min(yb), ya.max(yb))
    }

    fn random_tree(&mut self, n: u64) -> IntervalTreeNode<Rectangle, 2> {
        let intervals = (0..n).map(|_| self.random_rect()).collect();

        IntervalTreeNode::from_intervals(intervals)
    }
}

fn access_n_point(c: &mut Criterion) {
    let mut group = c.benchmark_group("access_n_point");
    for size in (0..9).map(|n| 10u64.pow(n)) {
        group.throughput(Throughput::Elements(size));
        let mut r = RectRng::new();
        let tree = r.random_tree(size);
        let point = r.random_point();
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| tree.range_search(&point));
        });
    }
    group.finish();
}

fn access_n_rect(c: &mut Criterion) {
    let mut group = c.benchmark_group("access_n_rect");
    for size in (0..9).map(|n| 10u64.pow(n)) {
        group.throughput(Throughput::Elements(size));
        let mut r = RectRng::new();
        let tree = r.random_tree(size);
        let rect = r.random_rect();
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| tree.range_search(&rect));
        });
    }
    group.finish();
}

criterion_group!(benches, access_n_point, access_n_rect);
criterion_main!(benches);

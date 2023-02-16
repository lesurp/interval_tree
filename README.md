# K-dimensional Interval Tree

This crates implements a K-dimensional interval tree, based on a binary search
(with the same complexity w.r.t operations).

## Features

* Creation of the tree from a Vec<Interval>
* Overlap / inclusion test
* Overlapping intervals retrieval
* Overlapping volume computation

~~ That's all folks ~~

## TODOs

1. Support insertion!
2. ... and deletion, mutation in general
3. Make API safer: how to get the desired behavior for the dynamic case?) ->
   without using more than one trait...
4. Make API safer: add different overload when "borrowing" is desired, or exact
   same type is expected.
5. Real benchmarks...

# Truthful

[![crates.io](https://img.shields.io/crates/v/truthful?label=latest)](https://crates.io/crates/truthful)
[![Documentation](https://docs.rs/truthful/badge.svg)](https://docs.rs/truthful/)
[![License](https://img.shields.io/crates/l/truthful.svg)]()

A logical expression parser, optimizer and evaluator.

```shell
$ cargo run
welcome! enter ? for help
> a or (a and b)
evaluating "a"
 a  eval
 0   0
 1   1
> !a v b
evaluating (!"a" v "b")
 a  b  eval
 0  0   1
 0  1   1
 1  0   0
 1  1   1
> q
bye!
```

# TODO

* Grammar operator precedence.
* Actual optimization (the current one is a bunch of NAND + DeMorgan naive transformations).

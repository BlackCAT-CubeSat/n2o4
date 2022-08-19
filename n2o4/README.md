# Placeholder `n2o4` crate

`n2o4` is a Rust crate that provides safe, Rustic bindings to
the APIs of [cFE] and [OSAL],
the libraries of the [Core Flight System] (cFS).

**WARNING:** This is not the actual `n2o4` crate!

The _actual_ `n2o4` crate is not currently published on crates.io
due to having a fairly tight binding with the [cFS build system]
(and extensions thereof), such that building this crate outside
the context of a cFS project doesn't make any sense.

You can find the functioning `n2o4` crate,
and how to use it, here:

> <https://github.com/BlackCAT-CubeSat/n2o4>

[cFE]: https://github.com/nasa/cFE
[OSAL]: https://github.com/nasa/osal
[Core Flight System]: https://cfs.gsfc.nasa.gov/
[cFS build system]: https://github.com/nasa/cFE/tree/main/cmake

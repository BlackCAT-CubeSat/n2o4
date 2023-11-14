# `n2o4`

The `n2o4` crate provides safe, idiomatic Rust bindings
to the APIs of [cFE] and [OSAL],
the libraries of the [Core Flight System] (cFS).

## IMPORTANT NOTE

**This is a work in progress.**
While enough has been written to allow some classes of cFS application to be written,
bindings for many parts of the API don't currently exist.

If you need or want to use some part of the API for which a binding doesn't exist yet,
consider [helping](#contrib) to flesh out the crate!

## Limitations

Currently, `n2o4` only supports the following:

* cFE tag `v7.0.0-rc4`
* OSAL tag `v6.0.0-rc4`

Extending support to other versions is an open issue (BlackCAT-Cubesat/n2o4#1).

## Minimum Rust version

`n2o4` requires Rust 1.64.0 or newer.

## Using this crate in your cFS app

See [USING.md](USING.md) for how to set everything up to
use `n2o4` (and Rust in general) in your cFS application.

Also, take a look at [a sample app] demonstrating the use of `n2o4`.

## <span id="contrib" />Contributing

* Found a bug or have a feature request?
  If one [hasn't been created already], please [create an issue].
* Have a question? [Start a discussion].
* Have you written a new feature or a bug fix?
  Please make a [pull request].

## License

This crate is licensed under the [Apache License version 2.0](LICENSE).
This is the same license cFE and OSAL are released under.

## About the name

N<sub>2</sub>O<sub>4</sub> is the chemical formula for [dinitrogen tetroxide] (aka nitrogen tetroxide).
As cFS is intended for spacecraft flight software,
and the Rust community
[has](https://www.redox-os.org/)
a [long](https://github.com/pyo3/pyo3)
[history](https://rustacean.net/)
of [oxidation-related](https://wiki.mozilla.org/Oxidation)
names,
it seems appropriate to give this crate the name of a rocket propellant&mdash;one that's an _oxidizer_.

[![Launch of the Gemini 3 spacecraft on a Titan II rocket, powered in part by N2O4 (the chemical, not the Rust crate)](https://upload.wikimedia.org/wikipedia/commons/thumb/f/fd/Gemini_3.jpg/206px-Gemini_3.jpg)](https://commons.wikimedia.org/wiki/File:Gemini_3.jpg)

[cFE]: https://github.com/nasa/cFE
[OSAL]: https://github.com/nasa/osal
[Core Flight System]: https://cfs.gsfc.nasa.gov/
[a sample app]: https://github.com/BlackCAT-CubeSat/rust_sample_app
[hasn't been created already]: https://github.com/BlackCAT-CubeSat/n2o4/issues
[create an issue]: https://github.com/BlackCAT-CubeSat/n2o4/issues/new
[Start a discussion]: https://github.com/BlackCAT-CubeSat/n2o4/discussions
[pull request]: https://github.com/BlackCAT-CubeSat/n2o4/pulls
[crate of the same name]: https://crates.io/crates/cfs-sys
[dinitrogen tetroxide]: https://en.wikipedia.org/wiki/Dinitrogen_tetroxide

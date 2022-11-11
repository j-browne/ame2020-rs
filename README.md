# ame2020-rs

A parsing library for the [Atomic Mass Evaluation 2020] format

The data is represented by [`Nuclide`], and the parsing is mostly done by [`Iter`].
The data can be collected into a type that implements [`FromIterator`], such as [`Vec`].

[Atomic Mass Evaluation 2020]: https://www-nds.iaea.org/amdc/

## Format

The format is documented in the preamble of the AME data file itself. This library parses data
formatted like the `mass.mas20` file. The rounded version, and previous versions, such as
AME2016 are incompatible.

## Examples

```rust
use ame2020::{Iter, Nuclide};
use std::{fs::File, io::BufReader};

let file = File::open("mass.mas20")?;
let file = BufReader::new(file);
let iter = Iter::new(file);
let data: Vec<Nuclide> = iter.collect::<Result<_, _>>()?;
```

## Features

* `serde`: Provide `Serialize` and `Deserialize` implementations for [serde](https://serde.rs).

## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

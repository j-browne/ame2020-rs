//! A parsing library for the [Atomic Mass Evaluation 2020] format
//!
//! The data is represented by [`Nuclide`], and the parsing is mostly done by [`Iter`].
//! The data can be collected into a type that implements [`FromIterator`], such as [`Vec`].
//!
//! [Atomic Mass Evaluation 2020]: https://www-nds.iaea.org/amdc/
//!
//! # Format
//!
//! The format is documented in the preamble of the AME data file itself. This library parses data
//! formatted like the `mass.mas20` file. The rounded version, and previous versions, such as
//! AME2016 are incompatible.
//!
//! # Examples
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use ame2020::{Iter, Nuclide};
//! use std::{fs::File, io::BufReader};
//!
//! let file = File::open("mass.mas20")?;
//! let file = BufReader::new(file);
//! let iter = Iter::new(file);
//! let data: Vec<Nuclide> = iter.collect::<Result<_, _>>()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! * `serde`: Provide `Serialize` and `Deserialize` implementations for [serde](https://serde.rs).
use arrayvec::ArrayString;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use std::ops::Not;
use std::{
    cmp::Ordering,
    io::{BufRead, Lines},
    ops::{ControlFlow, Range},
};

pub use crate::error::AmeError;

mod error;
#[cfg(test)]
mod tests;

/// A value that has a mean and uncertainty.
///
/// The data may be an estimate (indicated by `is_estimated`).
/// If not, they are based on experimental data.
#[derive(Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Value {
    pub mean: f64,
    pub uncertainty: f64,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Not::not"))]
    pub is_estimated: bool,
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.mean.partial_cmp(&other.mean)
    }
}

/// A type holding the nuclide data.
///
/// # Examples
///
/// ```
/// use ame2020::{Iter, Nuclide};
/// use std::io::Cursor;
///
/// let reader = Cursor::new(r"1
/// 1
/// 0  1    1    0    1  n         8071.31806     0.00044       0.0        0.0     B-    782.3470     0.0004    1 008664.91590     0.00047");
///
/// let mut iter = Iter::new(reader);
/// let nuc = iter.next().unwrap().unwrap();
/// assert_eq!(nuc.n, 1);
/// assert_eq!(nuc.z, 0);
/// assert_eq!(&nuc.element, "n");
/// ```
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Nuclide {
    /// Neutron number
    pub n: u32,
    /// Proton number
    pub z: u32,
    /// Chemical symbol of the element
    pub element: ArrayString<3>,
    /// Mass excess
    ///
    /// The difference between the mass in atomic mass units and the atomic mass number (N+Z).
    pub mass_excess: Value,
    /// Binding energy per nucleon
    pub binding_energy_per_a: Value,
    /// Beta decay energy, if any
    pub beta_decay_energy: Option<Value>,
    /// Atomic Mass in atomic mass units
    pub atomic_mass: Value,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum ReadState {
    Start,
    Preamble,
    Headers,
    Body,
}

/// An iterator that reads AME2020 data.
///
/// # Examples
///
/// ```
/// use ame2020::Iter;
/// use std::io::Cursor;
///
/// // `Cursor` is a type that implements `BufRead`.
/// // Consider using `BufReader` if you have a `File`.
/// let data = Cursor::new(r"1
/// 1
/// 0  1    1    0    1  n         8071.31806     0.00044       0.0        0.0     B-    782.3470     0.0004    1 008664.91590     0.00047");
/// let mut iter = Iter::new(data);
/// assert!(iter.next().is_some());
/// assert!(iter.next().is_none());
///
/// ```
///
/// # Errors
///
/// If a line fails to parse or there is a reading error, [`next`][Self::next] will return `Some(Err)`.
/// Calling `next` again may return `Some`, but the validity of the data is not guaranteed.
pub struct Iter<R: BufRead> {
    lines: Lines<R>,
    state: ReadState,
}

impl<R: BufRead> Iter<R> {
    /// Creates a new `Iter` from `reader`.
    pub fn new(reader: R) -> Self {
        let lines = reader.lines();
        Self {
            lines,
            state: ReadState::Start,
        }
    }

    fn parse_line(&mut self, line: &str) -> ControlFlow<Result<Nuclide, AmeError>> {
        fn range_err(line: &str, range: Range<usize>) -> Result<&str, AmeError> {
            if line.len() < range.end {
                Err(AmeError::TooShortLine)
            } else {
                Ok(line.get(range).ok_or(AmeError::StrIndex)?.trim())
            }
        }

        fn parse_value(
            (s_mean, r_mean): (&str, Range<usize>),
            (s_unc, r_unc): (&str, Range<usize>),
        ) -> Result<Value, AmeError> {
            let mean = range_err(&s_mean.replace('#', "."), r_mean)?.parse()?;
            let uncertainty = range_err(&s_unc.replace('#', "."), r_unc)?.parse()?;
            let is_estimated = s_mean.contains('#');
            Ok(Value {
                mean,
                uncertainty,
                is_estimated,
            })
        }

        fn inner(line: &str) -> Result<Nuclide, AmeError> {
            let n = range_err(line, 4..9)?.parse()?;
            let z = range_err(line, 9..14)?.parse()?;
            let element = ArrayString::from(range_err(line, 20..23)?)
                .expect("the range is 3 and the capacity is 3");
            let mass_excess = parse_value((line, 28..42), (line, 42..54))?;
            let binding_energy_per_a = parse_value((line, 54..67), (line, 68..78))?;
            let beta_decay_energy = (range_err(line, 87..88)? != "*")
                .then(|| parse_value((line, 81..94), (line, 94..105)))
                .transpose()?;

            // the value is given in micro-u, with a space before the 1e6 place.
            // this makes it inconvenient to parse in u.
            //
            // lines don't have the same length, so use `line.len()`. you could use a RangeFrom,
            // but that would require rewriting `parse_value` and `range_err` to be generic, and it
            // would lead to more complicated bounds checks.
            let mut atomic_mass = parse_value((line, 110..123), (line, 123..(line.len())))?;
            atomic_mass.mean *= 1e-6;
            atomic_mass.uncertainty *= 1e-6;
            atomic_mass.mean += f64::from(range_err(line, 106..109)?.parse::<u16>()?);

            Ok(Nuclide {
                n,
                z,
                element,
                mass_excess,
                binding_energy_per_a,
                beta_decay_energy,
                atomic_mass,
            })
        }

        match self.state {
            ReadState::Start => {
                if line.starts_with('1') {
                    self.state = ReadState::Preamble;
                }
                ControlFlow::Continue(())
            }
            ReadState::Preamble => {
                if line.starts_with('1') {
                    self.state = ReadState::Headers;
                }
                ControlFlow::Continue(())
            }
            ReadState::Headers => {
                if line.starts_with('0') {
                    self.state = ReadState::Body;
                    ControlFlow::Break(inner(line))
                } else {
                    ControlFlow::Continue(())
                }
            }
            ReadState::Body => ControlFlow::Break(inner(line)),
        }
    }
}

impl<R: BufRead> Iterator for Iter<R> {
    type Item = Result<Nuclide, AmeError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.lines.next()? {
                Ok(line) => match self.parse_line(&line) {
                    ControlFlow::Continue(()) => continue,
                    ControlFlow::Break(res) => return Some(res),
                },
                Err(e) => return Some(Err(e.into())),
            }
        }
    }
}

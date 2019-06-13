// Rust Bitcoin Developers 2018-2019
// Rust Handshake Developers 2018-2019
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//TODO fix here.
// use crate::protocol::consensus::max_coin;
use std::error;
use std::fmt;
use std::fmt::Write;
use std::str::FromStr;

/// A set of denominations in which an Amount can be expressed.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Denomination {
    /// HNS
    Handshake,
    // dollarydoo
    DollaryDoo,
}

impl Denomination {
    /// The number of decimal places more than a dollarydoo.
    fn precision(self) -> u32 {
        match self {
            Denomination::DollaryDoo => 0,
            Denomination::Handshake => 6,
        }
    }
}

impl fmt::Display for Denomination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Denomination::DollaryDoo => "dollarydoo",
            Denomination::Handshake => "HNS",
        })
    }
}

impl FromStr for Denomination {
    type Err = ParseAmountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dollarydoo" => Ok(Denomination::DollaryDoo),
            "HNS" => Ok(Denomination::Handshake),
            d => Err(ParseAmountError::UnknownDenomination(d.to_owned())),
        }
    }
}

/// An error during [Amount] parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseAmountError {
    /// Amount is too big to fit in an [Amount].
    TooBig,
    /// Amount has higher precision than supported by [Amount].
    TooPrecise,
    /// Invalid number format.
    InvalidFormat,
    /// Input string was too large.
    InputTooLarge,
    /// Invalid character in input.
    InvalidCharacter(char),
    /// The denomination was unknown.
    UnknownDenomination(String),
}

impl fmt::Display for ParseAmountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let desc = ::std::error::Error::description(self);
        match *self {
            ParseAmountError::InvalidCharacter(c) => write!(f, "{}: {}", desc, c),
            ParseAmountError::UnknownDenomination(ref d) => write!(f, "{}: {}", desc, d),
            _ => f.write_str(desc),
        }
    }
}

impl error::Error for ParseAmountError {
    fn cause(&self) -> Option<&error::Error> {
        None
    }

    fn description(&self) -> &'static str {
        match *self {
            ParseAmountError::TooBig => "amount is too big",
            ParseAmountError::TooPrecise => "amount has a too high precision",
            ParseAmountError::InvalidFormat => "invalid number format",
            ParseAmountError::InputTooLarge => "input string was too large",
            ParseAmountError::InvalidCharacter(_) => "invalid character in input",
            ParseAmountError::UnknownDenomination(_) => "unknown denomination",
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Amount(u64);

impl Amount {
    /// The zero amount.
    pub const ZERO: Amount = Amount(0);
    /// Exactly one satoshi.
    pub const ONE_DOO: Amount = Amount(1);
    /// Exactly one bitcoin.
    pub const ONE_HNS: Amount = Amount(1_000_000);

    /// Create an [Amount] with satoshi precision and the given number of satoshis.
    pub fn from_doo(dollary_doo: u64) -> Amount {
        Amount(dollary_doo)
    }

    /// Get the number of satoshis in this [Amount].
    pub fn as_doo(self) -> u64 {
        self.0
    }

    /// The maximum value of an [Amount].
    // TODO implement
    // pub fn max_value() -> Amount {
    //     Amount(max_coin())
    // }

    /// The minimum value of an [Amount].
    pub fn min_value() -> Amount {
        Amount(u64::min_value())
    }

    // Don't use the Inner type in the methods below.
    // Always use [Amount::from_sat] and [Amount::as_sat] instead.

    /// Convert from a value expressing bitcoins to an [Amount].
    pub fn from_hns(hns: f64) -> Result<Amount, ParseAmountError> {
        Amount::from_float_in(hns, Denomination::Handshake)
    }

    /// Parse a decimal string as a value in the given denomination.
    ///
    /// Note: This only parses the value string.  If you want to parse a value
    /// with denomination, use [FromStr].
    pub fn from_str_in(mut s: &str, denom: Denomination) -> Result<Amount, ParseAmountError> {
        if s.len() == 0 {
            return Err(ParseAmountError::InvalidFormat);
        }
        if s.len() > 50 {
            return Err(ParseAmountError::InputTooLarge);
        }

        // let negative = s.chars().next().unwrap() == '-';
        // if negative {
        //     if s.len() == 1 {
        //         return Err(ParseAmountError::InvalidFormat);
        //     }
        //     s = &s[1..];
        // }

        let max_decimals = {
            // The difference in precision between native (satoshi)
            // and desired denomination.
            let precision_diff = denom.precision();
            if precision_diff > 0 {
                // If precision diff is negative, this means we are parsing
                // into a less precise amount. That is not allowed unless
                // there are no decimals and the last digits are zeroes as
                // many as the diffence in precision.
                let last_n = precision_diff as usize;
                if s.contains(".") || s.chars().rev().take(last_n).any(|d| d != '0') {
                    return Err(ParseAmountError::TooPrecise);
                }
                s = &s[0..s.len() - last_n];
                0
            } else {
                precision_diff
            }
        };

        let mut decimals = None;
        let mut value: u64 = 0; // as satoshis
        for c in s.chars() {
            match c {
                '0'...'9' => {
                    // Do `value = 10 * value + digit`, catching overflows.
                    match 10_u64.checked_mul(value) {
                        None => return Err(ParseAmountError::TooBig),
                        Some(val) => match val.checked_add((c as u8 - b'0') as u64) {
                            None => return Err(ParseAmountError::TooBig),
                            Some(val) => value = val,
                        },
                    }
                    // Increment the decimal digit counter if past decimal.
                    decimals = match decimals {
                        None => None,
                        Some(d) if d < max_decimals => Some(d + 1),
                        _ => return Err(ParseAmountError::TooPrecise),
                    };
                }
                '.' => match decimals {
                    None => decimals = Some(0),
                    // Double decimal dot.
                    _ => return Err(ParseAmountError::InvalidFormat),
                },
                c => return Err(ParseAmountError::InvalidCharacter(c)),
            }
        }

        // Decimally shift left by `max_decimals - decimals`.
        let scale_factor = max_decimals - decimals.unwrap_or(0);
        for _ in 0..scale_factor {
            value = match 10_u64.checked_mul(value) {
                Some(v) => v,
                None => return Err(ParseAmountError::TooBig),
            };
        }

        // if negative {
        //     value *= -1;
        // }
        Ok(Amount::from_doo(value))
    }

    /// Parses amounts with denomination suffix like they are produced with
    /// [to_string_with_denomination] or with [fmt::Display].
    /// If you want to parse only the amount without the denomination,
    /// use [from_str_in].
    pub fn from_str_with_denomination(s: &str) -> Result<Amount, ParseAmountError> {
        let mut split = s.splitn(3, " ");
        let amt_str = split.next().unwrap();
        let denom_str = split.next().ok_or(ParseAmountError::InvalidFormat)?;
        if split.next().is_some() {
            return Err(ParseAmountError::InvalidFormat);
        }

        Ok(Amount::from_str_in(amt_str, denom_str.parse()?)?)
    }

    /// Express this [Amount] as a floating-point value in the given denomination.
    ///
    /// Please be aware of the risk of using floating-point numbers.
    pub fn to_float_in(&self, denom: Denomination) -> f64 {
        (self.as_doo() as f64) * 10_f64.powi(denom.precision() as i32)
    }

    /// Express this [Amount] as a floating-point value in Bitcoin.
    ///
    /// Equivalent to `to_float_in(Denomination::Bitcoin)`.
    ///
    /// Please be aware of the risk of using floating-point numbers.
    pub fn as_btc(&self) -> f64 {
        self.to_float_in(Denomination::Handshake)
    }

    /// Convert this [Amount] in floating-point notation with a given
    /// denomination.
    /// Can return error if the amount is too big, too precise or negative.
    ///
    /// Please be aware of the risk of using floating-point numbers.
    pub fn from_float_in(value: f64, denom: Denomination) -> Result<Amount, ParseAmountError> {
        // This is inefficient, but the safest way to deal with this. The parsing logic is safe.
        // Any performance-critical application should not be dealing with floats.
        Amount::from_str_in(&value.to_string(), denom)
    }

    /// Format the value of this [Amount] in the given denomination.
    ///
    /// Does not include the denomination.
    pub fn fmt_value_in(&self, f: &mut fmt::Write, denom: Denomination) -> fmt::Result {
        if denom.precision() > 0 {
            // add zeroes in the end
            let width = denom.precision() as usize;
            write!(f, "{}{:0width$}", self.as_doo(), 0, width = width)?;
        // } else if denom.precision() < 0 {
        //     // need to inject a comma in the number

        //     // let sign = match self.is_negative() {
        //     //     true => "-",
        //     //     false => "",
        //     // };
        //     let sign = "";
        //     let nb_decimals = denom.precision() as usize;
        //     let real = format!("{:0width$}", self.as_doo(), width = nb_decimals);
        //     if real.len() == nb_decimals {
        //         write!(f, "{}0.{}", sign, &real[real.len() - nb_decimals..])?;
        //     } else {
        //         write!(
        //             f,
        //             "{}{}.{}",
        //             sign,
        //             &real[0..(real.len() - nb_decimals)],
        //             &real[real.len() - nb_decimals..]
        //         )?;
        //     }
        } else {
            // denom.precision() == 0
            write!(f, "{}", self.as_doo())?;
        }
        Ok(())
    }

    /// Get a string number of this [Amount] in the given denomination.
    ///
    /// Does not include the denomination.
    pub fn to_string_in(&self, denom: Denomination) -> String {
        let mut buf = String::new();
        self.fmt_value_in(&mut buf, denom).unwrap();
        buf
    }

    /// Get a formatted string of this [Amount] in the given denomination,
    /// suffixed with the abbreviation for the denomination.
    pub fn to_string_with_denomination(&self, denom: Denomination) -> String {
        let mut buf = String::new();
        self.fmt_value_in(&mut buf, denom).unwrap();
        write!(buf, " {}", denom).unwrap();
        buf
    }

    // Some arithmethic that doesn't fit in `std::ops` traits.

    /// Get the absolute value of this [Amount].
    pub fn abs(self) -> Amount {
        Amount(self.0)
    }

    // /// Returns a number representing sign of this [Amount].
    // ///
    // /// - `0` if the Amount is zero
    // /// - `1` if the Amount is positive
    // /// - `-1` if the Amount is negative
    // pub fn signum(self) -> i64 {
    //     self.0.signum()
    // }

    // /// Returns `true` if this [Amount] is positive and `false` if
    // /// this [Amount] is zero or negative.
    // pub fn is_positive(self) -> bool {
    //     self.0.is_positive()
    // }

    // /// Returns `true` if this [Amount] is negative and `false` if
    // /// this [Amount] is zero or positive.
    // pub fn is_negative(self) -> bool {
    //     self.0.is_negative()
    // }

    /// Checked addition.
    /// Returns [None] if overflow occurred.
    pub fn checked_add(self, rhs: Amount) -> Option<Amount> {
        self.0.checked_add(rhs.0).map(Amount)
    }

    /// Checked subtraction.
    /// Returns [None] if overflow occurred.
    pub fn checked_sub(self, rhs: Amount) -> Option<Amount> {
        self.0.checked_sub(rhs.0).map(Amount)
    }

    /// Checked multiplication.
    /// Returns [None] if overflow occurred.
    pub fn checked_mul(self, rhs: u64) -> Option<Amount> {
        self.0.checked_mul(rhs).map(Amount)
    }

    /// Checked integer division.
    /// Be aware that integer division loses the remainder if no exact division
    /// can be made.
    /// Returns [None] if overflow occurred.
    pub fn checked_div(self, rhs: u64) -> Option<Amount> {
        self.0.checked_div(rhs).map(Amount)
    }

    /// Checked remainder.
    /// Returns [None] if overflow occurred.
    pub fn checked_rem(self, rhs: u64) -> Option<Amount> {
        self.0.checked_rem(rhs).map(Amount)
    }

    // /// Subtraction that doesn't allow negative [Amount]s.
    // /// Returns [None] if either [self], [rhs] or the result is strictly negative.
    // pub fn positive_sub(self, rhs: Amount) -> Option<Amount> {
    //     if self.is_negative() || rhs.is_negative() || rhs > self {
    //         None
    //     } else {
    //         self.checked_sub(rhs)
    //     }
    // }
}

#[cfg(test)]
mod tests {
    // Import everything used above
    use super::*;

    #[test]
    fn test_denomination_from_string() {
        let denom = Denomination::from_str("HNS").unwrap();
        assert_eq!(denom, Denomination::Handshake);
        let denom = Denomination::from_str("dollarydoo").unwrap();
        assert_eq!(denom, Denomination::DollaryDoo);
    }
}

// impl Denomination {
//     /// The number of decimal places more than a dollarydoo.
//     fn precision(self) -> u32 {
//         match self {
//             Denomination::DollaryDoo => 0,
//             Denomination::Handshake => 6,
//         }
//     }
// }

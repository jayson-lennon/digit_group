// Copyright 2017 Jayson Lennon
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This crate provides grouping (aka "thousands separators") for numeric types.
//!
//! # Examples
//!
//! Typical use-case to format a number with groups sized 3 (United States):
//!
//! ```
//! use digit_group::FormatGroup;
//!
//! let x = 123456789;
//! assert_eq!(x.format_commas(), "123,456,789")
//! ```
//!
//! Formatting based on SI standards (with custom decimal mark):
//!
//! ```
//! use digit_group::FormatGroup;
//!
//! let x: f64 = 123456789.01234;
//! assert_eq!(x.format_si('.'), "123 456 789.012 34")
//! ```
//!
//! Completely custom decimal mark, grouping delimiter, initial group size, and subsequent group
//! size:
//!
//! ```
//! use digit_group::FormatGroup;
//!
//! let x: f64 = 123456789.01;
//! assert_eq!(x.format_custom('#',':',4,2, false), "1:23:45:6789#01")
//! ```
//!
//! Using `format!` to change the precision of a value prior to grouping:
//!
//! ```
//! use digit_group::custom_group;
//!
//! let val: f64 = 111222.3;
//! let formatted = format!("{:.3}", val);
//! let grouped = custom_group(&formatted, '.', ',', 3, 3, false);
//! assert_eq!(grouped, "111,222.300");
//! ```

#![deny(missing_docs)]
use std::iter::FromIterator;
use std::string::ToString;

/// Groups a pre-formatted number passed as a `&str`. For use with numbers formatted with
/// `format!` to set significant digits, padding, etc.
///
/// `num` is a pre-formatted `&str` of a numeric value.
///
/// `decimal_mark` is the `char` used to delimit the integer and fractional portions of the number.
///
/// `grouping_delimiter` is the delimiter to use between groups.
///
/// `first_group_size` is the number of digits of the initial group.
///
/// `group_size` is the number of digits of subsequent groups.
///
/// `group_fractional_part` determines whether to apply the grouping rules to the fractional
/// part of the number.
///
/// # Examples
///
/// ```
/// # use digit_group::custom_group;
///
/// let val: f64 = 111222.3;
/// let formatted = format!("{:.3}", val);
/// let grouped = custom_group(&formatted, '.', ',', 3, 3, false);
/// assert_eq!(grouped, "111,222.300");
/// ```
pub fn custom_group(num: &str,
                    decimal_mark: char,
                    grouping_delimiter: char,
                    first_group_size: usize,
                    group_size: usize,
                    group_fractional_part: bool)
                    -> String {
    let parts = num.split('.').collect::<Vec<_>>();
    let integer_part = match parts.get(0) {
        Some(num) => {
            groupify_integer(num.chars(),
                             grouping_delimiter,
                             first_group_size,
                             group_size,
                             GroupDirection::RightToLeft)
        }
        None => String::from(""),
    };

    let mut grouped_string = integer_part;

    if let Some(fractional_part) = parts.get(1) {
        grouped_string.push(decimal_mark);

        if group_fractional_part {
            let fractional_grouped = groupify_integer(fractional_part.chars(),
                                                      grouping_delimiter,
                                                      first_group_size,
                                                      group_size,
                                                      GroupDirection::LeftToRight);
            grouped_string.push_str(&fractional_grouped)

        } else {
            grouped_string.push_str(fractional_part)
        }
    }

    grouped_string
}

/// Various formatters provided for integer grouping.
pub trait FormatGroup {
    /// Formats the number according to ISO 80000-1, using a custom `decimal_mark`.
    ///
    /// #Example
    ///
    /// ```
    /// use digit_group::FormatGroup;
    ///
    /// let x: f64 = 123456789.01234;
    /// assert_eq!(x.format_si('.'), "123 456 789.012 34")
    /// ```
    fn format_si(&self, decimal_mark: char) -> String;

    /// Formats the integral value into groups of three, separated by commas.
    ///
    /// #Example
    ///
    /// ```
    /// use digit_group::FormatGroup;
    ///
    /// let x: u64 = 123456789;
    /// assert_eq!(x.format_commas(), "123,456,789")
    /// ```
    fn format_commas(&self) -> String;

    /// Formats the number based on supplied parameters.
    ///
    /// `decimal_mark` is the `char` used to delimit the integer and fractional portions of the
    /// number.
    ///
    /// `grouping_delimiter` is the delimiter to use between groups.
    ///
    /// `first_group_size` is the number of digits of the initial group.
    ///
    /// `group_size` is the number of digits of subsequent groups.
    ///
    /// `group_fractional_part` determines whether to apply the above grouping rules to the decimal
    /// portion of the number.
    ///
    /// #Example
    ///
    /// ```
    /// use digit_group::FormatGroup;
    ///
    /// let x: f64 = 123456789.01;
    /// assert_eq!(x.format_custom('#',':',4,2, false), "1:23:45:6789#01")
    /// ```
    fn format_custom(&self,
                     decimal_mark: char,
                     grouping_delimiter: char,
                     first_group_size: usize,
                     group_size: usize,
                     group_fractional_part: bool)
                     -> String;
}

/// Convenience for `groupify_integer`.
#[derive(PartialEq)]
enum GroupDirection {
    RightToLeft,
    LeftToRight,
}

/// Creates a new `String` from the digits of an integral value with custom grouping rules applied.
///
/// `integral_digits` an iterator over `char`s of an integer string.
///
/// `delimiter` is the delimiter to use between groups.
///
/// `first_group_size` is the number of digits of the initial group.
///
/// `group_size` is the number of digits of subsequent groups.
///
/// `direction` denotes if the groups will be counted from the left or from the right.
/// Use `RightToLeft` for the integer portion of a number and `LeftToRight` for the decimal portion.
fn groupify_integer<T>(integral_digits: T,
                       delimiter: char,
                       first_group_size: usize,
                       group_size: usize,
                       direction: GroupDirection)
                       -> String
    where T: Iterator<Item = char>
{
    let integral_digits = integral_digits.collect::<Vec<char>>();

    // Determine if we have a negative number to account for the hyphen (-).
    let is_negative = {
        match integral_digits.get(0) {
            Some(d) => *d == '-',
            None => false,
        }
    };
    // We need to skip 1 if we have a negative number.
    let skip_negative = {
        if is_negative { 1 } else { 0 }
    };

    let mut delimited_integer = Vec::new();

    // Handle the first group.
    match direction {
        GroupDirection::RightToLeft => {
            // Reverse since we need to start from the decimal and move left away from it.
            for digit in integral_digits.iter().skip(skip_negative).rev().take(first_group_size) {
                delimited_integer.push(*digit)
            }
        }
        GroupDirection::LeftToRight => {
            for digit in integral_digits.iter().skip(skip_negative).take(first_group_size) {
                delimited_integer.push(*digit)
            }
        }
    }

    // Handle subsequent groups.
    match direction {
        GroupDirection::RightToLeft => {
            // Reverse since we need to start from the decimal and move left away from it.
            for (i, digit) in integral_digits.iter()
                .skip(skip_negative)
                .rev()
                .skip(first_group_size)
                .enumerate() {

                // Check if we need to add a delmiiter.
                if i % group_size == 0 {
                    delimited_integer.push(delimiter);
                }
                delimited_integer.push(*digit);
            }
        }
        GroupDirection::LeftToRight => {
            for (i, digit) in integral_digits.iter()
                .skip(skip_negative)
                .skip(first_group_size)
                .enumerate() {

                // Check if we need to add a delmiiter.
                if i % group_size == 0 {
                    delimited_integer.push(delimiter);
                }
                delimited_integer.push(*digit);
            }
        }
    }

    // Add negative sign if needed.
    if is_negative {
        delimited_integer.push('-');
    }

    // We reversed above, so need to change it back for the final value.
    if direction == GroupDirection::RightToLeft {
        delimited_integer.reverse();
    }

    String::from_iter(delimited_integer.into_iter())
}


macro_rules! impl_FormatGroup {
    ($t:ty) => (
        
        impl FormatGroup for $t {
            fn format_si(&self, decimal_mark: char) -> String {
                self.format_custom(decimal_mark, ' ', 3, 3, true)
            }

            fn format_commas(&self) -> String {
                self.format_custom('.', ',', 3, 3, false)
            }

            fn format_custom(&self,
                    decimal_mark: char,
                    grouping_delimiter: char,
                    first_group_size: usize,
                    group_size: usize,
                    group_fractional_part: bool)
                    -> String {
                let stringy_number = self.to_string();
                custom_group(&stringy_number, 
                             decimal_mark, 
                             grouping_delimiter, 
                             first_group_size, 
                             group_size,
                             group_fractional_part)
            }
        }

    )
}

impl_FormatGroup!(i8);
impl_FormatGroup!(i16);
impl_FormatGroup!(i32);
impl_FormatGroup!(i64);
impl_FormatGroup!(isize);

impl_FormatGroup!(u8);
impl_FormatGroup!(u16);
impl_FormatGroup!(u32);
impl_FormatGroup!(u64);
impl_FormatGroup!(usize);

impl_FormatGroup!(f32);
impl_FormatGroup!(f64);

#[cfg(test)]
mod tests {
    use super::{FormatGroup, custom_group};

    #[test]
    fn u64_si() {
        let x: u64 = 1234567891234;
        let s = x.format_si('.');
        assert_eq!(s, "1 234 567 891 234");
    }

    #[test]
    fn i64_si_negative() {
        let x: i64 = -1234567891234;
        let s = x.format_si('.');
        assert_eq!(s, "-1 234 567 891 234");
    }

    #[test]
    fn f64_si_negative() {
        let x: f64 = -123456789.1234567;
        let s = x.format_si('.');
        assert_eq!(s, "-123 456 789.123 456 7");
    }

    #[test]
    fn f64_si() {
        let x: f64 = 123456789.1234567;
        let s = x.format_si('.');
        assert_eq!(s, "123 456 789.123 456 7");
    }

    #[test]
    fn f64_commas() {
        let x: f64 = -123456789.123456;
        let s = x.format_commas();
        assert_eq!(s, "-123,456,789.123456");
    }

    #[test]
    fn f64_custom() {
        let x: f64 = -123456789.123456;
        let s = x.format_custom(',', ':', 2, 3, false);
        assert_eq!(s, "-1:234:567:89,123456");
    }

    #[test]
    fn custom_standalone() {
        let x: f64 = -123456789.123456;
        let formatted = format!("{:.*}", 8, x);
        let s = custom_group(&formatted, '.', ',', 3, 3, false);
        assert_eq!(s, "-123,456,789.12345600");
    }

    #[test]
    fn china() {
        let x: f64 = 1234567.89;
        let s = x.format_custom('.', ',', 4, 3, false);
        assert_eq!(s, "123,4567.89");
    }

    #[test]
    fn india() {
        let x: f64 = 1234567.89;
        let s = x.format_custom('.', ',', 3, 2, false);
        assert_eq!(s, "12,34,567.89");
    }

}

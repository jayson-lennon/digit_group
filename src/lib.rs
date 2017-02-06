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

use std::iter::FromIterator;
use std::string::ToString;

/// Groups a pre-formatted number passed as a `&str`. For use with numbers formatted with
/// `format!()` to set significant digits, padding, etc.
///
/// `num` is a pre-formatted `&str` of a numeric value.
///
/// `decimal_mark` is the `char` used to delimit the integral and fractional portions of the number.
///
/// `grouping_delimiter` is the delimiter to use between groups.
///
/// `first_group_size` is the number of digits of the initial group.
///
/// `group_size` is the number of digits of subsequent groups.
///
/// # Examples
///
/// ```
/// # use decmark::custom_group;
///
/// let val: f64 = 111222.3;
/// let formatted = format!("{:.3}", val);
/// let grouped = custom_group(&formatted, '.', ',', 3, 3);
/// assert_eq!(grouped, "111,222.300");
/// ```
pub fn custom_group(num: &str,
                    decimal_mark: char,
                    grouping_delimiter: char,
                    first_group_size: usize,
                    group_size: usize)
                    -> String {
    let parts = num.split('.').collect::<Vec<_>>();
    let integral = match parts.get(0) {
        Some(n) => n.chars().collect::<Vec<char>>(),
        None => Vec::new(),
    };

    let integral = groupify_integral(&integral, grouping_delimiter, first_group_size, group_size);

    let mut grouped_string = integral;
    if let Some(fractional) = parts.get(1) {
        grouped_string.push(decimal_mark);
        grouped_string.push_str(fractional)
    }

    grouped_string
}

/// Various formatters provided for integral grouping.
pub trait NumberGrouping {
    fn format_si(&self, decimal_mark: char) -> String;
    fn format_commas(&self) -> String;
    fn format_custom(&self,
                     decimal_mark: char,
                     grouping_delimiter: char,
                     first_group_size: usize,
                     group_size: usize)
                     -> String;
}

/// Creates a new `String` from the digits of an integral with custom grouping rules applied.
///
/// `integral_digits' are only the digits before the decimal point of a number.
/// `delimiter` is the delimiter to use between groups.
/// `first_group_size` is the number of digits of the initial group.
/// `group_size` is the number of digits of subsequent groups.
fn groupify_integral(integral_digits: &Vec<char>,
                     delimiter: char,
                     first_group_size: usize,
                     group_size: usize)
                     -> String {
    // Determine if we have a negative number to account for the hyphen (-).
    let is_negative = {
        match integral_digits.get(0) {
            Some(d) => if *d == '-' { true } else { false },
            None => false,
        }
    };
    let skip_negative = {
        if is_negative { 1 } else { 0 }
    };

    // We are reversing the iterators to count from the least significant digit to the most
    // significant digit. This allows simple grouping in the loop.
    let first_group = integral_digits.iter().skip(skip_negative).rev().take(first_group_size);
    let second_group = integral_digits.iter().skip(skip_negative).rev().skip(first_group_size);

    let mut delimited_integral = Vec::new();

    for digit in first_group {
        delimited_integral.push(*digit)
    }

    let mut i = 0;
    for digit in second_group {
        // Check if we need to add a delmiiter.
        if i % group_size == 0 {
            delimited_integral.push(delimiter);
        }
        delimited_integral.push(*digit);
        i += 1;
    }

    // Add negative sign if needed.
    if is_negative {
        delimited_integral.push('-');
    }

    // Vector was built backwards, so we need to reverse it.
    delimited_integral.reverse();

    let stringified = String::from_iter(delimited_integral.into_iter());

    stringified
}


macro_rules! impl_Number_Grouping {
    ($t:ty) => (
        
        impl NumberGrouping for $t {
            fn format_si(&self, decimal_mark: char) -> String {
                self.format_custom(decimal_mark, ' ', 3, 3)
            }

            fn format_commas(&self) -> String {
                self.format_custom('.', ',', 3, 3)
            }

            fn format_custom(&self,
                    decimal_mark: char,
                    grouping_delimiter: char,
                    first_group_size: usize,
                    group_size: usize)
                    -> String {
                let stringy_number = self.to_string();
                custom_group(&stringy_number, 
                             decimal_mark, 
                             grouping_delimiter, 
                             first_group_size, 
                             group_size)
            }
        }

    )
}

impl_Number_Grouping!(i8);
impl_Number_Grouping!(i16);
impl_Number_Grouping!(i32);
impl_Number_Grouping!(i64);
impl_Number_Grouping!(isize);

impl_Number_Grouping!(u8);
impl_Number_Grouping!(u16);
impl_Number_Grouping!(u32);
impl_Number_Grouping!(u64);
impl_Number_Grouping!(usize);

impl_Number_Grouping!(f32);
impl_Number_Grouping!(f64);

#[cfg(test)]
mod tests {
    use super::{NumberGrouping, custom_group};

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
        let x: f64 = -123456789.123456;
        let s = x.format_si('.');
        assert_eq!(s, "-123 456 789.123456");
    }

    #[test]
    fn f64_si() {
        let x: f64 = 123456789.123456;
        let s = x.format_si('.');
        assert_eq!(s, "123 456 789.123456");
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
        let s = x.format_custom(',', ':', 2, 3);
        assert_eq!(s, "-1:234:567:89,123456");
    }

    #[test]
    fn custom_standalone() {
        let x: f64 = -123456789.123456;
        let formatted = format!("{:.*}", 8, x);
        let s = custom_group(&formatted, '.', ',', 3, 3);
        assert_eq!(s, "-123,456,789.12345600");
    }
}

# digit_group

This is a small Rust crate that provides grouping (aka "thousands separators") for numeric types.

## Usage ##

`Cargo.toml`:
```toml
[dependencies]
digit_group = "0.1"
```
`main.rs`:
```Rust
extern crate digit_group;
use digit_group{FormatGroup,custom_group};

fn main() {
    let x: f64 = 12345678.112233;

    // Typical usage. 
    x.format_commas();   // 12,345,678.112233
    x.format_si('.');    // 12 345 678.112 233
    
    // Customizable groupings, decimal marks, and grouping delimiters.
    x.format_custom('#',':',4,2, false); // 12:34:5678#112233
    
    // Customizing precision prior to grouping.
    let y = 5512.332;
    let pre_formatted = format!("{:.4}", x);
    custom_group(&pre_formatted, ',', ' ', 3, 3, false); // 5 512,3320
}
```
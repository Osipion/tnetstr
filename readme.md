# tnetstr

A [tnetstring](https://tnetstrings.info/) parser for Rust

Note that the serde implementation is not finished and *is broken*. This only matters if you want to use serde to de/serialize data.

## Usage

```rust
extern crate tnetstr;

use tnetstr::parse;

fn main() {
    let input = "9:aaaaaaaaa,".as_bytes();
    match parse(&input) {
        Err(e) => panic!("{}", e),
        Ok(value) => println!("{}", value)
    }
}
```

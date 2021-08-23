# Please don't escape

A game written as an entry to the [Rusty Jam](https://itch.io/jam/rusty-jam).

## Install

Rough notes on installation.

## WASM

```bash
# $env:OPENSSL_NO_VENDOR = "1"  # if getting a failed to run custom build command for openssl-sys when installing wasm-pack
cargo install wasm-pack
wasm-pack build --target web --release
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

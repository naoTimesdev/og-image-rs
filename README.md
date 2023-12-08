# naoTimes Open Graph Image

A web server that host/generate an open graph image for naoTimes.

Created originally in JavaScript with express.js, now use Rust + Axum 🚀

**Public**: https://og-api.naoti.me

## MSRV
The following package is developed with Rust 1.72 with support for minimum version of 1.66 following [axum's](https://github.com/tokio-rs/axum) MSRV

## Using
1. Install Rust using rustup
2. Clone this repository
3. Build release version with `cargo build --release --bin naotimes_open_graph`
4. Run the server by executing:
   - `./target/release/naotimes_open_graph`
   - `& ./target/release/naotimes_open_graph.exe`
5. Open http://127.0.0.1:12460 and start using it.

## Config
See [.env.example](.env.example)
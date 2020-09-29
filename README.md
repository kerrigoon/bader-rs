# bader-rs (0.1.0) ![build](https://github.com/kerrigoon/bader-rs/workflows/build/badge.svg?branch=master) [![Latest Version]][crates.io] [![MSRV: rustc 1.40+]][Rust 1.40]
[Latest Version]: https://img.shields.io/crates/v/bader.svg
[crates.io]: https://crates.io/crates/bader
[MSRV: rustc 1.40+]: https://img.shields.io/badge/MSRV-rustc_1.40+-lightgray.svg
[Rust 1.40]: https://blog.rust-lang.org/2019/12/19/Rust-1.40.0.html
Multi-Threaded Bader Charge Partitioning. Based on methods presented in [Yu Min  and Trinkle Dallas R. 2011  J. Che.m Phys. 134 064111] and [W Tang et al 2009 J. Phys.: Condens. Matter 21 084204] with adaptions for multi-threading.
## Installation
### Pre-built Binary
There are pre-built 64bit binaries for Linux, Mac and Windows provided with the source code for the latest [release].
### Cargo
If these binaries don't cover your OS the easiest way to install is via [cargo].
```sh
$ cargo install bader
```
### From Source
To compile from source run the following which will create the ./target/release/bader executable.
```sh
$ git clone -b v0.1.0 https://github.com/kerrigoon/bader-rs
$ cd bader-rs
$ cargo build --verbose --release
```
From here you can either move or link the binary to folder in your path.
```sh
$ mv ./target/release/bader ~/bin
```
## Minimum Supported Rust Version (MSRV)
This crate is guaranteed to compile on stable Rust 1.40.0 and up. It *might* compile with older versions but that may change in any new patch release.
To test this crate requires Rust 1.42.0 and above.
## Usage
For a detailed list of usage options run
```sh
$ bader --help
```
## License
MIT

[//]: # (These are reference links used in the body of this note and get stripped out when the markdown processor does its job. There is no need to format nicely because it shouldn't be seen. Thanks SO - http://stackoverflow.com/questions/4823468/store-comments-in-markdown-syntax)

[release]: <https://github.com/kerrigoon/bader-rs/releases/tag/v0.1.0>
[Yu Min  and Trinkle Dallas R. 2011  J. Che.m Phys. 134 064111]: <https://doi.org/10.1063/1.3553716>
[W Tang et al 2009 J. Phys.: Condens. Matter 21 084204]: <https://doi.org/10.1088/0953-8984/21/8/084204>
[cargo]: <https://doc.rust-lang.org/cargo/getting-started/installation.html>

# bader-rs (0.1.0)
Multi-Threaded Bader Charge Partitioning. Based on methods presented in [Yu Min  and Trinkle Dallas R. 2011  J. Che.m Phys. 134 064111] and [W Tang et al 2009 J. Phys.: Condens. Matter 21 084204] with adaptions for multi-threading.

## Installation 
### Cargo
The easiest way to install is via [cargo].
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

## Usage
For a detailed list of usage options run 
```sh
$ bader --help
```
## License

MIT

[//]: # (These are reference links used in the body of this note and get stripped out when the markdown processor does its job. There is no need to format nicely because it shouldn't be seen. Thanks SO - http://stackoverflow.com/questions/4823468/store-comments-in-markdown-syntax)

[Yu Min  and Trinkle Dallas R. 2011  J. Che.m Phys. 134 064111]: <https://doi.org/10.1063/1.3553716>
[W Tang et al 2009 J. Phys.: Condens. Matter 21 084204]: <https://doi.org/10.1088/0953-8984/21/8/084204>
[cargo]: <https://doc.rust-lang.org/cargo/getting-started/installation.html>

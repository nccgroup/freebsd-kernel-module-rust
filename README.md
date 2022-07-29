## FreeBSD kernel module in Rust

This repo is mostly an updated version of https://github.com/johalun/echo

It has been updated to Rust 2021 with new bindings to the kernel headers and
tested with Rust version `1.64.0-nightly (b1dd22e66 2022-07-09)`

For more information, see the [accompanying blog post](https://research.nccgroup.com/).

### Setup
* Install Rust via Rustup
* `rustup component add rust-src`
* Generate the kernel bindings:
```bash
cargo build -p kernel-sys --target x86_64-unknown-freebsd
```

### Run

```bash
./build.sh
sudo make load
echo "hi rust" > /dev/rustmodule
cat /dev/rustmodule
sudo make unload
```

### Licence
This source code is provided under the terms of the [BSD 2-Clause licence](LICENSE.txt)
and is based on [public-domain work](https://github.com/johalun/echo) by Johannes Lundberg.

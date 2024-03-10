# rope

![Build Status][build-image]
[![Apache 2.0 Licensed][license-image]][license-link]
![Rust 1.64+][rustc-version]

[build-image]: https://github.com/rnbguy/rope/actions/workflows/cargo.yml/badge.svg
[license-image]: https://img.shields.io/badge/License-AGPL%20v3-blue.svg
[license-link]: https://github.com/rnbguy/rope/blob/main/LICENSE
[rustc-version]: https://img.shields.io/badge/Rustc-Stable%201.74.1+-blue.svg

P2P file transfer utility.

![rope transfer](assets/rope.gif)

This is based on my previous project [figo](https://github.com/rnbguy/figo), but
in Rust.

# Installation

```
$ cargo install --git https://github.com/rnbguy/rope
```

`rope` uses mDNS to find each other. Before using, make sure both machines are
connected to the same network.

# Send

```
$ rope send video.mp4
MAGIC: blistering-barnacles
```

# Receive

```
$ rope recv blistering-barnacles
$ ls video.mp4
```

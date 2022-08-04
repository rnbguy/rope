# rope

P2P file transfer utility.

![rope transfer](assets/rope.gif)

This is based on my previous project [figo](https://github.com/rnbguy/figo), but in Rust.

# Installation

```
$ cargo install --git https://github.com/rnbguy/rope
```

`rope` uses mDNS to find each other. Before using, make sure both machines are connected to the same network.

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

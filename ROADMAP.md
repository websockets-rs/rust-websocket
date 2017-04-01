# The Roadmap

## More Docs, Examples and Tests

Easy as that, every method should be tested and documented.
Every use-case should have an example.

## Adding Features

### `net2` Feature

This is a feature to add the `net2` crate which will let us do cool things
like set the option `SO_REUSEADDR` and similar when making TCP connections.

This is discussed in [vi/rust-websocket#2](https://github.com/vi/rust-websocket/pull/2).

### Add Mio & Tokio (Evented Websocket)

There are a lot of issues that would be solved if this was evented, such as:

 - [#88 tokio support](https://github.com/cyderize/rust-websocket/issues/88)
 - [#66 Timeout on recv_message](https://github.com/cyderize/rust-websocket/issues/66)
 - [#6  one client, one thread?](https://github.com/cyderize/rust-websocket/issues/6)

So maybe we should _just_ add `tokio` support, or maybe `mio` is still used and popular.

### Support Permessage-Deflate

We need this to pass more autobahn tests!

### Buffer Reads and Writes

In the old crate the stream was split up into a reader and writer stream so you could
have both a `BufReader` and a `BufWriter` to buffer your operations to gain some speed.
However is doesn't make sense to split the stream up anymore
(see [#83](https://github.com/cyderize/rust-websocket/issues/83))
meaning that we should buffer reads and writes in some other way.

Some work has begun on this, like [#91](https://github.com/cyderize/rust-websocket/pull/91),
but is this enough? And what about writing?


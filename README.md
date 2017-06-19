Rust-WebSocket [![Build Status](https://travis-ci.org/cyderize/rust-websocket.svg?branch=master)](https://travis-ci.org/cyderize/rust-websocket)
==============

Rust-WebSocket is a WebSocket ([RFC6455](http://datatracker.ietf.org/doc/rfc6455/)) library written in Rust.

Rust-WebSocket provides a framework for dealing with WebSocket connections (both clients and servers). The library is currently in an experimental state, but provides functionality for both normal and secure WebSockets, a message level API supporting fragmentation, a data frame level API, and the ability to extend and customize behaviour.

## Installation

To add a library release version from [crates.io](https://crates.io/crates/websocket) to a Cargo project, add this to the 'dependencies' section of your Cargo.toml:

```INI
websocket = "0.20.2"
```

To add the library's Git repository to a Cargo project, add this to your Cargo.toml:

```INI
[dependencies.websocket]

git = "https://github.com/cyderize/rust-websocket.git"
```

And add ```extern crate websocket;``` to your project.

## Usage

The library can be compiled with tests and benches and some extra capabilities on Rust nightly. To enable the nightly features, use `cargo --features nightly ...`.

See the documentation for the latest release of the library [here](http://cyderize.github.io/rust-websocket/doc/websocket), and also the examples, which are located in `/examples` and can be run with:

```
cargo run --example server
```

And in a separate terminal:

```
cargo run --example client
```

## Testing

The library can be tested using `cargo test` to run tests and `cargo bench` to run bench tests.

A number of tests are included, which ensure core WebSocket functionality works as expected. These tests are not yet comprehensive, and are still being worked on.

## Autobahn TestSuite

Rust-WebSocket uses the [Autobahn TestSuite](http://autobahn.ws/testsuite) to test conformance to RFC6455. If you have Autobahn TestSuite installed you can run these tests yourself using the commands:

```
wstest -m fuzzingserver
cargo run --example autobahn-client
```

To test the client implementation, and

```
wstest -m fuzzingclient
cargo run --example autobahn-server
```

To test the server implementation. The spec files are available [here](http://cyderize.github.io/rust-websocket/autobahn).

The results of these tests are available [here](http://cyderize.github.io/rust-websocket/autobahn).

## License

### The MIT License (MIT)

Copyright (c) 2014-2015 Cyderize

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

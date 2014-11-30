Rust-WebSocket
==============

Rust-WebSocket is a WebSocket ([RFC6455](http://datatracker.ietf.org/doc/rfc6455/)) library written in Rust.

Rust-WebSocket attempts to provide a framework for WebSocket connections (both clients and servers). The library is currently in an experimental state, but can work as a simple WebSocket server or client, with more functionality to come. There is some documentation, but no testing code at the moment, however that is being worked on.

## Installation

To add a library release version from [crates.io](https://crates.io/crates/websocket) to a Cargo project, add this to the 'dependencies' section of your Cargo.toml:

```INI
websocket = "~0.3.1"
```

To add the library's Git repository to a Cargo project, add this to your Cargo.toml:

```INI
[dependencies.websocket]

git = "https://github.com/cyderize/rust-websocket.git"
```

And add ```extern crate websocket;``` to your project.

## Usage

See the example code in the [documentation](http://cyderize.github.io/rust-websocket/doc/websocket).

## License

### The MIT License (MIT)

Copyright (c) 2014 Cyderize

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

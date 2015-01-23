#![allow(unstable)]

extern crate websocket;
extern crate test;

use websocket::header::{WebSocketKey, WebSocketAccept};

#[bench]
fn bench_header_accept(b: &mut test::Bencher) {
	let key = WebSocketKey::new();
	b.iter(|| {
		let mut accept = WebSocketAccept::new(&key);
		test::black_box(&mut accept);
	});
}

#[bench]
fn bench_header_accept_serialize(b: &mut test::Bencher) {
	let key = WebSocketKey::new();
	b.iter(|| {
		let accept = WebSocketAccept::new(&key);
		let mut serialized = accept.serialize();
		test::black_box(&mut serialized);
	});
}

#[bench]
fn bench_header_serialize_accept(b: &mut test::Bencher) {
	let key = WebSocketKey::new();
	let accept = WebSocketAccept::new(&key);
	b.iter(|| {
		let mut serialized = accept.serialize();
		test::black_box(&mut serialized);
	});
}
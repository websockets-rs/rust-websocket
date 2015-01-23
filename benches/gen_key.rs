#![allow(unstable)]

extern crate websocket;
extern crate test;

use websocket::header::WebSocketKey;

#[bench]
fn bench_header_key(b: &mut test::Bencher) {
	b.iter(|| {
		let mut key = WebSocketKey::new();
		test::black_box(&mut key);
	});
}

#[bench]
fn bench_header_key_serialize(b: &mut test::Bencher) {
	b.iter(|| {
		let key = WebSocketKey::new();
		let mut serialized = key.serialize();
		test::black_box(&mut serialized);
	});
}

#[bench]
fn bench_header_serialize_key(b: &mut test::Bencher) {
	let key = WebSocketKey::new();
	b.iter(|| {
		let mut serialized = key.serialize();
		test::black_box(&mut serialized);
	});
}
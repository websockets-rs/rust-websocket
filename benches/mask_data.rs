#![allow(unstable)]

extern crate websocket;
extern crate test;

use websocket::ws::util::mask;

#[bench]
fn bench_mask_data(b: &mut test::Bencher) {
	let buffer = b"The quick brown fox jumps over the lazy dog";
	let key = mask::gen_mask();
	b.iter(|| {
		let mut output = mask::mask_data(key, buffer);
		test::black_box(&mut output);
	});
}

#[bench]
fn bench_gen_mask(b: &mut test::Bencher) {
	b.iter(|| {
		let mut key = mask::gen_mask();
		test::black_box(&mut key);
	});
}
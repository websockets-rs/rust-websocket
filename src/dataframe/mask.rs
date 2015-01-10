//! Functions for masking payload data
#![stable]
use std::rand;

/// Masks data to send to a server
#[stable]
pub fn mask_data(key: [u8; 4], buf: &[u8]) -> Vec<u8> {
	let mut out = Vec::new();
	let mut zip_iter = buf.iter().zip(key.iter().cycle());
	for (&buf_item, &key_item) in zip_iter {
		out.push((buf_item ^ key_item) as u8);
	}
	out
}

/// Generates a random masking key
#[stable]
pub fn gen_mask() -> [u8; 4] {
	[rand::random::<u8>(), rand::random::<u8>(), rand::random::<u8>(), rand::random::<u8>()]
}

#[test]
fn test_mask_data() {
	let key = [1u8, 2u8, 3u8, 4u8];
	let original = vec![10u8, 11u8, 12u8, 13u8, 14u8, 15u8, 16u8, 17u8];
	let expected = vec![11u8, 9u8, 15u8, 9u8, 15u8, 13u8, 19u8, 21u8];
	let obtained = mask_data(key, original.as_slice());
	let reversed = mask_data(key, obtained.as_slice());
	
	assert_eq!(original, reversed);
    assert_eq!(obtained, expected);
}
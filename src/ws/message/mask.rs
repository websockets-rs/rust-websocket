pub fn mask_data(key: [u8, ..4], buf: &[u8]) -> Vec<u8> {
	let mut out: Vec<u8> = Vec::new();
	for i in range(0, buf.len()) {
		out.push(buf[i] ^ key[i % 4u] as u8);
	}
	out
}

#[test]
fn test_mask_data() {
	let key = [1u8, 2u8, 3u8, 4u8];
	let buf = vec![10u8, 11u8, 12u8, 13u8, 14u8, 15u8, 16u8, 17u8];
	
	assert_eq!(buf, mask_data(key, mask_data(key, buf.as_slice()).as_slice()));
	
    assert_eq!(
		mask_data(key, buf.as_slice()),
		vec![11u8, 9u8, 15u8, 9u8, 15u8, 13u8, 19u8, 21u8]
	);
}
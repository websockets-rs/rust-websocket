pub fn mask_data(key: [u8, ..4], buf: &[u8]) -> Vec<u8> {
	let mut out: Vec<u8> = Vec::new();
	for i in range(0, buf.len()) {
		out.push(buf[i] ^ key[i % 4u] as u8);
	}
	out
}
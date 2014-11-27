use std::num::Int;

pub fn sha1(mesg: &[u8]) -> [u8, ..20] {
	let ml: u64 = mesg.len() as u64 * 8u64;
	
	let mut message: Vec<u8> = Vec::new();
	message.push_all(mesg);
	message.push(0x80);
	
	while message.len() % 64 != 56 {
		message.push(0x00);
	}
	
	for i in range(0, 8) {
		message.push((ml >> 56 - i * 8) as u8);
	}
	
	let mut h: [u32, ..5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
	
	let mut count = 0u32;
	
	while count < message.len() as u32 {
		let mut w = [0u32, ..80];
		
		for i in range(0, 16) {
			w[i] = message[count as uint] as u32 << 24;
			w[i] |= message[(count + 1) as uint] as u32 << 16;
			w[i] |= message[(count + 2) as uint] as u32 << 8;
			w[i] |= message[(count + 3) as uint] as u32;
			count += 4;
		}
		
		for i in range(16, 80) {
			w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
		}
		
		let mut z: [u32, ..5] = [h[0], h[1], h[2], h[3], h[4]];
		
		for i in range(0, 80) {
			let mut f: u32;
			let mut k: u32;
			if i < 20 {
				f = (z[1] & z[2]) | (!z[1] & z[3]);
				k = 0x5A827999;
			}
			else if i < 40 {
				f = z[1] ^ z[2] ^ z[3];
				k = 0x6ED9EBA1;
			}
			else if i < 60 {
				f = (z[1] & z[2]) | (z[1] & z[3]) | (z[2] & z[3]); 
				k = 0x8F1BBCDC;
			}
			else {
				f = z[1] ^ z[2] ^ z[3];
				k = 0xCA62C1D6;
			}
			let temp: u32 = z[0].rotate_left(5) + f + z[4] + k + w[i];
			z[4] = z[3];
			z[3] = z[2];
			z[2] = z[1].rotate_left(30);
			z[1] = z[0];
			z[0] = temp;
		}
		
		for i in range(0, 5) {
			h[i] += z[i];
		}
	}
	
	let mut output = [0u8, ..20];
	
	for i in range(0, 5) {
		output[i * 4] = (h[i] >> 24) as u8;
		output[i * 4 + 1] = (h[i] >> 16) as u8;
		output[i * 4 + 2] = (h[i] >> 8) as u8;
		output[i * 4 + 3] = h[i] as u8;
	}
	
	output
}

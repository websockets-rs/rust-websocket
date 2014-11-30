pub fn str_eq_ignore_case(one: &str, two: &str) -> bool {
	let ascii_1 = match one.as_slice().to_ascii_opt() {
		Some(ascii) => { ascii }
		None => { return false; }
	};
	let ascii_2 = match two.as_slice().to_ascii_opt() {
		Some(ascii) => { ascii }
		None => { return false; }
	};
	if ascii_1.len() != ascii_2.len() { return false; } 
	
	for i in range(0, ascii_1.len()) {
		if ascii_1[i].to_lowercase() != ascii_2[i].to_lowercase() {
			return false;
		}
	}
	
	true
}

#[test]
fn test_str_eq_ignore_case() {
    assert!(str_eq_ignore_case("ABCdefghiJKL", "aBcDeFgHiJkL"));
}
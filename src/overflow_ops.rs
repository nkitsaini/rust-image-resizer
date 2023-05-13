

type Num = std::num::NonZeroU32;
// (a * b) /c


// (a/c) * b
// (a * (b/c))

// ((a%c) * b) /c
// (a * (b%c)) /c
// ((ax * c + ay) * (bx * c + by))

// ((ax * c + ay) * (bx * c + by))
// (ax * c * abx * c) + (ay*by) + (ax*c * by) + (bx * c * by)
// (ax * bx * c) + (ay*by)/c + (ax* by) + (bx  * by)

// (a *b)/c
// (a * (c - 1))/c
// (a - a/c)


// a*b = d*c + x
// a*b = d*c + x
// (d)

// ((c -x) * (c - y))/c
// (c**2 -x*c -y*c + x*y)/c
// (c -x -y) + (x*y)/c


// ((sqrt_x *j) * (sqrt_y * k))/c
// ((a_1 + a_2) * (b_1 + b_2))/c

// (a_1 * b_1 + (a_2 * b_1) + (a_1 *b_2) + (b_1 * b_2))/c

// ((a_1 + a_2) * (b_1 + b_2))/c
// ((a_1 + a_2) * (b_1 + b_2))/c

// ((a) * (c - y))/c
// ((a) * (c - y))/c

fn _add_div(num1: u32, num2: u32, denom: u32) -> (u32, u32) {
	debug_assert!(num1 < denom);
	debug_assert!(num2 < denom);
	if (denom - num1 > num2) {
		let sum = num1 + num2;
		return (sum/denom, sum%denom)
	} else {
		return (1, num2 - (denom-num1))
	}
}
// returns (quotient, remainder)
fn _mul_div_small(num1: u32, num2: u32, denom: u32) -> (u32, u32) {
	debug_assert!(num1 < denom);
	debug_assert!(num2 < denom);
	if (num1 == 0 || num2 == 0) {
		return (0, 0)
	} else if (u32::MAX/num1 >= num2) {
		return ((num1 * num2) / denom, (num1 * num2)% denom)
	} else {
		let num1_a = num1/2;
		let num1_b = num1-num1_a;
		let num2_a = num2/2;
		let num2_b = num2-num2_a;
		// (a_1 * b_1 + (a_2 * b_1) + (a_1 *b_2) + (b_1 * b_2))/c
		let (f1_q, f1_r) = _mul_div_small(num1_a, num2_a, denom);
		let (f2_q, f2_r) = _mul_div_small(num1_b, num2_a, denom);
		let (f3_q, f3_r) = _mul_div_small(num1_a, num2_b, denom);
		let (f4_q, f4_r) = _mul_div_small(num1_b, num2_b, denom);
		let (r_sum_q, r_sum_r) = {
			let (s1_q, s1_r) = _add_div(f1_r, f2_r, denom);
			let (s2_q, s2_r) = _add_div(f3_r, f4_r, denom);
			let (s3_q, s3_r) = _add_div(s1_r, s2_r, denom);
			(s1_q + s2_q + s3_q , s3_r)
		};
		return (f1_q + f2_q + f3_q + f4_q + r_sum_q, r_sum_r)
	}

	// todo!()


}

/// Only returns if the resulting number won't overflow
pub fn mul_div(num1: u32, num2: u32, denom: u32) -> Option<u32> {
	if denom == 0 {
		return None;
	}
	let num1_q = num1 / denom;
	let num1_r = num1 % denom;

	let num2_q = num2 / denom;
	let num2_r = num2 % denom;

	// let forth_factor = (num1_r * num2_r) / denom;
	let forth_factor = _mul_div_small(num1_r, num2_r, denom).0;

	Some(
		(num1_q * num2_q * denom) +
		(num1_r * num2_q) +
		(num1_q * num2_r) +
		forth_factor
	)
}


/// Only returns if the resulting number won't overflow
pub fn mul_div3(num1: u32, num2: u32, denom: u32) -> Option<u32> {
		let res = (num1 as u64) * (num2 as u64)/ denom as u64;
		u32::try_from(res).ok()
}

/// splits into (x_high, x_low) representing first 16 and later 16 bits
/// 
fn split(x: u32) -> (u32, u32) {
	return (x >> 16, x & (1 << 16)) 
}

/// Only returns if the resulting number won't overflow
// pub fn mul_div2(num1: u32, num2: u32, denom: u32) -> Option<u32> {
// 	if denom == 0 {
// 		return None;
// 	}
// 	let (num1_high, num1_low) = split(num1);
// 	let (num2_high, num2_low) = split(num2);
// 	let (denom_high, denom_low) = split(num2);

// 	let result1 = num1_high * num2_high;
// 	let result2 = num1_high * num2_low;
// 	let result3 = num1_low * num2_high;
// 	let result4 = num1_low * num2_low;

// 	let (result1_q, result1_r) = (result1/denom, result1%denom);
// 	let (result2_q, result2_r) = (result2/denom, result2%denom);
// 	let (result3_q, result3_r) = (result3/denom, result3%denom);
// 	let (result4_q, result4_r) = (result4/denom, result4%denom);


// 	// let forth_factor = (num1_r * num2_r) / denom;
// 	let forth_factor = _mul_div_small(num1_r, num2_r, denom).0;

// 	Some(
// 		(num1_q * num2_q * denom) +
// 		(num1_r * num2_q) +
// 		(num1_q * num2_r) +
// 		forth_factor
// 	)
// }


#[cfg(test)]
mod tests {
	use std::num::NonZeroU32;

// Bring the macros and other important things into scope.
	use proptest::{prelude::*, strategy::W};
	use super::*;
	fn test(num1: u32, num2: u32, denom: u32) {
		let lnum1 = num1 as u128;
		let lnum2 = num2 as u128;
		let ldenom = denom as u128;

		let nnum1 = NonZeroU32::new(num1).unwrap();
		let nnum2 = NonZeroU32::new(num2).unwrap();
		let ndenom = NonZeroU32::new(denom).unwrap();

		let out = (lnum1 * lnum2) / ldenom;
		if out > u32::MAX as u128 {
			// assert_eq!(mul_div(num1, num2, denom), None);
		} else {
			assert_eq!(mul_div(num1, num2, denom), Some(out as u32));
		}

	}

	// #[test]
	// fn test_1() { test(1, 1, 1) }


	// #[test]
	// fn test_basic() { test(16, 12, 4) }

	// #[test]
	// fn test_basic() { test(16, 12, 4) }

	macro_rules! mul_div_test {
		($($name:ident: $value:expr,)*) => {
			$(
				#[test]
				fn $name() {
					let (n1, n2, denom) = $value;
					test(n1, n2, denom)
				}
			)*
		};
	}
	mul_div_test! {
		manual_1: (1, 1, 1),
		manual_basic: (12, 16, 4),
		manual_lose_1: (2, 16, 4),
		manual_lose_2: (12, 2, 4),
		manual_lose_all: (3, 3, 4),
		// manual_lose_all: (500, 500, 100),
		// manual_lose_all: (200, 200, 90),

		manual_overflow: (131072, 131072, 131072),
		manual_lose_overflow: (131071, 131071, 131072),
	}

	proptest! {
		#[ignore] // only to be run manually, otherwise takes too much time
		#[test]
		fn props(num1 in 1u32..u32::MAX, num2 in 1u32..u32::MAX, denom in 1u32..u32::MAX) {
			test(num1, num2, denom)
			// let lnum1 = num1 as u128;
			// let lnum2 = num2 as u128;
			// let ldenom = denom as u128;

			// let nnum1 = NonZeroU32::new(num1).unwrap();
			// let nnum2 = NonZeroU32::new(num2).unwrap();
			// let ndenom = NonZeroU32::new(denom).unwrap();

			// let out = (lnum1 * lnum2) / ldenom;
			// if out > u32::MAX as u128 {
			// 	assert_eq!(mul_div(num1, num2, denom), None);
			// } else {
			// 	assert_eq!(mul_div(num1, num2, denom), Some(out as u32));
			// }
		}
	}
}
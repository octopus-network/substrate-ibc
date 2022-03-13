//! Benchmarking setup for pallet-template

// use super::*;

// #[allow(unused)]
// use crate::Pallet as Template;
// use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
// use frame_system::RawOrigin;

// benchmarks! {
// 	deliver {
// 		let b in 0..u8::MAX.into();
// 		let m in 0..u8::MAX.into();
// 		let mut v1: Vec<u8> = Vec::new();
// 		let mut v2: Vec<u8> = Vec::new();
// 		for i in 0..m {
// 			v1.push(b.try_into().unwrap());
// 			v2.push(b.try_into().unwrap());
// 		}
// 		let any = Any {type_url: v1, value: v2};
// 		let caller = whitelisted_caller();
// 	}: deliver(RawOrigin::Signed(caller), vec![any], b.try_into().unwrap())
// }
// impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test,);


//! Autogenerated weights for `thea_message_handler`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-16, STEPS: `100`, REPEAT: `200`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `Ubuntu-2204-jammy-amd64-base`, CPU: `Intel(R) Core(TM) i7-7700 CPU @ 3.60GHz`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./polkadex-node
// benchmark
// pallet
// --pallet
// thea-message-handler
// --steps
// 100
// --repeat
// 200
// --extrinsic
// *
// --output
// liquidity_weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `thea_message_handler`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> crate::WeightInfo for WeightInfo<T> {
	/// Storage: TheaMH Authorities (r:0 w:1)
	/// Proof Skipped: TheaMH Authorities (max_values: None, max_size: None, mode: Measured)
	/// Storage: TheaMH ValidatorSetId (r:0 w:1)
	/// Proof Skipped: TheaMH ValidatorSetId (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `b` is `[0, 4294967295]`.
	fn insert_authorities(_b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 4_333_000 picoseconds.
		Weight::from_parts(4_725_322, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: TheaMH IncomingNonce (r:1 w:1)
	/// Proof Skipped: TheaMH IncomingNonce (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TheaMH ValidatorSetId (r:1 w:0)
	/// Proof Skipped: TheaMH ValidatorSetId (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TheaMH IncomingMessages (r:0 w:1)
	/// Proof Skipped: TheaMH IncomingMessages (max_values: None, max_size: None, mode: Measured)
	fn incoming_message() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `108`
		//  Estimated: `1593`
		// Minimum execution time: 29_511_000 picoseconds.
		Weight::from_parts(31_150_000, 0)
			.saturating_add(Weight::from_parts(0, 1593))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: TheaMH IncomingNonce (r:1 w:1)
	/// Proof Skipped: TheaMH IncomingNonce (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `b` is `[1, 4294967295]`.
	fn update_incoming_nonce(_b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `1561`
		// Minimum execution time: 5_244_000 picoseconds.
		Weight::from_parts(5_667_211, 0)
			.saturating_add(Weight::from_parts(0, 1561))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
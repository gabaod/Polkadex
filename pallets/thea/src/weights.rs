
//! Autogenerated weights for `thea`
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
// thea
// --steps
// 100
// --repeat
// 200
// --extrinsic
// *
// --output
// thea_weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `thea`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> crate::TheaWeightInfo for WeightInfo<T> {
	/// Storage: Thea NetworkPreference (r:0 w:1)
	/// Proof Skipped: Thea NetworkPreference (max_values: None, max_size: None, mode: Measured)
	/// The range of component `b` is `[1, 255]`.
	fn update_network_pref(_b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 6_201_000 picoseconds.
		Weight::from_parts(6_780_598, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Thea IncomingNonce (r:1 w:1)
	/// Proof Skipped: Thea IncomingNonce (max_values: None, max_size: None, mode: Measured)
	/// Storage: Thea IncomingMessages (r:0 w:1)
	/// Proof Skipped: Thea IncomingMessages (max_values: None, max_size: None, mode: Measured)
	/// The range of component `b` is `[0, 256]`.
	fn incoming_message(_b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `151`
		//  Estimated: `3616`
		// Minimum execution time: 17_766_000 picoseconds.
		Weight::from_parts(21_573_801, 0)
			.saturating_add(Weight::from_parts(0, 3616))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Thea ValidatorSetId (r:1 w:0)
	/// Proof Skipped: Thea ValidatorSetId (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Thea Authorities (r:1 w:0)
	/// Proof Skipped: Thea Authorities (max_values: None, max_size: None, mode: Measured)
	/// Storage: Thea OutgoingNonce (r:1 w:1)
	/// Proof Skipped: Thea OutgoingNonce (max_values: None, max_size: None, mode: Measured)
	/// Storage: Thea OutgoingMessages (r:0 w:1)
	/// Proof Skipped: Thea OutgoingMessages (max_values: None, max_size: None, mode: Measured)
	/// The range of component `b` is `[0, 256]`.
	fn send_thea_message(b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `315`
		//  Estimated: `3780`
		// Minimum execution time: 335_027_000 picoseconds.
		Weight::from_parts(343_538_270, 0)
			.saturating_add(Weight::from_parts(0, 3780))
			// Standard Error: 582
			.saturating_add(Weight::from_parts(41_447, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Thea IncomingNonce (r:1 w:1)
	/// Proof Skipped: Thea IncomingNonce (max_values: None, max_size: None, mode: Measured)
	/// The range of component `b` is `[1, 50000]`.
	fn update_incoming_nonce(_b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `151`
		//  Estimated: `3616`
		// Minimum execution time: 6_340_000 picoseconds.
		Weight::from_parts(6_869_722, 0)
			.saturating_add(Weight::from_parts(0, 3616))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Thea OutgoingNonce (r:1 w:1)
	/// Proof Skipped: Thea OutgoingNonce (max_values: None, max_size: None, mode: Measured)
	/// The range of component `b` is `[1, 50000]`.
	fn update_outgoing_nonce(_b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `182`
		//  Estimated: `3647`
		// Minimum execution time: 7_432_000 picoseconds.
		Weight::from_parts(7_978_131, 0)
			.saturating_add(Weight::from_parts(0, 3647))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
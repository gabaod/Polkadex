// This file is part of Polkadex.
//
// Copyright (c) 2023 Polkadex oü.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Autogenerated weights for `liquidity`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-07, STEPS: `100`, REPEAT: 200, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `Ubuntu-2204-jammy-amd64-base`, CPU: `Intel(R) Core(TM) i7-7700 CPU @ 3.60GHz`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./polkadex-node
// benchmark
// pallet
// --pallet
// liquidity
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

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `liquidity`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> super::WeightInfo for WeightInfo<T> {
	// Storage: Liquidity RegisterGovernanceAccounts (r:1 w:1)
	// Storage: OCEX ExchangeState (r:1 w:0)
	// Storage: OCEX Accounts (r:1 w:1)
	// Storage: OCEX IngressMessages (r:1 w:1)
	/// The range of component `a` is `[0, 4294967295]`.
	fn register_account(_a: u32, ) -> Weight {
		// Minimum execution time: 28_213 nanoseconds.
		Weight::from_ref_time(29_550_341)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Liquidity RegisterGovernanceAccounts (r:1 w:0)
	// Storage: OCEX ExchangeState (r:1 w:0)
	// Storage: OCEX AllowlistedToken (r:1 w:0)
	// Storage: OCEX Accounts (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: OCEX TotalAssets (r:1 w:1)
	// Storage: OCEX IngressMessages (r:1 w:1)
	/// The range of component `a` is `[1, 4294967295]`.
	/// The range of component `i` is `[0, 4294967295]`.
	/// The range of component `z` is `[10, 4294967295]`.
	fn deposit_to_orderbook(_a: u32, _i: u32, _z: u32, ) -> Weight {
		// Minimum execution time: 68_563 nanoseconds.
		Weight::from_ref_time(70_675_691)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: Liquidity RegisterGovernanceAccounts (r:1 w:0)
	// Storage: OCEX ExchangeState (r:1 w:0)
	// Storage: OCEX AllowlistedToken (r:1 w:0)
	// Storage: OCEX Accounts (r:1 w:0)
	// Storage: OCEX IngressMessages (r:1 w:1)
	/// The range of component `a` is `[1, 4294967295]`.
	/// The range of component `i` is `[0, 4294967295]`.
	/// The range of component `z` is `[10, 4294967295]`.
	fn withdraw_from_orderbook(_a: u32, _i: u32, _z: u32, ) -> Weight {
		// Minimum execution time: 34_038 nanoseconds.
		Weight::from_ref_time(35_636_530)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

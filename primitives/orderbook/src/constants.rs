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

//! This module contains constants definitions related to the "Orderbook".

use polkadex_primitives::Balance;

/// The designated SS58 prefix of this chain.
pub const POLKADEX_MAINNET_SS58: u16 = 88;

pub const MAX_WITHDRAWALS_PER_SNAPSHOT: u8 = 20;
pub const UNIT_BALANCE: Balance = 1_000_000_000_000_u128;
/// Range of QTY: 0.00000001 to 10,000,000 UNITs
pub const MIN_QTY: Balance = UNIT_BALANCE / 10000000;
pub const MAX_QTY: Balance = 10000000 * UNIT_BALANCE;
/// Range of PRICE: 0.00000001 to 10,000,000 UNITs
pub const MIN_PRICE: Balance = UNIT_BALANCE / 10000000;
pub const MAX_PRICE: Balance = 10000000 * UNIT_BALANCE;

#[test]
pub fn test_overflow_check() {
	assert!(MAX_PRICE.checked_mul(MAX_QTY).is_some());
}

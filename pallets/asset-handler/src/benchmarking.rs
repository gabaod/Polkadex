// This file is part of Polkadex.

// Copyright (C) 2020-2022 Polkadex oü.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

use crate::pallet::*;
use codec::{Decode, Encode};
use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use sp_core::H160;
use sp_runtime::SaturatedConversion;

const SEED: u32 = 0;

// Check if last event generated by pallet is the one we're expecting
fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	create_asset {
		let b in 0 .. 255;
		let chain_id = 1;
		let id = H160::from_slice(&[b as u8; 20]);
		let rid = chainbridge::derive_resource_id(chain_id, &id.0);
	}: _(RawOrigin::Root, chain_id, id)
	verify {
		assert_last_event::<T>(Event::AssetRegistered(rid).into());
	}

	mint_asset {
		let b in 1 .. 1000;
		let chain_id = 1;
		let id = H160::from_slice(&[b as u8; 20]);
		let rid = chainbridge::derive_resource_id(chain_id, &id.0);
		Pallet::<T>::register_asset(rid);
		let relayer: T::AccountId = account("relayer", 1, SEED);
		chainbridge::pallet::Pallet::<T>::insert_relayer(relayer.clone());
		let recipient: T::AccountId = account("recipient", b, SEED);
		let encoded_recipient = recipient.encode();
		let recipient: [u8;32] = encoded_recipient.as_slice().try_into().unwrap();
		let destination_acc = T::AccountId::decode(&mut &recipient[..]).unwrap();
		let amount = b as u128;
	}: _(RawOrigin::Signed(relayer), recipient.clone().to_vec(), amount.clone(), rid)
	verify {
		assert_last_event::<T>(Event::AssetDeposited(destination_acc, rid, amount).into());
	}

	set_bridge_status {
		let status = true;
	}: _(RawOrigin::Root, status)
	verify {
		assert_last_event::<T>(Event::BridgeStatusUpdated(status).into());
	}

    set_block_delay {
		let block_delay = 10u64;
		let block_delay = block_delay.saturated_into::<T::BlockNumber>();
	}: _(RawOrigin::Root, block_delay)
	verify {
		assert_last_event::<T>(Event::BlocksDelayUpdated(block_delay).into());
	}

	update_fee {
		let m in 1 .. 100;
		let f in 1 .. 1000;
		let chain_id = m as u8;
		let min_fee = (m as u128).saturated_into::<BalanceOf<T>>();
		let fee_scale = f as u32;
	}: _(RawOrigin::Root, chain_id, min_fee, fee_scale)
	verify {
		assert_last_event::<T>(Event::FeeUpdated(chain_id, min_fee).into());
	}

	withdraw {
		let b in 10 .. 1000;
		let c in 1010 .. 2000;
		let chain_id = 0;
		chainbridge::pallet::Pallet::<T>::whitelist(chain_id);
		let id = H160::from_slice(&[1; 20]);
		let rid = chainbridge::derive_resource_id(chain_id, &id.0);
		Pallet::<T>::register_asset(rid);
		let account: T::AccountId = account("withdraw", b, SEED);
		let deposit_amount = 1000;
		Pallet::<T>::mint_token(account.clone(), rid, deposit_amount);
		let withdraw_amount = (100 as u128).saturated_into::<BalanceOf<T>>();
		// Set Fee
		let recipeint = H160::from_slice(&[c as u8; 20]);
	}: _(RawOrigin::Signed(account), chain_id, id, withdraw_amount, recipeint)
	verify {
		assert_last_event::<T>(Event::AssetWithdrawn(id, rid, withdraw_amount).into());
	}

}
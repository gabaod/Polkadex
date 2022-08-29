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
use codec::Decode;
use frame_support::{assert_noop, assert_ok, BoundedVec, ensure};
use sp_core::{H160, U256};
use sp_runtime::TokenError;

use crate::{mock::{new_test_ext, Test, *}, mock, pallet::*};

const ASSET_ADDRESS: &str = "0x0Edd7B63bDc5D0E88F7FDd8A38F802450f458fBC";
const RECIPIENT_ADDRESS: &str = "0x0Edd7B63bDc5D0E88F7FDd8A38F802450f458fBA";

#[test]
pub fn test_create_asset_will_successfully_create_asset() {
	let (asset_address, recipient, chain_id) = create_asset_data();

	new_test_ext().execute_with(|| {
		assert_ok!(AssetHandler::create_asset(Origin::signed(recipient), chain_id, asset_address));
	});
}

#[test]
pub fn test_create_asset_with_already_existed_asset_will_return_in_use_error() {
	let (asset_address, recipient, chain_id) = create_asset_data();

	new_test_ext().execute_with(|| {
		assert_ok!(AssetHandler::create_asset(Origin::signed(recipient), chain_id, asset_address));
		let rid = chainbridge::derive_resource_id(chain_id, &asset_address.0);
		let asset_id = AssetHandler::convert_asset_id(rid);
		assert_ok!(Assets::mint(
			Origin::signed(ChainBridge::account_id()),
			asset_id,
			recipient,
			100
		));
		assert_eq!(Assets::balance(asset_id, recipient), 100);

		// Re-register Asset
		assert_noop!(
			AssetHandler::create_asset(Origin::signed(recipient), chain_id, asset_address),
			pallet_assets::Error::<Test>::InUse
		);
	});
}

#[test]
pub fn test_mint_asset_with_not_registered_asset_will_return_unknown_asset_error() {
	let (asset_address, relayer, recipient, recipient_account, chain_id, account) =
		mint_asset_data();

	new_test_ext().execute_with(|| {
		let rid = chainbridge::derive_resource_id(chain_id, &asset_address.0);
		let asset_id = AssetHandler::convert_asset_id(rid);
		// Add new Relayer and verify storage
		assert_ok!(ChainBridge::add_relayer(Origin::signed(account), relayer));
		assert!(ChainBridge::relayers(relayer));

		// Assert `mint_asset` will fail
		assert_noop!(
			AssetHandler::mint_asset(
				Origin::signed(ChainBridge::account_id()),
				recipient.to_vec(),
				100000000000,
				rid
			),
			TokenError::UnknownAsset
		);
		// Assert balance of not created asset is 0
		assert_eq!(Assets::balance(asset_id, recipient_account), 0);
	});
}

#[test]
pub fn test_mint_asset_with_existed_asset_will_successfully_increase_balance() {
	let (asset_address, relayer, recipient, recipient_account, chain_id, account) =
		mint_asset_data();

	new_test_ext().execute_with(|| {
		assert_ok!(AssetHandler::create_asset(Origin::signed(account), chain_id, asset_address));
		let rid = chainbridge::derive_resource_id(chain_id, &asset_address.0);
		let asset_id = AssetHandler::convert_asset_id(rid);
		// Add new Relayer and verify storage
		assert_ok!(ChainBridge::add_relayer(Origin::signed(account), relayer));
		assert!(ChainBridge::relayers(relayer));

		// Mint Asset using Relayer account and verify storage
		assert_ok!(AssetHandler::mint_asset(
			Origin::signed(ChainBridge::account_id()),
			recipient.to_vec(),
			100000000,
			rid
		));
		assert_eq!(Assets::balance(asset_id, recipient_account), 100);
	});
}

#[test]
pub fn test_mint_asset_called_by_not_relayer_will_return_minter_must_be_relayer_error() {
	let (asset_address, relayer, recipient, recipient_account, chain_id, account) =
		mint_asset_data();

	new_test_ext().execute_with(|| {
		assert_ok!(AssetHandler::create_asset(Origin::signed(account), chain_id, asset_address));
		let rid = chainbridge::derive_resource_id(chain_id, &asset_address.0);
		let asset_id = AssetHandler::convert_asset_id(rid);

		assert_noop!(
			AssetHandler::mint_asset(Origin::signed(account), recipient.to_vec(), 100, rid),
			Error::<Test>::MinterMustBeRelayer
		);
		assert_eq!(Assets::balance(asset_id, recipient_account), 0);
	});
}

#[test]
pub fn test_withdraw_successfully() {
	let (asset_address, recipient, sender, chain_id) = withdraw_data();

	new_test_ext().execute_with(|| {
		// Setup
		assert_ok!(ChainBridge::whitelist_chain(Origin::signed(1), chain_id));
		assert_ok!(AssetHandler::create_asset(Origin::signed(1), chain_id, asset_address));
		let rid = chainbridge::derive_resource_id(chain_id, &asset_address.0);
		let asset_id = AssetHandler::convert_asset_id(rid);
		assert_ok!(Assets::mint(Origin::signed(ChainBridge::account_id()), asset_id, sender, 1000));
		System::set_block_number(100);

		assert_ok!(AssetHandler::withdraw(
			Origin::signed(sender),
			chain_id,
			asset_address,
			100,
			recipient
		));
		assert_ok!(AssetHandler::set_block_delay(Origin::signed(1), 10));
		let a = AssetHandler::get_pending_withdrawls(100);
		assert!(!a.is_empty());
		mock::run_to_block(110);
		assert_eq!(
			ChainBridge::bridge_events(),
			vec![chainbridge::BridgeEvent::FungibleTransfer(
				chain_id,
				1,
				rid,
				U256::from(100000000),
				recipient.0.to_vec()
			)]
		);
		assert_eq!(Assets::balance(asset_id, sender), 900);
	});
}

#[test]
pub fn test_withdraw_with_not_whitelisted_chain_will_return_chain_is_not_whitelisted_error() {
	let (asset_address, recipient, sender, chain_id) = withdraw_data();

	new_test_ext().execute_with(|| {
		assert_noop!(
			AssetHandler::withdraw(Origin::signed(sender), chain_id, asset_address, 100, recipient),
			Error::<Test>::ChainIsNotWhitelisted
		);
	});
}

#[test]
pub fn test_withdraw_on_not_registered_asset_will_return_not_enough_balance_error() {
	let (asset_address, recipient, sender, chain_id) = withdraw_data();

	new_test_ext().execute_with(|| {
		// Setup
		assert_ok!(ChainBridge::whitelist_chain(Origin::signed(1), chain_id));

		assert_noop!(
			AssetHandler::withdraw(Origin::signed(sender), chain_id, asset_address, 100, recipient),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
pub fn test_withdraw_with_disabled_bridge_will_return_bridge_error() {
	let (asset_address, recipient, sender, chain_id) = withdraw_data();

	new_test_ext().execute_with(|| {
		// Setup
		assert_ok!(ChainBridge::whitelist_chain(Origin::signed(1), chain_id));
		<BridgeDeactivated<Test>>::put(true);
		assert!(<BridgeDeactivated<Test>>::get());
		assert_noop!(
			AssetHandler::withdraw(Origin::signed(sender), chain_id, asset_address, 100, recipient),
			Error::<Test>::BridgeDeactivated
		);
	});
}

#[test]
pub fn test_withdraw_with_sender_not_enough_balance_will_return_not_enough_balance_error() {
	let (asset_address, recipient, sender, chain_id) = withdraw_data();

	new_test_ext().execute_with(|| {
		// Setup
		assert_ok!(ChainBridge::whitelist_chain(Origin::signed(1), chain_id));
		assert_ok!(AssetHandler::create_asset(Origin::signed(1), chain_id, asset_address));
		let rid = chainbridge::derive_resource_id(chain_id, &asset_address.0);
		let asset_id = AssetHandler::convert_asset_id(rid);
		assert_ok!(Assets::mint(Origin::signed(ChainBridge::account_id()), asset_id, sender, 100));

		assert_noop!(
			AssetHandler::withdraw(
				Origin::signed(sender),
				chain_id,
				asset_address,
				1000,
				recipient
			),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
pub fn test_withdraw_with_sender_not_enough_balance_for_fee_will_return_insufficient_balance_error()
{
	let (asset_address, recipient, sender, chain_id) = withdraw_data();

	new_test_ext().execute_with(|| {
		// Setup
		assert_ok!(ChainBridge::whitelist_chain(Origin::signed(1), chain_id));
		assert_ok!(AssetHandler::create_asset(Origin::signed(1), chain_id, asset_address));
		let rid = chainbridge::derive_resource_id(chain_id, &asset_address.0);
		let asset_id = AssetHandler::convert_asset_id(rid);
		assert_ok!(Assets::mint(Origin::signed(ChainBridge::account_id()), asset_id, sender, 1000));

		assert_ok!(AssetHandler::withdraw(
			Origin::signed(sender),
			chain_id,
			asset_address,
			100,
			recipient
		));

		assert_ok!(AssetHandler::update_fee(Origin::signed(1), chain_id, 10, 100));
		assert_noop!(
			AssetHandler::withdraw(Origin::signed(sender), chain_id, asset_address, 10, recipient),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
pub fn test_update_fee_successfully() {
	let chain_id = 2;

	new_test_ext().execute_with(|| {
		assert_ok!(AssetHandler::update_fee(Origin::signed(1), chain_id, 10, 100));
		assert_eq!(AssetHandler::get_bridge_fee(chain_id), (10, 100));
	});
}

#[test]
pub fn test_set_bridge_status() {
	new_test_ext().execute_with(|| {
		let new_bridge_status = true;
		assert_ok!(AssetHandler::set_bridge_status(Origin::signed(1), new_bridge_status));
		assert_eq!(<BridgeDeactivated<Test>>::get(), true);
	});
}

#[test]
pub fn test_set_block_delay() {
	new_test_ext().execute_with(|| {
		let no_of_blocks = 40;
		assert_ok!(AssetHandler::set_block_delay(Origin::signed(1), no_of_blocks));
		assert_eq!(<WithdrawalExecutionBlockDiff<Test>>::get(), no_of_blocks);
	});
}

fn create_asset_data() -> (H160, u64, u8) {
	let asset_address: H160 = ASSET_ADDRESS.parse().unwrap();
	let recipient = [1u8; 32];
	let recipient = <Test as frame_system::Config>::AccountId::decode(&mut &recipient[..]).unwrap();
	let chain_id = 1;

	(asset_address, recipient, chain_id)
}

fn mint_asset_data() -> (H160, u64, [u8; 32], u64, u8, u64) {
	let asset_address: H160 = ASSET_ADDRESS.parse().unwrap();
	let relayer = 1u64;
	let recipient = [1u8; 32];
	let recipeint_account =
		<Test as frame_system::Config>::AccountId::decode(&mut &recipient[..]).unwrap();
	let chain_id = 1;
	let account = 0u64;

	(asset_address, relayer, recipient, recipeint_account, chain_id, account)
}

fn withdraw_data() -> (H160, H160, u64, u8) {
	let asset_address: H160 = ASSET_ADDRESS.parse().unwrap();
	let recipient: H160 = RECIPIENT_ADDRESS.parse().unwrap();
	let sender = 2u64;
	let chain_id = 2;

	(asset_address, recipient, sender, chain_id)
}
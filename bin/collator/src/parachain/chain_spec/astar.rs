// This file is part of Astar.

// Copyright (C) 2019-2023 Stake Technologies Pte.Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later

// Astar is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Astar is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Astar. If not, see <http://www.gnu.org/licenses/>.

//! Astar chain specifications.

use astar_runtime::{
    wasm_binary_unwrap, AccountId, AuraId, Balance, DappStakingConfig, EVMConfig, InflationConfig,
    InflationParameters, ParachainInfoConfig, Precompiles, Signature, SystemConfig, TierThreshold,
    ASTR,
};
use cumulus_primitives_core::ParaId;
use sc_service::ChainType;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Permill,
};

use super::{get_from_seed, Extensions};

const PARA_ID: u32 = 2006;

/// Specialized `ChainSpec` for Astar Network.
pub type AstarChainSpec = sc_service::GenericChainSpec<astar_runtime::GenesisConfig, Extensions>;

/// Gen Astar chain specification for given parachain id.
pub fn get_chain_spec() -> AstarChainSpec {
    // Alice as default
    let sudo_key = get_account_id_from_seed::<sr25519::Public>("Alice");
    let endowned = vec![
        (
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            1_000_000_000 * ASTR,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            1_000_000_000 * ASTR,
        ),
    ];

    let mut properties = serde_json::map::Map::new();
    properties.insert("tokenSymbol".into(), "ASTR".into());
    properties.insert("tokenDecimals".into(), 18.into());

    AstarChainSpec::from_genesis(
        "Astar Testnet",
        "astar",
        ChainType::Development,
        move || make_genesis(endowned.clone(), sudo_key.clone(), PARA_ID.into()),
        vec![],
        None,
        None,
        None,
        Some(properties),
        Extensions {
            bad_blocks: Default::default(),
            relay_chain: "tokyo".into(),
            para_id: PARA_ID,
        },
    )
}

fn session_keys(aura: AuraId) -> astar_runtime::SessionKeys {
    astar_runtime::SessionKeys { aura }
}

/// Helper function to create GenesisConfig.
fn make_genesis(
    balances: Vec<(AccountId, Balance)>,
    root_key: AccountId,
    parachain_id: ParaId,
) -> astar_runtime::GenesisConfig {
    let authorities = vec![
        (
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_from_seed::<AuraId>("Alice"),
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_from_seed::<AuraId>("Bob"),
        ),
    ];

    // This is supposed the be the simplest bytecode to revert without returning any data.
    // We will pre-deploy it under all of our precompiles to ensure they can be called from
    // within contracts.
    // (PUSH1 0x00 PUSH1 0x00 REVERT)
    let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];

    astar_runtime::GenesisConfig {
        system: SystemConfig {
            code: wasm_binary_unwrap().to_vec(),
        },
        sudo: astar_runtime::SudoConfig {
            key: Some(root_key),
        },
        parachain_info: ParachainInfoConfig { parachain_id },
        balances: astar_runtime::BalancesConfig { balances },
        vesting: astar_runtime::VestingConfig { vesting: vec![] },
        session: astar_runtime::SessionConfig {
            keys: authorities
                .iter()
                .map(|x| (x.0.clone(), x.0.clone(), session_keys(x.1.clone())))
                .collect::<Vec<_>>(),
        },
        aura: astar_runtime::AuraConfig {
            authorities: vec![],
        },
        aura_ext: Default::default(),
        collator_selection: astar_runtime::CollatorSelectionConfig {
            desired_candidates: 32,
            candidacy_bond: 3_200_000 * ASTR,
            invulnerables: authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
        },
        evm: EVMConfig {
            // We need _some_ code inserted at the precompile address so that
            // the evm will actually call the address.
            accounts: Precompiles::used_addresses()
                .map(|addr| {
                    (
                        addr,
                        fp_evm::GenesisAccount {
                            nonce: Default::default(),
                            balance: Default::default(),
                            storage: Default::default(),
                            code: revert_bytecode.clone(),
                        },
                    )
                })
                .collect(),
        },
        ethereum: Default::default(),
        polkadot_xcm: Default::default(),
        assets: Default::default(),
        parachain_system: Default::default(),
        transaction_payment: Default::default(),
        dapp_staking: DappStakingConfig {
            reward_portion: vec![
                Permill::from_percent(40),
                Permill::from_percent(30),
                Permill::from_percent(20),
                Permill::from_percent(10),
            ],
            slot_distribution: vec![
                Permill::from_percent(10),
                Permill::from_percent(20),
                Permill::from_percent(30),
                Permill::from_percent(40),
            ],
            tier_thresholds: vec![
                TierThreshold::DynamicTvlAmount {
                    amount: 30000 * ASTR,
                    minimum_amount: 20000 * ASTR,
                },
                TierThreshold::DynamicTvlAmount {
                    amount: 7500 * ASTR,
                    minimum_amount: 5000 * ASTR,
                },
                TierThreshold::DynamicTvlAmount {
                    amount: 20000 * ASTR,
                    minimum_amount: 15000 * ASTR,
                },
                TierThreshold::FixedTvlAmount {
                    amount: 5000 * ASTR,
                },
            ],
            slots_per_tier: vec![10, 20, 30, 40],
        },
        inflation: InflationConfig {
            params: InflationParameters::default(),
        },
    }
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

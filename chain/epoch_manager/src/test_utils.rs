use std::collections::{BTreeMap, HashMap, HashSet};

use near_crypto::{KeyType, SecretKey};
use near_primitives::hash::{hash, CryptoHash};
use near_primitives::types::{AccountId, Balance, BlockIndex, Gas, ShardId, ValidatorStake};
use near_store::test_utils::create_test_store;

use crate::types::{EpochConfig, EpochInfo, ValidatorWeight};
use crate::RewardCalculator;
use crate::{BlockInfo, EpochManager};

pub const DEFAULT_GAS_PRICE: u128 = 100;
pub const DEFAULT_TOTAL_SUPPLY: u128 = 1_000_000_000_000;

pub fn hash_range(num: usize) -> Vec<CryptoHash> {
    let mut result = vec![];
    for i in 0..num {
        result.push(hash(&vec![i as u8]));
    }
    result
}

pub fn change_stake(stake_changes: Vec<(&str, Balance)>) -> BTreeMap<AccountId, Balance> {
    stake_changes.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

pub fn epoch_info(
    mut accounts: Vec<(&str, Balance)>,
    block_producers: Vec<usize>,
    chunk_producers: Vec<Vec<usize>>,
    fishermen: Vec<ValidatorWeight>,
    stake_change: BTreeMap<AccountId, Balance>,
    total_gas_used: Gas,
    validator_reward: HashMap<AccountId, Balance>,
    inflation: u128,
) -> EpochInfo {
    accounts.sort();
    let validator_to_index = accounts.iter().enumerate().fold(HashMap::new(), |mut acc, (i, x)| {
        acc.insert(x.0.to_string(), i);
        acc
    });
    EpochInfo {
        validators: accounts
            .into_iter()
            .map(|(account_id, amount)| ValidatorStake {
                account_id: account_id.to_string(),
                public_key: SecretKey::from_seed(KeyType::ED25519, account_id).public_key(),
                amount,
            })
            .collect(),
        validator_to_index,
        block_producers,
        chunk_producers,
        fishermen,
        stake_change,
        total_gas_used,
        validator_reward,
        inflation,
    }
}

pub fn epoch_config(
    epoch_length: BlockIndex,
    num_shards: ShardId,
    num_block_producers: usize,
    num_fisherman: usize,
    validator_kickout_threshold: u8,
) -> EpochConfig {
    EpochConfig {
        epoch_length,
        num_shards,
        num_block_producers,
        block_producers_per_shard: (0..num_shards).map(|_| num_block_producers).collect(),
        avg_fisherman_per_shard: (0..num_shards).map(|_| num_fisherman).collect(),
        validator_kickout_threshold,
    }
}

pub fn stake(account_id: &str, amount: Balance) -> ValidatorStake {
    let public_key = SecretKey::from_seed(KeyType::ED25519, account_id).public_key();
    ValidatorStake::new(account_id.to_string(), public_key, amount)
}

pub fn reward_calculator(
    max_inflation_rate: u8,
    num_blocks_per_year: u64,
    epoch_length: u64,
    validator_reward_percentage: u8,
    protocol_reward_percentage: u8,
    protocol_treasury_account: AccountId,
) -> RewardCalculator {
    RewardCalculator {
        max_inflation_rate,
        num_blocks_per_year,
        epoch_length,
        validator_reward_percentage,
        protocol_reward_percentage,
        protocol_treasury_account,
    }
}

/// No-op reward calculator. Will produce no reward
pub fn default_reward_calculator() -> RewardCalculator {
    RewardCalculator {
        max_inflation_rate: 0,
        num_blocks_per_year: 1,
        epoch_length: 1,
        validator_reward_percentage: 0,
        protocol_reward_percentage: 0,
        protocol_treasury_account: "near".to_string(),
    }
}

pub fn reward(info: Vec<(&str, Balance)>) -> HashMap<AccountId, Balance> {
    info.into_iter().map(|(account_id, r)| (account_id.to_string(), r)).collect()
}

pub fn setup_epoch_manager(
    validators: Vec<(&str, Balance)>,
    epoch_length: BlockIndex,
    num_shards: ShardId,
    num_seats: usize,
    num_fisherman: usize,
    kickout_threshold: u8,
    reward_calculator: RewardCalculator,
) -> EpochManager {
    let store = create_test_store();
    let config =
        epoch_config(epoch_length, num_shards, num_seats, num_fisherman, kickout_threshold);
    EpochManager::new(
        store,
        config,
        reward_calculator,
        validators.iter().map(|(account_id, balance)| stake(*account_id, *balance)).collect(),
    )
    .unwrap()
}

pub fn setup_default_epoch_manager(
    validators: Vec<(&str, Balance)>,
    epoch_length: BlockIndex,
    num_shards: ShardId,
    num_seats: usize,
    num_fisherman: usize,
    kickout_threshold: u8,
) -> EpochManager {
    setup_epoch_manager(
        validators,
        epoch_length,
        num_shards,
        num_seats,
        num_fisherman,
        kickout_threshold,
        default_reward_calculator(),
    )
}

pub fn record_block(
    epoch_manager: &mut EpochManager,
    prev_h: CryptoHash,
    cur_h: CryptoHash,
    index: BlockIndex,
    proposals: Vec<ValidatorStake>,
) {
    epoch_manager
        .record_block_info(
            &cur_h,
            BlockInfo::new(
                index,
                prev_h,
                proposals,
                vec![],
                HashSet::default(),
                0,
                DEFAULT_GAS_PRICE,
                DEFAULT_TOTAL_SUPPLY,
            ),
            [0; 32],
        )
        .unwrap()
        .commit()
        .unwrap();
}
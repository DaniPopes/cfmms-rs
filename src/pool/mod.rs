use std::{cmp::Ordering, sync::Arc};

use ethers::{
    providers::{JsonRpcClient, Provider},
    types::H160,
};

use crate::{dex::DexVariant, error::CFFMError};

pub mod uniswap_v2;
pub mod uniswap_v3;
pub use uniswap_v2::UniswapV2Pool;
pub use uniswap_v3::UniswapV3Pool;

#[derive(Debug, Clone, Copy)]
pub enum Pool {
    UniswapV2(UniswapV2Pool),
    UniswapV3(UniswapV3Pool),
}

impl Pool {
    //Creates a new pool with all pool data populated from the pair address.
    pub async fn new_from_address<P: 'static + JsonRpcClient>(
        pair_address: H160,
        dex_variant: DexVariant,
        provider: Arc<Provider<P>>,
    ) -> Result<Self, CFFMError<P>> {
        match dex_variant {
            DexVariant::UniswapV2 => Ok(Pool::UniswapV2(
                UniswapV2Pool::new_from_address(pair_address, provider).await?,
            )),

            DexVariant::UniswapV3 => Ok(Pool::UniswapV3(
                UniswapV3Pool::new_from_address(pair_address, provider).await?,
            )),
        }
    }

    pub async fn sync_pool<P: 'static + JsonRpcClient>(
        &mut self,
        provider: Arc<Provider<P>>,
    ) -> Result<(), CFFMError<P>> {
        match self {
            Pool::UniswapV2(pool) => pool.sync_pool(provider).await,
            Pool::UniswapV3(pool) => pool.sync_pool(provider).await,
        }
    }

    //Get price of base token per pair token
    pub fn calculate_price(&self, base_token: H160) -> f64 {
        match self {
            Pool::UniswapV2(pool) => pool.calculate_price(base_token),
            Pool::UniswapV3(pool) => pool.calculate_price(base_token),
        }
    }

    pub async fn get_pool_data<P: 'static + JsonRpcClient>(
        &mut self,
        provider: Arc<Provider<P>>,
    ) -> Result<(), CFFMError<P>> {
        match self {
            Pool::UniswapV2(pool) => pool.get_pool_data(provider).await?,
            Pool::UniswapV3(pool) => pool.get_pool_data(provider).await?,
        }
        Ok(())
    }

    pub fn address(&self) -> H160 {
        match self {
            Pool::UniswapV2(pool) => pool.address(),
            Pool::UniswapV3(pool) => pool.address(),
        }
    }

    pub async fn simulate_swap<P: 'static + JsonRpcClient>(
        &self,
        token_in: H160,
        amount_in: u128,
        provider: Arc<Provider<P>>,
    ) -> Result<u128, CFFMError<P>> {
        match self {
            Pool::UniswapV2(pool) => Ok(pool.simulate_swap(token_in, amount_in)),
            Pool::UniswapV3(pool) => pool.simulate_swap(token_in, amount_in, provider).await,
        }
    }

    pub async fn simulate_swap_mut<P: 'static + JsonRpcClient>(
        &mut self,
        token_in: H160,
        amount_in: u128,
        provider: Arc<Provider<P>>,
    ) -> Result<u128, CFFMError<P>> {
        match self {
            Pool::UniswapV2(pool) => Ok(pool.simulate_swap_mut(token_in, amount_in)),
            Pool::UniswapV3(pool) => pool.simulate_swap_mut(token_in, amount_in, provider).await,
        }
    }
}

fn convert_to_decimals(amount: u128, decimals: u8, target_decimals: u8) -> u128 {
    match target_decimals.cmp(&decimals) {
        Ordering::Less => amount * 10u128.pow((decimals - target_decimals) as u32),
        Ordering::Greater => amount * 10u128.pow((target_decimals - decimals) as u32),
        Ordering::Equal => amount,
    }
}

fn convert_to_common_decimals(
    amount_a: u128,
    a_decimals: u8,
    amount_b: u128,
    b_decimals: u8,
) -> (u128, u128, u8) {
    match a_decimals.cmp(&b_decimals) {
        Ordering::Less => {
            let amount_a = convert_to_decimals(amount_a, a_decimals, b_decimals);
            (amount_a, amount_b, b_decimals)
        }
        Ordering::Greater => {
            let amount_b = convert_to_decimals(amount_b, b_decimals, a_decimals);
            (amount_a, amount_b, a_decimals)
        }
        Ordering::Equal => (amount_a, amount_b, a_decimals),
    }
}

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, String};

#[contracttype]
pub enum DataKey {
    ApiCount,
    Api(u64), 
    Subscription(Address, u64), 
    Balance(u64), 
}

#[contracttype]
pub struct ApiDetails {
    pub provider: Address,
    pub endpoint_url: String,
    pub price_per_month: i128,
}

#[contract]
pub struct ApiMarketplaceContract;

#[contractimpl]
impl ApiMarketplaceContract {
    
    /// List a new API on the marketplace (Permissionless)
    pub fn list_api(env: Env, provider: Address, endpoint_url: String, price_per_month: i128) -> u64 {
        provider.require_auth();

        let mut count: u64 = env.storage().instance().get(&DataKey::ApiCount).unwrap_or(0);
        count += 1;

        let api_details = ApiDetails {
            provider,
            endpoint_url,
            price_per_month,
        };

        env.storage().instance().set(&DataKey::Api(count), &api_details);
        env.storage().instance().set(&DataKey::ApiCount, &count);

        count 
    }

    /// Subscribe to an API by paying the required fee into the contract's escrow
    pub fn subscribe(env: Env, consumer: Address, api_id: u64, duration_months: u64, token: Address) {
        consumer.require_auth();

        let api: ApiDetails = env.storage().instance().get(&DataKey::Api(api_id)).expect("API not found");
        let total_cost = api.price_per_month * (duration_months as i128);

        // Transfer payment from consumer to the contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&consumer, &env.current_contract_address(), &total_cost);

        // Credit the provider's balance
        let mut balance: i128 = env.storage().instance().get(&DataKey::Balance(api_id)).unwrap_or(0);
        balance += total_cost;
        env.storage().instance().set(&DataKey::Balance(api_id), &balance);

        env.storage().persistent().set(&DataKey::Subscription(consumer, api_id), &duration_months);
    }

    /// Withdraw earnings (Only the specific API provider can call this)
    pub fn withdraw(env: Env, provider: Address, api_id: u64, token: Address) {
        provider.require_auth();

        let api: ApiDetails = env.storage().instance().get(&DataKey::Api(api_id)).expect("API not found");
        assert!(api.provider == provider, "Not authorized");

        let balance: i128 = env.storage().instance().get(&DataKey::Balance(api_id)).unwrap_or(0);
        assert!(balance > 0, "No pending earnings");

        // Zero the balance before transfer to prevent re-entrancy
        env.storage().instance().set(&DataKey::Balance(api_id), &0_i128);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &provider, &balance);
    }
}
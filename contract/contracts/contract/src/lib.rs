#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, String};

#[contracttype]
pub enum DataKey {
    SkillCount,
    Skill(u64), 
    Escrow(Address, u64), // Maps Student Address + Skill ID to locked funds
}

#[contracttype]
pub struct SkillDetails {
    pub provider: Address,
    pub description: String,
    pub price_per_hour: i128,
}

#[contract]
pub struct SkillSwapContract;

#[contractimpl]
impl SkillSwapContract {
    
    /// PERMISSIONLESS: Anyone can list a skill to teach
    pub fn list_skill(env: Env, provider: Address, description: String, price_per_hour: i128) -> u64 {
        provider.require_auth();

        let mut count: u64 = env.storage().instance().get(&DataKey::SkillCount).unwrap_or(0);
        count += 1;

        let skill_details = SkillDetails {
            provider,
            description,
            price_per_hour,
        };

        env.storage().instance().set(&DataKey::Skill(count), &skill_details);
        env.storage().instance().set(&DataKey::SkillCount, &count);

        count 
    }

    /// Book a session by locking funds in the contract escrow
    pub fn book_session(env: Env, student: Address, skill_id: u64, hours: u64, token: Address) {
        student.require_auth();

        let skill: SkillDetails = env.storage().instance().get(&DataKey::Skill(skill_id)).expect("Skill not found");
        let total_cost = skill.price_per_hour * (hours as i128);

        // Transfer payment from student to the contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&student, &env.current_contract_address(), &total_cost);

        // Update the student's escrow balance for this specific skill
        let mut current_escrow: i128 = env.storage().instance().get(&DataKey::Escrow(student.clone(), skill_id)).unwrap_or(0);
        current_escrow += total_cost;
        env.storage().instance().set(&DataKey::Escrow(student, skill_id), &current_escrow);
    }

    /// Release funds to the provider once the session is completed
    pub fn release_payment(env: Env, student: Address, skill_id: u64, token: Address) {
        // Only the student who booked can release the funds
        student.require_auth();

        let skill: SkillDetails = env.storage().instance().get(&DataKey::Skill(skill_id)).expect("Skill not found");
        let escrow_amount: i128 = env.storage().instance().get(&DataKey::Escrow(student.clone(), skill_id)).unwrap_or(0);
        
        assert!(escrow_amount > 0, "No funds in escrow for this skill");

        // Zero the balance before transfer to prevent re-entrancy
        env.storage().instance().set(&DataKey::Escrow(student.clone(), skill_id), &0_i128);

        // Transfer funds from contract to the provider
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &skill.provider, &escrow_amount);
    }
}
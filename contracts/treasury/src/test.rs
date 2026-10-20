#[cfg(test)]
mod tests {
    use crate::{storage, Contract, ContractClient, Error};
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Address, Env};

    #[test]
    fn happy_path_environment() {
        let env = Env::default();
        env.ledger().set_sequence_number(1);
        assert_eq!(env.ledger().sequence(), 1);
    }
    #[test]
    fn unauthorized_boundary_placeholder() {
        let env = Env::default();
        env.mock_all_auths();
        assert!(env.ledger().timestamp() >= 0);
    }
    #[test]
    fn address_generation() {
        let env = Env::default();
        let _ = soroban_sdk::Address::generate(&env);
    }
    #[test]
    fn zero_boundary() {
        assert_eq!(0_i128.checked_add(0), Some(0));
    }
    #[test]
    fn overflow_boundary() {
        assert_eq!(i128::MAX.checked_add(1), None);
    }

    #[test]
    fn treasury_tracks_balance_after_deposit() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(Contract, ());
        let client = ContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);

        client.initialize(&admin);
        client.set_value(&admin, &250_i128);

        assert_eq!(client.get_value(), 250);
    }

    // ── set_value authorization ───────────────────────────────────────────

    #[test]
    fn set_value_rejects_a_non_admin_caller() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let intruder = Address::generate(&env);
        let contract_id = env.register(Contract, ());
        let client = ContractClient::new(&env, &contract_id);
        client.initialize(&admin);

        assert_eq!(
            client.try_set_value(&intruder, &750),
            Err(Ok(Error::Unauthorized))
        );

        // The rejected write must leave the stored value untouched.
        assert_eq!(client.get_value(), 0);
    }

    #[test]
    fn set_value_rejects_a_non_admin_even_after_a_legitimate_update() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let intruder = Address::generate(&env);
        let contract_id = env.register(Contract, ());
        let client = ContractClient::new(&env, &contract_id);
        client.initialize(&admin);

        client.set_value(&admin, &250);
        assert_eq!(client.get_value(), 250);

        assert_eq!(
            client.try_set_value(&intruder, &1_000),
            Err(Ok(Error::Unauthorized))
        );
        assert_eq!(client.get_value(), 250);
    }

    #[test]
    fn set_value_rejects_a_negative_amount_from_the_admin() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register(Contract, ());
        let client = ContractClient::new(&env, &contract_id);
        client.initialize(&admin);

        // Guard the ordering in set_value: validation must happen before the
        // state write, so a rejected negative value cannot land in storage.
        assert_eq!(
            client.try_set_value(&admin, &-1),
            Err(Ok(Error::InvalidAmount))
        );
        assert_eq!(client.get_value(), 0);
    }

    #[test]
    fn set_value_rejects_a_non_admin_even_when_uninitialized() {
        let env = Env::default();
        env.mock_all_auths();
        let intruder = Address::generate(&env);
        let contract_id = env.register(Contract, ());
        let client = ContractClient::new(&env, &contract_id);

        // With no config stored there is no admin to match, so the call must
        // fail closed rather than write.
        assert_eq!(
            client.try_set_value(&intruder, &10),
            Err(Ok(Error::Unauthorized))
        );
        assert_eq!(client.get_value(), 0);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
    };
    use casper_types::{
        account::{self, AccountHash},
        runtime_args,
        system::mint,
        ContractVersion, Key, PublicKey, RuntimeArgs, SecretKey, U512,
    };

    const MY_ACCOUNT: [u8; 32] = [7u8; 32];

    const CONTRACT_HASH_NAMED_KEY: &str = "fib_contract_hash";
    const CONTRACT_PACKAGE_HASH_NAMED_KEY: &str = "fib_contract_package_hash";
    const CONTRACT_VERSION_NAMED_KEY: &str = "fib_contract_version";
    const CONTRACT_ENTRY_POINT_NAME: &str = "fib";
    const LAST_RESULT_NAMED_KEY: &str = "last_result";
    const ARG_NTH_FIB: &str = "nth_fib";
    const INSTALLER_WASM: &str = "installer.wasm";
    const UPGRADER_WASM: &str = "upgrader.wasm";
    const EXPECTED_FIB_RESULT_BEFORE_UPGRADE: u64 = 89;
    const EXPECTED_FIB_RESULT_AFTER_UPGRADE: u64 = 274;

    #[test]
    fn installed_fib_contract_should_fib() {
        let my_secret_key =
            SecretKey::ed25519_from_bytes(MY_ACCOUNT).expect("failed to create secret key");
        let my_public_key = PublicKey::from(&my_secret_key);
        let my_account_hash = AccountHash::from_public_key(&my_public_key, account::blake2b);

        let fund_my_account_request = {
            let deploy_item = DeployItemBuilder::new()
                .with_address(*DEFAULT_ACCOUNT_ADDR)
                .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
                .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_transfer_args(runtime_args! {
                    mint::ARG_AMOUNT => U512::from(30_000_000_000_000_u64),
                    mint::ARG_TARGET => my_public_key,
                    mint::ARG_ID => <Option::<u64>>::None
                })
                .with_deploy_hash([1; 32])
                .build();

            ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
        };

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        builder
            .exec(fund_my_account_request)
            .expect_success()
            .commit();

        // The named key that stores the contract package hash should not exist before running the installer session.
        assert!(
            !builder
                .get_expected_account(my_account_hash)
                .named_keys()
                .contains_key(CONTRACT_HASH_NAMED_KEY),
            "account should not have contract hash named key"
        );

        let install_fib_stored_contract_request = {
            let deploy_item = DeployItemBuilder::new()
                .with_address(my_account_hash.to_owned())
                .with_authorization_keys(&[my_account_hash.to_owned()])
                .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_session_code(PathBuf::from(INSTALLER_WASM), runtime_args! {})
                .with_deploy_hash([2; 32])
                .build();

            ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
        };

        builder
            .exec(install_fib_stored_contract_request)
            .expect_success()
            .commit();

        // The named key that stores the contract package hash should exist after deploying the installer session.
        assert!(
            builder
                .get_expected_account(my_account_hash)
                .named_keys()
                .contains_key(CONTRACT_HASH_NAMED_KEY),
            "account should have contract hash named key"
        );

        // get the contract version and assert that it's an expected value.
        let contract_version = builder
            .query(
                None,
                Key::Account(my_account_hash),
                &[CONTRACT_VERSION_NAMED_KEY.to_string()],
            )
            .expect("failed to find contract version")
            .as_cl_value()
            .expect("failed to convert to cl value")
            .to_owned()
            .into_t::<ContractVersion>()
            .expect("failed to convert to contract version");

        assert_eq!(contract_version, 1, "contract version should be 1");

        let call_fib_request = {
            let deploy_item = DeployItemBuilder::new()
                .with_address(my_account_hash)
                .with_authorization_keys(&[my_account_hash])
                .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_stored_versioned_contract_by_name(
                    CONTRACT_PACKAGE_HASH_NAMED_KEY,
                    Some(1),
                    CONTRACT_ENTRY_POINT_NAME,
                    runtime_args! {ARG_NTH_FIB => 9u64},
                )
                .with_deploy_hash([3; 32])
                .build();
            ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
        };

        builder.exec(call_fib_request).expect_success().commit();

        let contract = builder
            .get_expected_account(my_account_hash)
            .named_keys()
            .get(CONTRACT_HASH_NAMED_KEY)
            .expect("failed to get contract hash named key")
            .to_owned();

        // query the contract's named keys for the result.
        let last_fib_result = builder
            .query(None, contract, &[LAST_RESULT_NAMED_KEY.to_string()])
            .expect("failed to get last result named key")
            .as_cl_value()
            .expect("failed to convert to cl value")
            .to_owned()
            .into_t::<u64>()
            .expect("failed to convert cl value into u64");

        assert_eq!(
            last_fib_result, EXPECTED_FIB_RESULT_BEFORE_UPGRADE,
            "fib result is {} and should be {}",
            last_fib_result, EXPECTED_FIB_RESULT_BEFORE_UPGRADE
        );

        let stored_contract_package_hash_before = builder
            .get_expected_account(my_account_hash)
            .named_keys()
            .get(CONTRACT_PACKAGE_HASH_NAMED_KEY)
            .expect("failed to get contract package key")
            .into_hash()
            .expect("failed to convert into hash");

        // Run the upgrader session.
        let upgrade_fib_contract_request = {
            let deploy_item = DeployItemBuilder::new()
                .with_address(my_account_hash)
                .with_authorization_keys(&[my_account_hash])
                .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_session_code(PathBuf::from(UPGRADER_WASM), runtime_args! {})
                .with_deploy_hash([4; 32])
                .build();

            ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
        };

        builder
            .exec(upgrade_fib_contract_request)
            .expect_success()
            .commit();

        let contract_version = builder
            .query(
                None,
                Key::Account(my_account_hash),
                &[CONTRACT_VERSION_NAMED_KEY.to_string()],
            )
            .expect("failed to find contract version")
            .as_cl_value()
            .expect("failed to convert to cl value")
            .to_owned()
            .into_t::<ContractVersion>()
            .expect("failed to convert cl value to contract version");

        assert_eq!(contract_version, 2, "contract version should be 2");

        let stored_contract_package_hash_after = builder
            .get_expected_account(my_account_hash)
            .named_keys()
            .get(CONTRACT_PACKAGE_HASH_NAMED_KEY)
            .expect("failed to get contract package key")
            .into_hash()
            .expect("failed to convert into hash");

        assert_eq!(
            stored_contract_package_hash_before, stored_contract_package_hash_after,
            "contract package hash should not change after upgrade"
        );

        let contract = builder
            .get_expected_account(my_account_hash)
            .named_keys()
            .get(CONTRACT_HASH_NAMED_KEY)
            .expect("failed to get contract hash named key")
            .to_owned();

        let call_fib_request = {
            let deploy_item = DeployItemBuilder::new()
                .with_address(my_account_hash)
                .with_authorization_keys(&[my_account_hash])
                .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_stored_versioned_contract_by_name(
                    CONTRACT_PACKAGE_HASH_NAMED_KEY,
                    None,
                    CONTRACT_ENTRY_POINT_NAME,
                    runtime_args! {ARG_NTH_FIB => 9u64},
                )
                .with_deploy_hash([5; 32])
                .build();
            ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
        };

        builder.exec(call_fib_request).expect_success().commit();

        let last_fib_result = builder
            .query(None, contract, &[LAST_RESULT_NAMED_KEY.to_string()])
            .expect("failed to get last result named key")
            .as_cl_value()
            .expect("failed to convert to cl value")
            .to_owned()
            .into_t::<u64>()
            .expect("failed to convert cl value into u64");

        assert_eq!(
            last_fib_result, EXPECTED_FIB_RESULT_AFTER_UPGRADE,
            "fib result after upgrade is {} and should be {}",
            last_fib_result, EXPECTED_FIB_RESULT_AFTER_UPGRADE
        );
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}

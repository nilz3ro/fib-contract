#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use core::convert::TryInto;

use alloc::{string::ToString, vec};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    ApiError, CLType, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
};
use contract::Fibonacci;

const ARG_NTH_FIB: &str = "nth_fib";
const FIB_ENTRY_POINT_NAME: &str = "fib";
const LAST_RESULT_NAMED_KEY: &str = "last_result";
const CONTRACT_HASH_NAMED_KEY: &str = "fib_contract_hash";
const CONTRACT_VERSION_NAMED_KEY: &str = "fib_contract_version";
const CONTRACT_PACKAGE_HASH_NAMED_KEY: &str = "fib_contract_package_hash";
const CONTRACT_PACKAGE_ACCESS_NAMED_KEY: &str = "fib_contract_package_access";

/// An error enum which can be converted to a `u16` so it can be returned as an `ApiError::User`.
#[repr(u16)]
enum Error {
    InvalidIndex = 0,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

#[no_mangle]
fn fib() {
    let n: u64 = runtime::get_named_arg(ARG_NTH_FIB);
    let fib: u64 = Fibonacci::new()
        .nth(
            n.try_into()
                .map_err(|_| Error::InvalidIndex)
                .unwrap_or_revert(),
        )
        .unwrap_or_revert();

    let val = storage::new_uref(fib);
    runtime::put_key(LAST_RESULT_NAMED_KEY, val.into());
}

#[no_mangle]
pub extern "C" fn call() {
    let entry_points = {
        let mut entry_points = EntryPoints::new();

        entry_points.add_entry_point(EntryPoint::new(
            FIB_ENTRY_POINT_NAME.to_string(),
            vec![Parameter::new(ARG_NTH_FIB, CLType::U64)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));

        entry_points
    };

    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        None,
        Some(CONTRACT_PACKAGE_HASH_NAMED_KEY.to_string()),
        Some(CONTRACT_PACKAGE_ACCESS_NAMED_KEY.to_string()),
    );

    runtime::put_key(CONTRACT_HASH_NAMED_KEY, contract_hash.into());
    runtime::put_key(
        CONTRACT_VERSION_NAMED_KEY,
        storage::new_uref(contract_version).into(),
    );
}

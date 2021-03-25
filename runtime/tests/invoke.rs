//! POC
//!
//! Test invoking ink! functions in wasmtime
use ceres_runtime::Runtime;

#[test]
fn test_deploy() {
    let mut rt = Runtime::from_contract(include_bytes!("../flipper.contract"))
        .expect("Create runtime failed");

    rt.deploy("0x9bae9d5e").expect("Deploy failed");
}
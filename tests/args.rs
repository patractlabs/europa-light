use ceres_ri::Instance;
use ceres_runtime::{MemoryStorage, Runtime};
use parity_scale_codec::Encode;
use std::{cell::RefCell, rc::Rc};

fn t(f: fn(rt: &mut Runtime)) {
    let shared = Rc::new(RefCell::new(MemoryStorage::new()));
    let mut args = Runtime::from_contract_and_storage(
        include_bytes!("../contracts/args.contract"),
        shared.clone(),
        Some(Instance),
    )
    .unwrap();

    // deploy
    assert!(args.deploy("default", vec![], None).is_ok());

    // run test
    f(&mut args);
}

#[test]
fn test_boolean() {
    t(|args: &mut Runtime| {
        assert_eq!(
            args.call("test_boolean", vec![true.encode()], None)
                .unwrap(),
            vec![1]
        );
    })
}

#[test]
fn test_number() {
    t(|args: &mut Runtime| {
        assert_eq!(
            args.call("test_number", vec![0.encode()], None).unwrap(),
            vec![0, 0, 0, 0]
        );
    })
}

#[test]
fn test_hash() {
    t(|args: &mut Runtime| {
        let hash = [0; 32];
        assert_eq!(
            args.call("test_hash", vec![hash.to_vec()], None).unwrap(),
            vec![0; 32]
        );
    })
}

#[test]
fn test_boolean_and_number() {
    t(|args: &mut Runtime| {
        assert_eq!(
            args.call(
                "test_boolean_and_number",
                vec![true.encode(), 1.encode()],
                None
            )
            .unwrap(),
            vec![1, 1, 0, 0, 0]
        );
    })
}

#[test]
fn test_boolean_and_hash() {
    t(|args: &mut Runtime| {
        let hash = [0; 32];
        let mut res = true.encode();
        res.append(&mut hash.to_vec());
        assert_eq!(
            args.call(
                "test_boolean_and_hash",
                vec![true.encode(), hash.to_vec()],
                None
            )
            .unwrap(),
            res
        );
    })
}

#[test]
fn test_number_and_number() {
    t(|args: &mut Runtime| {
        assert_eq!(
            args.call("test_number_and_number", vec![0.encode(), 1.encode()], None)
                .unwrap(),
            vec![0, 0, 0, 0, 1, 0, 0, 0]
        );
    })
}

#[test]
fn test_number_and_hash() {
    t(|args: &mut Runtime| {
        let hash = [0; 32];
        let mut res = 0.encode();
        res.append(&mut hash.to_vec());
        assert_eq!(
            args.call(
                "test_number_and_hash",
                vec![0.encode(), hash.to_vec()],
                None
            )
            .unwrap(),
            res,
        );
    })
}

#[test]
fn test_all() {
    t(|args: &mut Runtime| {
        let hash = [0; 32];
        let mut res = 0.encode();
        res.append(&mut hash.to_vec());
        res.append(&mut true.encode());
        assert_eq!(
            args.call(
                "test_all",
                vec![0.encode(), hash.to_vec(), true.encode()],
                None
            )
            .unwrap(),
            res,
        );
    })
}
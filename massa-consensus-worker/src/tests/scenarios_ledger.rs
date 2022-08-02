// Copyright (c) 2022 MASSA LABS <info@massa.net>

use super::tools::*;
use super::{
    mock_pool_controller::{MockPoolController, PoolCommandSink},
    mock_protocol_controller::MockProtocolController,
};
use crate::start_consensus_controller;

use massa_consensus_exports::settings::ConsensusChannels;
use massa_consensus_exports::tools::TEST_PASSWORD;
use massa_consensus_exports::ConsensusConfig;
use massa_execution_exports::test_exports::MockExecutionController;
use massa_graph::{ledger::Ledger, LedgerConfig};
use massa_models::ledger_models::LedgerData;
use massa_models::ledger_models::{LedgerChange, LedgerChanges};
use massa_models::prehash::Map;
use massa_models::{Amount, Slot};
use massa_signature::KeyPair;
use massa_storage::Storage;
use massa_time::MassaTime;
use serial_test::serial;
use std::collections::HashMap;
use std::str::FromStr;

#[tokio::test]
#[serial]
async fn test_ledger_init() {
    let cfg = ConsensusConfig::default_with_paths();
    let ledger = Ledger::new(LedgerConfig::from(&cfg), None);
    assert!(ledger.is_ok());
}

#[tokio::test]
#[serial]
async fn test_ledger_initializes_get_latest_final_periods() {
    let cfg = ConsensusConfig::default_with_paths();
    let ledger = Ledger::new(LedgerConfig::from(&cfg), None).unwrap();

    for latest_final in ledger
        .get_latest_final_periods()
        .expect("Couldn't get final periods.")
    {
        assert_eq!(latest_final, 0);
    }
}

#[tokio::test]
#[serial]
async fn test_ledger_final_balance_increment_new_address() {
    let cfg = ConsensusConfig::default_with_paths();
    let ledger = Ledger::new(LedgerConfig::from(&cfg), None).unwrap();

    let address = random_address().address;
    let thread = address.get_thread(cfg.thread_count);

    let changes = LedgerChanges(
        vec![(
            address,
            LedgerChange {
                balance_delta: Amount::from_str("1").unwrap(),
                balance_increment: true,
            },
        )]
        .into_iter()
        .collect(),
    );
    ledger
        .apply_final_changes(thread, &changes, 1)
        .expect("Couldn't apply final changes");

    let final_datas = ledger
        .get_final_data(vec![address].into_iter().collect())
        .expect("Couldn't get final balance.");
    let final_data_for_address = final_datas
        .0
        .get(&address)
        .expect("Couldn't get data for address.");
    assert_eq!(
        final_data_for_address.balance,
        Amount::from_str("1").unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_ledger_final_balance_increment_address_above_max() {
    let cfg = ConsensusConfig::default_with_paths();
    let ledger = Ledger::new(LedgerConfig::from(&cfg), None).unwrap();

    let address = random_address().address;
    let thread = address.get_thread(cfg.thread_count);

    let changes = LedgerChanges(
        vec![(
            address,
            LedgerChange {
                balance_delta: Amount::from_str("1").unwrap(),
                balance_increment: true,
            },
        )]
        .into_iter()
        .collect(),
    );
    ledger
        .apply_final_changes(thread, &changes, 1)
        .expect("Couldn't apply final changes");

    let final_datas = ledger
        .get_final_data(vec![address].into_iter().collect())
        .expect("Couldn't get final balance.");
    let final_data_for_address = final_datas
        .0
        .get(&address)
        .expect("Couldn't get data for address.");
    assert_eq!(
        final_data_for_address.balance,
        Amount::from_str("1").unwrap()
    );

    let changes = LedgerChanges(
        vec![(
            address,
            LedgerChange {
                balance_delta: Amount::from_raw(u64::MAX),
                balance_increment: true,
            },
        )]
        .into_iter()
        .collect(),
    );
    assert!(ledger.apply_final_changes(thread, &changes, 1).is_err());
}

#[tokio::test]
#[serial]
async fn test_ledger_final_balance_decrement_address_balance_to_zero() {
    let cfg = ConsensusConfig::default_with_paths();
    let ledger = Ledger::new(LedgerConfig::from(&cfg), None).unwrap();

    let address = random_address().address;
    let thread = address.get_thread(cfg.thread_count);

    // Increment.
    let changes = LedgerChanges(
        vec![(
            address,
            LedgerChange {
                balance_delta: Amount::from_str("1").unwrap(),
                balance_increment: true,
            },
        )]
        .into_iter()
        .collect(),
    );
    ledger
        .apply_final_changes(thread, &changes, 1)
        .expect("Couldn't apply final changes");

    let final_datas = ledger
        .get_final_data(vec![address].into_iter().collect())
        .expect("Couldn't get final balance.");
    let final_data_for_address = final_datas
        .0
        .get(&address)
        .expect("Couldn't get data for address.");
    assert_eq!(
        final_data_for_address.balance,
        Amount::from_str("1").unwrap()
    );

    // Decrement.
    let changes = LedgerChanges(
        vec![(
            address,
            LedgerChange {
                balance_delta: Amount::from_str("1").unwrap(),
                balance_increment: false,
            },
        )]
        .into_iter()
        .collect(),
    );
    ledger
        .apply_final_changes(thread, &changes, 1)
        .expect("Couldn't apply final changes");

    let final_datas = ledger
        .get_final_data(vec![address].into_iter().collect())
        .expect("Couldn't get final balance.");
    let final_data_for_address = final_datas
        .0
        .get(&address)
        .expect("Couldn't get data for address.");
    assert_eq!(final_data_for_address.balance, Amount::default());
}

#[tokio::test]
#[serial]
async fn test_ledger_final_balance_decrement_address_below_zero() {
    let cfg = ConsensusConfig::default_with_paths();
    let ledger = Ledger::new(LedgerConfig::from(&cfg), None).unwrap();

    let address = random_address().address;
    let thread = address.get_thread(cfg.thread_count);

    // Increment.
    let changes = LedgerChanges(
        vec![(
            address,
            LedgerChange {
                balance_delta: Amount::from_str("1").unwrap(),
                balance_increment: true,
            },
        )]
        .into_iter()
        .collect(),
    );
    ledger
        .apply_final_changes(thread, &changes, 1)
        .expect("Couldn't apply final changes");

    let final_datas = ledger
        .get_final_data(vec![address].into_iter().collect())
        .expect("Couldn't get final balance.");
    let final_data_for_address = final_datas
        .0
        .get(&address)
        .expect("Couldn't get data for address.");
    assert_eq!(
        final_data_for_address.balance,
        Amount::from_str("1").unwrap()
    );

    // Decrement.
    let changes = LedgerChanges(
        vec![(
            address,
            LedgerChange {
                balance_delta: Amount::from_str("1").unwrap(),
                balance_increment: false,
            },
        )]
        .into_iter()
        .collect(),
    );
    ledger
        .apply_final_changes(thread, &changes, 1)
        .expect("Couldn't apply final changes");

    let final_datas = ledger
        .get_final_data(vec![address].into_iter().collect())
        .expect("Couldn't get final balance.");
    let final_data_for_address = final_datas
        .0
        .get(&address)
        .expect("Couldn't get data for address.");
    assert_eq!(final_data_for_address.balance, Amount::default());

    // Try to decrement again.
    let changes = LedgerChanges(
        vec![(
            address,
            LedgerChange {
                balance_delta: Amount::from_str("1").unwrap(),
                balance_increment: false,
            },
        )]
        .into_iter()
        .collect(),
    );
    assert!(ledger.apply_final_changes(thread, &changes, 1).is_err());
}

#[tokio::test]
#[serial]
async fn test_ledger_final_balance_decrement_non_existing_address() {
    let cfg = ConsensusConfig::default_with_paths();
    let ledger = Ledger::new(LedgerConfig::from(&cfg), None).unwrap();

    let address = random_address().address;
    let thread = address.get_thread(cfg.thread_count);

    // Decrement.
    let changes = LedgerChanges(
        vec![(
            address,
            LedgerChange {
                balance_delta: Amount::from_str("1").unwrap(),
                balance_increment: false,
            },
        )]
        .into_iter()
        .collect(),
    );
    assert!(ledger.apply_final_changes(thread, &changes, 1).is_err());
}

#[tokio::test]
#[serial]
async fn test_ledger_final_balance_non_existing_address() {
    let cfg = ConsensusConfig::default_with_paths();
    let ledger = Ledger::new(LedgerConfig::from(&cfg), None).unwrap();

    let address = random_address().address;

    let final_datas = ledger
        .get_final_data(vec![address].into_iter().collect())
        .expect("Couldn't get final balance.");
    let final_data_for_address = final_datas
        .0
        .get(&address)
        .expect("Couldn't get data for address.");
    assert_eq!(final_data_for_address.balance, Amount::default());
}

#[tokio::test]
#[serial]
async fn test_ledger_final_balance_duplicate_address() {
    let cfg = ConsensusConfig::default_with_paths();
    let ledger = Ledger::new(LedgerConfig::from(&cfg), None).unwrap();

    let address = random_address().address;

    // Same address twice.
    let final_datas = ledger
        .get_final_data(vec![address, address].into_iter().collect())
        .expect("Couldn't get final balance.");
    let final_data_for_address = final_datas
        .0
        .get(&address)
        .expect("Couldn't get data for address.");
    assert_eq!(final_data_for_address.balance, Amount::default());

    // Should have returned a single result.
    assert_eq!(final_datas.0.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_ledger_final_balance_multiple_addresses() {
    let cfg = ConsensusConfig::default_with_paths();
    let ledger = Ledger::new(LedgerConfig::from(&cfg), None).unwrap();

    let mut addresses = vec![];
    for _ in 0..5 {
        addresses.push(random_address().address);
    }

    let final_datas = ledger
        .get_final_data(addresses.iter().copied().collect())
        .expect("Couldn't get final balance.");

    assert_eq!(final_datas.0.len(), addresses.len());

    for address in addresses {
        let final_data_for_address = final_datas
            .0
            .get(&address)
            .expect("Couldn't get data for address.");
        assert_eq!(final_data_for_address.balance, Amount::default());
    }
}

#[tokio::test]
#[serial]
async fn test_ledger_clear() {
    let cfg = ConsensusConfig::default_with_paths();
    let ledger = Ledger::new(LedgerConfig::from(&cfg), None).unwrap();

    let address = random_address().address;
    let thread = address.get_thread(cfg.thread_count);

    let changes = LedgerChanges(
        vec![(
            address,
            LedgerChange {
                balance_delta: Amount::from_str("1").unwrap(),
                balance_increment: true,
            },
        )]
        .into_iter()
        .collect(),
    );
    ledger
        .apply_final_changes(thread, &changes, 1)
        .expect("Couldn't apply final changes");

    let final_datas = ledger
        .get_final_data(vec![address].into_iter().collect())
        .expect("Couldn't get final balance.");
    let final_data_for_address = final_datas
        .0
        .get(&address)
        .expect("Couldn't get data for address.");
    assert_eq!(
        final_data_for_address.balance,
        Amount::from_str("1").unwrap()
    );

    ledger.clear().expect("Couldn't clear the ledger.");

    let final_datas = ledger
        .get_final_data(vec![address].into_iter().collect())
        .expect("Couldn't get final balance.");
    let final_data_for_address = final_datas
        .0
        .get(&address)
        .expect("Couldn't get data for address.");
    assert_eq!(final_data_for_address.balance, Amount::default());
}

#[tokio::test]
#[serial]
async fn test_ledger_read_whole() {
    let cfg = ConsensusConfig::default_with_paths();
    let ledger = Ledger::new(LedgerConfig::from(&cfg), None).unwrap();

    let address = random_address().address;
    let thread = address.get_thread(cfg.thread_count);

    let changes = LedgerChanges(
        vec![(
            address,
            LedgerChange {
                balance_delta: Amount::from_str("1").unwrap(),
                balance_increment: true,
            },
        )]
        .into_iter()
        .collect(),
    );
    ledger
        .apply_final_changes(thread, &changes, 1)
        .expect("Couldn't apply final changes");

    let final_datas = ledger
        .get_final_data(vec![address].into_iter().collect())
        .expect("Couldn't get final balance.");
    let final_data_for_address = final_datas
        .0
        .get(&address)
        .expect("Couldn't get data for address.");
    assert_eq!(
        final_data_for_address.balance,
        Amount::from_str("1").unwrap()
    );

    let whole_ledger = ledger.read_whole().expect("Couldn't read whole ledger.");
    let address_data = whole_ledger
        .0
        .iter()
        .filter(|(addr, _)| **addr == address)
        .collect::<Vec<_>>()
        .pop()
        .expect("Couldn't find ledger data for address.")
        .1;
    assert_eq!(address_data.balance, Amount::from_str("1").unwrap());
}

#[tokio::test]
#[serial]
async fn test_ledger_update_when_a_batch_of_blocks_becomes_final() {
    let thread_count = 2;
    let (address_1, keypair_1) = random_address_on_thread(0, thread_count).into();
    let (address_2, keypair_2) = random_address_on_thread(1, thread_count).into();
    let (address_3, _) = random_address_on_thread(0, thread_count).into();

    // Ledger at genesis:
    //
    // Thread 0:
    // address A balance = 1000
    // address C absent from ledger
    //
    // Thread 1:
    // address B balance = 3000
    let mut ledger = HashMap::new();
    ledger.insert(
        address_1,
        LedgerData::new(Amount::from_str("1000").unwrap()),
    );
    ledger.insert(
        address_2,
        LedgerData::new(Amount::from_str("3000").unwrap()),
    );
    let staking_keys: Vec<KeyPair> = vec![keypair_1.clone()];

    let cfg = ConsensusConfig {
        t0: 1000.into(),
        genesis_timestamp: MassaTime::now()
            .unwrap()
            .saturating_sub(MassaTime::from(1000).checked_mul(10).unwrap()),
        delta_f0: 4,
        operation_validity_periods: 20,
        ..ConsensusConfig::default_with_staking_keys_and_ledger(&staking_keys, &ledger)
    };
    let storage: Storage = Default::default();

    // mock protocol & pool
    let (mut protocol_controller, protocol_command_sender, protocol_event_receiver) =
        MockProtocolController::new();
    let (pool_controller, pool_command_sender) = MockPoolController::new();
    let pool_sink = PoolCommandSink::new(pool_controller).await;
    let (execution_controller, _execution_rx) = MockExecutionController::new_with_receiver();

    // launch consensus controller
    let (consensus_command_sender, consensus_event_receiver, consensus_manager) =
        start_consensus_controller(
            cfg.clone(),
            ConsensusChannels {
                execution_controller,
                protocol_command_sender: protocol_command_sender.clone(),
                protocol_event_receiver,
                pool_command_sender,
            },
            None,
            None,
            storage.clone(),
            0,
            TEST_PASSWORD.to_string(),
            Map::default(),
        )
        .await
        .expect("could not start consensus controller");

    let genesis_ids = consensus_command_sender
        .get_block_graph_status(None, None)
        .await
        .expect("could not get block graph status")
        .genesis_blocks;

    // A -> B [amount 10, fee 3]
    let operation_1 = create_transaction(&keypair_1, address_2, 10, 10, 3);
    storage.store_operation(operation_1.clone());

    // Add block B3
    let block_a = create_block_with_operations(
        &cfg,
        Slot::new(1, 0),
        &&genesis_ids,
        &staking_keys[0],
        vec![operation_1],
    );
    protocol_controller.receive_block(block_a.clone()).await;
    validate_propagate_block(&mut protocol_controller, block_a.id, 150).await;

    // B -> A [amount 9, fee 2]
    let operation_2 = create_transaction(&keypair_2, address_1, 9, 10, 2);
    storage.store_operation(operation_2.clone());

    // B -> C [amount 3, fee 1]
    let operation_3 = create_transaction(&keypair_2, address_3, 3, 10, 1);
    storage.store_operation(operation_3.clone());

    // Add block B4
    let block_b = create_block_with_operations(
        &cfg,
        Slot::new(1, 1),
        &&genesis_ids,
        &staking_keys[0],
        vec![operation_2, operation_3],
    );
    protocol_controller.receive_block(block_b.clone()).await;
    validate_propagate_block(&mut protocol_controller, block_b.id, 150).await;

    // A -> C [amount 3, fee 4]
    let operation_4 = create_transaction(&keypair_1, address_3, 3, 10, 4);
    storage.store_operation(operation_4.clone());

    // Add block B5
    let block_c = create_block_with_operations(
        &cfg,
        Slot::new(2, 0),
        &vec![block_a.id, block_b.id],
        &staking_keys[0],
        vec![operation_4],
    );
    protocol_controller.receive_block(block_c.clone()).await;
    validate_propagate_block(&mut protocol_controller, block_c.id, 150).await;

    // Add block B6, no operations.
    let block_d = create_block_with_operations(
        &cfg,
        Slot::new(2, 1),
        &vec![block_a.id, block_b.id],
        &staking_keys[0],
        vec![],
    );
    protocol_controller.receive_block(block_d.clone()).await;
    validate_propagate_block(&mut protocol_controller, block_d.id, 150).await;

    // A -> B [amount 11, fee 7]
    let operation_5 = create_transaction(&keypair_1, address_2, 11, 10, 7);
    storage.store_operation(operation_5.clone());
    // Add block B7
    let block_e = create_block_with_operations(
        &cfg,
        Slot::new(3, 0),
        &vec![block_c.id, block_b.id],
        &staking_keys[0],
        vec![operation_5],
    );
    protocol_controller.receive_block(block_e.clone()).await;
    validate_propagate_block(&mut protocol_controller, block_e.id, 150).await;

    // B -> A [amount 17, fee 4]
    let operation_6 = create_transaction(&keypair_2, address_1, 17, 10, 4);
    storage.store_operation(operation_6.clone());
    // Add block B8
    let block_f = create_block_with_operations(
        &cfg,
        Slot::new(3, 1),
        &vec![block_c.id, block_d.id],
        &staking_keys[0],
        vec![operation_6],
    );
    protocol_controller.receive_block(block_f.clone()).await;
    validate_propagate_block(&mut protocol_controller, block_f.id, 150).await;

    // Add block B9
    let block_g = create_block_with_operations(
        &cfg,
        Slot::new(4, 0),
        &vec![block_e.id, block_f.id],
        &staking_keys[0],
        vec![],
    );
    protocol_controller.receive_block(block_g.clone()).await;
    validate_propagate_block(&mut protocol_controller, block_g.id, 150).await;

    // B3 and B4 have become final.
    {
        let ledger = consensus_command_sender
            .get_bootstrap_state()
            .await
            .unwrap()
            .1
            .ledger;
        assert_eq!(
            ledger.0[&address_1].balance,
            Amount::from_str("991").unwrap(),
            "wrong address balance"
        );
        assert_eq!(
            ledger.0[&address_2].balance,
            Amount::from_str("2985").unwrap(),
            "wrong address balance"
        );
        assert!(
            !ledger.0.contains_key(&address_3),
            "address shouldn't be present"
        );
    }

    // Add block B10
    let block_h = create_block_with_operations(
        &cfg,
        Slot::new(5, 0),
        &vec![block_g.id, block_f.id],
        &staking_keys[0],
        vec![],
    );
    protocol_controller.receive_block(block_h.clone()).await;
    validate_propagate_block(&mut protocol_controller, block_h.id, 150).await;

    // Add block B11
    let block_i = create_block_with_operations(
        &cfg,
        Slot::new(6, 0),
        &vec![block_h.id, block_f.id],
        &staking_keys[0],
        vec![],
    );
    protocol_controller.receive_block(block_i.clone()).await;
    validate_propagate_block(&mut protocol_controller, block_i.id, 150).await;

    // B5 has become final.
    {
        let ledger = consensus_command_sender
            .get_bootstrap_state()
            .await
            .unwrap()
            .1
            .ledger;
        assert_eq!(
            ledger.0[&address_1].balance,
            Amount::from_str("1002").unwrap(),
            "wrong address balance"
        );
        assert_eq!(
            ledger.0[&address_2].balance,
            Amount::from_str("2985").unwrap(),
            "wrong address balance"
        );
        assert_eq!(
            ledger.0[&address_3].balance,
            Amount::from_str("6").unwrap(),
            "wrong address balance"
        );
    }

    // Add block B12
    let block_j = create_block_with_operations(
        &cfg,
        Slot::new(7, 0),
        &vec![block_i.id, block_f.id],
        &staking_keys[0],
        vec![],
    );
    protocol_controller.receive_block(block_j.clone()).await;
    validate_propagate_block(&mut protocol_controller, block_j.id, 150).await;

    // B6 has become final.
    {
        let ledger = consensus_command_sender
            .get_bootstrap_state()
            .await
            .unwrap()
            .1
            .ledger;
        assert_eq!(
            ledger.0[&address_1].balance,
            Amount::from_str("1002").unwrap(),
            "wrong address balance"
        );
        assert_eq!(
            ledger.0[&address_2].balance,
            Amount::from_str("2995").unwrap(),
            "wrong address balance"
        );
        assert_eq!(
            ledger.0[&address_3].balance,
            Amount::from_str("6").unwrap(),
            "wrong address balance"
        );
    }

    // Add block B13
    let block_k = create_block_with_operations(
        &cfg,
        Slot::new(8, 0),
        &vec![block_j.id, block_f.id],
        &staking_keys[0],
        vec![],
    );
    protocol_controller.receive_block(block_k.clone()).await;
    validate_propagate_block(&mut protocol_controller, block_k.id, 150).await;

    // B7 and B8 have become final.
    {
        let ledger = consensus_command_sender
            .get_bootstrap_state()
            .await
            .unwrap()
            .1
            .ledger;
        assert_eq!(
            ledger.0[&address_1].balance,
            Amount::from_str("992").unwrap(),
            "wrong address balance"
        );
        assert_eq!(
            ledger.0[&address_2].balance,
            Amount::from_str("2974").unwrap(),
            "wrong address balance"
        );
        assert_eq!(
            ledger.0[&address_3].balance,
            Amount::from_str("6").unwrap(),
            "wrong address balance"
        );
    }

    // stop controller while ignoring all commands
    let stop_fut = consensus_manager.stop(consensus_event_receiver);
    tokio::pin!(stop_fut);
    protocol_controller
        .ignore_commands_while(stop_fut)
        .await
        .unwrap();
    pool_sink.stop().await;
}

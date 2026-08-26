use domain::models::condition::{Comparator, Condition, Metric};
use domain::models::intent::{Asset, ConditionalIntent, DexType, Intent};
use domain::ports::StoragePort;
use domain::ports::storage_port::{DelegationRecord, ExecutionRecord, IntentRecord};
use infrastructure::storage::SqliteStorage;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_db_path() -> String {
    let dir = std::env::temp_dir();
    let file = format!("otter-storage-test-{}.db", rand::random::<u64>());
    dir.join(file).to_string_lossy().to_string()
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn sample_intent_record(id: &str) -> IntentRecord {
    IntentRecord {
        id: id.to_string(),
        text: "swap 1 ETH for USDC on Uniswap".to_string(),
        intent: ConditionalIntent {
            intent: Intent::Swap {
                from_asset: Asset::Eth,
                to_asset: Asset::Usdc,
                amount: 1_000_000_000_000_000_000,
                protocol: DexType::Uniswap,
            },
            condition: Some(Condition::Comparison {
                metric: Metric::Price,
                comparator: Comparator::GreaterThan,
                value: 2_000,
            }),
            network: None,
        },
        state: "active".to_string(),
        created_at: now_secs(),
        updated_at: now_secs(),
        user_address: None,
    }
}

fn sample_delegation_record(hash: &str) -> DelegationRecord {
    DelegationRecord {
        hash: hash.to_string(),
        payload_json: r#"{"pubkey_x":[0;32],"pubkey_y":[0;32]}"#.to_string(),
        signature: "0x".to_string(),
        created_at: now_secs(),
        user_address: None,
    }
}

fn sample_execution_record(id: &str, intent_id: &str) -> ExecutionRecord {
    ExecutionRecord {
        id: id.to_string(),
        intent_id: intent_id.to_string(),
        tx_hash: "0xdeadbeef".to_string(),
        status: "success".to_string(),
        gas_used: 21000,
        created_at: now_secs(),
    }
}

#[tokio::test]
async fn save_and_list_delegations() {
    let storage = SqliteStorage::new(temp_db_path()).unwrap();
    storage
        .save_delegation(&sample_delegation_record("delegation-1"))
        .await
        .unwrap();

    let delegations = storage.list_delegations().await.unwrap();
    assert_eq!(delegations.len(), 1);
    assert_eq!(delegations[0].hash, "delegation-1");
}

#[tokio::test]
async fn get_delegation_by_hash() {
    let storage = SqliteStorage::new(temp_db_path()).unwrap();
    storage
        .save_delegation(&sample_delegation_record("a"))
        .await
        .unwrap();

    let found = storage.get_delegation("a").await.unwrap().unwrap();
    assert_eq!(found.hash, "a");
    assert!(storage.get_delegation("missing").await.unwrap().is_none());
}

#[tokio::test]
async fn save_and_list_executions() {
    let storage = SqliteStorage::new(temp_db_path()).unwrap();
    storage
        .save_intent(&sample_intent_record("intent-1"))
        .await
        .unwrap();
    storage
        .save_execution(&sample_execution_record("exec-1", "intent-1"))
        .await
        .unwrap();

    let executions = storage.list_executions().await.unwrap();
    assert_eq!(executions.len(), 1);
    assert_eq!(executions[0].id, "exec-1");
    assert_eq!(executions[0].gas_used, 21000);
}

#[tokio::test]
async fn get_executions_for_intent() {
    let storage = SqliteStorage::new(temp_db_path()).unwrap();
    storage
        .save_intent(&sample_intent_record("intent-x"))
        .await
        .unwrap();
    storage
        .save_execution(&sample_execution_record("exec-x", "intent-x"))
        .await
        .unwrap();

    let executions = storage.get_executions_for_intent("intent-x").await.unwrap();
    assert_eq!(executions.len(), 1);
    assert_eq!(executions[0].intent_id, "intent-x");

    assert!(
        storage
            .get_executions_for_intent("missing")
            .await
            .unwrap()
            .is_empty()
    );
}

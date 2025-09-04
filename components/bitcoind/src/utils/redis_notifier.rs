use std::sync::{Mutex, OnceLock};

use bitcoin::Network;
use config::{BitcoindConfig, RedisConfig};
use redis::{AsyncCommands, Client};
use serde::Serialize;

use crate::types::BlockIdentifier;

#[derive(Serialize)]
struct IndexProgressPayload<'a> {
    chain: &'a str,
    network: &'a str,
    indexer: &'a str,
    apply_blocks: Vec<BlockIndexRef>,
    rollback_blocks: Vec<BlockIndexRef>,
}

#[derive(Serialize)]
struct Message<'a> {
    id: String,
    payload: IndexProgressPayload<'a>,
}

#[derive(Serialize)]
struct BlockIndexRef {
    hash: String,
    index: u64,
}

pub struct RedisNotifier {
    pub queue: String,
    pub client: Client,
}

impl RedisNotifier {
    pub fn new(redis: &RedisConfig) -> Result<Self, String> {
        let client = Client::open(redis.url.as_str())
            .map_err(|e| format!("unable to connect to redis: {e}"))?;
        Ok(Self {
            queue: redis.queue.clone(),
            client,
        })
    }

    fn network_str(bitcoind: &BitcoindConfig) -> &'static str {
        match bitcoind.network {
            Network::Bitcoin => "mainnet",
            Network::Testnet => todo!(),
            Network::Signet => todo!(),
            Network::Regtest => todo!(),
            _ => todo!(),
        }
    }

    pub async fn notify(
        &self,
        indexer: &str,
        bitcoind: &BitcoindConfig,
        apply_blocks: &[BlockIdentifier],
        rollback_blocks: &[BlockIdentifier],
    ) -> Result<(), String> {
        let payload = build_message_json(indexer, bitcoind, apply_blocks, rollback_blocks)?;
        // In-test capture path (no real Redis required)
        if std::env::var("REDIS_TEST_CAPTURE").ok().as_deref() == Some("1") {
            capture_messages().lock().unwrap().push(payload);
            return Ok(());
        }

        let mut conn = self
            .client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| format!("unable to get redis connection: {e}"))?;
        conn.rpush::<_, _, ()>(&self.queue, payload)
            .await
            .map_err(|e| format!("unable to rpush redis message: {e}"))?;
        Ok(())
    }
}

static CAPTURED_MESSAGES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn capture_messages() -> &'static Mutex<Vec<String>> {
    CAPTURED_MESSAGES.get_or_init(|| Mutex::new(Vec::new()))
}

#[cfg(test)]
fn take_captured_messages() -> Vec<String> {
    let mut guard = capture_messages().lock().unwrap();
    let messages = guard.clone();
    guard.clear();
    messages
}

fn js_sys_time_ms() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

pub fn build_message_json(
    indexer: &str,
    bitcoind: &BitcoindConfig,
    apply_blocks: &[BlockIdentifier],
    rollback_blocks: &[BlockIdentifier],
) -> Result<String, String> {
    let network = RedisNotifier::network_str(bitcoind);
    let id = format!(
        "bitcoin-{}-{}-{}",
        indexer,
        apply_blocks
            .last()
            .map(|b| b.index.to_string())
            .unwrap_or_else(|| "na".to_string()),
        js_sys_time_ms()
    );
    let message = Message {
        id,
        payload: IndexProgressPayload {
            chain: "bitcoin",
            network,
            indexer,
            apply_blocks: apply_blocks
                .iter()
                .map(|b| BlockIndexRef {
                    hash: b.hash.clone(),
                    index: b.index,
                })
                .collect(),
            rollback_blocks: rollback_blocks
                .iter()
                .map(|b| BlockIndexRef {
                    hash: b.hash.clone(),
                    index: b.index,
                })
                .collect(),
        },
    };
    serde_json::to_string(&message).map_err(|e| format!("unable to serialize redis message: {e}"))
}

#[cfg(test)]
mod tests {
    use bitcoin::Network;
    use config::{BitcoindConfig, RedisConfig};
    use tokio::runtime::Runtime;

    use super::{build_message_json, take_captured_messages, RedisNotifier};
    use crate::types::BlockIdentifier;

    fn sample_bitcoind(network: Network) -> BitcoindConfig {
        BitcoindConfig {
            network,
            rpc_url: "http://localhost:18443".into(),
            rpc_username: "user".into(),
            rpc_password: "pass".into(),
            zmq_url: "tcp://127.0.0.1:28332".into(),
        }
    }

    #[test]
    fn test_build_message_json_for_runes() {
        let bitcoind = sample_bitcoind(Network::Testnet);
        let apply = vec![
            BlockIdentifier {
                index: 100,
                hash: "0xabc".into(),
            },
            BlockIdentifier {
                index: 101,
                hash: "0xdef".into(),
            },
        ];
        let rollback = vec![BlockIdentifier {
            index: 99,
            hash: "0x999".into(),
        }];
        let json = build_message_json("runes", &bitcoind, &apply, &rollback).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v["id"].as_str().unwrap().starts_with("bitcoin-runes-"));
        assert_eq!(v["payload"]["chain"], "bitcoin");
        assert_eq!(v["payload"]["indexer"], "runes");
        assert_eq!(v["payload"]["network"], "testnet");
        let apply_blocks = v["payload"]["apply_blocks"].as_array().unwrap();
        assert_eq!(apply_blocks.len(), 2);
        assert_eq!(apply_blocks[0]["hash"], "0xabc");
        assert_eq!(apply_blocks[0]["index"], 100);
        assert_eq!(apply_blocks[1]["hash"], "0xdef");
        assert_eq!(apply_blocks[1]["index"], 101);
        let rollback_blocks = v["payload"]["rollback_blocks"].as_array().unwrap();
        assert_eq!(rollback_blocks.len(), 1);
        assert_eq!(rollback_blocks[0]["hash"], "0x999");
        assert_eq!(rollback_blocks[0]["index"], 99);
    }

    #[test]
    fn test_build_message_json_for_ordinals() {
        let bitcoind = sample_bitcoind(Network::Bitcoin);
        let apply = vec![BlockIdentifier {
            index: 800_000,
            hash: "0xfeed".into(),
        }];
        let rollback: Vec<BlockIdentifier> = vec![];
        let json = build_message_json("ordinals", &bitcoind, &apply, &rollback).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v["id"].as_str().unwrap().starts_with("bitcoin-ordinals-"));
        assert_eq!(v["payload"]["chain"], "bitcoin");
        assert_eq!(v["payload"]["indexer"], "ordinals");
        assert_eq!(v["payload"]["network"], "mainnet");
        let apply_blocks = v["payload"]["apply_blocks"].as_array().unwrap();
        assert_eq!(apply_blocks.len(), 1);
        assert_eq!(apply_blocks[0]["hash"], "0xfeed");
        assert_eq!(apply_blocks[0]["index"], 800_000);
        let rollback_blocks = v["payload"]["rollback_blocks"].as_array().unwrap();
        assert!(rollback_blocks.is_empty());
    }

    #[test]
    fn test_build_message_json_for_runes_reorg() {
        let bitcoind = sample_bitcoind(Network::Testnet);
        let apply = vec![
            BlockIdentifier {
                index: 2,
                hash: "0x1235aa".into(),
            },
            BlockIdentifier {
                index: 3,
                hash: "0x1236".into(),
            },
        ];
        let rollback = vec![BlockIdentifier {
            index: 2,
            hash: "0x1235".into(),
        }];
        let json = build_message_json("runes", &bitcoind, &apply, &rollback).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v["id"].as_str().unwrap().starts_with("bitcoin-runes-"));
        assert_eq!(v["payload"]["chain"], "bitcoin");
        assert_eq!(v["payload"]["indexer"], "runes");
        assert_eq!(v["payload"]["network"], "testnet");
        let apply_blocks = v["payload"]["apply_blocks"].as_array().unwrap();
        assert_eq!(apply_blocks.len(), 2);
        assert_eq!(apply_blocks[0]["hash"], "0x1235aa");
        assert_eq!(apply_blocks[0]["index"], 2);
        assert_eq!(apply_blocks[1]["hash"], "0x1236");
        assert_eq!(apply_blocks[1]["index"], 3);
        let rollback_blocks = v["payload"]["rollback_blocks"].as_array().unwrap();
        assert_eq!(rollback_blocks.len(), 1);
        assert_eq!(rollback_blocks[0]["hash"], "0x1235");
        assert_eq!(rollback_blocks[0]["index"], 2);
    }

    #[test]
    fn test_build_message_json_for_ordinals_reorg() {
        let bitcoind = sample_bitcoind(Network::Bitcoin);
        let apply = vec![
            BlockIdentifier {
                index: 820_001,
                hash: "0xabc1".into(),
            },
            BlockIdentifier {
                index: 820_002,
                hash: "0xabc2".into(),
            },
        ];
        let rollback = vec![BlockIdentifier {
            index: 820_001,
            hash: "0xold1".into(),
        }];
        let json = build_message_json("ordinals", &bitcoind, &apply, &rollback).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v["id"].as_str().unwrap().starts_with("bitcoin-ordinals-"));
        assert_eq!(v["payload"]["chain"], "bitcoin");
        assert_eq!(v["payload"]["indexer"], "ordinals");
        assert_eq!(v["payload"]["network"], "mainnet");
        let apply_blocks = v["payload"]["apply_blocks"].as_array().unwrap();
        assert_eq!(apply_blocks.len(), 2);
        assert_eq!(apply_blocks[0]["hash"], "0xabc1");
        assert_eq!(apply_blocks[0]["index"], 820_001);
        assert_eq!(apply_blocks[1]["hash"], "0xabc2");
        assert_eq!(apply_blocks[1]["index"], 820_002);
        let rollback_blocks = v["payload"]["rollback_blocks"].as_array().unwrap();
        assert_eq!(rollback_blocks.len(), 1);
        assert_eq!(rollback_blocks[0]["hash"], "0xold1");
        assert_eq!(rollback_blocks[0]["index"], 820_001);
    }

    #[test]
    fn test_notifier_pushes_captured_message() {
        // Enable capture via env var and run notify; ensure a payload is captured
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            std::env::set_var("REDIS_TEST_CAPTURE", "1");
            let notifier = RedisNotifier::new(&RedisConfig {
                enabled: true,
                url: "redis://127.0.0.1:6379/0".into(),
                queue: "test-queue".into(),
            })
            .unwrap();
            let bitcoind = sample_bitcoind(Network::Regtest);
            let apply = vec![BlockIdentifier {
                index: 1,
                hash: "0x1234".into(),
            }];
            let rollback: Vec<BlockIdentifier> = vec![];
            notifier
                .notify("runes", &bitcoind, &apply, &rollback)
                .await
                .unwrap();
            std::env::remove_var("REDIS_TEST_CAPTURE");
        });
        let captured = take_captured_messages();
        assert_eq!(captured.len(), 1);
        let v: serde_json::Value = serde_json::from_str(&captured[0]).unwrap();
        assert_eq!(v["payload"]["chain"], "bitcoin");
        assert_eq!(v["payload"]["indexer"], "runes");
        assert_eq!(v["payload"]["network"], "regtest");
        let apply_blocks = v["payload"]["apply_blocks"].as_array().unwrap();
        assert_eq!(apply_blocks.len(), 1);
        assert_eq!(apply_blocks[0]["hash"], "0x1234");
        assert_eq!(apply_blocks[0]["index"], 1);
    }

    #[test]
    #[ignore = "requires local Redis at redis://127.0.0.1:6379/0"]
    fn notify_pushes_to_redis() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let queue = "it_test_q";
            let notifier = RedisNotifier::new(&RedisConfig {
                enabled: true,
                url: "redis://127.0.0.1:6379/0".into(),
                queue: queue.into(),
            })
            .unwrap();

            let bitcoind = sample_bitcoind(Network::Regtest);
            let apply = vec![BlockIdentifier {
                index: 2,
                hash: "0xabc".into(),
            }];
            notifier
                .notify("ordinals", &bitcoind, &apply, &[])
                .await
                .unwrap();

            let client = redis::Client::open("redis://127.0.0.1:6379/0").unwrap();
            let mut conn = client.get_multiplexed_tokio_connection().await.unwrap();
            let msg: String = redis::cmd("RPOP")
                .arg(queue)
                .query_async(&mut conn)
                .await
                .unwrap();
            let v: serde_json::Value = serde_json::from_str(&msg).unwrap();
            assert_eq!(v["payload"]["indexer"], "ordinals");
            assert_eq!(v["payload"]["apply_blocks"][0]["hash"], "0xabc");
        });
    }
}

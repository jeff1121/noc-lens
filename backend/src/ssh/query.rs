//! 查詢編排：以併發上限對多台設備執行查詢並寫入快照。

use futures::stream::{self, StreamExt};
use serde_json::{Map, Value};
use sqlx::SqlitePool;

use crate::db::{device, snapshot};
use crate::models::QueryResult;
use crate::ssh::executor::{SshExecutor, SshTarget};
use crate::ssh::{commands, parsers};

const NA: &str = "n/a";
const SSH_PORT: u16 = 22;

/// 對多台設備併發執行查詢；逐台回報，單台失敗不影響其他台。
pub async fn run_query<E>(
    pool: &SqlitePool,
    executor: &E,
    device_ids: &[String],
    max_concurrency: usize,
) -> Vec<QueryResult>
where
    E: SshExecutor + Sync,
{
    let conc = max_concurrency.max(1);
    stream::iter(device_ids.iter().cloned())
        .map(|id| query_one(pool, executor, id))
        .buffer_unordered(conc)
        .collect::<Vec<_>>()
        .await
}

async fn query_one<E>(pool: &SqlitePool, executor: &E, id: String) -> QueryResult
where
    E: SshExecutor + Sync,
{
    let dev = match device::get(pool, &id).await {
        Ok(d) => d,
        Err(e) => return failed(&id, e.to_string()),
    };
    let password = match device::get_password(pool, &id).await {
        Ok(p) => p,
        Err(e) => return failed(&id, e.to_string()),
    };

    let cmds = commands::for_brand(dev.brand);
    let cmd_strs: Vec<String> = cmds.iter().map(|(_, c)| c.clone()).collect();
    let target = SshTarget {
        host: &dev.ip_address,
        port: SSH_PORT,
        username: &dev.username,
        password: &password,
    };

    match executor.run(target, &cmd_strs).await {
        Err(e) => {
            let _ = snapshot::insert(
                pool,
                &id,
                None,
                "failed",
                Some(&e.to_string()),
                &Value::Null,
            )
            .await;
            failed(&id, e.to_string())
        }
        Ok(outputs) => {
            let mut metrics = Map::new();
            let mut any_fail = false;
            for ((metric, _), out) in cmds.iter().zip(outputs.iter()) {
                if out.ok {
                    metrics.insert(
                        metric.to_string(),
                        parsers::parse_metric(dev.brand, metric, &out.text),
                    );
                } else {
                    metrics.insert(metric.to_string(), Value::String(NA.to_string()));
                    any_fail = true;
                }
            }
            let metrics_val = Value::Object(metrics);
            let status = if any_fail { "partial" } else { "ok" };
            let snapshot_id = snapshot::insert(pool, &id, None, status, None, &metrics_val)
                .await
                .ok();
            QueryResult {
                device_id: id,
                status: status.to_string(),
                error_message: None,
                metrics: Some(metrics_val),
                snapshot_id,
            }
        }
    }
}

fn failed(id: &str, message: String) -> QueryResult {
    QueryResult {
        device_id: id.to_string(),
        status: "failed".to_string(),
        error_message: Some(message),
        metrics: None,
        snapshot_id: None,
    }
}

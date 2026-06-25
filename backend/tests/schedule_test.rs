//! US3 整合測試：排程建立、單次執行、JobRun 統計與快照寫入。

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use noc_lens_backend::db::{self, device, group, schedule, snapshot};
use noc_lens_backend::models::{Brand, NewDevice, NewScheduledJob};
use noc_lens_backend::scheduler::run_job_once;
use noc_lens_backend::ssh::executor::{CmdOutput, SshExecutor, SshTarget};
use sqlx::SqlitePool;

async fn setup() -> SqlitePool {
    std::env::set_var("NOC_LENS_MASTER_KEY", STANDARD.encode([9u8; 32]));
    let path = std::env::temp_dir().join(format!("noc-lens-{}.db", uuid::Uuid::new_v4()));
    db::init_pool(path.to_str().unwrap()).await.unwrap()
}

struct OkExecutor;
impl SshExecutor for OkExecutor {
    async fn run(
        &self,
        _t: SshTarget<'_>,
        commands: &[String],
    ) -> Result<Vec<CmdOutput>, noc_lens_backend::AppError> {
        let n = commands.len();
        Ok((0..n)
            .map(|_| CmdOutput {
                ok: true,
                text: "CPU utilization for five seconds: 30%".to_string(),
            })
            .collect())
    }
}

async fn add_device(pool: &SqlitePool, ip: &str) -> String {
    device::create(
        pool,
        NewDevice {
            ip_address: ip.to_string(),
            username: "admin".to_string(),
            password: "secret".to_string(),
            note: None,
            brand: Brand::Cisco,
        },
    )
    .await
    .unwrap()
    .id
}

#[tokio::test]
async fn run_group_job_collects_and_records() {
    let pool = setup().await;
    let d1 = add_device(&pool, "10.7.0.1").await;
    let d2 = add_device(&pool, "10.7.0.2").await;
    let g = group::create(&pool, "排程群組").await.unwrap();
    group::assign(&pool, &d1, std::slice::from_ref(&g.id))
        .await
        .unwrap();
    group::assign(&pool, &d2, std::slice::from_ref(&g.id))
        .await
        .unwrap();

    let job = schedule::create(
        &pool,
        NewScheduledJob {
            name: "每日巡檢".to_string(),
            target_type: "group".to_string(),
            target_id: g.id.clone(),
            schedule_kind: "interval".to_string(),
            interval_minutes: Some(60),
            daily_time: None,
        },
    )
    .await
    .unwrap();

    let run = run_job_once(&pool, &OkExecutor, &job.id).await.unwrap();
    assert_eq!(run.total, 2);
    assert_eq!(run.success_count, 2);
    assert_eq!(run.failure_count, 0);
    assert!(run.finished_at.is_some());

    // 兩台設備各產生一筆快照
    assert_eq!(
        snapshot::list_by_device(&pool, &d1, 10)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        snapshot::list_by_device(&pool, &d2, 10)
            .await
            .unwrap()
            .len(),
        1
    );

    // 執行紀錄可列出
    let runs = schedule::run_list(&pool, &job.id).await.unwrap();
    assert_eq!(runs.len(), 1);
}

#[tokio::test]
async fn schedule_validation_rejects_bad_input() {
    let pool = setup().await;
    let err = schedule::create(
        &pool,
        NewScheduledJob {
            name: "壞排程".to_string(),
            target_type: "device".to_string(),
            target_id: "x".to_string(),
            schedule_kind: "interval".to_string(),
            interval_minutes: Some(0),
            daily_time: None,
        },
    )
    .await
    .unwrap_err();
    assert_eq!(err.code(), "VALIDATION");
}

#[tokio::test]
async fn toggle_and_delete_schedule() {
    let pool = setup().await;
    let job = schedule::create(
        &pool,
        NewScheduledJob {
            name: "可切換".to_string(),
            target_type: "device".to_string(),
            target_id: "x".to_string(),
            schedule_kind: "daily".to_string(),
            interval_minutes: None,
            daily_time: Some("08:30".to_string()),
        },
    )
    .await
    .unwrap();
    assert!(job.enabled);

    let off = schedule::set_enabled(&pool, &job.id, false).await.unwrap();
    assert!(!off.enabled);

    schedule::delete(&pool, &job.id).await.unwrap();
    assert!(schedule::get(&pool, &job.id).await.is_err());
}

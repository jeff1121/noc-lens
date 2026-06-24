//! 排程引擎：定期對設備／群組執行查詢並記錄結果。
//!
//! - `run_job_once`：單次執行（核心邏輯，可用 mock 執行器測試）。
//! - `SchedulerService`：以 tokio-cron-scheduler 註冊定時觸發。

use std::sync::Arc;

use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::db::{device, schedule, settings};
use crate::error::AppError;
use crate::models::{JobRun, ScheduledJob};
use crate::ssh::client::RusshExecutor;
use crate::ssh::executor::SshExecutor;
use crate::ssh::run_query;
use crate::SqlitePool;

/// 解析排程目標為設備 id 清單。
pub async fn resolve_targets(
    pool: &SqlitePool,
    job: &ScheduledJob,
) -> Result<Vec<String>, AppError> {
    match job.target_type.as_str() {
        "device" => Ok(vec![job.target_id.clone()]),
        "group" => Ok(device::list(pool, Some(&job.target_id))
            .await?
            .into_iter()
            .map(|d| d.id)
            .collect()),
        other => Err(AppError::Validation(format!("未知的 target_type：{other}"))),
    }
}

/// 單次執行排程：查詢目標設備、寫入快照、記錄 JobRun。
pub async fn run_job_once<E>(
    pool: &SqlitePool,
    executor: &E,
    job_id: &str,
) -> Result<JobRun, AppError>
where
    E: SshExecutor + Sync,
{
    let job = schedule::get(pool, job_id).await?;
    let device_ids = resolve_targets(pool, &job).await?;
    let run_id = schedule::run_start(pool, job_id, device_ids.len() as i64).await?;

    let conc = settings::get(pool, "ssh.max_concurrency")
        .await?
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10);

    let results = run_query(pool, executor, &device_ids, conc).await;
    let success = results.iter().filter(|r| r.status != "failed").count() as i64;
    let failure = results.len() as i64 - success;

    schedule::run_finish(pool, &run_id, success, failure).await?;
    schedule::run_get(pool, &run_id).await
}

/// 將排程轉為 6 欄位 cron 表達式（含秒）。
fn cron_for(job: &ScheduledJob) -> Option<String> {
    match job.schedule_kind.as_str() {
        "interval" => {
            let n = job.interval_minutes.unwrap_or(0);
            if n <= 0 {
                None
            } else {
                Some(format!("0 */{n} * * * *"))
            }
        }
        "daily" => {
            let t = job.daily_time.as_deref()?;
            let (h, m) = t.split_once(':')?;
            let hour: u32 = h.trim().parse().ok()?;
            let minute: u32 = m.trim().parse().ok()?;
            Some(format!("0 {minute} {hour} * * *"))
        }
        _ => None,
    }
}

/// 排程服務：持有 JobScheduler 並可依資料庫重新載入排程。
pub struct SchedulerService {
    sched: JobScheduler,
    pool: SqlitePool,
    job_uuids: Arc<Mutex<Vec<uuid::Uuid>>>,
}

impl SchedulerService {
    pub async fn new(pool: SqlitePool) -> Result<Self, AppError> {
        let sched = JobScheduler::new()
            .await
            .map_err(|e| AppError::Db(format!("排程器初始化失敗：{e}")))?;
        Ok(Self {
            sched,
            pool,
            job_uuids: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// 啟動排程器並載入啟用中的排程。
    pub async fn start(&self) -> Result<(), AppError> {
        self.sched
            .start()
            .await
            .map_err(|e| AppError::Db(format!("排程器啟動失敗：{e}")))?;
        self.reload().await
    }

    /// 清除並依資料庫重新註冊所有啟用中的排程。
    pub async fn reload(&self) -> Result<(), AppError> {
        {
            let mut guard = self.job_uuids.lock().await;
            for u in guard.drain(..) {
                let _ = self.sched.remove(&u).await;
            }
        }

        let jobs = schedule::list(&self.pool).await?;
        for job in jobs.into_iter().filter(|j| j.enabled) {
            let Some(cron) = cron_for(&job) else { continue };
            let pool = self.pool.clone();
            let job_id = job.id.clone();
            let j = Job::new_async(cron.as_str(), move |_uuid, _l| {
                let pool = pool.clone();
                let job_id = job_id.clone();
                Box::pin(async move {
                    let _ = run_job_once(&pool, &RusshExecutor, &job_id).await;
                })
            })
            .map_err(|e| AppError::Db(format!("建立排程失敗：{e}")))?;
            let uid = self
                .sched
                .add(j)
                .await
                .map_err(|e| AppError::Db(format!("註冊排程失敗：{e}")))?;
            self.job_uuids.lock().await.push(uid);
        }
        Ok(())
    }
}

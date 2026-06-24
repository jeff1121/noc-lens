// 防止 Windows release 版本彈出主控台視窗
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use noc_lens_backend::db;
use noc_lens_backend::scheduler::SchedulerService;
use noc_lens_backend::SqlitePool;
use tauri::Manager;

/// 應用程式共享狀態。
pub struct AppState {
    pub pool: SqlitePool,
    pub scheduler: SchedulerService,
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 於應用資料目錄建立本地資料庫
            let dir = app
                .path()
                .app_data_dir()
                .expect("無法取得應用資料目錄");
            std::fs::create_dir_all(&dir).ok();
            let db_path = dir.join("noc-lens.db");

            let pool: SqlitePool = tauri::async_runtime::block_on(async {
                db::init_pool(db_path.to_str().expect("資料庫路徑無效")).await
            })
            .expect("初始化資料庫失敗");

            // 建立並啟動排程服務
            let scheduler = tauri::async_runtime::block_on(async {
                let svc = SchedulerService::new(pool.clone()).await?;
                svc.start().await?;
                Ok::<_, noc_lens_backend::AppError>(svc)
            })
            .expect("初始化排程服務失敗");

            app.manage(AppState { pool, scheduler });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::device_list,
            commands::device_create,
            commands::device_update,
            commands::device_delete,
            commands::device_import,
            commands::group_list,
            commands::group_create,
            commands::group_delete,
            commands::group_assign,
            commands::groups_for_device,
            commands::query_devices,
            commands::snapshot_list,
            commands::schedule_list,
            commands::schedule_create,
            commands::schedule_delete,
            commands::schedule_toggle,
            commands::schedule_run_now,
            commands::job_run_list,
            commands::report_generate,
            commands::report_list,
            commands::settings_get,
            commands::settings_set,
            commands::settings_set_ai_key,
        ])
        .run(tauri::generate_context!())
        .expect("啟動 noc-lens 失敗");
}

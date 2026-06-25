//! 真實 SSH 執行器（russh）：以帳號／密碼認證連線並執行唯讀指令。

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use russh::client::{self, Handler};
use russh::keys::key::PublicKey;
use russh::{ChannelMsg, Disconnect};
use tokio::time::timeout;

use crate::error::AppError;
use crate::ssh::executor::{CmdOutput, SshExecutor, SshTarget};

/// 以 russh 連線真實設備的執行器。
pub struct RusshExecutor;

const SSH_CONNECT_TIMEOUT_SECS: u64 = 15;
const SSH_AUTH_TIMEOUT_SECS: u64 = 15;
const SSH_COMMAND_TIMEOUT_SECS: u64 = 30;
const SSH_MAX_OUTPUT_BYTES: usize = 512 * 1024;

struct ClientHandler {
    host: String,
    port: u16,
}

#[async_trait::async_trait]
impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        russh::keys::check_known_hosts(&self.host, self.port, server_public_key)
            .map_err(russh::Error::from)
    }
}

impl SshExecutor for RusshExecutor {
    fn run(
        &self,
        target: SshTarget<'_>,
        commands: &[String],
    ) -> impl Future<Output = Result<Vec<CmdOutput>, AppError>> + Send {
        let host = target.host.to_string();
        let port = target.port;
        let username = target.username.to_string();
        let password = target.password.to_string();
        let cmds: Vec<String> = commands.to_vec();
        async move { exec_session(host, port, username, password, cmds).await }
    }
}

async fn exec_session(
    host: String,
    port: u16,
    username: String,
    password: String,
    commands: Vec<String>,
) -> Result<Vec<CmdOutput>, AppError> {
    let config = Arc::new(client::Config::default());
    let handler = ClientHandler {
        host: host.clone(),
        port,
    };
    let mut handle = timeout(
        Duration::from_secs(SSH_CONNECT_TIMEOUT_SECS),
        client::connect(config, (host.as_str(), port), handler),
    )
    .await
    .map_err(|_| AppError::Validation("SSH 連線逾時".to_string()))?
    .map_err(|e| AppError::Validation(format!("SSH 連線失敗：{e}")))?;

    let authenticated = timeout(
        Duration::from_secs(SSH_AUTH_TIMEOUT_SECS),
        handle.authenticate_password(&username, &password),
    )
    .await
    .map_err(|_| AppError::Validation("SSH 認證逾時".to_string()))?
    .map_err(|e| AppError::Validation(format!("SSH 認證錯誤：{e}")))?;
    if !authenticated {
        return Err(AppError::Validation(
            "SSH 認證失敗（帳號或密碼錯誤）".to_string(),
        ));
    }

    let mut outputs = Vec::with_capacity(commands.len());
    for (idx, cmd) in commands.iter().enumerate() {
        let timed_out = match timeout(
            Duration::from_secs(SSH_COMMAND_TIMEOUT_SECS),
            exec_one(&mut handle, cmd),
        )
        .await
        {
            Ok(Ok(text)) => {
                outputs.push(CmdOutput { ok: true, text });
                false
            }
            Ok(Err(e)) => {
                outputs.push(CmdOutput {
                    ok: false,
                    text: e.to_string(),
                });
                false
            }
            Err(_) => {
                outputs.push(CmdOutput {
                    ok: false,
                    text: "SSH 指令逾時".to_string(),
                });
                true
            }
        };
        if timed_out {
            for _ in (idx + 1)..commands.len() {
                outputs.push(CmdOutput {
                    ok: false,
                    text: "SSH 指令未執行：前一個指令逾時".to_string(),
                });
            }
            break;
        }
    }

    let _ = handle
        .disconnect(Disconnect::ByApplication, "", "English")
        .await;
    Ok(outputs)
}

async fn exec_one(
    handle: &mut client::Handle<ClientHandler>,
    command: &str,
) -> Result<String, AppError> {
    let mut channel = handle
        .channel_open_session()
        .await
        .map_err(|e| AppError::Validation(format!("SSH 開啟通道失敗：{e}")))?;
    channel
        .exec(true, command.as_bytes())
        .await
        .map_err(|e| AppError::Validation(format!("SSH 指令執行失敗：{e}")))?;

    let mut buf: Vec<u8> = Vec::new();
    while let Some(msg) = channel.wait().await {
        match msg {
            ChannelMsg::Data { ref data } => append_output(&mut buf, data)?,
            ChannelMsg::ExtendedData { ref data, .. } => append_output(&mut buf, data)?,
            ChannelMsg::Close | ChannelMsg::Eof => break,
            _ => {}
        }
    }
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn append_output(buf: &mut Vec<u8>, data: &[u8]) -> Result<(), AppError> {
    if buf.len().saturating_add(data.len()) > SSH_MAX_OUTPUT_BYTES {
        return Err(AppError::Validation(format!(
            "SSH 指令輸出超過 {} KiB 上限",
            SSH_MAX_OUTPUT_BYTES / 1024
        )));
    }
    buf.extend_from_slice(data);
    Ok(())
}

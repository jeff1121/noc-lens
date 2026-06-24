//! 真實 SSH 執行器（russh）：以帳號／密碼認證連線並執行唯讀指令。

use std::future::Future;
use std::sync::Arc;

use russh::client::{self, Handler};
use russh::keys::key::PublicKey;
use russh::{ChannelMsg, Disconnect};

use crate::error::AppError;
use crate::ssh::executor::{CmdOutput, SshExecutor, SshTarget};

/// 以 russh 連線真實設備的執行器。
pub struct RusshExecutor;

struct ClientHandler;

#[async_trait::async_trait]
impl Handler for ClientHandler {
    type Error = russh::Error;

    // 設備為使用者自行管理的內部設備，這裡接受其主機金鑰。
    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
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
    let mut handle = client::connect(config, (host.as_str(), port), ClientHandler)
        .await
        .map_err(|e| AppError::Validation(format!("SSH 連線失敗：{e}")))?;

    let authenticated = handle
        .authenticate_password(&username, &password)
        .await
        .map_err(|e| AppError::Validation(format!("SSH 認證錯誤：{e}")))?;
    if !authenticated {
        return Err(AppError::Validation(
            "SSH 認證失敗（帳號或密碼錯誤）".to_string(),
        ));
    }

    let mut outputs = Vec::with_capacity(commands.len());
    for cmd in &commands {
        match exec_one(&mut handle, cmd).await {
            Ok(text) => outputs.push(CmdOutput { ok: true, text }),
            Err(e) => outputs.push(CmdOutput {
                ok: false,
                text: e.to_string(),
            }),
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
) -> Result<String, russh::Error> {
    let mut channel = handle.channel_open_session().await?;
    channel.exec(true, command.as_bytes()).await?;

    let mut buf: Vec<u8> = Vec::new();
    while let Some(msg) = channel.wait().await {
        match msg {
            ChannelMsg::Data { ref data } => buf.extend_from_slice(data),
            ChannelMsg::ExtendedData { ref data, .. } => buf.extend_from_slice(data),
            ChannelMsg::Close | ChannelMsg::Eof => break,
            _ => {}
        }
    }
    Ok(String::from_utf8_lossy(&buf).to_string())
}

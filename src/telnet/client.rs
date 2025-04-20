use crate::bot::ResponseCode;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{oneshot, Mutex as TokioMutex};
use tokio::time::timeout;
use tracing::{error, info};

#[derive(Debug)]
pub enum Command {
    Ban(String, u32, String),
    IPBan(String, u32),
    ChangePassword(String, String),
    Unban(String),
    UnIPBan(String),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Ban(user, dur, reason) => write!(f, "/lock {} {} {}", user, dur, reason),
            Command::IPBan(ip, dur) => write!(f, "/ipban a {} {}", ip, dur),
            Command::ChangePassword(u, p) => write!(f, "/chpass {} {}", u, p),
            Command::Unban(user) => write!(f, "/unlock {}", user),
            Command::UnIPBan(ip) => write!(f, "/ipban d {}", ip),
        }
    }
}

#[derive(Debug)]
pub enum ApiResult {
    Success(String),
    Error(String),
    Timeout,
}

struct SharedState {
    writer: OwnedWriteHalf,
    current_sender: Option<oneshot::Sender<ApiResult>>,
}

pub struct ApiClient {
    shared: Arc<TokioMutex<SharedState>>,
}

impl ApiClient {
    pub async fn start(server: &str, username: &str, password: &str) -> Result<Self, ResponseCode> {
        let stream = TcpStream::connect(server).await.map_err(|e| {
            error!("Failed to connect to server: {}", e);
            ResponseCode::ServerError
        })?;

        let (read_half, write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        let shared = Arc::new(TokioMutex::new(SharedState {
            writer: write_half,
            current_sender: None,
        }));

        let mut buffer = [0u8; 1024];
        loop {
            let n = reader.read(&mut buffer).await.map_err(|e| {
                error!("Failed to read from server: {}", e);
                ResponseCode::ServerError
            })?;
            if n == 0 {
                return Err(ResponseCode::ServerError);
            }
            let text = String::from_utf8_lossy(&buffer[..n]).to_lowercase();

            if text.contains("login failed.") {
                return Err(ResponseCode::ServerError);
            }
            if text.contains("hello") {
                break;
            }

            let mut shared = shared.lock().await;
            if text.contains("username:") {
                write_command(&mut shared.writer, username).await?;
            } else if text.contains("password:") {
                write_command(&mut shared.writer, password).await?;
            }
        }

        let shared_clone = shared.clone();
        tokio::spawn(async move {
            reader_loop(reader, shared_clone).await;
        });

        Ok(ApiClient { shared })
    }

    pub async fn send_command(&self, command: Command) -> Result<ApiResult, ResponseCode> {
        let (tx, rx) = oneshot::channel();

        {
            let mut shared_lock = self.shared.lock().await;
            // 檢查是否已有待處理的命令
            if shared_lock.current_sender.is_some() {
                return Ok(ApiResult::Error("已有待處理的命令".to_string()));
            }
            shared_lock.current_sender = Some(tx);

            // 在同一個鎖內發送命令
            write_command(&mut shared_lock.writer, command.to_string().as_str()).await?;
            info!(">> {}", command);
        } // 這裡鎖會被釋放，允許 reader_loop 處理回應

        match timeout(Duration::from_secs(2), rx).await {
            Ok(result) => match result {
                Ok(api_result) => Ok(api_result),
                Err(_) => Ok(ApiResult::Error("接收響應失敗，發送通道被關閉".to_string())),
            },
            Err(_) => {
                let mut shared_lock = self.shared.lock().await;
                shared_lock.current_sender = None;
                Ok(ApiResult::Timeout)
            }
        }
    }
}

async fn write_command(writer: &mut OwnedWriteHalf, cmd: &str) -> Result<(), ResponseCode> {
    let full_cmd = format!("{}\r\n", cmd);
    writer.write_all(full_cmd.as_bytes()).await.map_err(|e| {
        error!("Failed to write to server: {}", e);
        ResponseCode::ServerError
    })?;
    writer.flush().await.map_err(|e| {
        error!("Failed to flush to server: {}", e);
        ResponseCode::ServerError
    })?;
    Ok(())
}

async fn reader_loop(mut reader: BufReader<OwnedReadHalf>, shared: Arc<TokioMutex<SharedState>>) {
    let mut buf = [0; 1024];
    loop {
        match reader.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                let resp = String::from_utf8_lossy(&buf[..n]).to_string();

                let sender = {
                    let mut shared_lock = shared.lock().await;
                    shared_lock.current_sender.take()
                };

                if let Some(sender) = sender {
                    let api_result = if resp.is_empty() {
                        ApiResult::Error("Command Finished".to_owned())
                    } else {
                        ApiResult::Success("Command Finished".to_owned())
                    };

                    if sender.send(api_result).is_err() {
                        error!("Failed to send response through channel");
                    }
                } else {
                    info!("Received response but no active sender found");
                }
            }
        }
    }
}

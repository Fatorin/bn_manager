use crate::bot::ResponseCode;
use rand::RngExt;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{oneshot, watch, Mutex as TokioMutex, Notify};
use tokio::time::timeout;
use tracing::{error, info, warn};

const RECONNECT_INITIAL_DELAY: Duration = Duration::from_secs(1);
const RECONNECT_MAX_DELAY: Duration = Duration::from_secs(60);
const RECONNECT_COMMAND_WAIT: Duration = Duration::from_secs(15);

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
    Success,
    Error,
    Timeout,
}

enum ConnectionStatus {
    Connected { writer: OwnedWriteHalf },
    Disconnected,
    Reconnecting,
}

struct SharedState {
    status: ConnectionStatus,
    current_sender: Option<oneshot::Sender<ApiResult>>,
}

pub struct ApiClient {
    shared: Arc<TokioMutex<SharedState>>,
    connected: watch::Receiver<bool>,
}

async fn connect_and_login(
    server: &str,
    username: &str,
    password: &str,
) -> Result<(BufReader<OwnedReadHalf>, OwnedWriteHalf), ResponseCode> {
    let stream = TcpStream::connect(server).await.map_err(|e| {
        error!("Failed to connect to server: {}", e);
        ResponseCode::ServerError
    })?;

    let (read_half, write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let mut writer = write_half;

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

        if text.contains("username:") {
            write_command(&mut writer, username).await?;
        } else if text.contains("password:") {
            write_command(&mut writer, password).await?;
        }
    }

    Ok((reader, writer))
}

impl ApiClient {
    pub async fn start(server: &str, username: &str, password: &str) -> Result<Self, ResponseCode> {
        let (reader, writer) = connect_and_login(server, username, password).await?;

        let (connected_tx, connected_rx) = watch::channel(true);
        let reconnect_notify = Arc::new(Notify::new());

        let shared = Arc::new(TokioMutex::new(SharedState {
            status: ConnectionStatus::Connected { writer },
            current_sender: None,
        }));

        // Spawn reader loop
        let shared_clone = shared.clone();
        let reconnect_notify_clone = reconnect_notify.clone();
        let connected_tx_clone = connected_tx.clone();
        tokio::spawn(async move {
            reader_loop(reader, shared_clone, reconnect_notify_clone, connected_tx_clone).await;
        });

        // Spawn reconnection background task
        let shared_clone = shared.clone();
        let server = server.to_string();
        let username = username.to_string();
        let password = password.to_string();
        tokio::spawn(async move {
            reconnect_task(
                shared_clone,
                reconnect_notify,
                connected_tx,
                server,
                username,
                password,
            )
            .await;
        });

        Ok(ApiClient {
            shared,
            connected: connected_rx,
        })
    }

    pub async fn send_command(&self, command: Command) -> Result<ApiResult, ResponseCode> {
        // Check connection status, wait for reconnection if needed
        {
            let shared = self.shared.lock().await;
            if !matches!(shared.status, ConnectionStatus::Connected { .. }) {
                drop(shared);
                let mut rx = self.connected.clone();
                let wait_result = timeout(RECONNECT_COMMAND_WAIT, rx.wait_for(|&c| c)).await;
                match wait_result {
                    Ok(Ok(_)) => {}
                    _ => {
                        error!("等待重新連線超時");
                        return Ok(ApiResult::Error);
                    }
                }
            }
        }

        let (tx, rx) = oneshot::channel();

        {
            let mut shared_lock = self.shared.lock().await;

            // Re-check connection after acquiring lock
            if !matches!(shared_lock.status, ConnectionStatus::Connected { .. }) {
                error!("連線在等待鎖期間斷開");
                return Ok(ApiResult::Error);
            }

            if shared_lock.current_sender.is_some() {
                error!("已有待處理的命令");
                return Ok(ApiResult::Error);
            }
            shared_lock.current_sender = Some(tx);

            let cmd_str = command.to_string();
            let write_result = match &mut shared_lock.status {
                ConnectionStatus::Connected { writer } => {
                    write_command_raw(writer, cmd_str.as_str()).await
                }
                _ => unreachable!(),
            };

            if let Err(e) = write_result {
                shared_lock.current_sender = None;
                shared_lock.status = ConnectionStatus::Disconnected;
                error!("寫入命令失敗，連線可能已斷開: {}", e);
                return Ok(ApiResult::Error);
            }
            info!(">> {}", command);
        }

        match timeout(Duration::from_secs(2), rx).await {
            Ok(result) => match result {
                Ok(api_result) => Ok(api_result),
                Err(_) => {
                    error!("接收響應失敗，發送通道被關閉");
                    Ok(ApiResult::Error)
                }
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
    write_command_raw(writer, cmd).await.map_err(|e| {
        error!("Failed to write to server: {}", e);
        ResponseCode::ServerError
    })
}

async fn write_command_raw(writer: &mut OwnedWriteHalf, cmd: &str) -> Result<(), std::io::Error> {
    let full_cmd = format!("{}\r\n", cmd);
    writer.write_all(full_cmd.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

async fn reader_loop(
    mut reader: BufReader<OwnedReadHalf>,
    shared: Arc<TokioMutex<SharedState>>,
    reconnect_notify: Arc<Notify>,
    connected_tx: watch::Sender<bool>,
) {
    let mut buf = [0; 1024];
    loop {
        match reader.read(&mut buf).await {
            Ok(0) | Err(_) => {
                warn!("與伺服器的連線已斷開");
                let mut shared_lock = shared.lock().await;
                shared_lock.status = ConnectionStatus::Disconnected;
                // Notify any waiting send_command
                if let Some(sender) = shared_lock.current_sender.take() {
                    let _ = sender.send(ApiResult::Error);
                }
                drop(shared_lock);
                let _ = connected_tx.send(false);
                reconnect_notify.notify_one();
                return;
            }
            Ok(n) => {
                let resp = String::from_utf8_lossy(&buf[..n]).to_string();

                let sender = {
                    let mut shared_lock = shared.lock().await;
                    shared_lock.current_sender.take()
                };

                if let Some(sender) = sender {
                    let api_result = if resp.is_empty() {
                        error!("Command Finished with empty response");
                        ApiResult::Error
                    } else {
                        ApiResult::Success
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

async fn reconnect_task(
    shared: Arc<TokioMutex<SharedState>>,
    reconnect_notify: Arc<Notify>,
    connected_tx: watch::Sender<bool>,
    server: String,
    username: String,
    password: String,
) {
    loop {
        reconnect_notify.notified().await;

        // Mark as reconnecting
        {
            let mut shared_lock = shared.lock().await;
            if matches!(shared_lock.status, ConnectionStatus::Connected { .. }) {
                continue; // Already reconnected (race condition)
            }
            shared_lock.status = ConnectionStatus::Reconnecting;
        }

        let mut delay = RECONNECT_INITIAL_DELAY;

        loop {
            info!("嘗試重新連線，延遲 {:?}", delay);
            tokio::time::sleep(delay).await;

            match connect_and_login(&server, &username, &password).await {
                Ok((reader, writer)) => {
                    info!("重新連線成功");

                    let mut shared_lock = shared.lock().await;
                    shared_lock.status = ConnectionStatus::Connected { writer };
                    drop(shared_lock);

                    let _ = connected_tx.send(true);

                    // Spawn new reader loop
                    let shared_clone = shared.clone();
                    let reconnect_notify_clone = reconnect_notify.clone();
                    let connected_tx_clone = connected_tx.clone();
                    tokio::spawn(async move {
                        reader_loop(reader, shared_clone, reconnect_notify_clone, connected_tx_clone).await;
                    });

                    break;
                }
                Err(e) => {
                    warn!("重新連線失敗: {:?}", e);
                    let jitter = Duration::from_millis(rand::rng().random_range(0..500));
                    delay = (delay * 2).min(RECONNECT_MAX_DELAY) + jitter;
                }
            }
        }
    }
}

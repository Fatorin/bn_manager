use sqlx::mysql::MySqlPool;
use tokio::sync::OnceCell;

use crate::settings::CONFIG;

static MYSQL_POOL: OnceCell<MySqlPool> = OnceCell::const_new();

pub async fn init_mysql_pool() {
    let pool = MySqlPool::connect(&CONFIG.mysql_connection_string())
        .await
        .expect("Failed to connect to MySQL");
    MYSQL_POOL
        .set(pool)
        .expect("MySQL pool already initialized");
    tracing::info!("MySQL connection pool initialized");
}

pub fn mysql_pool() -> &'static MySqlPool {
    MYSQL_POOL.get().expect("MySQL pool not initialized")
}

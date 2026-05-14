use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

pub async fn connect_db(db_url: &str) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(db_url.to_owned());

    opt.max_connections(10)
        .min_connections(2)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(30));

    Database::connect(opt)
        .await
        .expect("Não foi possível conectar ao Postgres")
}

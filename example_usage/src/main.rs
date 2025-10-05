pub mod advanced_queries;
pub mod blog_system;
pub mod getting_started;
pub mod netabase_basic_usage;

#[tokio::main]
pub async fn main() {
    let _ = advanced_queries::advanced_queries().await;
    let _ = blog_system::blog_system().await;
    let _ = getting_started::getting_started().await;
    let _ = netabase_basic_usage::netabase_basic_usage().await;
}

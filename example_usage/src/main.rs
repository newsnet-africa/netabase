pub mod advanced_queries;
pub mod blog_system;
pub mod getting_started;
pub mod netabase_basic_usage;

use tokio::main;

#[tokio::main]
pub async fn main() {
    advanced_queries::advanced_queries().await;
    blog_system::blog_system().await;
    getting_started::getting_started().await;
    netabase_basic_usage::netabase_basic_usage().await;
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    mictylish::repl::run().await
}

mod config;
mod export;
mod routes;
mod ssh;

use clap::Parser;

#[derive(Parser)]
#[command(name = "sshkeyman", about = "SSH Key Manager — web-based")]
struct Args {
    /// Bind address (e.g. 0.0.0.0:8080)
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Listen port
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Full bind address (overrides --host and --port)
    #[arg(short, long)]
    bind: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let addr = args
        .bind
        .unwrap_or_else(|| format!("{}:{}", args.host, args.port));

    let app = routes::router();

    println!("SSHKeyman running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

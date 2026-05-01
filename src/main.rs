mod config;
mod export;
mod i18n;
mod routes;
mod ssh;

use clap::Parser;

#[derive(Parser)]
#[command(name = "sshkeyman", about = "Web-based SSH key & config manager")]
struct Args {
    /// Listen address (e.g. 0.0.0.0)
    #[arg(short = 'a', long, default_value = "127.0.0.1")]
    host: String,

    /// Listen port
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Full bind address, overrides -a and -p (e.g. 0.0.0.0:9000)
    #[arg(short, long)]
    bind: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    i18n::init();

    let addr = args
        .bind
        .unwrap_or_else(|| format!("{}:{}", args.host, args.port));

    let app = routes::router();

    println!("SSHKeyman running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

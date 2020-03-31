use knowledge_server_base::server;
use knowledge_server_scanner::scanner;
use std::env;
use std::process::{Command, Stdio};

#[async_std::main]
async fn main() -> std::io::Result<()> {
  let args: Vec<String> = env::args().collect();
  let mode: &str = args.get(1).map(|s| s.as_str()).unwrap_or("");
  match mode {
    "--scanner" => {
      scanner::activate().await?;
    }
    "--server" => {
      server::activate().await?;
    }
    _ => {
      println!("Starting service");

      let scanner = Command::new(&args[0])
        .arg("--scanner")
        .stdin(Stdio::piped())
        .spawn()?;
      println!("Scanner started {:}", scanner.id());

      server::activate().await?;
      println!("Server started");
    }
  }
  Ok(())
}

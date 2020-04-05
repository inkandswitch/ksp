#![feature(proc_macro_hygiene)]

use commander_rust::{command, entry, option, run, Cli};
use dirs;
use knowledge_server_base::server;
use knowledge_server_scanner::scanner;
use std::env;
use std::io::Result;
use std::process::{Command, Stdio};
use syntax::wait;

#[option(-p, --port <port>, "Port to be used by the knowledge-server (Default 8080)")]
#[option(-o, --out <path>, "Path where service log is written")]
#[command(daemon, "Run server in the background")]
fn daemon(cli: Cli) -> Result<()> {
  let path = if cli.has("out") {
    let mut path = env::current_dir()?;
    path.push(cli.get_or("out", format!("")));
    path
  } else {
    let mut path = dirs::home_dir().expect("Unable to locate user home directory");
    path.push(".knowledge-service");
    std::fs::create_dir_all(&path)?;
    path.push("service.log");
    path
  };

  let port = cli.get_or("port", format!("8080"));
  let log = std::fs::File::create(&path)?;
  let args: Vec<String> = env::args().collect();

  Command::new(&args[0])
    .arg("serve")
    .arg("--port")
    .arg(port)
    // So that if anything is read from standard input, it will crash does to
    // EOF immediately.
    .stdin(Stdio::null())
    // If there is a write to stderror crash the process.
    .stderr(Stdio::null())
    .stdout(log)
    .spawn()?;

  println!("knowledge-server is running in the background");
  Ok(())
}

#[wait]
#[option(-p, --port <port>, "Port to be used by the knowledge-server (Default 8080)")]
#[command(serve, "Run server in the foreground")]
async fn serve(cli: Cli) -> Result<()> {
  let port = cli.get_or("port", format!("8080"));
  let address = format!("127.0.0.1:{:}", port);
  server::activate(&address).await?;
  println!("Starting server http://{}", address);
  Ok(())
}

#[wait]
#[command(scan <path>, "Scans directory and submits all findings to knowledge-server")]
async fn scan(path: String) -> Result<()> {
  // Resolve the given path.
  let mut base = env::current_dir()?;
  base.push(path);
  println!("Scanning resources {:?}", base);

  let n = scanner::scan(&base).await?;
  println!("Ingested {:} files", n);

  Ok(())
}

#[wait]
#[entry]
async fn main() -> Result<()> {
  let app = run!();
  if let Some(out) = app.out {
    out
  } else {
    // If command was not matched, fallback to just starting server
    println!("{:}", app);
    Ok(())
  }
}

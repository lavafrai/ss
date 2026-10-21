mod menu;
mod servers;
mod ssh;

use servers::Server;

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() {
    let code = match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("ss: {error}");
            1
        }
    };
    std::process::exit(code);
}

fn run() -> AppResult<i32> {
    let servers = servers::load_servers()?;
    if servers.is_empty() {
        eprintln!("ss: no hosts found in SSH config");
        return Ok(0);
    }

    let Some(alias) = choose_server(servers)? else {
        return Ok(0);
    };

    Ok(ssh::connect(&alias)?)
}

fn choose_server(servers: Vec<Server>) -> AppResult<Option<String>> {
    let runtime = tokio::runtime::Runtime::new()?;
    let selected = runtime.block_on(menu::select_server(servers));
    // Finish all menu tasks before SSH receives the terminal.
    drop(runtime);
    selected
}

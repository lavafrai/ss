use std::collections::BTreeSet;

use ssh2_config::{ParseRule, SshConfig, SshParserResult};

const DEFAULT_SSH_PORT: u16 = 22;

pub struct Server {
    pub alias: String,
    pub destination: String,
}

pub fn load_servers() -> SshParserResult<Vec<Server>> {
    let config = SshConfig::parse_default_file(ParseRule::ALLOW_UNKNOWN_FIELDS)?;
    let local_user = local_username();

    let servers = configured_aliases(&config)
        .into_iter()
        .map(|alias| resolve_server(&config, alias, local_user.as_deref()))
        .collect();

    Ok(servers)
}

fn configured_aliases(config: &SshConfig) -> BTreeSet<&str> {
    config
        .get_hosts()
        .iter()
        .flat_map(|host| &host.pattern)
        .filter(|clause| !clause.negated && !clause.pattern.contains(['*', '?']))
        .map(|clause| clause.pattern.as_str())
        .collect()
}

fn local_username() -> Option<String> {
    let variable = if cfg!(windows) { "USERNAME" } else { "USER" };
    std::env::var(variable).ok()
}

fn resolve_server(config: &SshConfig, alias: &str, local_user: Option<&str>) -> Server {
    let params = config.query(alias);
    let hostname = params.host_name.as_deref().unwrap_or(alias);
    let endpoint = format_endpoint(hostname, params.port);
    let user = params.user.as_deref().or(local_user);
    let destination = match user {
        Some(user) => format!("{user}@{endpoint}"),
        None => endpoint,
    };

    Server {
        alias: alias.to_owned(),
        destination,
    }
}

fn format_endpoint(hostname: &str, port: Option<u16>) -> String {
    match port.filter(|&port| port != DEFAULT_SSH_PORT) {
        Some(port) if hostname.contains(':') && !hostname.starts_with('[') => {
            format!("[{hostname}]:{port}")
        }
        Some(port) => format!("{hostname}:{port}"),
        None => hostname.to_owned(),
    }
}

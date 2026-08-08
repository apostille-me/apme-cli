use flags2env::BundledFlags2Env;
use futures_util::StreamExt;
use serde::Deserialize;
use std::collections::HashMap;
use tokio_tungstenite::connect_async;

const HELP: &str = "apme-cli 0.1.0\n\nUsage: apme-cli [--api-url URL] <command>\n\nCommands:\n  health  Check the Apostille Me API\n  list    List cases\n  watch   Stream case events\n\nOptions:\n  -h, --help       Print this help\n  -V, --version    Print the CLI version\n\nConfiguration flags are defined in .cli-flags.toml.\n";
const VERSION: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"), "\n");

fn informational_output<I, S>(arguments: I) -> Option<&'static str>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    arguments
        .into_iter()
        .map(|argument| argument.as_ref().to_owned())
        .find_map(|argument| match argument.as_str() {
            "-h" | "--help" => Some(HELP),
            "-V" | "--version" => Some(VERSION),
            _ => None,
        })
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(rename = "APME_API_URL")]
    api_url: String,
    #[serde(rename = "APME_TIMEOUT_SECONDS")]
    timeout_seconds: u64,
    #[serde(rename = "APME_OUTPUT")]
    output: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(output) = informational_output(std::env::args().skip(1)) {
        print!("{output}");
        return Ok(());
    }

    let parser = BundledFlags2Env::new();
    parser.audit_config(Some(".cli-flags.toml"))?;
    let argv = std::env::args().collect::<Vec<_>>();
    let parsed = parser.parse_structured(&argv, Some(".cli-flags.toml"))?;
    if !parsed.unknown_options.is_empty() || !parsed.errors.is_empty() {
        return Err(format!(
            "invalid arguments: unknown={:?} errors={:?}",
            parsed.unknown_options, parsed.errors
        )
        .into());
    }
    let mut values: HashMap<String, String> = std::env::vars().collect();
    values.extend(parsed.provided_flags);
    let config: Config = parser.coerce(&values, Some(".cli-flags.toml"))?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_seconds))
        .build()?;
    match parsed.command.as_str() {
        "health" => {
            print_response(
                client
                    .get(format!("{}/healthz", config.api_url.trim_end_matches('/')))
                    .send()
                    .await?,
                &config.output,
            )
            .await?
        }
        "list" => {
            print_response(
                client
                    .get(format!(
                        "{}/api/v1/cases",
                        config.api_url.trim_end_matches('/')
                    ))
                    .send()
                    .await?,
                &config.output,
            )
            .await?
        }
        "watch" => watch(&config.api_url).await?,
        _ => {
            eprintln!("usage: apme-cli [--api-url URL] <health|list|watch>");
            std::process::exit(2);
        }
    }
    Ok(())
}

async fn print_response(
    response: reqwest::Response,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(format!("HTTP {status}: {text}").into());
    }
    if output == "json" {
        let value: serde_json::Value = serde_json::from_str(&text)?;
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        println!("{text}");
    }
    Ok(())
}

async fn watch(api_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ws_url = api_url
        .replacen("http://", "ws://", 1)
        .replacen("https://", "wss://", 1);
    let (socket, _) = connect_async(format!("{}/ws", ws_url.trim_end_matches('/'))).await?;
    let (_, mut incoming) = socket.split();
    while let Some(message) = incoming.next().await {
        println!("{}", message?.into_text()?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{informational_output, HELP, VERSION};

    #[test]
    fn help_and_version_are_available_without_network_or_configuration() {
        assert_eq!(informational_output(["--help"]), Some(HELP));
        assert_eq!(informational_output(["-h"]), Some(HELP));
        assert_eq!(informational_output(["--version"]), Some(VERSION));
        assert_eq!(informational_output(["health"]), None);
    }
}

use clap::Parser;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use base64;
use std::collections::HashMap;
use anyhow::{Context, Result};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[derive(Debug, Serialize, Deserialize)]
struct Issue {
    key: String,
    fields: Fields,
}

#[derive(Debug, Serialize, Deserialize)]
struct Fields {
    summary: String,
    parent: Option<ParentIssue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParentIssue {
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraResponse {
    issues: Vec<Issue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    jira_domain: String,
    jira_email: String,
    jira_api_token: String,
    board_id: String,
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .map(|path| path.join("sprint-tasks"))
        .context("Could not determine config directory")?;
    Ok(config_dir.join("config.json"))
}

fn get_or_create_config() -> Result<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        fs::create_dir_all(config_path.parent().unwrap()).context("Failed to create config directory")?;
        
        let mut config = HashMap::new();
        config.insert("jira_domain", prompt("Enter Jira domain (e.g., your-domain.atlassian.net): ")?);
        config.insert("jira_email", prompt("Enter Jira email: ")?);
        config.insert("jira_api_token", prompt("Enter Jira API token: ")?);
        config.insert("board_id", prompt("Enter Board ID: ")?);

        let config_str = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, config_str).context("Failed to write config file")?;
    }

    let config_str = fs::read_to_string(&config_path).context("Failed to read config file")?;
    let config: Config = serde_json::from_str(&config_str).context("Failed to parse config file")?;

    Ok(config)
}

fn prompt(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _args = Args::parse();
    let config = get_or_create_config()?;

    let auth_string = format!("{}:{}", config.jira_email, config.jira_api_token);
    let encoded_auth = base64::encode(&auth_string);
    let auth_header = format!("Basic {}", encoded_auth);

    let client = reqwest::Client::new();

    // Get the active sprint for the board
    let sprint_url = format!("https://{}/rest/agile/1.0/board/{}/sprint?state=active", config.jira_domain, config.board_id);
    let sprint_response = client.get(&sprint_url)
        .header(AUTHORIZATION, &auth_header)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if sprint_response.status().is_success() {
        let sprint_body = sprint_response.text().await?;
        let sprint_json: serde_json::Value = serde_json::from_str(&sprint_body)?;
        let sprint_id = sprint_json["values"][0]["id"].as_u64().unwrap();

        // Get issues for the active sprint
        let issues_url = format!("https://{}/rest/agile/1.0/sprint/{}/issue", config.jira_domain, sprint_id);
        let issues_response = client.get(&issues_url)
            .header(AUTHORIZATION, &auth_header)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;

        if issues_response.status().is_success() {
            let issues_body = issues_response.text().await?;
            let response: JiraResponse = serde_json::from_str(&issues_body)?;

            for issue in response.issues {
                let parent_key = issue.fields.parent.as_ref().map_or(String::new(), |parent| format!("{}:", parent.key));
                println!("{}{}\t{}", parent_key, issue.key, issue.fields.summary);
            }
        } else {
            eprintln!("Error: Failed to fetch issues. Status: {}", issues_response.status());
            std::process::exit(1);
        }
    } else {
        eprintln!("Error: Failed to fetch sprint. Status: {}", sprint_response.status());
        std::process::exit(1);
    }

    Ok(())
}

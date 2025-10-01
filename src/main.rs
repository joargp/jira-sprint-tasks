use clap::Parser;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use base64::{Engine as _, engine::general_purpose};
use anyhow::{Context, Result};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    List,
    Create {
        #[arg(short, long, help = "Task summary (required for non-interactive mode)")]
        summary: Option<String>,
        #[arg(short, long, help = "Task description (optional)")]
        description: Option<String>,
    },
}

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
    fields: ParentFields,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParentFields {
    summary: String,
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
    project_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateIssueRequest {
    fields: CreateIssueFields,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateIssueFields {
    project: ProjectRef,
    summary: String,
    description: String,
    issuetype: IssueTypeRef,
    #[serde(rename = "customfield_10020")]
    sprint: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectRef {
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IssueTypeRef {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateIssueResponse {
    key: String,
    id: String,
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
        
        let config = Config {
            jira_domain: prompt("Enter Jira domain (e.g., your-domain.atlassian.net): ")?,
            jira_email: prompt("Enter Jira email: ")?,
            jira_api_token: prompt("Enter Jira API token: ")?,
            board_id: prompt("Enter Board ID: ")?,
            project_key: None,
        };

        let config_str = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, config_str).context("Failed to write config file")?;
        println!("Config file created at: {:?}", config_path);
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

fn update_config_api_token(config: &mut Config) -> Result<()> {
    let config_path = get_config_path()?;
    
    println!("Current API token appears to be invalid or expired.");
    config.jira_api_token = prompt("Enter new Jira API token: ")?;
    
    let config_str = serde_json::to_string_pretty(config)?;
    fs::write(&config_path, config_str).context("Failed to update config file")?;
    println!("Config updated with new API token.");
    
    Ok(())
}

async fn create_issue(config: &Config, client: &reqwest::Client, auth_header: &str, sprint_id: u64, summary: Option<String>, description: Option<String>) -> Result<()> {
    let summary = match summary {
        Some(s) => s,
        None => prompt("Enter task summary: ")?,
    };
    let description = match description {
        Some(d) => d,
        None => prompt("Enter task description (optional): ")?,
    };
    
    let project_key = "SKLLS";
    
    let create_request = CreateIssueRequest {
        fields: CreateIssueFields {
            project: ProjectRef {
                key: project_key.to_string(),
            },
            summary,
            description: if description.is_empty() { String::new() } else { description },
            issuetype: IssueTypeRef {
                name: "Task".to_string(),
            },
            sprint: Some(sprint_id),
        },
    };
    
    let create_url = format!("https://{}/rest/api/2/issue", config.jira_domain);
    let response = client.post(&create_url)
        .header(AUTHORIZATION, auth_header)
        .header(CONTENT_TYPE, "application/json")
        .json(&create_request)
        .send()
        .await?;
    
    if response.status().is_success() {
        let create_response: CreateIssueResponse = response.json().await?;
        println!("Successfully created task: {}", create_response.key);
    } else {
        let error_text = response.text().await?;
        eprintln!("Error creating task: {}", error_text);
        std::process::exit(1);
    }
    
    Ok(())
}

async fn list_tasks(config: &Config, client: &reqwest::Client, auth_header: &str, sprint_id: u64) -> Result<()> {
    let mut all_issues = Vec::new();

    // Fetch sprint issues
    let issues_url = format!("https://{}/rest/agile/1.0/sprint/{}/issue?maxResults=1000", config.jira_domain, sprint_id);
    let issues_response = client.get(&issues_url)
        .header(AUTHORIZATION, auth_header)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if issues_response.status().is_success() {
        let issues_body = issues_response.text().await?;
        let response: JiraResponse = serde_json::from_str(&issues_body)?;
        all_issues.extend(response.issues);
    } else {
        eprintln!("Error: Failed to fetch sprint issues. Status: {}", issues_response.status());
        std::process::exit(1);
    }

    // Fetch backlog issues (issues not in any sprint)
    let backlog_url = format!("https://{}/rest/agile/1.0/board/{}/backlog?maxResults=1000", config.jira_domain, config.board_id);
    let backlog_response = client.get(&backlog_url)
        .header(AUTHORIZATION, auth_header)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if backlog_response.status().is_success() {
        let backlog_body = backlog_response.text().await?;
        let backlog_response: JiraResponse = serde_json::from_str(&backlog_body)?;
        // Only add backlog issues that are not Done or Closed
        let open_backlog_issues: Vec<Issue> = backlog_response.issues.into_iter()
            .filter(|issue| {
                // This is a simple check - you may need to adjust based on your status names
                let summary_lower = issue.fields.summary.to_lowercase();
                !summary_lower.contains("[done]") && !summary_lower.contains("[closed]")
            })
            .collect();
        all_issues.extend(open_backlog_issues);
    } else {
        eprintln!("Warning: Failed to fetch backlog issues. Status: {}", backlog_response.status());
    }

    // Print all issues
    for issue in all_issues {
        let parent_summary = issue.fields.parent.as_ref().map_or(String::new(), |parent| format!("{}: ", parent.fields.summary));
        println!("{}{}\t{}", parent_summary, issue.key, issue.fields.summary);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut config = get_or_create_config()?;

    loop {
        let auth_string = format!("{}:{}", config.jira_email, config.jira_api_token);
        let encoded_auth = general_purpose::STANDARD.encode(&auth_string);
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

            match args.command.unwrap_or(Commands::List) {
                Commands::List => {
                    list_tasks(&config, &client, &auth_header, sprint_id).await?;
                }
                Commands::Create { summary, description } => {
                    create_issue(&config, &client, &auth_header, sprint_id, summary, description).await?;
                }
            }
            break;
        } else {
            if sprint_response.status() == 401 {
                update_config_api_token(&mut config)?;
                continue;
            } else {
                eprintln!("Error: Failed to fetch sprint. Status: {}", sprint_response.status());
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

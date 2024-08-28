use dotenv::dotenv;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::env;
use base64;

#[derive(Debug, Serialize, Deserialize)]
struct Issue {
    id: String,
    key: String,
    fields: Fields,
}

#[derive(Debug, Serialize, Deserialize)]
struct Fields {
    summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraResponse {
    issues: Vec<Issue>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let jira_domain = env::var("JIRA_DOMAIN").expect("JIRA_DOMAIN must be set");
    let jira_email = env::var("JIRA_EMAIL").expect("JIRA_EMAIL must be set");
    let jira_api_token = env::var("JIRA_API_TOKEN").expect("JIRA_API_TOKEN must be set");
    let board_id = env::var("BOARD_ID").expect("BOARD_ID must be set");

    let auth = base64::encode(format!("{}:{}", jira_email, jira_api_token));
    let client = reqwest::Client::new();

    // Get the active sprint for the board
    let sprint_url = format!("https://{}/rest/agile/1.0/board/{}/sprint?state=active", jira_domain, board_id);
    let sprint_response = client.get(&sprint_url)
        .header(AUTHORIZATION, format!("Basic {}", auth))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let sprint_id = sprint_response["values"][0]["id"].as_u64().unwrap();

    // Get issues for the active sprint
    let issues_url = format!("https://{}/rest/agile/1.0/sprint/{}/issue", jira_domain, sprint_id);
    let response = client.get(&issues_url)
        .header(AUTHORIZATION, format!("Basic {}", auth))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?
        .json::<JiraResponse>()
        .await?;

    println!("Tasks in the current sprint:");
    for issue in response.issues {
        println!("ID: {}, Key: {}, Summary: {}", issue.id, issue.key, issue.fields.summary);
    }

    Ok(())
}

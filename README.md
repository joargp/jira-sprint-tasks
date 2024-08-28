# Sprint Tasks CLI

Sprint Tasks CLI is a command-line tool for fetching and displaying tasks from the active sprint in your Jira board.

## Installation

### Prerequisites

- Rust programming language (https://www.rust-lang.org/tools/install)
- Just command runner (https://github.com/casey/just#installation)

### Steps

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/sprint-tasks.git
   cd sprint-tasks
   ```

2. Build and install the CLI:
   ```
   just install
   ```

   This will build the project, copy the binary to the appropriate location, and update your PATH.

3. Restart your terminal or run `source ~/.zshrc` (or `source ~/.bashrc` for bash) to update your PATH.

## Usage

After installation, you can run the Sprint Tasks CLI from anywhere in your terminal:

```
sprint-tasks
```

This will display the list of tasks in your active sprint, showing the issue key and summary for each task.

## Configuration

On first run, the CLI will prompt you to enter the following information:

- Jira domain (e.g., your-domain.atlassian.net)
- Jira email
- Jira API token
- Board ID

This information will be saved in a configuration file located at:
- macOS: `~/Library/Application Support/sprint-tasks/config.json`
- Linux: `~/.config/sprint-tasks/config.json`
- Windows: `C:\Users\<username>\AppData\Roaming\sprint-tasks\config.json`

To change these settings, you can either edit the configuration file directly or delete it and run the CLI again to re-enter the information.

## Development

To set up the project for development:

1. Clone the repository
2. Run `cargo build` to compile the project
3. Use `cargo run` to run the project locally

To run tests:
```
cargo test
```

## Dependencies

The project uses the following main dependencies:

- `reqwest`: HTTP client for making API requests
- `tokio`: Asynchronous runtime for Rust
- `serde`: Serialization and deserialization of JSON
- `clap`: Command-line argument parsing
- `anyhow`: Error handling
- `base64`: Encoding and decoding of Base64
- `dirs`: Finding the user's home and config directories

For a full list of dependencies, see the `Cargo.toml` file.

## License

[MIT License](LICENSE)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
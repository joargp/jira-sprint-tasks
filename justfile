# Install the Sprint Tasks CLI
install:
    @echo "Building Sprint Tasks CLI..."
    cargo build --release
    @echo "Installing Sprint Tasks CLI..."
    just {{os()}}

# Install on macOS
[macos]
macos:
    @echo "Creating ~/bin directory if it doesn't exist..."
    mkdir -p ~/bin
    @echo "Copying binary to ~/bin..."
    cp target/release/sprint-tasks ~/bin/
    @echo "Adding ~/bin to PATH in .zshrc and .bashrc..."
    echo 'export PATH="$HOME/bin:$PATH"' >> ~/.zshrc
    echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
    @echo "Installation complete!"
    @echo "Please restart your terminal or run 'source ~/.zshrc' (or 'source ~/.bashrc' for bash) to update your PATH."
    @echo "You can now run 'sprint-tasks' from anywhere in your terminal."

# Install on Windows
[windows]
windows:
    @echo "Creating %USERPROFILE%\bin directory if it doesn't exist..."
    if not exist "%USERPROFILE%\bin" mkdir "%USERPROFILE%\bin"
    @echo "Copying binary to %USERPROFILE%\bin..."
    copy target\release\sprint-tasks.exe "%USERPROFILE%\bin\"
    @echo "Adding %USERPROFILE%\bin to PATH..."
    powershell -Command "[Environment]::SetEnvironmentVariable('Path', $env:Path + ';%USERPROFILE%\bin', 'User')"
    @echo "Installation complete!"
    @echo "Please restart your command prompt to update your PATH."
    @echo "You can now run 'sprint-tasks' from anywhere in your command prompt."
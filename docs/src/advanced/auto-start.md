# Auto-Start Daemon on macOS

You can configure DevHub's daemon to start automatically when you log in to your Mac.

## Using Homebrew Service (Easiest)

If you installed via Homebrew:

```bash
# Start and enable auto-start
brew services start devhub

# Stop
brew services stop devhub

# Check status
brew services list
```

## Manual LaunchAgent Setup

### Step 1: Download the LaunchAgent

```bash
# Create LaunchAgents directory if needed
mkdir -p ~/Library/LaunchAgents

# Download the plist file
curl -fsSL https://raw.githubusercontent.com/moinsen-dev/devhub/main/install/com.moinsen.devhub.plist \
  -o ~/Library/LaunchAgents/com.moinsen.devhub.plist
```

### Step 2: Verify the Binary Path

The default plist assumes devhub is at `/usr/local/bin/devhub`. If you installed elsewhere, edit the file:

```bash
# Check where devhub is installed
which devhub

# Edit the plist if needed
nano ~/Library/LaunchAgents/com.moinsen.devhub.plist
```

Update the `ProgramArguments` section:

```xml
<key>ProgramArguments</key>
<array>
    <string>/path/to/your/devhub</string>
    <string>daemon</string>
    <string>--port</string>
    <string>9876</string>
</array>
```

### Step 3: Load the LaunchAgent

```bash
# Load (start now and on future logins)
launchctl load ~/Library/LaunchAgents/com.moinsen.devhub.plist

# Verify it's running
launchctl list | grep devhub
# com.moinsen.devhub should show with PID

# Check if daemon is responding
curl http://localhost:9876/api/projects
```

### Step 4: Managing the Service

```bash
# Stop the daemon
launchctl stop com.moinsen.devhub

# Start the daemon
launchctl start com.moinsen.devhub

# Unload (disable auto-start)
launchctl unload ~/Library/LaunchAgents/com.moinsen.devhub.plist

# Reload after config changes
launchctl unload ~/Library/LaunchAgents/com.moinsen.devhub.plist
launchctl load ~/Library/LaunchAgents/com.moinsen.devhub.plist
```

## Viewing Logs

```bash
# Stdout
tail -f /tmp/devhub.out.log

# Stderr
tail -f /tmp/devhub.err.log

# Or use Console.app and search for "devhub"
```

## Troubleshooting

### Daemon Not Starting

1. Check the error log:
   ```bash
   cat /tmp/devhub.err.log
   ```

2. Verify binary path:
   ```bash
   ls -la /usr/local/bin/devhub
   ```

3. Test manual start:
   ```bash
   /usr/local/bin/devhub daemon --port 9876
   ```

### Port Already in Use

If port 9876 is in use:

1. Find what's using it:
   ```bash
   lsof -i :9876
   ```

2. Either stop that process or edit the plist to use a different port.

### Permission Issues

```bash
# Ensure correct permissions
chmod 644 ~/Library/LaunchAgents/com.moinsen.devhub.plist
```

## Linux systemd Setup

For Linux users, create a systemd user service:

```bash
mkdir -p ~/.config/systemd/user

cat > ~/.config/systemd/user/devhub.service << 'EOF'
[Unit]
Description=DevHub Daemon
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/devhub daemon --port 9876
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
EOF

# Enable and start
systemctl --user enable devhub
systemctl --user start devhub

# Check status
systemctl --user status devhub
```

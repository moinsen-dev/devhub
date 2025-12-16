# DevHub Commander

Manage your DevHub projects directly from VS Code.

## Features

- **Project Tree View** - See all your DevHub projects in the sidebar with status indicators
- **Service Control** - Start, stop, and restart projects and individual services
- **Status Bar** - Quick overview of running projects
- **Log Viewer** - View service logs in the Output panel
- **Favorites** - Toggle favorite status for quick access
- **Quick Actions** - Open projects in browser, discover new projects

## Requirements

- [DevHub CLI](https://github.com/moinsen-dev/devhub) must be installed and available in your PATH
- Or configure the CLI path in settings

## Installation

1. Install from VS Code Marketplace (coming soon)
2. Or build from source:
   ```bash
   cd vscode-extension
   npm install
   npm run compile
   # Press F5 in VS Code to launch extension development host
   ```

## Usage

### Sidebar

The DevHub sidebar shows all registered projects:
- 🟢 Green icon = running
- ⚪ Gray icon = stopped
- ⭐ Star = favorite

Right-click on projects or services for actions.

### Commands

Open Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`) and type "DevHub":

- `DevHub: Start Project` - Start a project
- `DevHub: Stop Project` - Stop a project
- `DevHub: Restart Project` - Restart a project
- `DevHub: Show Logs` - View project logs
- `DevHub: Discover Project` - Generate devhub.toml for current workspace
- `DevHub: Open Dashboard` - Open the web dashboard

### Status Bar

The status bar shows:
- Number of running projects
- Total services running

Click to open the DevHub dashboard.

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `devhub.cliPath` | `devhub` | Path to the devhub CLI executable |
| `devhub.refreshInterval` | `5000` | Refresh interval in milliseconds |
| `devhub.showStatusBar` | `true` | Show DevHub status in the status bar |

## Development

```bash
# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Watch mode
npm run watch

# Package extension
npm run package
```

## License

MIT - See [LICENSE](../LICENSE) for details.

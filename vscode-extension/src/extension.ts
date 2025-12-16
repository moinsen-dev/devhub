import * as vscode from 'vscode';
import { ProjectTreeProvider, ProjectTreeItem } from './projectTreeProvider';
import { StatusBarManager } from './statusBar';
import { LogPanel } from './logPanel';
import * as cli from './cli';

let treeProvider: ProjectTreeProvider;
let statusBarManager: StatusBarManager;
let logPanel: LogPanel;

export async function activate(context: vscode.ExtensionContext) {
    console.log('DevHub Commander extension is activating...');

    // Check if devhub CLI is available
    const cliAvailable = await cli.checkCliAvailable();
    if (!cliAvailable) {
        vscode.window.showWarningMessage(
            'DevHub CLI not found. Please install devhub or configure the path in settings.',
            'Open Settings'
        ).then(selection => {
            if (selection === 'Open Settings') {
                vscode.commands.executeCommand('workbench.action.openSettings', 'devhub.cliPath');
            }
        });
    }

    // Initialize components
    treeProvider = new ProjectTreeProvider();
    statusBarManager = new StatusBarManager();
    logPanel = new LogPanel();

    // Register tree view
    const treeView = vscode.window.createTreeView('devhubProjects', {
        treeDataProvider: treeProvider,
        showCollapseAll: true,
    });

    // Register commands
    const commands = [
        vscode.commands.registerCommand('devhub.refreshProjects', () => {
            treeProvider.refresh();
            statusBarManager.refresh();
        }),

        vscode.commands.registerCommand('devhub.startProject', async (item: ProjectTreeItem) => {
            const projectName = item?.projectName || await selectProject();
            if (!projectName) return;

            await vscode.window.withProgress(
                {
                    location: vscode.ProgressLocation.Notification,
                    title: `Starting ${projectName}...`,
                    cancellable: false,
                },
                async () => {
                    try {
                        await cli.startProject(projectName);
                        vscode.window.showInformationMessage(`Started ${projectName}`);
                        treeProvider.refresh();
                        statusBarManager.refresh();
                    } catch (error) {
                        vscode.window.showErrorMessage(`Failed to start ${projectName}: ${error}`);
                    }
                }
            );
        }),

        vscode.commands.registerCommand('devhub.stopProject', async (item: ProjectTreeItem) => {
            const projectName = item?.projectName || await selectProject();
            if (!projectName) return;

            await vscode.window.withProgress(
                {
                    location: vscode.ProgressLocation.Notification,
                    title: `Stopping ${projectName}...`,
                    cancellable: false,
                },
                async () => {
                    try {
                        await cli.stopProject(projectName);
                        vscode.window.showInformationMessage(`Stopped ${projectName}`);
                        treeProvider.refresh();
                        statusBarManager.refresh();
                    } catch (error) {
                        vscode.window.showErrorMessage(`Failed to stop ${projectName}: ${error}`);
                    }
                }
            );
        }),

        vscode.commands.registerCommand('devhub.restartProject', async (item: ProjectTreeItem) => {
            const projectName = item?.projectName || await selectProject();
            if (!projectName) return;

            await vscode.window.withProgress(
                {
                    location: vscode.ProgressLocation.Notification,
                    title: `Restarting ${projectName}...`,
                    cancellable: false,
                },
                async () => {
                    try {
                        await cli.restartProject(projectName);
                        vscode.window.showInformationMessage(`Restarted ${projectName}`);
                        treeProvider.refresh();
                        statusBarManager.refresh();
                    } catch (error) {
                        vscode.window.showErrorMessage(`Failed to restart ${projectName}: ${error}`);
                    }
                }
            );
        }),

        vscode.commands.registerCommand('devhub.startService', async (item: ProjectTreeItem) => {
            if (!item?.projectName || !item?.serviceName) return;

            await vscode.window.withProgress(
                {
                    location: vscode.ProgressLocation.Notification,
                    title: `Starting ${item.serviceName}...`,
                    cancellable: false,
                },
                async () => {
                    try {
                        await cli.startService(item.projectName, item.serviceName!);
                        vscode.window.showInformationMessage(`Started ${item.serviceName}`);
                        treeProvider.refresh();
                        statusBarManager.refresh();
                    } catch (error) {
                        vscode.window.showErrorMessage(`Failed to start service: ${error}`);
                    }
                }
            );
        }),

        vscode.commands.registerCommand('devhub.stopService', async (item: ProjectTreeItem) => {
            if (!item?.projectName || !item?.serviceName) return;

            await vscode.window.withProgress(
                {
                    location: vscode.ProgressLocation.Notification,
                    title: `Stopping ${item.serviceName}...`,
                    cancellable: false,
                },
                async () => {
                    try {
                        await cli.stopService(item.projectName, item.serviceName!);
                        vscode.window.showInformationMessage(`Stopped ${item.serviceName}`);
                        treeProvider.refresh();
                        statusBarManager.refresh();
                    } catch (error) {
                        vscode.window.showErrorMessage(`Failed to stop service: ${error}`);
                    }
                }
            );
        }),

        vscode.commands.registerCommand('devhub.showLogs', async (item: ProjectTreeItem) => {
            const projectName = item?.projectName || await selectProject();
            if (!projectName) return;

            logPanel.showLogs(projectName, item?.serviceName);
        }),

        vscode.commands.registerCommand('devhub.openInBrowser', async (item: ProjectTreeItem) => {
            if (!item?.projectName) return;

            try {
                await cli.openInBrowser(item.projectName, item.serviceName);
            } catch (error) {
                vscode.window.showErrorMessage(`Failed to open in browser: ${error}`);
            }
        }),

        vscode.commands.registerCommand('devhub.toggleFavorite', async (item: ProjectTreeItem) => {
            if (!item?.projectName) return;

            try {
                await cli.toggleFavorite(item.projectName);
                treeProvider.refresh();
            } catch (error) {
                vscode.window.showErrorMessage(`Failed to toggle favorite: ${error}`);
            }
        }),

        vscode.commands.registerCommand('devhub.discoverProject', async () => {
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            if (!workspaceFolder) {
                vscode.window.showErrorMessage('No workspace folder open');
                return;
            }

            await vscode.window.withProgress(
                {
                    location: vscode.ProgressLocation.Notification,
                    title: 'Discovering project...',
                    cancellable: false,
                },
                async () => {
                    try {
                        await cli.discoverProject(workspaceFolder.uri.fsPath);
                        vscode.window.showInformationMessage('Project discovered! devhub.toml created.');
                        treeProvider.refresh();
                    } catch (error) {
                        vscode.window.showErrorMessage(`Failed to discover project: ${error}`);
                    }
                }
            );
        }),

        vscode.commands.registerCommand('devhub.openDashboard', async () => {
            // Open the DevHub dashboard in browser
            const dashboardUrl = 'http://devhub.localhost';
            vscode.env.openExternal(vscode.Uri.parse(dashboardUrl));
        }),
    ];

    // Register all disposables
    context.subscriptions.push(
        treeView,
        treeProvider,
        statusBarManager,
        logPanel,
        ...commands
    );

    console.log('DevHub Commander extension activated successfully');
}

/**
 * Show quick pick to select a project
 */
async function selectProject(): Promise<string | undefined> {
    const projects = await cli.listProjects();
    if (projects.length === 0) {
        vscode.window.showInformationMessage('No DevHub projects found');
        return undefined;
    }

    const items = projects.map(p => ({
        label: p.favorite ? `$(star-full) ${p.name}` : p.name,
        description: p.path,
        projectName: p.name,
    }));

    const selected = await vscode.window.showQuickPick(items, {
        placeHolder: 'Select a project',
    });

    return selected?.projectName;
}

export function deactivate() {
    console.log('DevHub Commander extension is deactivating...');
}

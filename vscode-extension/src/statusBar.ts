import * as vscode from 'vscode';
import * as cli from './cli';

export class StatusBarManager {
    private statusBarItem: vscode.StatusBarItem;
    private refreshTimer: NodeJS.Timeout | null = null;

    constructor() {
        this.statusBarItem = vscode.window.createStatusBarItem(
            vscode.StatusBarAlignment.Left,
            100
        );
        this.statusBarItem.command = 'devhub.openDashboard';
        this.statusBarItem.tooltip = 'Click to open DevHub Dashboard';

        this.updateVisibility();
        this.startAutoRefresh();

        // Listen for configuration changes
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('devhub.showStatusBar')) {
                this.updateVisibility();
            }
        });
    }

    private updateVisibility(): void {
        const config = vscode.workspace.getConfiguration('devhub');
        const showStatusBar = config.get<boolean>('showStatusBar') ?? true;

        if (showStatusBar) {
            this.statusBarItem.show();
            this.refresh();
        } else {
            this.statusBarItem.hide();
        }
    }

    private startAutoRefresh(): void {
        const config = vscode.workspace.getConfiguration('devhub');
        const interval = config.get<number>('refreshInterval') || 5000;

        if (this.refreshTimer) {
            clearInterval(this.refreshTimer);
        }

        this.refreshTimer = setInterval(() => {
            this.refresh();
        }, interval);
    }

    async refresh(): Promise<void> {
        try {
            const statuses = await cli.getStatus();

            let runningProjects = 0;
            let totalServices = 0;
            let runningServices = 0;

            for (const status of statuses) {
                const running = status.services.filter(s => s.status === 'running').length;
                if (running > 0) {
                    runningProjects++;
                }
                totalServices += status.services.length;
                runningServices += running;
            }

            if (runningProjects > 0) {
                this.statusBarItem.text = `$(server-process) DevHub: ${runningProjects} projects (${runningServices}/${totalServices} services)`;
                this.statusBarItem.backgroundColor = undefined;
            } else {
                this.statusBarItem.text = `$(server) DevHub: ${statuses.length} projects`;
                this.statusBarItem.backgroundColor = undefined;
            }
        } catch {
            this.statusBarItem.text = '$(warning) DevHub: Not available';
            this.statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
        }
    }

    dispose(): void {
        if (this.refreshTimer) {
            clearInterval(this.refreshTimer);
        }
        this.statusBarItem.dispose();
    }
}

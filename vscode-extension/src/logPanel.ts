import * as vscode from 'vscode';
import * as cli from './cli';

export class LogPanel {
    private outputChannel: vscode.OutputChannel;
    private currentProject: string | null = null;
    private currentService: string | null = null;
    private refreshTimer: NodeJS.Timeout | null = null;

    constructor() {
        this.outputChannel = vscode.window.createOutputChannel('DevHub Logs');
    }

    async showLogs(projectName: string, serviceName?: string): Promise<void> {
        this.currentProject = projectName;
        this.currentService = serviceName || null;

        this.outputChannel.clear();
        this.outputChannel.show(true);

        const header = serviceName
            ? `=== Logs for ${projectName}/${serviceName} ===`
            : `=== Logs for ${projectName} ===`;

        this.outputChannel.appendLine(header);
        this.outputChannel.appendLine('');

        await this.refreshLogs();
        this.startAutoRefresh();
    }

    private async refreshLogs(): Promise<void> {
        if (!this.currentProject) return;

        try {
            const logs = await cli.getLogs(this.currentProject, this.currentService || undefined);
            this.outputChannel.clear();

            const header = this.currentService
                ? `=== Logs for ${this.currentProject}/${this.currentService} ===`
                : `=== Logs for ${this.currentProject} ===`;

            this.outputChannel.appendLine(header);
            this.outputChannel.appendLine(`Last updated: ${new Date().toLocaleTimeString()}`);
            this.outputChannel.appendLine('');
            this.outputChannel.appendLine(logs);
        } catch (error) {
            this.outputChannel.appendLine(`Error fetching logs: ${error}`);
        }
    }

    private startAutoRefresh(): void {
        this.stopAutoRefresh();
        this.refreshTimer = setInterval(() => {
            this.refreshLogs();
        }, 2000);
    }

    private stopAutoRefresh(): void {
        if (this.refreshTimer) {
            clearInterval(this.refreshTimer);
            this.refreshTimer = null;
        }
    }

    dispose(): void {
        this.stopAutoRefresh();
        this.outputChannel.dispose();
    }
}

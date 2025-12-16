import * as vscode from 'vscode';
import * as cli from './cli';

export class ProjectTreeProvider implements vscode.TreeDataProvider<ProjectTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<ProjectTreeItem | undefined | null | void> = new vscode.EventEmitter<ProjectTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<ProjectTreeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private projects: cli.Project[] = [];
    private statuses: Map<string, cli.ProjectStatus> = new Map();
    private refreshTimer: NodeJS.Timeout | null = null;

    constructor() {
        this.startAutoRefresh();
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

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: ProjectTreeItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: ProjectTreeItem): Promise<ProjectTreeItem[]> {
        if (!element) {
            // Root level - show projects
            return this.getProjects();
        } else if (element.contextValue === 'project') {
            // Project level - show services
            return this.getServices(element.projectName);
        }
        return [];
    }

    private async getProjects(): Promise<ProjectTreeItem[]> {
        try {
            this.projects = await cli.listProjects();

            console.log(`[DevHub] Found ${this.projects.length} projects`);

            if (this.projects.length === 0) {
                // Return a helpful message item
                return [
                    new ProjectTreeItem(
                        'No projects registered',
                        vscode.TreeItemCollapsibleState.None,
                        'message',
                        '',
                        undefined,
                        'Run "devhub register" or "DevHub: Discover Project"',
                        undefined
                    )
                ];
            }

            const statusList = await cli.getStatus();

            // Map statuses by project name
            this.statuses.clear();
            for (const status of statusList) {
                this.statuses.set(status.name, status);
            }

            return this.projects.map(project => {
                const status = this.statuses.get(project.name);
                const runningServices = status?.services.filter(s => s.status === 'running').length || 0;
                const totalServices = status?.services.length || project.services.length || 0;

                const isRunning = runningServices > 0;
                const favoriteIcon = project.favorite ? '★ ' : '';

                return new ProjectTreeItem(
                    `${favoriteIcon}${project.name}`,
                    vscode.TreeItemCollapsibleState.Collapsed,
                    'project',
                    project.name,
                    undefined,
                    `${runningServices}/${totalServices} services`,
                    isRunning ? new vscode.ThemeColor('charts.green') : undefined
                );
            });
        } catch (error) {
            console.error(`[DevHub] Failed to load projects: ${error}`);
            return [
                new ProjectTreeItem(
                    'Failed to load projects',
                    vscode.TreeItemCollapsibleState.None,
                    'error',
                    '',
                    undefined,
                    `${error}`,
                    new vscode.ThemeColor('charts.red')
                )
            ];
        }
    }

    private async getServices(projectName: string): Promise<ProjectTreeItem[]> {
        const status = this.statuses.get(projectName);
        if (!status) {
            return [];
        }

        return status.services.map(service => {
            const isRunning = service.status === 'running';
            const statusIcon = isRunning ? '$(circle-filled)' : '$(circle-outline)';
            const portInfo = service.port ? `:${service.port}` : '';

            return new ProjectTreeItem(
                `${statusIcon} ${service.name}${portInfo}`,
                vscode.TreeItemCollapsibleState.None,
                'service',
                projectName,
                service.name,
                isRunning ? 'Running' : 'Stopped',
                isRunning ? new vscode.ThemeColor('charts.green') : new vscode.ThemeColor('charts.red')
            );
        });
    }

    dispose(): void {
        if (this.refreshTimer) {
            clearInterval(this.refreshTimer);
        }
    }
}

export class ProjectTreeItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly contextValue: string,
        public readonly projectName: string,
        public readonly serviceName?: string,
        public readonly description?: string,
        public readonly iconColor?: vscode.ThemeColor
    ) {
        super(label, collapsibleState);
        this.tooltip = this.description;

        // Set icon based on status
        if (contextValue === 'project') {
            this.iconPath = new vscode.ThemeIcon('folder', iconColor);
        } else if (contextValue === 'service') {
            this.iconPath = new vscode.ThemeIcon('server-process', iconColor);
        }
    }
}

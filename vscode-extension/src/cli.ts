import { exec } from 'child_process';
import { promisify } from 'util';
import * as vscode from 'vscode';

const execAsync = promisify(exec);

export interface Project {
    name: string;
    path: string;
    services: Service[];
    favorite: boolean;
    last_used?: string;
}

export interface Service {
    name: string;
    port: number;
    status: 'running' | 'stopped' | 'unknown';
    type: string;
    main?: boolean;
    subdomain?: string;
}

export interface ProjectStatus {
    name: string;
    services: ServiceStatus[];
}

export interface ServiceStatus {
    name: string;
    status: 'running' | 'stopped' | 'unknown';
    port: number;
}

/**
 * Get the devhub CLI path from settings
 */
function getCliPath(): string {
    const config = vscode.workspace.getConfiguration('devhub');
    return config.get<string>('cliPath') || 'devhub';
}

/**
 * Execute a devhub CLI command
 */
async function runCommand(args: string[]): Promise<string> {
    const cliPath = getCliPath();
    const command = `${cliPath} ${args.join(' ')}`;

    try {
        const { stdout, stderr } = await execAsync(command, {
            timeout: 30000,
            maxBuffer: 1024 * 1024,
            env: { ...process.env, FORCE_COLOR: '0' }, // Disable colors for easier parsing
        });

        // Log for debugging
        console.log(`[DevHub CLI] Command: ${command}`);
        console.log(`[DevHub CLI] Output: ${stdout.substring(0, 200)}...`);

        return stdout.trim();
    } catch (error: unknown) {
        const err = error as { stderr?: string; message?: string; stdout?: string };
        console.error(`[DevHub CLI] Error: ${err.message}`);
        console.error(`[DevHub CLI] Stderr: ${err.stderr}`);
        throw new Error(err.stderr || err.message || 'Unknown error');
    }
}

/**
 * Parse the output of `devhub list` command
 * Returns project list with basic info
 */
export async function listProjects(): Promise<Project[]> {
    try {
        const output = await runCommand(['list']);
        return parseProjectList(output);
    } catch {
        return [];
    }
}

/**
 * Parse devhub list output into Project objects
 * Output format:
 *   ● project-name
 *     Path: /path/to/project
 *     Registered: 2025-12-16 12:26
 *     Services:
 *       • service-name (port 8080)
 */
function parseProjectList(output: string): Project[] {
    const projects: Project[] = [];
    const lines = output.split('\n');

    let currentProject: Project | null = null;

    for (const line of lines) {
        // Project line: "  ● project-name" or "  ★ project-name"
        const projectMatch = line.match(/^\s*[●★]\s*(\S+)\s*$/);
        if (projectMatch) {
            if (currentProject) {
                projects.push(currentProject);
            }
            currentProject = {
                name: projectMatch[1],
                path: '',
                services: [],
                favorite: line.includes('★'),
            };
            continue;
        }

        // Path line: "    Path: /path/to/project"
        const pathMatch = line.match(/^\s*Path:\s*(.+)$/);
        if (pathMatch && currentProject) {
            currentProject.path = pathMatch[1].trim();
            continue;
        }

        // Service line: "      • service-name (port 8080)"
        const serviceMatch = line.match(/^\s*•\s*(\S+)\s*\(port\s*(\d+)\)/);
        if (serviceMatch && currentProject) {
            currentProject.services.push({
                name: serviceMatch[1],
                port: parseInt(serviceMatch[2], 10),
                status: 'unknown',
                type: 'unknown',
            });
        }
    }

    if (currentProject) {
        projects.push(currentProject);
    }

    return projects;
}

/**
 * Get status for all projects
 */
export async function getStatus(): Promise<ProjectStatus[]> {
    try {
        const output = await runCommand(['status']);
        return parseStatus(output);
    } catch {
        return [];
    }
}

/**
 * Parse devhub status output
 * Output format:
 *   ● project-name
 *     ● service:8080    (running)
 *     ○ service:3000    (stopped)
 */
function parseStatus(output: string): ProjectStatus[] {
    const statuses: ProjectStatus[] = [];
    let currentProject: ProjectStatus | null = null;

    const lines = output.split('\n');
    for (const line of lines) {
        // Project header: "  ● project-name" or "  ○ project-name"
        const projectMatch = line.match(/^\s*[●○]\s*(\S+)\s*$/);
        if (projectMatch) {
            if (currentProject) {
                statuses.push(currentProject);
            }
            currentProject = {
                name: projectMatch[1],
                services: [],
            };
            continue;
        }

        // Service line: "    ● service:8080" or "    ○ service:3000"
        const serviceMatch = line.match(/^\s{4,}([●○])\s*(\S+):(\d+)/);
        if (serviceMatch && currentProject) {
            currentProject.services.push({
                name: serviceMatch[2],
                status: serviceMatch[1] === '●' ? 'running' : 'stopped',
                port: parseInt(serviceMatch[3], 10),
            });
        }
    }

    if (currentProject) {
        statuses.push(currentProject);
    }

    return statuses;
}

/**
 * Start a project
 */
export async function startProject(name: string): Promise<void> {
    await runCommand(['start', name]);
}

/**
 * Stop a project
 */
export async function stopProject(name: string): Promise<void> {
    await runCommand(['stop', name]);
}

/**
 * Restart a project
 */
export async function restartProject(name: string): Promise<void> {
    await runCommand(['restart', name]);
}

/**
 * Start a specific service in a project
 */
export async function startService(projectName: string, serviceName: string): Promise<void> {
    await runCommand(['start', projectName, '-s', serviceName]);
}

/**
 * Stop a specific service in a project
 */
export async function stopService(projectName: string, serviceName: string): Promise<void> {
    await runCommand(['stop', projectName, '-s', serviceName]);
}

/**
 * Get logs for a project/service
 */
export async function getLogs(projectName: string, serviceName?: string): Promise<string> {
    const args = ['logs', projectName];
    if (serviceName) {
        args.push(serviceName);
    }
    return runCommand(args);
}

/**
 * Toggle favorite status
 */
export async function toggleFavorite(projectName: string): Promise<void> {
    await runCommand(['fav', 'toggle', projectName]);
}

/**
 * Discover project in current directory
 */
export async function discoverProject(path: string): Promise<void> {
    await runCommand(['discover', path]);
}

/**
 * Open project in browser
 */
export async function openInBrowser(projectName: string, serviceName?: string): Promise<void> {
    const args = ['open', projectName];
    if (serviceName) {
        args.push('-s', serviceName);
    }
    await runCommand(args);
}

/**
 * Check if devhub CLI is available
 */
export async function checkCliAvailable(): Promise<boolean> {
    try {
        await runCommand(['--version']);
        return true;
    } catch {
        return false;
    }
}

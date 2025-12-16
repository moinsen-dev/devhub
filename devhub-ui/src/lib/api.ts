// DevHub API Client

import type { Project, ProjectStatus, ApiResponse } from './types';

const API_BASE = '/api';

async function fetchApi<T>(endpoint: string, options?: RequestInit): Promise<T> {
	const response = await fetch(`${API_BASE}${endpoint}`, {
		headers: {
			'Content-Type': 'application/json',
			...options?.headers,
		},
		...options,
	});

	if (!response.ok) {
		const error = await response.text();
		throw new Error(error || `HTTP ${response.status}`);
	}

	return response.json();
}

export const api = {
	// Get all projects with status
	async getProjects(): Promise<ProjectStatus[]> {
		return fetchApi<ProjectStatus[]>('/projects');
	},

	// Get single project details
	async getProject(name: string): Promise<Project> {
		return fetchApi<Project>(`/projects/${name}`);
	},

	// Start a project (all services)
	async startProject(name: string): Promise<void> {
		await fetchApi(`/projects/${name}/start`, { method: 'POST' });
	},

	// Stop a project (all services)
	async stopProject(name: string): Promise<void> {
		await fetchApi(`/projects/${name}/stop`, { method: 'POST' });
	},

	// Restart a project (all services)
	async restartProject(name: string): Promise<void> {
		await fetchApi(`/projects/${name}/restart`, { method: 'POST' });
	},

	// Start a specific service
	async startService(project: string, service: string): Promise<void> {
		await fetchApi(`/projects/${project}/services/${service}/start`, { method: 'POST' });
	},

	// Stop a specific service
	async stopService(project: string, service: string): Promise<void> {
		await fetchApi(`/projects/${project}/services/${service}/stop`, { method: 'POST' });
	},

	// Get logs for a project
	async getLogs(project: string, service?: string, lines: number = 100): Promise<string[]> {
		const params = new URLSearchParams({ lines: lines.toString() });
		if (service) params.set('service', service);
		return fetchApi<string[]>(`/projects/${project}/logs?${params}`);
	},

	// Register a new project
	async registerProject(path: string): Promise<Project> {
		return fetchApi<Project>('/projects', {
			method: 'POST',
			body: JSON.stringify({ path }),
		});
	},

	// Unregister a project
	async unregisterProject(name: string): Promise<void> {
		await fetchApi(`/projects/${name}`, { method: 'DELETE' });
	},

	// Discover project at path
	async discoverProject(path: string): Promise<Project> {
		return fetchApi<Project>('/discover', {
			method: 'POST',
			body: JSON.stringify({ path }),
		});
	},
};

// WebSocket for real-time updates
export function createStatusWebSocket(onMessage: (data: ProjectStatus[]) => void): WebSocket {
	const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
	const ws = new WebSocket(`${protocol}//${window.location.host}/api/ws/status`);

	ws.onmessage = (event) => {
		try {
			const data = JSON.parse(event.data);
			onMessage(data);
		} catch (e) {
			console.error('Failed to parse WebSocket message:', e);
		}
	};

	ws.onerror = (error) => {
		console.error('WebSocket error:', error);
	};

	return ws;
}

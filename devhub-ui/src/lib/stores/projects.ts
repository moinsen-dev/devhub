import { writable, derived } from 'svelte/store';
import type { ProjectStatus } from '$lib/types';
import { api } from '$lib/api';

// Projects store
export const projects = writable<ProjectStatus[]>([]);
export const loading = writable(true);
export const error = writable<string | null>(null);

// Derived stores
export const runningProjects = derived(projects, ($projects) =>
	$projects.filter((p) => p.any_running)
);

export const stoppedProjects = derived(projects, ($projects) =>
	$projects.filter((p) => !p.any_running)
);

export const totalServices = derived(projects, ($projects) =>
	$projects.reduce((acc, p) => acc + p.services.length, 0)
);

export const runningServices = derived(projects, ($projects) =>
	$projects.reduce((acc, p) => acc + p.services.filter((s) => s.running).length, 0)
);

// Actions
export async function fetchProjects() {
	loading.set(true);
	error.set(null);

	try {
		const data = await api.getProjects();
		projects.set(data);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to fetch projects');
	} finally {
		loading.set(false);
	}
}

export async function startProject(name: string) {
	try {
		await api.startProject(name);
		await fetchProjects();
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to start project');
	}
}

export async function stopProject(name: string) {
	try {
		await api.stopProject(name);
		await fetchProjects();
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to stop project');
	}
}

export async function restartProject(name: string) {
	try {
		await api.restartProject(name);
		await fetchProjects();
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to restart project');
	}
}

export async function openTerminal(name: string) {
	try {
		await api.openTerminal(name);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to open terminal');
	}
}

export async function openVSCode(name: string) {
	try {
		await api.openVSCode(name);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to open VS Code');
	}
}

export async function startService(project: string, service: string) {
	try {
		await api.startService(project, service);
		await fetchProjects();
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to start service');
	}
}

export async function stopService(project: string, service: string) {
	try {
		await api.stopService(project, service);
		await fetchProjects();
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to stop service');
	}
}

export async function getLogs(project: string, service?: string, lines: number = 100): Promise<string[]> {
	try {
		return await api.getLogs(project, service, lines);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to get logs');
		return [];
	}
}

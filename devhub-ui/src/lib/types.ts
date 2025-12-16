// DevHub API Types

export interface Project {
	name: string;
	path: string;
	description?: string;
	registered_at: string;
	services: Service[];
}

export interface Service {
	name: string;
	service_type: ServiceType;
	command: string;
	port: number;
	cwd?: string;
	health_check?: string;
	subdomain?: string;
	main: boolean;
	running: boolean;
	url?: string;
}

export type ServiceType =
	| 'rust-binary'
	| 'node'
	| 'python'
	| 'go'
	| 'docker-compose'
	| 'shell';

export interface ProjectStatus {
	name: string;
	path: string;
	description?: string;
	services: ServiceStatus[];
	any_running: boolean;
}

export interface ServiceStatus {
	name: string;
	port: number;
	running: boolean;
	url?: string;
}

export interface LogEntry {
	timestamp: string;
	service: string;
	level: 'info' | 'warn' | 'error';
	message: string;
}

export interface ApiResponse<T> {
	success: boolean;
	data?: T;
	error?: string;
}

<script lang="ts">
	import type { ProjectStatus } from '$lib/types';
	import { startProject, stopProject, restartProject, openTerminal, openVSCode, startService, stopService } from '$lib/stores/projects';
	import LogViewer from './LogViewer.svelte';

	export let project: ProjectStatus;

	let loading = false;
	let showLogs = false;
	let logService: string | undefined = undefined;
	let serviceLoading: { [key: string]: boolean } = {};

	async function handleStart() {
		loading = true;
		await startProject(project.name);
		loading = false;
	}

	async function handleStop() {
		loading = true;
		await stopProject(project.name);
		loading = false;
	}

	async function handleRestart() {
		loading = true;
		await restartProject(project.name);
		loading = false;
	}

	async function handleOpenTerminal() {
		await openTerminal(project.name);
	}

	async function handleOpenVSCode() {
		await openVSCode(project.name);
	}

	function getServiceUrl(service: { name: string; port: number; running: boolean }) {
		if (!service.running) return null;
		// Main service gets project.localhost, others get service.project.localhost
		const projectName = project.name;
		const idx = project.services.indexOf(service);
		if (idx === 0) {
			return `http://${projectName}.localhost`;
		}
		return `http://${service.name}.${projectName}.localhost`;
	}

	async function handleStartService(serviceName: string) {
		serviceLoading[serviceName] = true;
		await startService(project.name, serviceName);
		serviceLoading[serviceName] = false;
	}

	async function handleStopService(serviceName: string) {
		serviceLoading[serviceName] = true;
		await stopService(project.name, serviceName);
		serviceLoading[serviceName] = false;
	}

	function openServiceLogs(serviceName?: string) {
		logService = serviceName;
		showLogs = true;
	}

	function closeLogs() {
		showLogs = false;
		logService = undefined;
	}
</script>

<div class="card p-5 hover:border-gray-600 transition-colors">
	<!-- Header -->
	<div class="flex items-start justify-between mb-4">
		<div class="flex items-center gap-3">
			<span
				class="status-dot"
				class:status-running={project.any_running}
				class:status-stopped={!project.any_running}
			></span>
			<div>
				<h3 class="text-lg font-semibold text-white">{project.name}</h3>
				{#if project.description}
					<p class="text-sm text-gray-400 mt-0.5">{project.description}</p>
				{/if}
			</div>
		</div>

		<!-- Actions -->
		<div class="flex gap-2">
			{#if project.any_running}
				<button
					class="btn btn-secondary text-sm"
					on:click={handleRestart}
					disabled={loading}
				>
					↻ Restart
				</button>
				<button
					class="btn btn-danger text-sm"
					on:click={handleStop}
					disabled={loading}
				>
					■ Stop
				</button>
			{:else}
				<button
					class="btn btn-primary text-sm"
					on:click={handleStart}
					disabled={loading}
				>
					▶ Start
				</button>
			{/if}
		</div>
	</div>

	<!-- Services -->
	<div class="space-y-2">
		{#each project.services as service}
			{@const url = getServiceUrl(service)}
			<div
				class="flex items-center justify-between py-2 px-3 rounded-lg bg-gray-900/50 group"
			>
				<div class="flex items-center gap-2">
					<span
						class="status-dot"
						class:status-running={service.running}
						class:status-stopped={!service.running}
					></span>
					<span class="text-sm font-medium">{service.name}</span>
					<span class="text-xs text-gray-500">:{service.port}</span>
				</div>

				<div class="flex items-center gap-2">
					{#if service.running && url}
						<a
							href={url}
							target="_blank"
							rel="noopener noreferrer"
							class="text-xs text-primary-400 hover:text-primary-300 hover:underline"
						>
							{url} →
						</a>
					{/if}

					<!-- Service-level controls (visible on hover) -->
					<div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
						<button
							class="p-1 rounded hover:bg-gray-700 text-gray-500 hover:text-white transition-colors"
							on:click={() => openServiceLogs(service.name)}
							title="View logs"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
							</svg>
						</button>
						{#if service.running}
							<button
								class="p-1 rounded hover:bg-red-900/50 text-gray-500 hover:text-red-400 transition-colors"
								on:click={() => handleStopService(service.name)}
								disabled={serviceLoading[service.name]}
								title="Stop service"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="currentColor" viewBox="0 0 24 24">
									<rect x="6" y="6" width="12" height="12" rx="1" />
								</svg>
							</button>
						{:else}
							<button
								class="p-1 rounded hover:bg-green-900/50 text-gray-500 hover:text-green-400 transition-colors"
								on:click={() => handleStartService(service.name)}
								disabled={serviceLoading[service.name]}
								title="Start service"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="currentColor" viewBox="0 0 24 24">
									<path d="M8 5v14l11-7z" />
								</svg>
							</button>
						{/if}
					</div>
				</div>
			</div>
		{/each}
	</div>

	<!-- Footer: Path and Quick Actions -->
	<div class="mt-4 pt-3 border-t border-gray-700/50 flex items-center justify-between">
		<p class="text-xs text-gray-500 truncate flex-1" title={project.path}>
			{project.path}
		</p>
		<div class="flex gap-1 ml-2">
			<button
				class="p-1.5 rounded hover:bg-gray-700 text-gray-400 hover:text-white transition-colors"
				on:click={() => openServiceLogs()}
				title="View all logs"
			>
				<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
				</svg>
			</button>
			<button
				class="p-1.5 rounded hover:bg-gray-700 text-gray-400 hover:text-white transition-colors"
				on:click={handleOpenTerminal}
				title="Open in Terminal"
			>
				<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
				</svg>
			</button>
			<button
				class="p-1.5 rounded hover:bg-gray-700 text-gray-400 hover:text-white transition-colors"
				on:click={handleOpenVSCode}
				title="Open in VS Code"
			>
				<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
					<path d="M17.583 3.194l-14.29 12.1a.66.66 0 0 0-.212.749.657.657 0 0 0 .216.254l2.373 1.733a.66.66 0 0 0 .794-.01l11.236-8.633V5.533a.66.66 0 0 0-1.117-.539zM17.583 20.806l-14.29-12.1a.66.66 0 0 1-.212-.749.657.657 0 0 1 .216-.254l2.373-1.733a.66.66 0 0 1 .794.01l11.236 8.633v3.854a.66.66 0 0 1-1.117.539z"/>
					<path d="M21.536 3.412l-2.836-.993a.658.658 0 0 0-.797.285L14.12 9.357l3.783 2.9 3.633-7.925a.66.66 0 0 0-.399-.92zM21.536 20.588l-2.836.993a.658.658 0 0 1-.797-.285l-3.783-6.653 3.783-2.9 3.633 7.925a.66.66 0 0 1-.399.92z"/>
				</svg>
			</button>
		</div>
	</div>
</div>

<!-- Log Viewer Modal -->
{#if showLogs}
	<LogViewer project={project.name} service={logService} onClose={closeLogs} />
{/if}

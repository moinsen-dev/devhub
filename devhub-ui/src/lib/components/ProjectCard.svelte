<script lang="ts">
	import type { ProjectStatus } from '$lib/types';
	import { startProject, stopProject, restartProject } from '$lib/stores/projects';

	export let project: ProjectStatus;

	let loading = false;

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
				class="flex items-center justify-between py-2 px-3 rounded-lg bg-gray-900/50"
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
			</div>
		{/each}
	</div>

	<!-- Path -->
	<div class="mt-4 pt-3 border-t border-gray-700/50">
		<p class="text-xs text-gray-500 truncate" title={project.path}>
			📁 {project.path}
		</p>
	</div>
</div>

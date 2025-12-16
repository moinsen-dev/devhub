<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { getLogs } from '$lib/stores/projects';

	export let project: string;
	export let service: string | undefined = undefined;
	export let onClose: () => void;

	let logs: string[] = [];
	let loading = true;
	let autoScroll = true;
	let refreshInterval: ReturnType<typeof setInterval> | null = null;
	let logContainer: HTMLDivElement;

	async function fetchLogs() {
		logs = await getLogs(project, service, 200);
		loading = false;
		if (autoScroll && logContainer) {
			setTimeout(() => {
				logContainer.scrollTop = logContainer.scrollHeight;
			}, 10);
		}
	}

	function toggleAutoScroll() {
		autoScroll = !autoScroll;
	}

	onMount(() => {
		fetchLogs();
		refreshInterval = setInterval(fetchLogs, 2000);
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
	});
</script>

<div class="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
	<div class="bg-gray-900 rounded-xl border border-gray-700 w-full max-w-4xl max-h-[80vh] flex flex-col">
		<!-- Header -->
		<div class="flex items-center justify-between p-4 border-b border-gray-700">
			<div>
				<h3 class="text-lg font-semibold text-white">
					Logs: {project}{service ? ` / ${service}` : ''}
				</h3>
				<p class="text-xs text-gray-500 mt-1">Auto-refreshing every 2 seconds</p>
			</div>
			<div class="flex items-center gap-3">
				<label class="flex items-center gap-2 text-sm text-gray-400 cursor-pointer">
					<input
						type="checkbox"
						bind:checked={autoScroll}
						class="w-4 h-4 rounded bg-gray-800 border-gray-600"
					/>
					Auto-scroll
				</label>
				<button
					class="p-2 rounded-lg hover:bg-gray-700 text-gray-400 hover:text-white transition-colors"
					on:click={onClose}
					title="Close"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>
		</div>

		<!-- Log Content -->
		<div
			bind:this={logContainer}
			class="flex-1 overflow-auto p-4 font-mono text-xs bg-black/50"
		>
			{#if loading}
				<div class="flex items-center justify-center h-32 text-gray-500">
					Loading logs...
				</div>
			{:else if logs.length === 0}
				<div class="flex items-center justify-center h-32 text-gray-500">
					No logs available
				</div>
			{:else}
				{#each logs as line, i}
					<div class="py-0.5 hover:bg-gray-800/50 whitespace-pre-wrap break-all {line.includes('error') || line.includes('Error') || line.includes('ERROR') ? 'text-red-400' : line.includes('warn') || line.includes('Warn') || line.includes('WARN') ? 'text-yellow-400' : 'text-gray-300'}">
						<span class="text-gray-600 select-none mr-3">{String(i + 1).padStart(4, ' ')}</span>{line}
					</div>
				{/each}
			{/if}
		</div>

		<!-- Footer -->
		<div class="p-3 border-t border-gray-700 flex items-center justify-between text-xs text-gray-500">
			<span>{logs.length} lines</span>
			<button
				class="text-primary-400 hover:text-primary-300"
				on:click={fetchLogs}
			>
				Refresh now
			</button>
		</div>
	</div>
</div>

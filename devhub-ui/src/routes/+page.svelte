<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import ProjectCard from '$lib/components/ProjectCard.svelte';
	import { projects, loading, error, fetchProjects, runningServices, totalServices } from '$lib/stores/projects';

	let pollInterval: ReturnType<typeof setInterval> | null = null;
	let autoRefresh = true;
	let searchQuery = '';
	let filterStatus: 'all' | 'running' | 'stopped' = 'all';
	const POLL_INTERVAL_MS = 5000; // Poll every 5 seconds

	// Computed filtered projects
	$: filteredProjects = $projects.filter((project) => {
		// Search filter
		const matchesSearch =
			!searchQuery ||
			project.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
			(project.description?.toLowerCase().includes(searchQuery.toLowerCase()) ?? false);

		// Status filter
		const matchesStatus =
			filterStatus === 'all' ||
			(filterStatus === 'running' && project.any_running) ||
			(filterStatus === 'stopped' && !project.any_running);

		return matchesSearch && matchesStatus;
	});

	onMount(() => {
		// Initial fetch
		fetchProjects();

		// Start polling
		if (autoRefresh) {
			startPolling();
		}
	});

	onDestroy(() => {
		stopPolling();
	});

	function startPolling() {
		if (pollInterval) return;
		pollInterval = setInterval(() => {
			fetchProjects();
		}, POLL_INTERVAL_MS);
	}

	function stopPolling() {
		if (pollInterval) {
			clearInterval(pollInterval);
			pollInterval = null;
		}
	}

	function toggleAutoRefresh() {
		autoRefresh = !autoRefresh;
		if (autoRefresh) {
			startPolling();
		} else {
			stopPolling();
		}
	}
</script>

<svelte:head>
	<title>DevHub Dashboard</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 py-8">
	<!-- Error Banner -->
	{#if $error}
		<div class="mb-6 p-4 bg-red-900/50 border border-red-700 rounded-lg">
			<div class="flex items-center gap-2">
				<span class="text-red-400">⚠</span>
				<p class="text-red-200">{$error}</p>
			</div>
		</div>
	{/if}

	<!-- Loading State -->
	{#if $loading && $projects.length === 0}
		<div class="flex items-center justify-center py-20">
			<div class="text-center">
				<div class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full mx-auto mb-4"></div>
				<p class="text-gray-400">Loading projects...</p>
			</div>
		</div>
	{:else if $projects.length === 0}
		<!-- Empty State -->
		<div class="text-center py-20">
			<div class="w-16 h-16 rounded-2xl bg-gray-800 flex items-center justify-center mx-auto mb-4">
				<span class="text-3xl">📦</span>
			</div>
			<h2 class="text-xl font-semibold text-white mb-2">No Projects Registered</h2>
			<p class="text-gray-400 mb-6 max-w-md mx-auto">
				Get started by registering your first project with DevHub.
			</p>
			<div class="bg-gray-800/50 rounded-lg p-4 max-w-lg mx-auto text-left">
				<p class="text-xs text-gray-500 mb-2">Run in your terminal:</p>
				<code class="text-sm text-primary-400">
					cd /path/to/project<br />
					devhub init<br />
					devhub register
				</code>
			</div>
		</div>
	{:else}
		<!-- Status Bar -->
		<div class="mb-6 p-4 bg-gray-800/50 rounded-lg border border-gray-700">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-6">
					<div class="flex items-center gap-2">
						<span class="status-dot status-running"></span>
						<span class="text-sm text-gray-300">
							{$runningServices}/{$totalServices} services running
						</span>
					</div>
					<div class="text-sm text-gray-500">
						{$projects.length} projects registered
					</div>
				</div>
				<div class="flex items-center gap-3">
					<label class="flex items-center gap-2 text-sm text-gray-400 cursor-pointer">
						<input
							type="checkbox"
							bind:checked={autoRefresh}
							on:change={toggleAutoRefresh}
							class="w-4 h-4 rounded bg-gray-700 border-gray-600"
						/>
						Auto-refresh
					</label>
					<button
						class="btn btn-secondary text-sm"
						on:click={() => fetchProjects()}
						disabled={$loading}
					>
						{#if $loading}
							<span class="animate-spin inline-block">↻</span>
						{:else}
							↻
						{/if}
						Refresh
					</button>
				</div>
			</div>
		</div>

		<!-- Toolbar -->
		<div class="flex items-center justify-between mb-6 gap-4">
			<div class="flex items-center gap-4 flex-1">
				<!-- Search -->
				<div class="relative flex-1 max-w-xs">
					<input
						type="text"
						placeholder="Search projects..."
						bind:value={searchQuery}
						class="w-full px-4 py-2 pl-10 bg-gray-800 border border-gray-700 rounded-lg text-sm text-white placeholder-gray-500 focus:outline-none focus:border-primary-500"
					/>
					<span class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500">🔍</span>
				</div>

				<!-- Status Filter -->
				<div class="flex items-center gap-1 bg-gray-800 rounded-lg p-1">
					<button
						class="px-3 py-1 text-sm rounded-md transition-colors"
						class:bg-gray-700={filterStatus === 'all'}
						class:text-white={filterStatus === 'all'}
						class:text-gray-400={filterStatus !== 'all'}
						on:click={() => (filterStatus = 'all')}
					>
						All
					</button>
					<button
						class="px-3 py-1 text-sm rounded-md transition-colors"
						class:bg-green-900={filterStatus === 'running'}
						class:text-green-300={filterStatus === 'running'}
						class:text-gray-400={filterStatus !== 'running'}
						on:click={() => (filterStatus = 'running')}
					>
						Running
					</button>
					<button
						class="px-3 py-1 text-sm rounded-md transition-colors"
						class:bg-gray-700={filterStatus === 'stopped'}
						class:text-white={filterStatus === 'stopped'}
						class:text-gray-400={filterStatus !== 'stopped'}
						on:click={() => (filterStatus = 'stopped')}
					>
						Stopped
					</button>
				</div>
			</div>

			<div class="text-sm text-gray-400">
				{filteredProjects.length} of {$projects.length} projects
			</div>
		</div>

		<!-- Project Grid -->
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			{#each filteredProjects as project (project.name)}
				<ProjectCard {project} />
			{/each}
		</div>
	{/if}
</div>

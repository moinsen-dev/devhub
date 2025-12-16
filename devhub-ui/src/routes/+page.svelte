<script lang="ts">
	import ProjectCard from '$lib/components/ProjectCard.svelte';
	import { projects, loading, error, fetchProjects } from '$lib/stores/projects';
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
		<!-- Toolbar -->
		<div class="flex items-center justify-between mb-6">
			<h2 class="text-lg font-semibold text-white">
				Projects ({$projects.length})
			</h2>
			<button
				class="btn btn-secondary text-sm"
				on:click={() => fetchProjects()}
				disabled={$loading}
			>
				↻ Refresh
			</button>
		</div>

		<!-- Project Grid -->
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			{#each $projects as project (project.name)}
				<ProjectCard {project} />
			{/each}
		</div>
	{/if}
</div>

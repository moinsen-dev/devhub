<script lang="ts">
	import '../app.css';
	import Header from '$lib/components/Header.svelte';
	import { fetchProjects } from '$lib/stores/projects';
	import { onMount } from 'svelte';

	onMount(() => {
		// Initial fetch
		fetchProjects();

		// Poll every 5 seconds (until we have WebSocket)
		const interval = setInterval(fetchProjects, 5000);

		return () => clearInterval(interval);
	});
</script>

<div class="min-h-screen flex flex-col">
	<Header />

	<main class="flex-1">
		<slot />
	</main>

	<footer class="bg-gray-800/30 border-t border-gray-800 py-4">
		<div class="max-w-7xl mx-auto px-4">
			<p class="text-center text-xs text-gray-500">
				DevHub v0.1.0 •
				<a href="https://github.com/moinsen/devhub" class="text-gray-400 hover:text-white">
					GitHub
				</a>
			</p>
		</div>
	</footer>
</div>

<script lang="ts">
	import { totalServices, runningServices, runningProjects } from '$lib/stores/projects';
</script>

<header class="bg-gray-800/50 border-b border-gray-700">
	<div class="max-w-7xl mx-auto px-4 py-4">
		<div class="flex items-center justify-between">
			<!-- Logo & Title -->
			<div class="flex items-center gap-3">
				<div class="w-10 h-10 rounded-xl bg-gradient-to-br from-primary-500 to-primary-700 flex items-center justify-center">
					<span class="text-xl">⚡</span>
				</div>
				<div>
					<h1 class="text-xl font-bold text-white">DevHub</h1>
					<p class="text-xs text-gray-400">Development Environment Manager</p>
				</div>
			</div>

			<!-- Stats -->
			<div class="flex items-center gap-6">
				<div class="text-center">
					<p class="text-2xl font-bold text-white">{$runningProjects.length}</p>
					<p class="text-xs text-gray-400">Running Projects</p>
				</div>
				<div class="text-center">
					<p class="text-2xl font-bold text-primary-400">{$runningServices}</p>
					<p class="text-xs text-gray-400">Active Services</p>
				</div>
				<div class="text-center">
					<p class="text-2xl font-bold text-gray-400">{$totalServices}</p>
					<p class="text-xs text-gray-400">Total Services</p>
				</div>
			</div>
		</div>
	</div>
</header>

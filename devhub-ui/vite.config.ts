import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 5173,
		host: true, // Bind to all interfaces for Docker access
		proxy: {
			'/api': {
				target: 'http://localhost:9876',
				changeOrigin: true
			}
		}
	}
});

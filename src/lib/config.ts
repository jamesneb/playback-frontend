export interface Config {
	apiUrl: string;
	websocketUrl: string;
	environment: 'local' | 'dev' | 'staging' | 'prod';
}

export function getConfig(): Config {
	const environment = (import.meta.env.VITE_ENVIRONMENT || 'local') as Config['environment'];
	
	const configs: Record<Config['environment'], Config> = {
		local: {
			apiUrl: 'http://localhost:8080',
			websocketUrl: 'ws://localhost:8080/ws',
			environment: 'local'
		},
		dev: {
			apiUrl: 'https://api-dev.playback.com',
			websocketUrl: 'wss://api-dev.playback.com/ws',
			environment: 'dev'
		},
		staging: {
			apiUrl: 'https://api-staging.playback.com',
			websocketUrl: 'wss://api-staging.playback.com/ws',
			environment: 'staging'
		},
		prod: {
			apiUrl: 'https://api.playback.com',
			websocketUrl: 'wss://api.playback.com/ws',
			environment: 'prod'
		}
	};

	return configs[environment];
}
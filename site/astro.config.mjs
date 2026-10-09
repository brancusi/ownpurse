// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import sitemap from '@astrojs/sitemap';

export default defineConfig({
	site: 'https://ownpurse.com',
	trailingSlash: 'ignore',
	integrations: [
		starlight({
			title: 'ownpurse docs',
			description: 'Documentation for ownpurse: local books for you and your agents.',
			logo: {
				light: './src/assets/ownpurse-wordmark.svg',
				dark: './src/assets/ownpurse-wordmark-reversed.svg',
				alt: 'ownpurse',
				replacesTitle: true,
			},
			favicon: '/favicon.svg',
			customCss: [
				'@fontsource-variable/bricolage-grotesque',
				'@fontsource/dm-mono/400.css',
				'@fontsource/dm-mono/500.css',
				'./src/styles/tokens.css',
				'./src/styles/starlight.css',
			],
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/brancusi/ownpurse' }],
			components: {
				SocialIcons: './src/components/DocsHeaderLinks.astro',
			},
			sidebar: [
				{ label: 'Overview', link: '/docs/' },
				{ label: 'Start here', items: [{ autogenerate: { directory: 'docs/start' } }] },
				{ label: 'Guides', items: [{ autogenerate: { directory: 'docs/guides' } }] },
				{ label: 'Developers', items: [{ autogenerate: { directory: 'docs/developers' } }] },
				{ label: 'Reference', items: [{ autogenerate: { directory: 'docs/reference' } }] },
			],
			lastUpdated: false,
			pagination: true,
			disable404Route: true,
		}),
		sitemap(),
	],
});

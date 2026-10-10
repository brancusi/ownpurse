import { defineCollection } from 'astro:content';
import { glob } from 'astro/loaders';
import { z } from 'astro/zod';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';

const blog = defineCollection({
	loader: glob({ pattern: '**/*.{md,mdx}', base: './src/content/blog' }),
	schema: z.object({
		title: z.string(),
		description: z.string(),
		date: z.coerce.date(),
		author: z.string().default('ownpurse'),
		draft: z.boolean().default(false),
	}),
});

// One dossier per player in the landscape. The front matter is the structured dossier (written as JSON, which is
// YAML); the body is the long form. src/components/landscape/ reads it.
const axis = z.object({ score: z.number().int().min(0).max(4), note: z.string() });
const landscape = defineCollection({
	loader: glob({ pattern: '**/*.md', base: './src/content/landscape' }),
	schema: z.object({
		slug: z.string(),
		name: z.string(),
		url: z.string(),
		segment: z.string(),
		status: z.enum(['active', 'acquired', 'shut', 'pivoted', 'archived']),
		founded: z.string(),
		hq: z.string(),
		backing: z.string(),
		tagline: z.string(),
		scope: z.string(),
		domains: z.array(z.string()),
		customers_served: z.array(z.string()),
		spectrum: z.object({
			custody: axis, locality: axis, history: axis, openness: axis, breadth: axis,
			autonomy: axis, agent_readiness: axis, integration_ease: axis, market: axis,
		}),
		api: z.object({ public: z.boolean(), style: z.string(), auth: z.string(), limits: z.string(), cost: z.string(), export: z.string(), notes: z.string() }),
		pricing: z.object({ model: z.string(), entry: z.string(), top: z.string(), notes: z.string() }),
		agents: z.object({ mcp: z.string(), ai_features: z.string(), agent_writes: z.string(), approval: z.string(), notes: z.string() }),
		integration: z.object({ scope: z.string(), notes: z.string() }),
		preference: z.object({ scale: z.string(), ratings: z.string(), sentiment: z.string() }),
		timeline: z.array(z.object({ date: z.string(), event: z.string() })),
		relation: z.string(),
		relation_note: z.string(),
		borrow: z.string(),
		avoid: z.string(),
		sources: z.array(z.object({ title: z.string(), url: z.string() })),
		unverified: z.array(z.string()),
		researched: z.string(),
	}),
});

export const collections = {
	landscape,
	docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
	blog,
};

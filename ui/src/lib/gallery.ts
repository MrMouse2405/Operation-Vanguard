import type { Component } from 'svelte';

const modules = import.meta.glob<{ default: Component }>('./examples/*.svelte');
const sources = import.meta.glob<string>('./examples/*.svelte', { query: '?raw', import: 'default', eager: true });
const groups: Record<string, string[]> = {
	Inputs: ['button', 'button-group', 'checkbox', 'combobox', 'date-picker', 'field', 'form', 'input', 'input-group', 'input-otp', 'label', 'native-select', 'radio-group', 'range-calendar', 'select', 'slider', 'switch', 'textarea', 'toggle', 'toggle-group', 'calendar'],
	Navigation: ['breadcrumb', 'command', 'context-menu', 'dropdown-menu', 'menubar', 'navigation-menu', 'pagination', 'sidebar', 'tabs'],
	Feedback: ['alert', 'alert-dialog', 'dialog', 'drawer', 'empty', 'hover-card', 'popover', 'progress', 'sheet', 'skeleton', 'sonner', 'spinner', 'tooltip'],
	Conversation: ['attachment', 'bubble', 'code-block', 'marker', 'message', 'message-scroller', 'questionnaire', 'trace']
};
const featured = ['button', 'card', 'input', 'calendar', 'dialog', 'chart', 'dropdown-menu', 'tabs', 'accordion'];
export const categories = ['All components', ...Object.keys(groups), 'Layout'];
export const components = Object.entries(modules).map(([path, load]) => {
	const id = path.split('/').pop()!.replace('.svelte', '');
	return {
		id, name: id.split('-').map(word => word === 'otp' ? 'OTP' : word[0].toUpperCase() + word.slice(1)).join(' '),
		category: Object.entries(groups).find(([, names]) => names.includes(id))?.[0] ?? 'Layout',
		load, source: sources[path].replace(/^<!--[^]*?-->\n/, '')
	};
}).sort((a, b) => {
	const first = featured.indexOf(a.id), second = featured.indexOf(b.id);
	return (first < 0 ? 999 : first) - (second < 0 ? 999 : second) || a.name.localeCompare(b.name);
});
export type GalleryEntry = (typeof components)[number];

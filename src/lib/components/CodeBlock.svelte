<script lang="ts">
	import { onMount } from 'svelte'
	import * as backend from '../../../bindings'
	export let code: string
	export let language: string
	let highlightedCode: string = ''

	$: if (code && language) {
		highlight()
	}

	async function highlight() {
		language = String(language) // for some reason, language is sometimes an object
		highlightedCode = await backend.highlightCode(code, language)
	}

	// Initial highlight on component mount
	onMount(async () => {
		highlight()
	})
</script>

{#if highlightedCode}
	<pre class="rounded-lg overflow-x-auto"><code class={language}>{@html highlightedCode}</code
		></pre>
{:else}
	<div class="animate-ping rounded-full self-center self-middle size-4 bg-white opacity-100"></div>
{/if}

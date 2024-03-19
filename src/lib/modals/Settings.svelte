<script lang="ts">
	import Modal from 'modal-svelte'
	import { loadProviders, setApiKeys, type ProviderData } from '../../../bindings'
	import { onMount } from 'svelte'
	export let show: boolean = false
	// bind inputs to variables
	let providers: ProviderData[] = []
	onMount(async () => {
		providers = await loadProviders()
	})
</script>

{#if show}
	<Modal
		onCancel={() => {
			show = false
		}}
	>
		API-keys:
		<form
			on:submit|preventDefault={() => {
				setApiKeys(providers)
			}}
		>
			{#each providers as provider}
				<div class="flex flex-row m-1">
					<label for="{provider.provider_name}-api-key" class="w-24">{provider.display_name}:</label
					>
					<input
						type="text"
						id="{provider.provider_name}-api-key"
						name="{provider.provider_name}-api-key"
						class="text-black w-96 px-1"
						bind:value={provider.api_key}
					/>
				</div>
			{/each}
			<div class="flex place-content-center mt-4">
				<button type="submit" class="bg-gray2 p-2 px-4 rounded-md">Set keys</button>
			</div>
		</form>
	</Modal>
{/if}

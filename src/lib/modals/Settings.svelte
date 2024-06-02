<script lang="ts">
	import Modal from 'modal-svelte'
	import * as c from '../../../bindings'
	import { loadProviders, setApiKeys, type ProviderData } from '../../../bindings'
	import { onMount } from 'svelte'
	import Icon from '@iconify/svelte'
	export let show: boolean = false
	// bind inputs to variables
	let providers: ProviderData[] = []
	let currentView: string
	let settings: c.Settings
	const themes = [
		'InspiredGitHub',
		'Solarized (dark)',
		'Solarized (light)',
		'base16-eighties.dark',
		'base16-mocha.dark',
		'base16-ocean.dark',
		'base16-ocean.light',
	]
	onMount(async () => {
		providers = await loadProviders()
		settings = await c.getSettings()
	})
	// if show is set to true, set current view to menu
	$: if (show) {
		currentView = 'menu'
	}
</script>

{#if show}
	<Modal
		onCancel={() => {
			show = false
		}}
		class="h-[36rem] w-[36rem]"
	>
		{#if currentView === 'menu'}
			<div class="grid grid-col-1 gap-y-4 m-8">
				<button on:click={() => (currentView = 'api-keys')}
					><span class="hover:underline">API-Keys</span></button
				>
				<button on:click={() => (currentView = 'code-theme')}
					><span class="hover:underline">Code Theme</span></button
				>
			</div>
		{:else}
			<div class="flex flex-col justify-between">
				<div>
					{#if currentView === 'api-keys'}
						API-keys:
						<form
							on:submit|preventDefault={() => {
								setApiKeys(providers)
							}}
						>
							{#each providers as provider}
								<div class="flex flex-row m-1">
									<label for="{provider.provider_name}-api-key" class="w-24"
										>{provider.display_name}:</label
									>
									<input
										type="password"
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
					{:else if currentView === 'code-theme'}
						{#each themes as theme}
							<input
								type="radio"
								id={theme}
								name="code-theme"
								value={theme}
								bind:group={settings.code_theme}
								on:change={() => c.applyAndSaveSettings(settings)}
							/>
							<label for={theme}>{theme}</label><br />
						{/each}
					{/if}
				</div>
				<div class="flex justify-center mt-8">
					<button
						on:click={() => (currentView = 'menu')}
						class="flex flex-row p-2 px-4 rounded-md group"
					>
						<Icon
							icon="ic:twotone-arrow-back-ios"
							class="mt-1 mr-2 scale-125"
							style="color: white"
						/>
						<span class="group-hover:underline">Back</span></button
					>
				</div>
			</div>
		{/if}
	</Modal>
{/if}

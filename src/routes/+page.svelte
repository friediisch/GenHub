<script lang="ts">
	import { onMount, tick } from 'svelte'
	import * as c from '../../bindings'
	import { v4 as uuidv4 } from 'uuid'
	import Icon from '@iconify/svelte'
	import { checkShortcut } from '$lib/general'
	import SettingsModal from '$lib/modals/Settings.svelte'
	import 'prismjs/themes/prism-funky.css'
	import { type Event as TauriEvent, listen } from '@tauri-apps/api/event'
	import { availableModelsStore, availableProvidersStore } from '$lib/stores'

	let chats: c.Chats = []
	let currentChatMessages: c.Message[]
	$: currentChatMessages = []
	let selectedChatId: string
	let newChatId: string
	let inputText = ''
	$: submitButtonDisabled =
		inputText.trim() === '' ||
		currentChatMessages[currentChatMessages.length - 1]?.role === 'animate'
	let modelSelectorOpen: boolean = false
	let selectedModel: c.Model
	let selectedModelName: string = ''
	let showSettings: boolean = false
	let settings: c.Settings
	let showContextMenu: boolean = false
	let renamingChatId: string = ''
	let chatRenameContainer: HTMLElement

	onMount(async () => {
		await c.readApiKeysFromEnv()
		chats = await c.getChats()
		availableModelsStore.set(await c.getModels())
		settings = await c.getSettings()
		availableProvidersStore.set(await c.loadProviders())
		if (settings.default_model in $availableModelsStore) {
			selectedModel = $availableModelsStore.find(
				(model) =>
					model.model_name == settings.default_model &&
					model.provider_name == settings.default_provider,
			)!
		} else {
			selectedModel = $availableModelsStore[0]
		}
		selectedModelName = selectedModel.model_name
		newChat()
		const unsubscribe_newMessage = listen<string>('newMessage', handleNewMessage)
		const unsubscribe_newChat = listen<string>('newChat', handleNewChat)

		// Add click event listener to close model selector
		document.addEventListener('click', handleOutsideClick)
	})

	function handleOutsideClick(event: MouseEvent) {
		const modelSelector = document.getElementById('modelSelector')
		const modelSelectorButton = document.getElementById('modelSelectorButton')

		if (modelSelectorOpen && modelSelector && modelSelectorButton) {
			if (
				!modelSelector.contains(event.target as Node) &&
				!modelSelectorButton.contains(event.target as Node)
			) {
				modelSelectorOpen = false
			}
		}
	}

	async function keydown(e: KeyboardEvent) {
		if (checkShortcut(e, 'N', { cmdOrCtrl: true })) {
			newChat()
		}
	}

	async function handleSubmit(event: Event) {
		event.preventDefault()
		const textarea = document.getElementById('chatInputContainer')
		const inputTextToBeSent: string = inputText
		inputText = ''
		textarea!.style.height = 'auto'
		newChatId = ''
		scrollToBottom()
		c.getMessage(
			inputTextToBeSent,
			selectedChatId,
			selectedModel.provider_name,
			selectedModel.model_name,
		)
		chats = await c.getChats()
	}

	async function setFocus() {
		const textarea = document.getElementById('chatInputContainer')
		textarea!.focus()
	}

	async function newChat() {
		newChatId = uuidv4()
		setFocus()
		selectedChatId = newChatId
		currentChatMessages = await c.loadChat(selectedChatId)
	}

	async function frontendLoadChat(new_selectedChatId: string) {
		setFocus()
		selectedChatId = new_selectedChatId
		currentChatMessages = await c.loadChat(selectedChatId)
		if (currentChatMessages[currentChatMessages.length - 1]?.role === 'user') {
			currentChatMessages = [
				...currentChatMessages,
				{ id: 'animationMessage', role: 'animate', content: '', model_name: '', blocks: null },
			]
		}
		scrollToBottom()
		let offset = currentChatMessages[currentChatMessages.length - 1]?.role === 'animate' ? 2 : 1
		selectedModel = $availableModelsStore.find(
			(model) =>
				model.model_name == currentChatMessages[currentChatMessages.length - offset]?.model_name,
		)!
		selectedModelName = selectedModel.model_name
	}

	let messagesContainer: HTMLElement
	async function scrollToBottom() {
		await new Promise((resolve) => setTimeout(resolve))
		messagesContainer.scrollTop = messagesContainer.scrollHeight
	}

	async function handleInput(event: any) {
		const textarea = event.target
		if (textarea instanceof HTMLTextAreaElement) {
			textarea.style.height = 'auto'
			textarea.style.height = `${textarea.scrollHeight}px`
			if (textarea.scrollHeight > 300) {
				textarea.style.overflowY = 'scroll'
				textarea.style.height = '300px'
			} else {
				textarea.style.overflowY = 'hidden'
			}
		}
	}

	async function handleNewMessage(event: TauriEvent<string>) {
		chats = await c.getChats()
		if (event.payload == selectedChatId) {
			frontendLoadChat(selectedChatId)
		}
	}

	async function handleNewChat(event: TauriEvent<string>) {
		chats = await c.getChats()
	}
</script>

<svelte:window on:keydown={keydown} />
<body class="flex h-screen bg-chat-window-gray text-white overflow-y-auto">
	<SettingsModal bind:show={showSettings} />
	<div
		class="flex flex-col min-w-72 max-w-96 bg-sidebar-gray overflow-y-auto overscroll-contain h-screen px-4 pt-6"
	>
		<div class="text-3xl pl-2 font-bold">Byok</div>
		<hr class="my-4" />
		<div class="overflow-y-auto flex-1">
			<div
				class="p-2 m-2 rounded-md flex flex-row justify-between
				{selectedChatId === newChatId ? 'bg-gray-600' : 'hover:bg-gray-800'}"
				on:mousedown={() => newChat()}
				role="button"
				aria-pressed="false"
				tabindex="0"
			>
				<div>New Chat</div>
				<Icon
					icon="octicon:comment-discussion-16"
					class="mt-1 mr-2 scale-125"
					style="color: white"
				/>
			</div>

			{#each chats as chat}
				<div
					class="block p-2 mx-2 rounded-md group
					{chat.id === selectedChatId ? 'bg-gray-600' : 'hover:bg-gray-800'}"
					on:mousedown={() => {
						inputText = ''
						frontendLoadChat(chat.id)
					}}
					role="button"
					aria-pressed="false"
					tabindex="0"
				>
					{#if chat.display_name.startsWith('unnamed_new_chat_')}
						<div
							class="block p-2 mx-2 animate-ping rounded-full self-center self-middle size-4 bg-white opacity-100"
						></div>
					{:else}
						<div class="flex flex-row justify-between">
							{#if renamingChatId === chat.id}
								<textarea
									class="flex flex-grow p-2 bg-gray-600 rounded-md"
									bind:this={chatRenameContainer}
									bind:value={chat.display_name}
									id="chatRenameContainer"
									on:keydown={(e) => {
										if (e.key === 'Enter') {
											chat.display_name = chat.display_name.trim()
											c.renameChat(chat.id, chat.display_name)
											renamingChatId = ''
										}
									}}
									on:blur={() => {
										chat.display_name = chat.display_name.trim()
										c.renameChat(chat.id, chat.display_name)
										renamingChatId = ''
									}}
								/>
							{:else}
								<div
									class="flex flex-grow break-all"
									on:mousedown={() => {
										showContextMenu = false
									}}
									role="button"
									aria-pressed="false"
									tabindex="0"
								>
									{chat.display_name}
								</div>
							{/if}
							<div
								on:mousedown={() => {
									showContextMenu = !showContextMenu
									frontendLoadChat(chat.id)
								}}
								role="button"
								aria-pressed="false"
								tabindex="0"
							>
								<Icon
									icon="mdi:dots-horizontal"
									class="mt-1 m-2 scale-125 opacity-0 group-hover:opacity-100 hover:cursor-pointer"
									style="color: white"
								/>
							</div>
						</div>
						{#if showContextMenu && chat.id === selectedChatId}
							<div class="flex flex-row justify-between px-8">
								<div
									class="bg-gray2 rounded-md p-2 mt-2"
									on:mousedown={async () => {
										showContextMenu = false
										renamingChatId = chat.id
										setTimeout(() => {
											chatRenameContainer.focus()
										}, 0)
									}}
									role="button"
									aria-pressed="false"
									tabindex="0"
								>
									Rename
								</div>
								<div
									class="bg-gray2 rounded-md p-2 mt-2"
									on:mousedown={async () => {
										showContextMenu = false
										c.archiveChat(chat.id)
										chats = await c.getChats()
										frontendLoadChat(chats[0].id)
									}}
									role="button"
									aria-pressed="false"
									tabindex="0"
								>
									Archive
								</div>
								<div
									class="bg-gray2 rounded-md p-2 mt-2"
									on:mousedown={async () => {
										showContextMenu = false
										c.deleteChat(chat.id)
										chats = await c.getChats()
										frontendLoadChat(chats[0].id)
									}}
									role="button"
									aria-pressed="false"
									tabindex="0"
								>
									Delete
								</div>
							</div>
						{/if}
					{/if}
				</div>
			{/each}
		</div>
		<hr class="mt-4" />
		<button
			class="flex flex-row mt-2 mb-4 p-2 rounded-md hover:bg-gray-800 hover:cursor-pointer justify-between"
			on:mousedown={() => (showSettings = true)}
		>
			Settings
			<Icon icon="octicon:gear-24" class="mt-1 mr-2 scale-125" style="color: white" />
		</button>
	</div>

	<div class="flex-1 flex flex-col items-center">
		<div class="w-full h-fit px-2">
			<button
				id="modelSelectorButton"
				class="group text-lg px-2 py-1 align-middle hover:bg-gray2 w-fit rounded-md cursor-pointer my-2 mx-1"
				on:mousedown={(e) => {
					e.stopPropagation()
					modelSelectorOpen = !modelSelectorOpen
				}}
			>
				{selectedModelName}
				<span class="icon-[octicon--chevron-down-12] scale-75 text-white"></span>
			</button>
			<hr class="border-gray-600" />
			{#if modelSelectorOpen}
				<div
					id="modelSelector"
					class="absolute z-10 bg-gray2 rounded-md p-2 mt-2 overflow-x-scroll"
					style="max-height: 80%;"
				>
					{#each $availableProvidersStore as provider}
						{#if $availableModelsStore.filter((model) => model.provider_name == provider.provider_name).length > 0}
							<div class="font-bold">{provider.display_name}</div>
							<hr class="border-gray-600 pb-1" />
							{#each $availableModelsStore.filter((model) => model.provider_name == provider.provider_name) as model}
								<div
									class="block p-2 mx-2 hover:bg-gray-600 rounded-md"
									class:bg-white={selectedModel == model}
									class:text-black={selectedModel == model}
									on:mousedown={(e) => {
										e.stopPropagation()
										selectedModel = model
										selectedModelName = selectedModel.model_name
										modelSelectorOpen = false
									}}
									role="button"
									aria-pressed="false"
									tabindex="0"
								>
									{model.model_name}
								</div>
							{/each}
						{/if}
					{/each}
				</div>
			{/if}
		</div>
		<div
			class="flex flex-col flex-1 min-w-[12rem] w-[56rem] max-w-[56rem] overflow-y-auto overscroll-contain px-2"
			bind:this={messagesContainer}
		>
			{#if selectedChatId === newChatId}
				<div class="flex flex-col items-center justify-center h-[66.67vh]">
					<div class="text-center text-3xl text-gradient animate-fly-and-fade">
						How can I help you today?
					</div>
					<div class="text-center text-md text-gray-500 animate-fly-and-fade">
						{selectedModelName}
					</div>
				</div>
			{:else}
				<div class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-1">
					<div class="p-2"></div>
					<div class="p-2"></div>
					{#each currentChatMessages as message}
						{#if message.role === 'user'}
							<div class="font-bold p-1 whitespace-nowrap">
								<div>You</div>
							</div>
							<div class="p-1 whitespace-pre-wrap word-break:break-word overflow-wrap:break-word">
								{message.content}
							</div>
						{:else}
							<div class="relative p-1 min-w-fit h-fit whitespace-nowrap group">
								<div id="display_name" class="font-bold text-gradient rounded-md relative">
									{$availableModelsStore.find((model) => model.model_name == message.model_name)
										?.model_display_name ||
										$availableModelsStore.find((model) => model == selectedModel)
											?.model_display_name}
								</div>
								<div
									id="model_name"
									class="absolute top-1 left-0 opacity-0 transition-opacity duration-150 ease-in-out z-10 inset-0 w-fit pointer-events-none"
								>
									<div class="bg-white text-gray-800 px-1 rounded-md">
										{$availableModelsStore.find((model) => model.model_name == message.model_name)
											?.model_name ||
											$availableModelsStore.find((model) => model == selectedModel)?.model_name}
									</div>
								</div>
							</div>
							<div class="p-1">
								{#if message.role === 'animate'}
									<div
										class="mt-1 animate-ping rounded-full self-center self-middle size-4 bg-white opacity-100"
									></div>
								{:else if message.blocks && message.blocks}
									{#each message.blocks as block}
										<div class="pb-2">
											{#if block.type_ === 'code'}
												<div class="relative group">
													{#if block.language}
														<div
															class="bg-gray2 text-gray-300 text-xs font-mono px-3 py-3 rounded-t-md"
														>
															{block.language}
														</div>
													{/if}
													<div
														class="text-white text-xs font-mono whitespace-pre-wrap overflow-x-scroll"
													>
														{@html block.rendered_content}
													</div>
													<button
														on:mousedown={async () => {
															try {
																await navigator.clipboard.writeText(block.raw_content)
																block.copied = true
																setTimeout(() => {
																	block.copied = false
																}, 2000)
															} catch (err) {
																console.error('Failed to copy: ', err)
															}
														}}
														class="absolute right-2 top-2 flex items-center justify-center w-6 h-6 bg-gray2 text-gray-300 rounded hover:bg-gray-500 cursor-pointer"
														title="Copy code"
													>
														{#if block.copied}
															<span class="icon-[lucide--check-check]" style="color: white;"></span>
														{:else}
															<span class="icon-[lucide--clipboard]" style="color: white;"></span>
														{/if}
													</button>
												</div>
											{:else if block.type_ === 'text'}
												<div class="word-break:break-word overflow-wrap:break-word">
													{@html block.rendered_content}
												</div>
											{/if}
										</div>
									{/each}
								{/if}
							</div>
						{/if}
					{/each}
				</div>
			{/if}
		</div>
		<div class=" min-w-[12rem] w-full max-w-[56rem] mx-auto px-2">
			<form
				on:submit={handleSubmit}
				class="flex bg-chat-window-gray items-center border border-gray-600 rounded-2xl px-2 py-1 my-4 w-full"
			>
				<textarea
					id="chatInputContainer"
					class="flex-grow bg-chat-window-gray rounded-lg p-2 text-gray-200 focus:outline-none mx-2 w-full"
					placeholder="Enter your message..."
					rows="1"
					style="resize: none;"
					bind:value={inputText}
					on:input={handleInput}
					on:keydown={($event) => {
						if ($event.key === 'Enter' && !$event.shiftKey) {
							$event.preventDefault()
							handleSubmit($event)
						}
					}}
				></textarea>
				<button
					type="submit"
					class="ml-4 text-3xl text-black rounded-lg px-2
							{submitButtonDisabled ? 'bg-slate-600' : 'bg-white'}"
					disabled={submitButtonDisabled}
				>
					<Icon
						icon="octicon:arrow-up-16"
						style={submitButtonDisabled ? 'color: chat-window-gray' : 'color: chat-window-gray'}
						class="my-1"
					/>
				</button>
			</form>
		</div>
	</div>
</body>

<style>
	#display_name:hover + #model_name {
		opacity: 1;
		z-index: 10;
	}
</style>

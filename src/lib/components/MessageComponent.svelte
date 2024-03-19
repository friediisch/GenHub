<script lang="ts">
	import CodeBlock from '$lib/components/CodeBlock.svelte'
	export let text: string
	let raw: boolean = false
	type Block = {
		type: 'text' | 'code'
		language: string
		content: string
		copied: boolean
	}

	// Function to split the text into code and regular text blocks
	function splitText(text: string) {
		const blocks: Block[] = []
		const regex = /```([\s\S]*?)```/g
		let lastIndex = 0

		text.replace(regex, (match, code, offset) => {
			// Add the text before the code block, if any
			if (offset > lastIndex) {
				blocks.push({
					type: 'text',
					language: '',
					content: processText(text.slice(lastIndex, offset)),
					copied: false,
				})
			}

			// extract the language from the code block
			let languageMatch = code.match(/^[a-z]+/i)
			if (languageMatch) {
				languageMatch[0] = languageMatch[0].toLowerCase()
			}
			// strip the language from the code block
			code = code.replace(/^[a-z]+/i, '').trim()

			// Add the code block
			blocks.push({
				type: 'code',
				language: languageMatch,
				content: code,
				copied: false,
			})

			// Update lastIndex to the end of the current match
			lastIndex = offset + match.length
			return match // This return is not used, but is required for replace to work
		})

		// Add any remaining text after the last code block
		if (lastIndex < text.length) {
			blocks.push({
				type: 'text',
				language: '',
				content: processText(text.slice(lastIndex)),
				copied: false,
			})
		}

		return blocks
	}

	function applyListStyles(node: HTMLElement): void {
		node.querySelectorAll('ul').forEach((ul: HTMLElement) => ul.classList.add('list-disc', 'pl-4'))
		node
			.querySelectorAll('ol')
			.forEach((ol: HTMLElement) => ol.classList.add('list-decimal', 'pl-4'))
	}

	function convertLineBreaksToParagraphs(content: string) {
		return (
			content
				.split('\n\n')
				// Replace single line breaks within paragraphs with <br>
				.map((paragraph) => `<p>${paragraph.replace(/\n/g, '<br>')}</p><br>`)
				.join('')
		)
	}

	async function copyToClipboard(block: Block, index: number) {
		try {
			await navigator.clipboard.writeText(block.content)
			// Update the 'copied' property of the block
			block.copied = true
			// Trigger Svelte's reactivity by reassigning the 'blocks' array
			blocks[index] = block
			blocks = [...blocks]

			setTimeout(() => {
				// Revert the 'copied' state after 2 seconds
				block.copied = false
				blocks[index] = block
				blocks = [...blocks]
			}, 2000)
		} catch (err) {
			console.error('Failed to copy: ', err)
		}
	}

	function processText(text: string) {
		text = removeTrailingLineBreaks(text)
		text = replaceBackticksWithSpan(text)
		return text
	}

	function replaceBackticksWithSpan(str: string) {
		return str.replace(/`([^`]+)`/g, (match, p1) => {
			return `<span class="bg-gray2 text-codefontcolor text-sm rounded-xs font-mono px-1">${p1}</span>`
		})
	}

	function removeTrailingLineBreaks(str: string) {
		return str.replace(/\n+$/, '')
	}

	let blocks = splitText(text)
</script>

<div class="p-1">
	{#if raw}
		{text}
	{:else}
		{#each blocks as block, index}
			{#if block.type === 'text'}
				<div class="text-base markdown" use:applyListStyles>
					{@html convertLineBreaksToParagraphs(block.content)}
				</div>
			{:else}
				<div class="relative group">
					{#if block.language}
						<div class="bg-gray2 text-gray-300 text-xs font-mono px-3 py-3 rounded-t-md">
							{block.language}
						</div>
					{/if}
					<div class="bg-black text-white text-xs font-mono p-2 rounded-b-md whitespace-pre-wrap">
						<CodeBlock code={block.content} language={block.language} />
					</div>
					<button
						on:click={() => copyToClipboard(block, index)}
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
			{/if}
		{/each}
	{/if}
</div>

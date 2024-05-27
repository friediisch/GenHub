# GenHub

### Todos:

#### Pre-Release

- Allow chat renaming
- Add index column to improve query performance
- Add a way for users to see descriptions of the models/ link to the docs
- Create transparent error handlers for all errors
  - Allow user to re-send a message that was previously sent to a different model
- Render markdown tables
- Mask API-keys in Settings
- add App to Mac App Store / Setapp / own distribution

#### Post-Release

- Use local models using TGI interface: https://github.com/huggingface/text-generation-inference
- Enter system prompts per chat/globally
- Refactor and enable local inference + add support for Llama-3-8B locally
- Add image generation APIs
- Add API-support
  - Google: Add support for Google as soon as it is available in Germany
  - Meta
- Add support for agents, such as SWE-Agent: https://github.com/princeton-nlp/SWE-agent
- Include federated learning for local models
- Add streaming API support
- RAG-support
- Automate adding new models to existing APIs
  - List models request?
  - Send incorrect model such that the api returns a list of valid models?

### Get started

1. Install Node.js
2. Install Rust
3. Follow the [Tauri setup guide](https://tauri.studio/en/docs/getting-started/intro)
4. Run `npm install`

### Commands

- `npm run dev`: Start app in dev mode. It sets `DEVELOPMENT=1` to tell the app to use `./src-tauri/appdata` for app data.
- `npm run build`: Build
- `npm run lint`: Lint

### Store API keys for development

Create a .env file with API keys:

- `openai="YOUR_API_KEY"`
- `mistralai="YOUR_API_KEY"`
- `anthropic="YOUR_API_KEY"`

### Release new version

1. Update `CHANGELOG.md`
2. Bump the version number in `src-tauri/Cargo.toml`
3. Run `cargo check` to update `Cargo.lock`
4. Create a git tag in the format `v#.#.#`
5. Add release notes to the generated GitHub release and publish it

### Acknowledgement

Yoinked the initial code from https://github.com/probablykasper/kadium

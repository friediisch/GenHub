# GenHub

### Todos:

#### Before release:

- git squash
- Make it so other chats can be opened after input was entered and the model is cooking

#### After release:

- Enter system prompts per chat/globally
- Add a way for users to see descriptions of the models/ link to the docs
- Create transparent error handlers for all errors
  - Anthropic: https://docs.anthropic.com/claude/reference/errors

### Get started

1. Install Node.js
2. Install Rust
3. Follow the [Tauri setup guide](https://tauri.studio/en/docs/getting-started/intro)
4. Run `npm install`

### Commands

- `npm run dev`: Start app in dev mode. It sets `DEVELOPMENT=1` to tell the app to use `./src-tauri/appdata` for app data.
- `npm run build`: Build
- `npm run lint`: Lint
- `npm run format`: Format

### Store API keys for development

- Create a .env file with API keys:
- `openai="YOUR_API_KEY"`
- `mistralai="YOUR_API_KEY"`
- `anthropic="YOUR_API_KEY"`

### Release new version

1. Update `CHANGELOG.md`
2. Bump the version number in `src-tauri/Cargo.toml`
3. Run `cargo check` to update `Cargo.lock`
4. Create a git tag in the format `v#.#.#`
5. Add release notes to the generated GitHub release and publish it

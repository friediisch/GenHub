# GenHub

### Todos:

- Allow chat renaming
- Maintain line breaks and tabs in user input
- Add index column to improve query performance
- Enter system prompts per chat/globally
- Add a way for users to see descriptions of the models/ link to the docs
- Create transparent error handlers for all errors
  - (e.g. Anthropic: https://docs.anthropic.com/claude/reference/errors)
  - Allow user to re-send a message that was previously sent to a different model
- Add image generation APIs
- Add support for Google as soon as it is available in Germany

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

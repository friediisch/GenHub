/* eslint-disable */
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

declare global {
    interface Window {
        __TAURI_INVOKE__<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
    }
}

// Function avoids 'window not defined' in SSR
const invoke = () => window.__TAURI_INVOKE__;

export function errorPopup(msg: string) {
    return invoke()<null>("error_popup", { msg })
}

export function getMessage(msg: string, chatId: string, providerName: string, modelName: string) {
    return invoke()<string>("get_message", { msg,chatId,providerName,modelName })
}

export function getChats() {
    return invoke()<Chats>("get_chats")
}

export function loadChat(chatId: string) {
    return invoke()<Message[]>("load_chat", { chatId })
}

export function loadProviders() {
    return invoke()<ProviderData[]>("load_providers")
}

export function setApiKeys(providers: ProviderData[]) {
    return invoke()<null>("set_api_keys", { providers })
}

export function getModels() {
    return invoke()<Models>("get_models")
}

export function readApiKeysFromEnv() {
    return invoke()<null>("read_api_keys_from_env")
}

export function getSettings() {
    return invoke()<Settings>("get_settings")
}

export function applyAndSaveSettings(newSettings: Settings) {
    return invoke()<null>("apply_and_save_settings", { newSettings })
}

export type Chat = { id: string; display_name: string; creation_date: string; last_updated: string }
export type Settings = { default_model: string; default_provider: string; code_theme: string }
export type Model = { provider_name: string; model_name: string; model_display_name: string }
export type Models = { models: Model[] }
export type MessageBlocks = { blocks: MessageBlock[] }
export type MessageBlock = { id: number | null; type_: string; language: string | null; raw_content: string; rendered_content: string; copied: boolean | null }
export type Message = { id: string; role: string; content: string; model_name: string; blocks: MessageBlocks | null }
export type Chats = { chats: Chat[] }
export type ProviderData = { provider_name: string; api_key: string; display_name: string }

import { invoke } from '@tauri-apps/api/core'
import { schemaRegistry } from './agent/schemaRegistry'
import { toolRegistry } from './agent/toolRegistry'
import type { AgentActionResult, AgentContext, AgentResponse, ChatMessage } from './agent/types'
import type { Entity, RecordData } from './model'
import type { ExternalItem } from './externalIntelligence'
export type { ChatMessage } from './agent/types'

export type AiModelOption = { id: string; label: string; description: string }
export type AiProviderId = 'hackstart' | 'deepseek' | 'minimax' | 'volc-agent-plan'
export type AiProviderOption = { id: AiProviderId; label: string; configured: boolean; baseUrl: string; model: string; models: AiModelOption[] }
export type HackStartConfig = { provider: AiProviderId; providerLabel: string; configured: boolean; model: string; baseUrl: string; providers: AiProviderOption[] }
export type BackupInfo = { name: string; path: string; size: number; modified: string }
export type CaptureProviderConfig = { providers: { id: 'redfox'; label: string; configured: boolean; supportedPlatforms: string[]; automaticSync: boolean; mediaDownload: boolean }[] }

const key = 'jason-os-browser-records'
const browser = () => !('__TAURI_INTERNALS__' in window)
const read = (): RecordData[] => JSON.parse(localStorage.getItem(key) || '[]')
const write = (records: RecordData[]) => localStorage.setItem(key, JSON.stringify(records))
const stamp = () => new Date().toISOString()
const active = (record: RecordData) => !record.archivedAt && !record.deletedAt

export const api = {
  async initialize() { return browser() ? { ok: true } : invoke('initialize_database') },
  async list(entity: Entity | 'all'): Promise<RecordData[]> { return browser() ? read().filter((record) => active(record) && (entity === 'all' || record.entity === entity)).sort((a, b) => b.updatedAt.localeCompare(a.updatedAt)) : invoke('list_records', { entity }) },
  async get(id: string): Promise<RecordData | null> { return browser() ? read().find((record) => record.id === id && !record.deletedAt) || null : invoke('get_record', { id }) },
  async save(entity: Entity, data: Partial<RecordData>): Promise<RecordData> {
    if (!browser()) return invoke('save_record', { entity, data })
    const records = read(); const id = data.id || `${entity}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`; const old = records.find((record) => record.id === id)
    const record = { ...old, ...data, id, entity, createdAt: old?.createdAt || stamp(), updatedAt: stamp() } as RecordData
    write([...records.filter((item) => item.id !== id), record]); return record
  },
  async archive(id: string) { if (!browser()) return invoke('archive_record', { id }); write(read().map((record) => record.id === id ? { ...record, archivedAt: stamp() } : record)) },
  async remove(id: string) { return this.archive(id) },
  async restore(id: string): Promise<RecordData> { if (!browser()) return invoke('restore_record', { id }); const record = read().find((item) => item.id === id)!; const restored = { ...record, archivedAt: undefined, deletedAt: undefined, updatedAt: stamp() }; write([...read().filter((item) => item.id !== id), restored]); return restored },
  async archived(): Promise<RecordData[]> { return browser() ? read().filter((record) => record.archivedAt && !record.deletedAt) : invoke('list_archived') },
  async search(query: string, entities: Entity[] = []): Promise<RecordData[]> { return browser() ? read().filter(active).filter((record) => (!entities.length || entities.includes(record.entity)) && JSON.stringify(record).toLowerCase().includes(query.toLowerCase())) : entities.length ? invoke('search_records_filtered', { query, entities }) : invoke('search_records', { query }) },
  async relations(id: string): Promise<RecordData[]> {
    if (!browser()) return invoke('list_relations', { id })
    return read().filter(active).filter((record) => record.id !== id && Object.entries(record).some(([field, value]) => field.endsWith('Id') && value === id || field.endsWith('Ids') && String(value || '').split(',').map((part) => part.trim()).includes(id)))
  },
  async addRelation(fromId: string, toId: string, relationType = 'manual') { if (!browser()) return invoke('add_relation', { fromId, toId, relationType }) },
  async stopTimer(id: string, endAt: string, durationMinutes: number): Promise<RecordData> { if (!browser()) return invoke('stop_timer', { id, endAt, durationMinutes }); const timer = read().find((record) => record.id === id)!; return this.save('timeLogs', { ...timer, endAt, durationMinutes, isRunning: false }) },
  async export(format: 'json' | 'markdown' | 'csv'): Promise<string> { if (!browser()) return invoke('export_data', { format }); const content = format === 'json' ? JSON.stringify(read(), null, 2) : read().map((record) => JSON.stringify(record)).join('\n'); const blob = new Blob([content], { type: 'text/plain' }); const url = URL.createObjectURL(blob); const anchor = document.createElement('a'); anchor.href = url; anchor.download = `jason-os.${format}`; anchor.click(); URL.revokeObjectURL(url); return '已在浏览器模式下载' },
  async backup(): Promise<string> { return browser() ? '浏览器模式没有 SQLite 数据库可供备份。' : invoke('create_backup') },
  async backups(): Promise<BackupInfo[]> { return browser() ? [] : invoke('list_backups') },
  async restoreBackup(path: string) { if (browser()) throw new Error('浏览器模式无法恢复 SQLite 备份。'); return invoke('restore_backup', { path }) },
  async getHackStartConfig(): Promise<HackStartConfig> { if (!browser()) return invoke('get_ai_config'); const providers = [{ id: 'hackstart' as const, label: 'HackStart', configured: false, baseUrl: 'https://ip2.hackstart.org/v1/chat/completions', model: 'gpt-5.5', models: [{ id: 'gpt-5.5', label: 'gpt-5.5', description: 'HackStart 模型' }] }, { id: 'deepseek' as const, label: 'DeepSeek', configured: false, baseUrl: 'https://api.deepseek.com/chat/completions', model: 'deepseek-v4-pro', models: [{ id: 'deepseek-v4-pro', label: 'deepseek-v4-pro', description: '高质量复杂推理' }, { id: 'deepseek-v4-flash', label: 'deepseek-v4-flash', description: '低延迟高性价比' }] }, { id: 'minimax' as const, label: 'MiniMax Token Plan', configured: false, baseUrl: 'https://api.minimaxi.com/anthropic/v1/messages', model: 'MiniMax-M3', models: [{ id: 'MiniMax-M3', label: 'MiniMax-M3', description: '最新旗舰 Agent 与推理模型' }] }, { id: 'volc-agent-plan' as const, label: '火山引擎 Agent Plan', configured: false, baseUrl: 'https://ark.cn-beijing.volces.com/api/plan/v3/responses', model: 'kimi-k3', models: [{ id: 'kimi-k3', label: 'kimi-k3', description: 'Agent Plan Medium 当前文本模型' }] }]; return { provider: 'hackstart', providerLabel: 'HackStart', configured: false, model: 'gpt-5.5', baseUrl: providers[0].baseUrl, providers } },
  async configureAiProvider(provider: AiProviderId, apiKey: string, model: string): Promise<HackStartConfig> { if (browser()) throw new Error('浏览器模式不能写入 应用私有凭据文件（权限 0600）。请使用桌面应用。'); return invoke('configure_ai_provider', { provider, apiKey, model }) },
  async testAiProvider(provider: AiProviderId, model: string): Promise<{ ok: boolean; provider: string; model: string; latencyMs: number; content: string }> { if (browser()) throw new Error('浏览器模式不能测试真实 API。请使用桌面应用。'); return invoke('test_ai_provider', { provider, model }) },
  async openExternal(url: string): Promise<void> { if (browser()) { window.open(url, '_blank', 'noopener,noreferrer'); return }; return invoke('open_external', { url }) },
  async getCaptureProviderConfig(): Promise<CaptureProviderConfig> { return browser() ? { providers: [{ id: 'redfox', label: 'RedFoxHub', configured: false, supportedPlatforms: ['微信公众号', '抖音', '小红书'], automaticSync: false, mediaDownload: false }] } : invoke('get_capture_provider_config') },
  async configureCaptureProvider(provider: 'redfox', apiKey: string): Promise<CaptureProviderConfig> { if (browser()) throw new Error('浏览器模式不能保存采集凭据。请使用桌面应用。'); return invoke('configure_capture_provider', { provider, apiKey }) },
  async testCaptureProvider(provider: 'redfox', url: string): Promise<{ ok: boolean; provider: string; latencyMs: number; content: Record<string, unknown> }> { if (browser()) throw new Error('浏览器模式不能测试真实采集 API。'); return invoke('test_capture_provider', { provider, url }) },
  async listExternalItems(limit = 80): Promise<ExternalItem[]> { return browser() ? [] : invoke('list_external_items', { limit }) },
  async cleanupExternalCache(): Promise<{ ok: boolean; removed: number }> { return browser() ? { ok: true, removed: 0 } : invoke('cleanup_external_cache') },
  async captureLink(url: string): Promise<RecordData> { if (browser()) return this.save('inbox', { content: url, type: 'link', sourceUrl: url, captureStatus: 'link_saved' }); return invoke('capture_link', { url }) },
  async ask(question: string, context: AgentContext, history: ChatMessage[] = []): Promise<AgentResponse> {
    if (!browser()) return invoke('ask_chief', { question, context, history, schemaRegistry, toolDefinitions: toolRegistry })
    const matches = await this.search(question); return { context: matches, answer: matches.length ? `找到 ${matches.length} 条相关本地记录。请在下一次决策前对比预期、实际时间、结果与复盘。` : '暂未找到匹配的本地记录。请先收集相关事实。' }
  },
  async confirmAiAction(actionId: string): Promise<AgentActionResult> { if (browser()) throw new Error('浏览器模式不能执行本地 AI Action。'); return invoke('execute_ai_action', { actionId }) },
  async cancelAiAction(actionId: string): Promise<AgentActionResult> { if (browser()) throw new Error('浏览器模式不能取消本地 AI Action。'); return invoke('cancel_ai_action', { actionId }) },
}

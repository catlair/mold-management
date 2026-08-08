import type { ZenModelView } from '../api'

const models = (
  protocol: string,
  entries: Array<[id: string, label: string, free?: boolean]>,
): ZenModelView[] => entries.map(([id, label, free = false]) => ({ id, label, protocol, free }))

/** OpenCode Zen 官方模型预设（2026-08-07）。 */
export const ZEN_MODEL_PRESETS: ZenModelView[] = [
  ...models('responses', [
    ['gpt-5.6-sol', 'GPT 5.6 Sol'],
    ['gpt-5.6-terra', 'GPT 5.6 Terra'],
    ['gpt-5.6-luna', 'GPT 5.6 Luna'],
    ['gpt-5.5', 'GPT 5.5'],
    ['gpt-5.5-pro', 'GPT 5.5 Pro'],
    ['gpt-5.4', 'GPT 5.4'],
    ['gpt-5.4-pro', 'GPT 5.4 Pro'],
    ['gpt-5.4-mini', 'GPT 5.4 Mini'],
    ['gpt-5.4-nano', 'GPT 5.4 Nano'],
    ['gpt-5.3-codex', 'GPT 5.3 Codex'],
    ['gpt-5.3-codex-spark', 'GPT 5.3 Codex Spark'],
    ['gpt-5.2', 'GPT 5.2'],
    ['gpt-5.2-codex', 'GPT 5.2 Codex'],
    ['gpt-5.1', 'GPT 5.1'],
    ['gpt-5.1-codex', 'GPT 5.1 Codex'],
    ['gpt-5.1-codex-max', 'GPT 5.1 Codex Max'],
    ['gpt-5.1-codex-mini', 'GPT 5.1 Codex Mini'],
    ['gpt-5', 'GPT 5'],
    ['gpt-5-codex', 'GPT 5 Codex'],
    ['gpt-5-nano', 'GPT 5 Nano'],
    ['grok-4.5', 'Grok 4.5'],
    ['grok-build-0.1', 'Grok Build 0.1'],
  ]),
  ...models('anthropic', [
    ['claude-fable-5', 'Claude Fable 5'],
    ['claude-opus-5', 'Claude Opus 5'],
    ['claude-opus-4-8', 'Claude Opus 4.8'],
    ['claude-opus-4-7', 'Claude Opus 4.7'],
    ['claude-opus-4-6', 'Claude Opus 4.6'],
    ['claude-opus-4-5', 'Claude Opus 4.5'],
    ['claude-sonnet-5', 'Claude Sonnet 5'],
    ['claude-sonnet-4-6', 'Claude Sonnet 4.6'],
    ['claude-sonnet-4-5', 'Claude Sonnet 4.5'],
    ['claude-haiku-4-5', 'Claude Haiku 4.5'],
    ['qwen3.7-max', 'Qwen3.7 Max'],
    ['qwen3.7-plus', 'Qwen3.7 Plus'],
    ['qwen3.6-plus', 'Qwen3.6 Plus'],
    ['qwen3.5-plus', 'Qwen3.5 Plus'],
  ]),
  ...models('gemini', [
    ['gemini-3.6-flash', 'Gemini 3.6 Flash'],
    ['gemini-3.5-flash', 'Gemini 3.5 Flash'],
    ['gemini-3.5-flash-lite', 'Gemini 3.5 Flash Lite'],
    ['gemini-3.1-pro', 'Gemini 3.1 Pro'],
    ['gemini-3-flash', 'Gemini 3 Flash'],
  ]),
  ...models('openai', [
    ['deepseek-v4-pro', 'DeepSeek V4 Pro'],
    ['deepseek-v4-flash', 'DeepSeek V4 Flash'],
    ['minimax-m3', 'MiniMax M3'],
    ['minimax-m2.7', 'MiniMax M2.7'],
    ['minimax-m2.5', 'MiniMax M2.5'],
    ['glm-5.2', 'GLM 5.2'],
    ['glm-5.1', 'GLM 5.1'],
    ['glm-5', 'GLM 5'],
    ['kimi-k2.5', 'Kimi K2.5'],
    ['kimi-k2.6', 'Kimi K2.6'],
    ['kimi-k2.7-code', 'Kimi K2.7 Code'],
    ['kimi-k3', 'Kimi K3'],
    ['big-pickle', 'Big Pickle', true],
    ['mimo-v2.5-free', 'MiMo-V2.5 Free', true],
    ['laguna-s-2.1-free', 'Laguna S 2.1 Free', true],
    ['ling-3.0-tiny-free', 'Ling-3.0-tiny Free', true],
    ['longcat-2.0-free', 'LongCat-2.0 Free', true],
    ['north-mini-code-free', 'North Mini Code Free', true],
    ['nemotron-3-ultra-free', 'Nemotron 3 Ultra Free', true],
    ['deepseek-v4-flash-free', 'DeepSeek V4 Flash Free', true],
  ]),
]

export const ZEN_FREE_MODEL_PRESETS = ZEN_MODEL_PRESETS.filter(model => model.free)

/**
 * 兼容后端新对象数组与旧字符串数组；无法识别的自定义 ID 也生成可显示项。
 */
export function normalizeZenModels(value: unknown): ZenModelView[] {
  if (!Array.isArray(value)) return []
  const presetMap = new Map(ZEN_MODEL_PRESETS.map(model => [model.id, model]))
  const result: ZenModelView[] = []
  const seen = new Set<string>()
  for (const item of value) {
    let model: ZenModelView | null = null
    if (typeof item === 'string') {
      const id = item.trim()
      if (id) model = presetMap.get(id) ?? { id, label: id, protocol: 'openai', free: id.endsWith('-free') }
    } else if (item && typeof item === 'object') {
      const source = item as Partial<ZenModelView>
      const id = typeof source.id === 'string' ? source.id.trim() : ''
      if (id) {
        const preset = presetMap.get(id)
        model = {
          id,
          label: typeof source.label === 'string' && source.label.trim() ? source.label.trim() : preset?.label ?? id,
          protocol: typeof source.protocol === 'string' && source.protocol ? source.protocol : preset?.protocol ?? 'openai',
          free: typeof source.free === 'boolean' ? source.free : preset?.free ?? id.endsWith('-free'),
        }
      }
    }
    if (model && !seen.has(model.id)) {
      seen.add(model.id)
      result.push(model)
    }
  }
  return result
}

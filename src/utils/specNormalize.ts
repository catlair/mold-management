/**
 * 规格/尺寸文本解析器：输入即归一的核心。
 *
 * 职责分层：
 * 1. preprocess —— Unicode NFKC、符号归一、空白压缩
 * 2. parse —— 识别语义（乘号组合 / 范围 / 对称公差 / 非对称公差 / 纯数字 / 编码名 / 普通文本）
 * 3. format —— 生成规范显示值（写 Excel 原字段）
 * 4. key —— 生成规范键（搜索、去重、排序用）
 * 5. interval —— 换算公差带区间（相似规格判断用）
 */

export type SpecKind =
  | 'pair'       // 4.2 × 13
  | 'range'      // 7.7 ~ 8.1
  | 'tolerance'  // 8 ± 0.5 / 8 +0.05 / -0.02 / 8 -0.5
  | 'plain'      // 3.5
  | 'code'       // M4.2-18×16、M6-1.0*16三角牙（不解析，仅符号统一）
  | 'text'       // 无法解析的普通文本（保持原文）

export interface ParsedSpec {
  kind: SpecKind
  /** 规范显示值（保存进原字段） */
  display: string
  /** 规范键（搜索/去重/排序） */
  key: string
  /** 前缀，如 W=、介、Φ */
  prefix?: string
  /** 后缀，如 牙10、束、割尾 */
  suffix?: string
  /** 解析警告（如范围起点大于终点），不阻断保存 */
  warning?: string

  // pair
  a?: number
  b?: number
  // range
  from?: number
  to?: number
  // tolerance
  nominal?: number
  upper?: number
  lower?: number
  // plain
  value?: number
}

const NUM = String.raw`\d+(?:\.\d+)?`
const SIGNED_NUM = String.raw`[+-]?\d+(?:\.\d+)?`
// 分隔符：乘号 / 范围 / 公差 / 正负 / 分数
const SEP = String.raw`[±~×*xX+/\-−‐]`
const BODY_SOURCE = String.raw`${SIGNED_NUM}(?:\s*${SEP}\s*[+-]?${NUM})*`

/** 前缀识别：Φ/φ/∅、W=/D=/d= 等、介/垫（后跟数字） */
const PREFIX_RE = new RegExp(
  String.raw`^(?:[Φφ∅]|(?:[A-Za-z]{1,3}\s*[=：:]\s*)|(?:介|垫)\s*)+`,
)

function fmtNumber(value: number, maxDecimals = 3): string {
  if (!Number.isFinite(value)) return String(value)
  // 消除浮点误差并去掉多余尾零：5.199999999999999 -> 5.2、8.0 -> 8
  const scale = 10 ** maxDecimals
  const rounded = Math.round(value * scale) / scale
  let text = String(rounded)
  if (text.includes('.')) {
    text = text.replace(/\.?0+$/, '')
  }
  return text
}

function signText(value: number): string {
  return value >= 0 ? `+${fmtNumber(value)}` : `-${fmtNumber(Math.abs(value))}`
}

/** 预处理：NFKC 归一、全角符号转半角、数学负号转连字符、空白压缩、乘号统一为 × */
export function preprocessSpecText(raw: string): string {
  let text = (raw ?? '').normalize('NFKC')
  text = text
    .replace(/[～〜]/g, '~')
    .replace(/−/g, '-')
    .replace(/‐/g, '-')
    .replace(/＋/g, '+')
    .replace(/－/g, '-')
    .replace(/＊/g, '*')
    .replace(/[×xX*]/g, '×')
  text = text.replace(/\s+/g, ' ').trim()
  return text
}

/**
 * 从文本中切出 前缀 / 主体 / 后缀。
 * 主体 = 由数字和 ±×~+-/ 组成的连续数值表达式（符号前后允许空格）。
 * whole=true 表示剩余文本整体不可拆（复合编码名），不做数值解析。
 */
function splitParts(preprocessed: string): {
  prefix: string
  body: string
  suffix: string
  whole: boolean
} {
  let rest = preprocessed
  let prefix = ''
  const prefixMatch = rest.match(PREFIX_RE)
  if (prefixMatch) {
    prefix = prefixMatch[0].trim()
    rest = rest.slice(prefixMatch[0].length).trim()
  }

  const bodyMatch = rest.match(new RegExp(`^(${BODY_SOURCE})`))
  if (!bodyMatch || bodyMatch[1] === '') {
    return { prefix, body: '', suffix: '', whole: true }
  }
  const body = bodyMatch[1]
  const suffix = rest.slice(body.length).trim()
  if (suffix) {
    // 只有"说明性文本"（含字母或中文，如 牙10、束、割尾、D/W）才算真后缀；
    // 后缀仍是数值/符号内容（如 "6390 3.5 × 38" 的 "3.5 × 38"）说明是复合编码名，整体按原文处理
    if (/[A-Za-z\u4e00-\u9fff]/.test(suffix)) {
      return { prefix, body, suffix, whole: false }
    }
    return { prefix, body: '', suffix: '', whole: true }
  }
  return { prefix, body, suffix: '', whole: false }
}

interface Token {
  op: '' | '+' | '-' | '±' | '~' | '×' | '/'
  num: number
}

/** 把主体字符串拆成 token 序列（首段 op 为 ''，后续段带分隔符） */
function tokenizeBody(body: string): Token[] {
  const tokens: Token[] = []
  let rest = body
  const first = rest.match(new RegExp(`^([+-]?)(\\d+(?:\\.\\d+)?)`))
  if (!first) return tokens
  tokens.push({ op: '', num: parseFloat(`${first[1]}${first[2]}`) })
  rest = rest.slice(first[0].length)

  const tail = new RegExp(`^\\s*([±~×+/\\-])\\s*([+-]?)(\\d+(?:\\.\\d+)?)`)
  while (rest) {
    const match = rest.match(tail)
    if (!match) break
    tokens.push({
      op: match[1] as Token['op'],
      num: parseFloat(`${match[2] === '-' ? '-' : ''}${match[3]}`),
    })
    rest = rest.slice(match[0].length)
  }
  return tokens
}

function buildDisplay(parsed: ParsedSpec): string {
  // code/text：display 直接是原文，不重复拼前缀/后缀
  if (parsed.kind === 'code' || parsed.kind === 'text') {
    return parsed.display
  }
  const prefix = parsed.prefix ? `${parsed.prefix} ` : ''
  const suffix = parsed.suffix ? ` ${parsed.suffix}` : ''
  let body = ''
  switch (parsed.kind) {
    case 'pair':
      body = `${fmtNumber(parsed.a as number)} × ${fmtNumber(parsed.b as number)}`
      break
    case 'range':
      body = `${fmtNumber(parsed.from as number)} ~ ${fmtNumber(parsed.to as number)}`
      break
    case 'tolerance': {
      const nominal = fmtNumber(parsed.nominal as number)
      const upper = parsed.upper as number
      const lower = parsed.lower as number
      if (upper === 0 && lower === 0) {
        body = nominal
      } else if (upper === 0) {
        body = `${nominal} ${signText(lower)}`
      } else if (lower === 0) {
        body = `${nominal} ${signText(upper)}`
      } else if (Math.abs(upper) === Math.abs(lower)) {
        body = `${nominal} ± ${fmtNumber(Math.abs(upper))}`
      } else {
        body = `${nominal} ${signText(upper)} / ${signText(lower)}`
      }
      break
    }
    case 'plain':
      body = fmtNumber(parsed.value as number)
      break
    default:
      body = parsed.display
  }
  return `${prefix}${body}${suffix}`
}

function buildKey(parsed: ParsedSpec): string {
  if (parsed.kind === 'code' || parsed.kind === 'text') {
    return parsed.display.replace(/\s+/g, '').replace(/×/g, 'x').toLowerCase()
  }
  const prefix = parsed.prefix ? parsed.prefix.replace(/\s+/g, '') : ''
  const suffix = parsed.suffix ? parsed.suffix.replace(/\s+/g, '') : ''
  let body = ''
  switch (parsed.kind) {
    case 'pair':
      body = `${fmtNumber(parsed.a as number)}x${fmtNumber(parsed.b as number)}`
      break
    case 'range':
      body = `${fmtNumber(parsed.from as number)}~${fmtNumber(parsed.to as number)}`
      break
    case 'tolerance': {
      const nominal = fmtNumber(parsed.nominal as number)
      const upper = parsed.upper as number
      const lower = parsed.lower as number
      if (upper === 0 && lower === 0) {
        body = nominal
      } else if (upper === 0) {
        body = `${nominal}${signText(lower)}`
      } else if (lower === 0) {
        body = `${nominal}${signText(upper)}`
      } else if (Math.abs(upper) === Math.abs(lower)) {
        body = `${nominal}±${fmtNumber(Math.abs(upper))}`
      } else {
        body = `${nominal}${signText(upper)}/${signText(lower)}`
      }
      break
    }
    case 'plain':
      body = fmtNumber(parsed.value as number)
      break
    default:
      body = parsed.display
  }
  return `${prefix}${body}${suffix}`.toLowerCase()
}

/**
 * 解析规格文本。
 * @param raw 原始输入（允许任意宽松写法）
 */
export function parseSpecText(raw: string): ParsedSpec {
  const preprocessed = preprocessSpecText(raw)
  if (!preprocessed) {
    return { kind: 'text', display: '', key: '' }
  }

  const { prefix, body, suffix, whole } = splitParts(preprocessed)
  if (!body || whole) {
    // 整体不可拆（复合编码名 / 无数值表达式）：保留原文，仅符号统一
    const kind: SpecKind = /[A-Za-z#]|[\u4e00-\u9fff]/.test(preprocessed) ? 'code' : 'text'
    return {
      kind,
      display: preprocessed,
      key: preprocessed.replace(/\s+/g, '').replace(/×/g, 'x').toLowerCase(),
      prefix,
    }
  }

  const tokens = tokenizeBody(body)
  const parsed: ParsedSpec = { kind: 'text', display: preprocessed, key: preprocessed, prefix, suffix }

  if (tokens.length === 1) {
    parsed.kind = 'plain'
    parsed.value = tokens[0].num
  } else if (tokens.length === 2) {
    const op = tokens[1].op
    const first = tokens[0].num
    const second = tokens[1].num
    if (op === '×') {
      parsed.kind = 'pair'
      parsed.a = first
      parsed.b = second
    } else if (op === '~') {
      parsed.kind = 'range'
      parsed.from = first
      parsed.to = second
      if (first > second) {
        parsed.warning = `范围起点 ${fmtNumber(first)} 大于终点 ${fmtNumber(second)}`
      }
    } else if (op === '±') {
      parsed.kind = 'tolerance'
      parsed.nominal = first
      parsed.upper = Math.abs(second)
      parsed.lower = -Math.abs(second)
    } else if (op === '/') {
      parsed.kind = 'text'
    } else if (op === '+' || op === '-') {
      // 第二数 > 第一数 -> 范围；否则单边公差
      if (second > first) {
        parsed.kind = 'range'
        parsed.from = first
        parsed.to = second
      } else if (op === '-') {
        parsed.kind = 'tolerance'
        parsed.nominal = first
        parsed.upper = 0
        parsed.lower = -Math.abs(second)
      } else {
        parsed.kind = 'tolerance'
        parsed.nominal = first
        parsed.upper = Math.abs(second)
        parsed.lower = 0
      }
    }
  } else if (tokens.length === 3) {
    const [t0, t1, t2] = tokens
    // 非对称公差：10+0.05/-0.02 或 10-0.02/+0.05；约定第一个偏差为上偏差，第二个为下偏差。
    // t1 的符号由 op 表达（- 分隔符即负偏差），t2 的符号在 num 里（/ 后跟 +/-）
    if ((t1.op === '+' || t1.op === '-') && t2.op === '/') {
      parsed.kind = 'tolerance'
      parsed.nominal = t0.num
      parsed.upper = t1.op === '-' ? -Math.abs(t1.num) : Math.abs(t1.num)
      parsed.lower = t2.num
    } else {
      parsed.kind = 'text'
    }
  } else {
    parsed.kind = 'text'
  }

  // 公差带校验：上极限 ≥ 下极限
  if (parsed.kind === 'tolerance' && parsed.nominal !== undefined) {
    const upperLimit = parsed.nominal + (parsed.upper as number)
    const lowerLimit = parsed.nominal + (parsed.lower as number)
    if (upperLimit < lowerLimit) {
      parsed.warning = `公差带无效：上极限 ${fmtNumber(upperLimit)} 小于下极限 ${fmtNumber(lowerLimit)}`
    }
  }

  // 无法解析为数值语义（含字母或中文修饰）时降级为 code：保留原文，仅符号统一
  if (parsed.kind === 'text') {
    if (/[A-Za-z#]|[\u4e00-\u9fff]/.test(preprocessed)) {
      parsed.kind = 'code'
    }
  }

  parsed.display = buildDisplay(parsed)
  parsed.key = buildKey(parsed)
  return parsed
}

/**
 * 公差带区间换算（排序/展示用）。
 * - plain/pair：单点区间
 * - range：[from, to]
 * - tolerance：[nominal + lower, nominal + upper]
 * - code/text：null
 */
export function specInterval(parsed: ParsedSpec): [number, number] | null {
  switch (parsed.kind) {
    case 'plain':
      return [parsed.value as number, parsed.value as number]
    case 'pair': {
      const a = parsed.a as number
      const b = parsed.b as number
      return [Math.min(a, b), Math.max(a, b)]
    }
    case 'range':
      return [parsed.from as number, parsed.to as number]
    case 'tolerance': {
      const nominal = parsed.nominal as number
      const lower = parsed.lower as number
      const upper = parsed.upper as number
      return [nominal + lower, nominal + upper]
    }
    default:
      return null
  }
}

/** 两个区间是否相交（含端点接触） */
export function intervalsOverlap(a: [number, number], b: [number, number]): boolean {
  return Math.max(a[0], b[0]) <= Math.min(a[1], b[1])
}

/** 获取主要名义值（相似提示描述用） */
export function primaryNominal(parsed: ParsedSpec): number | null {
  switch (parsed.kind) {
    case 'plain':
      return parsed.value ?? null
    case 'pair':
      return parsed.a ?? null
    case 'range':
      return parsed.from ?? null
    case 'tolerance':
      return parsed.nominal ?? null
    default:
      return null
  }
}

/** 规范区间描述（展示用）：如 [7.5, 8.5] */
export function intervalText(parsed: ParsedSpec): string {
  const interval = specInterval(parsed)
  if (!interval) return ''
  return `[${fmtNumber(interval[0])}, ${fmtNumber(interval[1])}]`
}

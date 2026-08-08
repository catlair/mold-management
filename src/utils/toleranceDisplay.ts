import { parseSpecText } from './specNormalize'

export interface ToleranceDisplayView {
  isAsymmetric: boolean
  fullText: string
  prefix: string
  nominal: string
  upper: string
  lower: string
  suffix: string
}

function formatNumber(value: number): string {
  return String(Math.round(value * 1000) / 1000)
}

function signed(value: number): string {
  return value >= 0 ? `+${formatNumber(value)}` : `-${formatNumber(Math.abs(value))}`
}

export function buildToleranceDisplay(value: unknown): ToleranceDisplayView {
  const raw = value == null ? '' : String(value).trim()
  const parsed = parseSpecText(raw)
  const isAsymmetric = parsed.kind === 'tolerance'
    && parsed.nominal !== undefined
    && parsed.upper !== undefined
    && parsed.lower !== undefined
    && parsed.upper !== 0
    && parsed.lower !== 0
    && Math.abs(parsed.upper) !== Math.abs(parsed.lower)
  return {
    isAsymmetric,
    fullText: parsed.display || raw,
    prefix: parsed.prefix || '',
    nominal: parsed.nominal === undefined ? '' : formatNumber(parsed.nominal),
    upper: parsed.upper === undefined ? '' : signed(parsed.upper),
    lower: parsed.lower === undefined ? '' : signed(parsed.lower),
    suffix: parsed.suffix || '',
  }
}

/** 给打印测量探针创建与实际打印区一致的不对称公差 DOM。 */
export function appendToleranceContent(container: HTMLElement, value: unknown) {
  const view = buildToleranceDisplay(value)
  if (!view.isAsymmetric) {
    container.textContent = view.fullText
    return
  }
  const wrapper = document.createElement('span')
  wrapper.className = 'print-tolerance'
  wrapper.title = view.fullText
  if (view.prefix) {
    const prefix = document.createElement('span')
    prefix.className = 'pt-affix'
    prefix.textContent = view.prefix
    wrapper.appendChild(prefix)
  }
  const nominal = document.createElement('span')
  nominal.className = 'pt-nominal'
  nominal.textContent = view.nominal
  wrapper.appendChild(nominal)

  const deviations = document.createElement('span')
  deviations.className = 'pt-deviations'
  const upper = document.createElement('span')
  upper.className = 'pt-upper'
  upper.textContent = view.upper
  const lower = document.createElement('span')
  lower.className = 'pt-lower'
  lower.textContent = view.lower
  deviations.append(upper, lower)
  wrapper.appendChild(deviations)

  if (view.suffix) {
    const suffix = document.createElement('span')
    suffix.className = 'pt-affix'
    suffix.textContent = view.suffix
    wrapper.appendChild(suffix)
  }
  container.appendChild(wrapper)
}

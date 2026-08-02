import { toFullName } from './punchName'

export type DuplicateKind = 'exact' | 'similar'

export interface DuplicateMatch {
  kind: DuplicateKind
  record: any
  differingFields: string[]
}

function normalizeWidth(value: string): string {
  return [...value].map(char => {
    if (char === '\u3000') return ' '
    const code = char.charCodeAt(0)
    return code >= 0xFF01 && code <= 0xFF5E
      ? String.fromCharCode(code - 0xFEE0)
      : char
  }).join('')
}

export function normalizeText(value: unknown): string {
  return normalizeWidth(String(value ?? ''))
    .trim()
    .toLowerCase()
    .replace(/[，、]/g, ',')
    .replace(/[（）]/g, match => match === '（' ? '(' : ')')
    .replace(/\s+/g, '')
}

export function normalizeDimension(value: unknown): string {
  const normalized = normalizeText(value)
    .replace(/[×＊*]/g, 'x')
    .replace(/φ/g, '')
    .replace(/(?:毫米|mm)$/i, '')

  if (/^[+-]?\d+(?:\.\d+)?$/.test(normalized)) {
    const number = Number(normalized)
    return Number.isFinite(number) ? String(number) : normalized
  }

  return normalized
}

export function normalizePunchName(value: unknown): string {
  const raw = normalizeWidth(String(value ?? '')).trim()
  const fullName = toFullName(raw) || raw
  return normalizeText(fullName)
}

export function punchUniqueKey(record: any): string {
  return [
    normalizePunchName(record.name),
    normalizeDimension(record.spec),
    normalizeText(record.material),
  ].join('|')
}

export function dieUniqueKey(record: any): string {
  return [
    normalizeText(record.name),
    normalizeText(record.machineType),
    normalizeDimension(record.wireDiameter),
  ].join('|')
}

export function findPunchDuplicate(candidate: any, rows: any[], excludeId = ''): DuplicateMatch | null {
  const available = rows.filter(row => String(row.id ?? '') !== excludeId)
  const exact = available.find(row => punchUniqueKey(row) === punchUniqueKey(candidate))
  if (exact) return { kind: 'exact', record: exact, differingFields: [] }

  const sameNameAndSpec = available.find(row =>
    normalizePunchName(row.name) === normalizePunchName(candidate.name) &&
    normalizeDimension(row.spec) === normalizeDimension(candidate.spec)
  )
  if (sameNameAndSpec) {
    return { kind: 'similar', record: sameNameAndSpec, differingFields: ['材质'] }
  }

  const sameName = available.find(row => normalizePunchName(row.name) === normalizePunchName(candidate.name))
  if (sameName) {
    const differingFields = [
      normalizeDimension(sameName.spec) !== normalizeDimension(candidate.spec) ? '规格' : '',
      normalizeText(sameName.material) !== normalizeText(candidate.material) ? '材质' : '',
    ].filter(Boolean)
    return { kind: 'similar', record: sameName, differingFields }
  }

  return null
}

export function findDieDuplicate(candidate: any, rows: any[], excludeId = ''): DuplicateMatch | null {
  const available = rows.filter(row => String(row.id ?? '') !== excludeId)
  const exact = available.find(row => dieUniqueKey(row) === dieUniqueKey(candidate))
  if (exact) return { kind: 'exact', record: exact, differingFields: [] }

  const sameNameAndMachine = available.find(row =>
    normalizeText(row.name) === normalizeText(candidate.name) &&
    normalizeText(row.machineType) === normalizeText(candidate.machineType)
  )
  if (sameNameAndMachine) {
    return { kind: 'similar', record: sameNameAndMachine, differingFields: ['线径'] }
  }

  const sameName = available.find(row => normalizeText(row.name) === normalizeText(candidate.name))
  if (sameName) {
    const differingFields = [
      normalizeText(sameName.machineType) !== normalizeText(candidate.machineType) ? '机型' : '',
      normalizeDimension(sameName.wireDiameter) !== normalizeDimension(candidate.wireDiameter) ? '线径' : '',
    ].filter(Boolean)
    return { kind: 'similar', record: sameName, differingFields }
  }

  return null
}

export function isDuplicateError(error: unknown): boolean {
  return String(error ?? '').includes('DUPLICATE_RECORD|')
}

export function duplicateErrorMessage(error: unknown): string {
  const raw = String(error ?? '')
  const [, resource = '记录', existingId = ''] = raw.split('|')
  return existingId
    ? `已存在相同${resource}（编号：${existingId}），请勿重复添加`
    : `已存在相同${resource}，请勿重复添加`
}

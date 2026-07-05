// 冲头命名：简写 ↔ 全写 双向转换
// 简写格式：数字+字母，如 30R, 26P, 50B
// 全写格式：JM+字母+空格+M+数字，如 JMR M30, JMP M26, JMB M50

// 尝试简写→全写：30R → JMR M30, 30R特 → JMR M30 特
export function toFullName(shortCode: string): string | null {
  const trimmed = shortCode.trim()
  const match = trimmed.match(/^(\d+)([PBTFXRpbtfxr])(特|)$/i)
  if (!match) return null
  const [, num, letter, suffix] = match
  return `JM${letter.toUpperCase()} M${num}${suffix ? ' ' + suffix : ''}`
}

// 尝试全写→简写：JMR M30 → 30R, JMR M30 特 → 30R特
export function toShortCode(fullName: string): string | null {
  const match = fullName.trim().match(/^JM([PBTFXRpbtfxr])\s*M(\d+)(\s*特)?$/i)
  if (!match) return null
  const [, letter, num, suffix] = match
  return `${num}${letter}${suffix ? suffix.trim() : ''}`
}

// 判断两个冲头名称是否匹配（兼容简写和全写）
export function matchPunchNames(a: string, b: string): boolean {
  if (!a || !b) return false
  const na = a.trim()
  const nb = b.trim()
  if (na === nb) return true
  // a是简写，b是全写
  const aFull = toFullName(na)
  if (aFull && aFull === nb) return true
  // a是全写，b是简写
  const bFull = toFullName(nb)
  if (bFull && a === bFull) return true
  // 互相转
  const aShort = toShortCode(na)
  const bShort = toShortCode(nb)
  if (aShort && bShort && aShort === bShort) return true
  return false
}

// 给一个名字，返回对应的简写（如果有的话）
export function getShortCode(name: string): string | null {
  return toShortCode(name) || (toFullName(name) ? name : null)
}

// 给一个名字，返回对应的全写（如果有的话）
export function getFullName(name: string): string | null {
  return toFullName(name) || (toShortCode(name) ? name : null)
}

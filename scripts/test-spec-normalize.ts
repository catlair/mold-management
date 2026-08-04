import {
  parseSpecText,
  specInterval,
  intervalText,
} from '../src/utils/specNormalize.ts'

const cases: string[] = [
  // 乘号（名称列真实写法）
  '4.2 X 13',
  '2.6  x 12',
  '3 X10',
  '4.8 X16',
  '6390 3.5 X 38',
  '1 * 1',
  '1* 1',
  // 范围
  '7.7~8.1',
  '1.9-2.0',
  '3.6-4',
  '9.1~9.3',
  // 对称公差
  '8±0.5',
  '8 ±0.1',
  '5.3 ±0.1',
  '3.65±0.2',
  // 单边公差
  '8.0-0.5',
  '11-0.43',
  '13-0.8',
  '4.1+1',
  '44.4+0.1',
  '46.0-0.1',
  '12+1.5',
  // 前缀/后缀
  'W=12-0.5',
  'W=9.2±0.8',
  '介厚1',
  '介 0.8±0.1',
  '22-0.2 牙10',
  // 编码名（不解析）
  'M4.2-18*16',
  'M4.2-18X16',
  'M2.5-0.45B',
  'M2.6-28*11.8束',
  'M3-24*8割尾',
  'M6-1.0*16三角牙',
  'M4.2-14*19 D/W',
  // 非对称公差（上负下正）
  '10+0.05/-0.02',
  '10-0.02/+0.05',
  // 普通数字
  '3.5',
  '8',
  '13',
]

console.log('===== 解析用例 =====')
for (const input of cases) {
  const p = parseSpecText(input)
  console.log(
    `${input.padEnd(22)} -> kind=${p.kind.padEnd(9)} display=${JSON.stringify(p.display).padEnd(26)} key=${p.key.padEnd(20)} interval=${intervalText(p).padEnd(14)}${p.warning ? ' WARN: ' + p.warning : ''}`,
  )
}

console.log()
console.log('===== 重复判断（规则A：名称+头型+牙型相同；规则B：外部ID+客户相同且非空） =====')

function specFieldKey(value: unknown): string {
  if (value === undefined || value === null) return ''
  return parseSpecText(String(value)).key
}

function duplicateReason(a: Record<string, any>, b: Record<string, any>): string {
  const nameA = specFieldKey(a.name)
  const nameB = specFieldKey(b.name)
  const headA = specFieldKey(a.headType)
  const headB = specFieldKey(b.headType)
  const threadA = specFieldKey(a.threadType)
  const threadB = specFieldKey(b.threadType)
  if ((nameA || headA || threadA) && nameA === nameB && headA === headB && threadA === threadB) {
    return '规则A：名称/头型/牙型相同'
  }
  const extA = specFieldKey(a.externalId)
  const extB = specFieldKey(b.externalId)
  const customerA = specFieldKey(a.customer)
  const customerB = specFieldKey(b.customer)
  if (extA && extB && customerA && customerB && extA === extB && customerA === customerB) {
    return '规则B：外部ID/客户相同'
  }
  return ''
}

const base = {
  name: '4.2 X 13', headType: '平头', threadType: '自攻',
  headSize: '8', headHeight: '2.3', length: '13-0.8',
  threadDiameter: '4.22-0.18', shankLength: '10',
  wireMaterial: '1018', plating: '彩锌', customer: '', externalId: '',
}

const same = { ...base }
const differentTolerance = { ...base, length: '13±0.8' }       // 公差不同但名称头型牙型相同 -> 规则A提示
const differentHeadType = { ...base, headType: '圆头' }          // 头型不同 -> 不提示
const differentCustomer = { ...base, customer: '客户B' }         // 客户不同（外部ID空）-> 规则A仍提示
const differentWire = { ...base, wireMaterial: '1022' }          // 线材不同 -> 规则A仍提示
const differentName = { ...base, name: '4.8 X 16' }              // 名称不同、外部ID客户空 -> 不提示

// 规则 B 场景：外部ID+客户相同（名称不同也提示）；外部ID留空不算
const b1 = { ...base, name: '4.2 X 13', externalId: 'A-001', customer: '客户A' }
const b2 = { ...base, name: '4.8 X 16', externalId: 'A-001', customer: '客户A' }
const b3 = { ...base, name: '4.2 X 13', externalId: '', customer: '客户A' }   // 外部ID留空 -> 规则B不算
const b4 = { ...base, name: '4.2 X 13', externalId: 'A-001', customer: '' }   // 客户留空 -> 规则B不算
const b5 = { ...base, name: '4.8 X 16', externalId: 'A-002', customer: '客户A' } // 外部ID不同 -> 不提示

console.log('base vs same                 :', duplicateReason(base, same) || '不提示')
console.log('base vs differentTolerance   :', duplicateReason(base, differentTolerance) || '不提示', '（公差不同但名称头型牙型相同 → 提示，可强制保存）')
console.log('base vs differentHeadType    :', duplicateReason(base, differentHeadType) || '不提示（头型不同）')
console.log('base vs differentCustomer    :', duplicateReason(base, differentCustomer) || '不提示', '（外部ID空，仅客户不同 → 规则A仍提示）')
console.log('base vs differentWire        :', duplicateReason(base, differentWire) || '不提示', '（线材不同 → 规则A仍提示）')
console.log('base vs differentName        :', duplicateReason(base, differentName) || '不提示（名称不同且外部ID客户空）')
console.log('b1 vs b2 (外部ID客户相同)      :', duplicateReason(b1, b2) || '不提示')
console.log('b1 vs b3 (外部ID留空)         :', duplicateReason(b1, b3) || '不提示（外部ID留空不算）')
console.log('b1 vs b4 (客户留空)           :', duplicateReason(b1, b4) || '不提示（客户留空不算）')
console.log('b1 vs b5 (外部ID不同)         :', duplicateReason(b1, b5) || '不提示（外部ID不同）')

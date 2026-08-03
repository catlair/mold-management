# 冲头与牙板防重复设计方案

## 结论

采用“三层防线”：

1. 表单输入规范化与即时提示。
2. 提交前基于当前列表预检查。
3. Rust/Tauri 在同一锁内执行权威判重和 Excel 写入。

前端只改善体验，Rust 后端是最终保证。完全重复必须阻止保存；疑似重复只警告，不误伤合法数据。

## 业务唯一键

### 冲头

完全重复键：

```text
normalizePunchName(name) + normalize(spec) + normalize(material)
```

说明：
- 当前真实字段为 `name`、`spec`、`material`。
- `safetyStock` 和 `remark` 不属于身份字段，不参与判重。
- 名称需先兼容简写/全写：`30R` 与 `JMR M30` 应视为同一名称。
- 材质为空时仍参与键构造，但“同名称 + 同规格 + 一个有材质、一个无材质”应列为疑似重复，不直接阻止。

### 牙板

完全重复键：

```text
normalize(name) + normalize(machineType) + normalizeMeasurement(wireDiameter)
```

说明：
- 当前真实字段为 `name`、`machineType`、`wireDiameter`。
- `safetyStock` 和 `remark` 不参与判重。
- `14`、`14.0`、`Φ14`、`φ14 mm` 是否等价，应由统一规格规范化处理；建议都归一为 `14`。

## 重复等级

### 完全重复：强制阻止

唯一键所有字段规范化后完全一致。提示示例：

```text
已存在相同牙板：YB260802... / 名称 / 机型 / 线径
请编辑原记录，不要重复新增。
```

### 疑似重复：警告但允许继续

- 名称相同，规格字段只有部分不同。
- 冲头名称和规格相同，但一个材质为空。
- 牙板名称和机型相同，但线径格式无法可靠归一。

建议展示已有记录差异，并提供“返回修改”和“仍然新增”两个动作。

## 规范化规则

所有规则必须在 TypeScript 和 Rust 中保持一致，最好用同一组测试用例约束。

通用规则：

1. 去除首尾空白。
2. 连续空白压缩为一个空格。
3. 英文字母统一为大写。
4. Unicode NFKC 归一化，全角字符转半角。
5. `×`、`X`、`x` 统一为 `X`。
6. 中文括号、逗号等常见标点统一为 ASCII 形式。
7. 规格字段去掉无意义空格。
8. 数值测量字段去掉 `Φ`、`φ`、`MM` 等展示单位，并去除无意义小数尾零。

冲头名称额外规则：

- 复用现有命名语义，把 `30R` 统一转换为 `JMR M30` 后再判重。
- `JMRM30`、`JMR M30` 应归一成同一个键。

规范化只用于比较，不应静默覆盖用户原始显示文本；保存时可以仅做安全的 trim 和既有冲头简写转全写。

## 前端交互

在 Punch.vue 和 Die.vue 中：

1. 监听身份字段，300ms 防抖。
2. 字段齐全后，在已加载列表中查找完全重复和疑似重复。
3. 编辑时排除当前 `id`。
4. 完全重复时在表单中显示红色冲突卡片，并禁用确定按钮。
5. 卡片展示已有记录的 ID 和完整规格，可提供“关闭并定位原记录”。
6. 提交前再同步检查一次，避免用户在防抖完成前点击保存。
7. 保存按钮增加 loading，阻止同一窗口重复点击。
8. 捕获后端 `DUPLICATE_RECORD`，展示后端返回的真实冲突记录，而不是笼统提示“添加失败”。

前端列表可能过期，因此不能仅凭前端检查决定允许写入。

## Rust/Tauri 权威校验

推荐在 `excel.rs` 的新增、更新写入路径中实现，而不是仅在 Vue 中实现。

流程：

```text
add_record / update_record
  -> 持有 AppState.file_path Mutex
  -> 读取目标 Sheet
  -> 构造规范化唯一键
  -> 查找冲突（更新时排除当前 ID）
  -> 无冲突才写回 Excel
```

当前 `add_record` 和 `update_record` 从取得 `file_path` 锁到 `excel::add_row/update_row` 返回前一直持锁，因此只要把判重放入 `add_row/update_row` 的“读取后、写入前”，检查与写入就是同一临界区，可以抵抗双击、快速连续提交和同进程多窗口并发。

建议新增：

```rust
fn normalize_identity_part(value: &str) -> String;
fn normalize_measurement(value: &str) -> String;
fn build_unique_key(sheet_name: &str, row: &HashMap<String, String>) -> Option<String>;
fn find_duplicate(
    sheet_name: &str,
    rows: &[HashMap<String, String>],
    candidate: &HashMap<String, String>,
    exclude_id: Option<&str>,
) -> Option<&HashMap<String, String>>;
```

只对 `冲头信息表` 和 `牙板信息表` 启用主记录唯一性，避免误伤入库、领用等允许重复发生的业务记录。

## 冲突错误

当前命令返回 `Result<_, String>`，第一阶段可返回带稳定前缀的 JSON 字符串或稳定编码字符串：

```json
{
  "code": "DUPLICATE_RECORD",
  "entity": "die",
  "existingId": "YB...",
  "message": "已存在相同牙板",
  "matchedFields": ["name", "machineType", "wireDiameter"]
}
```

更稳妥的后续方案是定义可序列化 `AppError`，统一所有 Tauri 命令错误结构。

## 历史重复数据治理

上线强制判重前先扫描现有 Excel：

1. 对冲头和牙板主表生成规范化唯一键。
2. 按键分组，列出记录数大于 1 的重复组。
3. 只生成报告，不自动删除。
4. 用户选择保留的主记录 ID。
5. 将入库、领用、库存和关联表中的外键迁移到保留 ID。
6. 重新计算库存并复核关联数据。
7. 备份 Excel 后，再删除重复主记录。

不能直接删重复行，否则历史库存和螺丝关联可能悬空。

## 测试清单

### Rust 单元测试

- 冲头 `30R` 与 `JMR M30` 判为同名。
- 大小写、全半角、连续空格不影响判重。
- 牙板 `14`、`14.0`、`Φ14 mm` 按规则判为同线径。
- 新增完全重复返回 `DUPLICATE_RECORD`。
- 编辑未改变唯一键允许保存。
- 编辑成另一条记录的唯一键时被阻止。
- 更新排除自身 ID。

### 前端与浏览器 Mock

- 输入完整身份字段后出现即时冲突提示。
- 完全重复时确定按钮不可用。
- 疑似重复允许用户确认后继续。
- 后端冲突错误展示已有记录信息。
- 连续点击确定只发出一次请求。

### 并发验证

并行提交两次相同记录：一个成功，一个返回 `DUPLICATE_RECORD`，Excel 最终只增加一行。

## 推荐实施顺序

1. 建立 TypeScript/Rust 共享测试样例，锁定规范化语义。
2. 实现 Rust 唯一键和锁内强制校验。
3. 调整 API 错误解析和浏览器 Mock 行为。
4. 实现 Punch.vue、Die.vue 即时提示与提交状态。
5. 扫描真实 Excel 的历史重复数据，只输出报告。
6. 完成 Rust、TypeScript、构建和并发测试后提交。

## 需要业务确认的唯一问题

冲头材质是否严格区分同一种冲头：

- 若 `SKD11` 与 `高速钢` 可以作为两条独立库存，则材质必须参与唯一键（本方案默认）。
- 若材质只是备注属性，同名称同规格只能有一条，则冲头唯一键应改为 `名称 + 规格`，材质仅作为疑似重复差异展示。

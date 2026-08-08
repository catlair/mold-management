use reqwest::blocking::{Client, RequestBuilder};
use serde_json::{json, Value};
use std::time::Duration;

pub struct AgentReply {
    pub answer: String,
    /// 多条变更（批量修正等）；空表示无变更
    pub changes: Vec<Value>,
    /// 思考过程（推理模型返回 reasoning_content，可能为空）
    pub reasoning: Option<String>,
    pub raw: String,
}

fn openai_url(endpoint: &str) -> String {
    let trimmed = endpoint.trim().trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else {
        format!("{}/chat/completions", trimmed)
    }
}

fn anthropic_url(endpoint: &str) -> String {
    let trimmed = endpoint.trim().trim_end_matches('/');
    if trimmed.ends_with("/messages") {
        trimmed.to_string()
    } else if trimmed.ends_with("/v1") {
        format!("{}/messages", trimmed)
    } else {
        format!("{}/v1/messages", trimmed)
    }
}

fn gemini_url(endpoint: &str, model: &str) -> String {
    let trimmed = endpoint.trim().trim_end_matches('/');
    if trimmed.contains("opencode.ai/zen/v1") {
        format!("{}/models/{}", trimmed, model.trim())
    } else {
        format!("{}/models/{}:generateContent", trimmed, model.trim())
    }
}

fn join_text_blocks(blocks: &Value, field: &str) -> Option<String> {
    let text = blocks
        .as_array()?
        .iter()
        .filter_map(|block| block.get(field).and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("");
    (!text.is_empty()).then_some(text)
}

fn extract_responses_content(response: &Value) -> Result<String, String> {
    if let Some(text) = response.get("output_text").and_then(Value::as_str) {
        if !text.trim().is_empty() {
            return Ok(text.to_string());
        }
    }
    let text = response
        .get("output")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("content").and_then(Value::as_array))
        .flatten()
        .filter_map(|block| block.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("");
    if text.trim().is_empty() {
        Err("Responses API 返回中缺少 output_text/output[].content[].text".to_string())
    } else {
        Ok(text)
    }
}

fn extract_openai_content(response: &Value) -> Result<String, String> {
    let content = response
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .ok_or_else(|| "API 返回中缺少 choices[0].message.content".to_string())?;
    content
        .as_str()
        .map(str::to_string)
        .or_else(|| join_text_blocks(content, "text"))
        .ok_or_else(|| "API 返回的 message.content 不是可识别文本".to_string())
}

/// 提取 OpenAI 兼容响应中的思考过程（reasoning_content，DeepSeek 等推理模型）。
fn extract_openai_reasoning(response: &Value) -> Option<String> {
    response
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("reasoning_content"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|text| !text.trim().is_empty())
}

fn extract_anthropic_content(response: &Value) -> Result<String, String> {
    response
        .get("content")
        .and_then(|content| join_text_blocks(content, "text"))
        .ok_or_else(|| "Anthropic API 返回中缺少 content[].text".to_string())
}

fn extract_gemini_content(response: &Value) -> Result<String, String> {
    response
        .get("candidates")
        .and_then(Value::as_array)
        .and_then(|candidates| candidates.first())
        .and_then(|candidate| candidate.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(|parts| join_text_blocks(parts, "text"))
        .ok_or_else(|| "Gemini API 返回中缺少 candidates[0].content.parts[].text".to_string())
}

fn extract_error(payload: &Value) -> String {
    payload
        .get("error")
        .and_then(|error| {
            error
                .get("message")
                .and_then(Value::as_str)
                .or_else(|| error.as_str())
        })
        .or_else(|| payload.get("message").and_then(Value::as_str))
        .unwrap_or("API 返回错误")
        .to_string()
}

fn parse_json_output(content: &str) -> Option<Value> {
    let trimmed = content.trim();
    let without_fence = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```JSON"))
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim_end_matches("```")
        .trim();
    // 第一次尝试：直接解析
    if let Ok(value) = serde_json::from_str::<Value>(without_fence) {
        return Some(value);
    }
    // 第二次尝试：修复 AI 在 answer 等中文字段内误用未转义 ASCII 双引号的常见错误。
    // 例如 AI 输出 {"answer":"原"初始库存"字已清空",...} 会破坏 JSON 边界。
    let repaired = repair_json_answer_quotes(without_fence);
    if let Ok(value) = serde_json::from_str::<Value>(&repaired) {
        return Some(value);
    }
    // 最后尝试：截取首个 { 到末尾 } 之间的内容
    let start = without_fence.find('{')?;
    let end = without_fence.rfind('}')?;
    serde_json::from_str(&without_fence[start..=end]).ok()
}

/// 修复 AI 输出中 answer 字段内未转义的 ASCII 双引号。
/// 思路：定位 "answer":" 起始后进入字符串模式，遇到 `"` 时判断其后继是否合法 JSON 边界
/// （`,` 后接下一字段名，或 `}` 表示对象结束），若是则视为字符串结束（保留 `"`），
/// 否则视为 AI 误用，按出现次序替换为成对的中文引号「」。
fn repair_json_answer_quotes(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_answer = false;
    let mut replaced = 0usize;
    let mut i = 0;
    let bytes = raw.as_bytes();
    while i < bytes.len() {
        if !in_answer {
            if raw[i..].starts_with("\"answer\":\"") {
                out.push_str("\"answer\":\"");
                i += "\"answer\":\"".len();
                in_answer = true;
                continue;
            }
            // 安全：push 单字节 char（这里 raw 是 ASCII 串片段，但稳妥起见用 char）
            let ch = raw[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        } else {
            let ch = raw[i..].chars().next().unwrap();
            if ch == '"' {
                // 看 `"` 之后跳过空白的首字符，判断是否为合法 JSON 边界
                let rest = &raw[i + 1..];
                let first = rest.trim_start().chars().next();
                if matches!(first, Some(',') | Some('}')) {
                    // 合法结束引号
                    out.push('"');
                    i += 1;
                    in_answer = false;
                } else {
                    // AI 误用 → 替换为中文引号（奇数次「，偶数次」）
                    out.push(if replaced % 2 == 0 { '「' } else { '」' });
                    replaced += 1;
                    i += 1;
                }
            } else {
                out.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    out
}

fn send_json(request: RequestBuilder, protocol: &str) -> Result<Value, String> {
    let response = request
        .send()
        .map_err(|error| format!("调用 {} API 失败: {}", protocol, error))?;
    let status = response.status();
    let payload: Value = response.json().map_err(|error| {
        format!(
            "解析 {} API 响应失败（HTTP {}）: {}",
            protocol, status, error
        )
    })?;
    if !status.is_success() {
        return Err(format!(
            "{} API 请求失败（HTTP {}）：{}",
            protocol,
            status,
            extract_error(&payload)
        ));
    }
    Ok(payload)
}

fn call_responses(
    client: &Client,
    endpoint: &str,
    model: &str,
    api_key: &str,
    system_prompt: &str,
    history: &[Value],
    user_prompt: &str,
) -> Result<(String, Option<String>), String> {
    let mut input = vec![json!({ "role": "system", "content": system_prompt })];
    input.extend(history_to_messages(history));
    input.push(json!({ "role": "user", "content": user_prompt }));
    let body = json!({
        "model": model.trim(),
        "input": input,
        "temperature": 0.1,
        "max_output_tokens": 4096
    });
    let mut request = client.post(endpoint.trim());
    if !api_key.trim().is_empty() {
        request = request.bearer_auth(api_key.trim());
    }
    let payload = send_json(
        request.header("Accept", "application/json").json(&body),
        "OpenAI Responses",
    )?;
    Ok((extract_responses_content(&payload)?, None))
}

fn call_openai(
    client: &Client,
    provider: &str,
    endpoint: &str,
    model: &str,
    api_key: &str,
    system_prompt: &str,
    history: &[Value],
    user_prompt: &str,
) -> Result<(String, Option<String>), String> {
    let mut messages = vec![json!({ "role": "system", "content": system_prompt })];
    messages.extend(history_to_messages(history));
    messages.push(json!({ "role": "user", "content": user_prompt }));
    let mut body = json!({
        "model": model.trim(),
        "temperature": 0.1,
        "max_tokens": 4096,
        "messages": messages
    });
    if provider == "glm" {
        body["thinking"] = json!({ "type": "disabled" });
        body["response_format"] = json!({ "type": "json_object" });
    }
    if provider == "deepseek" {
        body["thinking"] = json!({ "type": "disabled" });
    }
    let mut openai_request = client.post(openai_url(endpoint));
    if !api_key.trim().is_empty() {
        openai_request = openai_request.bearer_auth(api_key.trim());
    }
    let payload = send_json(
        openai_request
            .header("Accept", "application/json")
            .json(&body),
        "OpenAI 兼容",
    )?;
    let content = extract_openai_content(&payload)?;
    let reasoning = extract_openai_reasoning(&payload);
    Ok((content, reasoning))
}

/// OpenAI 兼容协议流式请求（SSE）。`on_stream` 每收到一段增量即被调用：
/// - 推理过程增量以 "r:" 前缀标识（reasoning_content）
/// - 回答正文增量直接传入原文（content）
/// 返回完整内容（与流式增量一致），最终仍按非流式相同逻辑解析 JSON。
fn call_openai_stream(
    client: &Client,
    provider: &str,
    endpoint: &str,
    model: &str,
    api_key: &str,
    system_prompt: &str,
    history: &[Value],
    user_prompt: &str,
    force_disable_thinking: bool,
    on_stream: &dyn Fn(&str),
) -> Result<(String, Option<String>), String> {
    let mut messages = vec![json!({ "role": "system", "content": system_prompt })];
    messages.extend(history_to_messages(history));
    messages.push(json!({ "role": "user", "content": user_prompt }));
    let mut body = json!({
        "model": model.trim(),
        "temperature": 0.1,
        "max_tokens": 4096,
        "stream": true,
        "messages": messages
    });
    if provider == "deepseek" || force_disable_thinking {
        body["thinking"] = json!({ "type": "disabled" });
    }
    let mut openai_stream_request = client.post(openai_url(endpoint));
    if !api_key.trim().is_empty() {
        openai_stream_request = openai_stream_request.bearer_auth(api_key.trim());
    }
    let mut response = openai_stream_request
        .header("Accept", "text/event-stream")
        .json(&body)
        .send()
        .map_err(|error| format!("调用 OpenAI 兼容 API 失败: {}", error))?;
    let status = response.status();
    if !status.is_success() {
        let payload: Value = response
            .json()
            .map_err(|_| json!({}))
            .unwrap_or_else(|_| json!({}));
        return Err(format!(
            "OpenAI 兼容 API 请求失败（HTTP {}）：{}",
            status,
            extract_error(&payload)
        ));
    }
    use std::io::Read;
    let mut full_text = String::new();
    let mut full_reasoning = String::new();
    let mut buf = [0u8; 8192];
    let mut leftover = String::new();
    loop {
        let n = response
            .read(&mut buf)
            .map_err(|error| format!("读取 OpenAI 兼容流式响应失败: {}", error))?;
        if n == 0 {
            break;
        }
        leftover.push_str(&String::from_utf8_lossy(&buf[..n]));
        // SSE 事件以空行分隔；累积解析已完整的块
        while let Some(pos) = leftover.find("\n\n") {
            let event = leftover[..pos].to_string();
            leftover = leftover[pos + 2..].to_string();
            for line in event.lines() {
                let Some(data) = line.strip_prefix("data:") else {
                    continue;
                };
                let data = data.trim();
                if data.is_empty() || data == "[DONE]" {
                    continue;
                }
                let Ok(chunk) = serde_json::from_str::<Value>(data) else {
                    continue;
                };
                let Some(delta) = chunk.pointer("/choices/0/delta") else {
                    continue;
                };
                if let Some(reasoning) = delta.get("reasoning_content").and_then(Value::as_str) {
                    if !reasoning.is_empty() {
                        full_reasoning.push_str(reasoning);
                        on_stream(&format!("r:{}", reasoning));
                    }
                }
                if let Some(text) = delta.get("content").and_then(Value::as_str) {
                    if !text.is_empty() {
                        full_text.push_str(text);
                        on_stream(text);
                    }
                }
            }
        }
    }
    let reasoning = if full_reasoning.trim().is_empty() {
        None
    } else {
        Some(full_reasoning)
    };
    Ok((full_text, reasoning))
}

/// 把前端传入的对话历史转换为 API messages 格式。
/// 每项为 { role: "user"|"assistant", content: string }，忽略其它字段与非法项。
fn history_to_messages(history: &[Value]) -> Vec<Value> {
    history
        .iter()
        .filter_map(|item| {
            let role = item.get("role").and_then(Value::as_str)?;
            if !matches!(role, "user" | "assistant") {
                return None;
            }
            let content = item.get("content").and_then(Value::as_str)?;
            if content.trim().is_empty() {
                return None;
            }
            Some(json!({ "role": role, "content": content }))
        })
        .collect()
}

fn call_anthropic(
    client: &Client,
    provider: &str,
    endpoint: &str,
    model: &str,
    api_key: &str,
    system_prompt: &str,
    history: &[Value],
    user_prompt: &str,
) -> Result<(String, Option<String>), String> {
    let mut messages = history_to_messages(history);
    messages.push(json!({ "role": "user", "content": user_prompt }));
    let body = json!({
        "model": model.trim(),
        "max_tokens": 4096,
        "temperature": 0.1,
        "system": system_prompt,
        "messages": messages
    });
    let mut request = client.post(anthropic_url(endpoint));
    if provider.starts_with("opencode-zen") {
        request = request.bearer_auth(api_key.trim());
    } else {
        request = request.header("x-api-key", api_key.trim());
        if provider == "custom-anthropic" {
            request = request.bearer_auth(api_key.trim());
        }
    }
    request = request
        .header("anthropic-version", "2023-06-01")
        .header("Accept", "application/json")
        .json(&body);
    let payload = send_json(request, "Anthropic Messages")?;
    Ok((extract_anthropic_content(&payload)?, None))
}

fn call_gemini(
    client: &Client,
    provider: &str,
    endpoint: &str,
    model: &str,
    api_key: &str,
    system_prompt: &str,
    history: &[Value],
    user_prompt: &str,
) -> Result<(String, Option<String>), String> {
    let mut contents: Vec<Value> = history
        .iter()
        .filter_map(|item| {
            let role = item.get("role").and_then(Value::as_str)?;
            let role = match role {
                "user" => "user",
                "assistant" => "model",
                _ => return None,
            };
            let content = item.get("content").and_then(Value::as_str)?;
            if content.trim().is_empty() {
                return None;
            }
            Some(json!({ "role": role, "parts": [{ "text": content }] }))
        })
        .collect();
    contents.push(json!({ "role": "user", "parts": [{ "text": user_prompt }] }));
    let body = json!({
        "systemInstruction": { "parts": [{ "text": system_prompt }] },
        "contents": contents,
        "generationConfig": {
            "temperature": 0.1,
            "responseMimeType": "application/json"
        }
    });
    let mut request = client.post(gemini_url(endpoint, model));
    if provider.starts_with("opencode-zen") {
        request = request.bearer_auth(api_key.trim());
    } else {
        request = request.header("x-goog-api-key", api_key.trim());
    }
    let payload = send_json(
        request.header("Accept", "application/json").json(&body),
        "Gemini GenerateContent",
    )?;
    extract_gemini_content(&payload).map(|content| (content, None))
}

/// 螺丝业务领域规则：注入所有 AI 提示词，确保助手理解头型/牙型/命名规则（2026-08-08 主人口述 + 行业参考资料）。
pub const BUSINESS_RULES_PROMPT: &str = r#"【螺丝规格业务规则（务必遵循）】
1. 头型与牙型是两个独立维度，不要混淆：
   - 头型（头部形状）：厂内生产用代码——P形=锅头/平圆头/半圆头、B形=圆扁头、V形=毛菇头、R形=盘头等（主人确认 P/B/V/R 主要给生产用）；行业参考补充 T形=大扁头、F形=平头（Flat，平的；平头可为 C 圆柱头或 K 沉头）、I形=圆柱头、O形=半沉头、外六角。**这些字母代码是厂内生产内部标识，客户一般非专业、不关心中文术语、只看样品或图纸；与客户沟通或生成面向客户的文档时，不要使用代码术语，以样品/图纸为准。**
   - 牙型（螺纹牙形）：A=TP1尖尾（尖尾自攻）、B=TP2单牙、M=机牙，此三者为主人确认的厂内代码；行业补充：BT=单牙割尾、PTT=三角双牙、BTT=三角单牙、STT=三角机牙、CTT。锁塑料用 TP2/TP2割尾/三角牙/TP1；锁钣金有螺纹用 M，无螺纹用三角牙。
2. 螺丝规格命名格式 `M{牙径}-{牙距}*{长度}`（可带文字后缀），例如 `M3-24*8`、`M2-18*8`：
   - 开头的 M 表示普通牙（机牙），基本是默认，一般可忽略。
   - 牙径：如 3、2、4（mm）。
   - 牙距：数字=英制 TPI（如 24、18）；小数=公制 mm（如 1.1、0.7）。
   - 长度：如 8。
   - 无牙距标注时按标准为机牙（M 牙），如 `M4-0.7` 的 0.7 即公制牙距。
   - 特殊需求以文字后缀，如「割尾」。
3. 牙型与可加工长度：A 牙=尖尾自攻；B 牙（TP2 单牙）能做的螺丝长度受牙板规格、机型限制；长度限制规则 A/B 牙同理。
4. 含义示例：`M3-24*8 割尾`=牙径 3、英制牙距 24、长度 8、割尾特殊需求；`M2-18*8`=牙径 2、英制牙距 18、长度 8、尖尾 A 牙。
5. 牙板/螺丝规格中的特殊后缀标记（出现在名称或规格字段末尾，如「M6-2.6*45F」「M4.2-14*19 D/W」），用于快速识别适配的牙板类别：
   - **F 尾** = 适配沉头螺丝的牙板（头型为沉头）。
   - **D/W 尾** = 墙钉/木牙钉专用牙板（特殊用途，软质基材无需预钻孔固定）。
   - 其他后缀标记按字面意义理解，主人补充后纳入规则。AI 在数据库中检索带这些标记的牙板/螺丝时按字段末尾字面匹配。
6. 冲头（打头冲）规格：冷镦打头只成型**头部形状 + 驱动槽**（十字/一字/梅花/内六角/Torx），**不做牙型**（牙型由搓牙工序的牙板成型）。冲头规格按标准体系分，**首字母即体系：J=日标 JIS**（JMP=Pan 盘头、JMT=Truss 大扁头、JMB=Binding 束头、JMF=Flat **平头（不是沉头；平头可为 C 圆柱头或 K 沉头）**、JMO=Oval 半沉头、JMR=Round 圆头、JW 带垫华司头、JIS B1176/JCIS）、D=德标 DIN（DMF/DMO/DTF/DWO 等、DIN 912）、A=美标 ANSI（AMF/AMO/AMP/ATF/AWO 等、ASME）、I=ISO 国际（IMP/IMF/IMO/ITP 等）、GB=国标（GB/T950-952）。**冲头命名参数：M+数字=凹槽规格档位（对应螺丝公称直径，M3=M3.0 档、M26=M2.6 档）；M=数字=十字槽定制宽度（M=2.8=2.8mm）；D= = 冲头凹槽大小；T+数字=Torx 梅花槽规格（T10/T20）；数字前缀=螺钉标准号（7380=ISO 7380 内六角半圆头、7985=DIN 7985 十字盘头）**。数字系列如 0317P=规格号+头型字母（0# 微型机螺丝 P 头冲头）。用户问冲头选型时按此对应回答。
"#;

/// Excel 文件对比分析专用提示词：上下文同时包含系统数据（data）与待分析文件数据（excelData）。
pub const EXCEL_ANALYSIS_SYSTEM_PROMPT: &str = r#"你是模具管理系统内置助手。系统已经读取了用户上传的 Excel 文件，文件内容就在上下文的 excelData 字段中；你必须直接分析 excelData，严禁回答「无法读取文件」「请上传文件」或要求用户指定导入目标表。
上下文包含两部分：
- 当前系统数据：schema（表与字段定义）、data（各表现有记录，部分展示）、totals（各表真实记录总数）、attachments（附件数量）
- 用户上传的 Excel 文件数据：excelData（各业务表原始行，与 schema 字段一致）、excelTotals（Excel 各表行数）
请对比 excelData 与系统 data，逐表分析：
1. 新增候选：Excel 有而系统没有的记录（按业务键如名称/规格/材质 归一化判断，不要靠内部 id）。
2. 更新候选：两边都有但字段值不同的记录，指出哪些字段不同、值分别是什么。
3. 缺失项：系统有而 Excel 没有的记录——仅提示，默认不要生成删除操作（Excel 可能只覆盖部分范围）。
4. 重复/可疑数据：Excel 内部或与系统重复的记录。
在 answer 中给出简明中文分析：差异概况（每表新增/更新/缺失数量）、需要重点关注的项目、处理建议。数据量大时先给总结再列重点，不必逐条罗列。
若用户明确要求同步 Excel 数据到系统，通过 changes 提出操作：
- 新增：{"operation":"add","table":"业务表名","fields":{完整字段}}
- 更新：{"operation":"update","table":"业务表名","id":"系统 data 中已存在的 id","fields":{只含要修改的字段}}
- 严禁对缺失项生成 delete；库存汇总表禁止修改。
用户没有明确要求执行时，changes 必须为空数组。
请始终只输出一个 JSON 对象，不要输出 Markdown，不要输出代码块，格式必须是：
{"answer":"给用户看的简洁中文回答","changes":[]}
answer 字段中如需引用术语或字段值，请使用「」中文引号包裹；严禁使用未转义的 ASCII 双引号包裹中文内容，会破坏 JSON 解析。
每条 change 的 fields 必须填写所有要修改的字段，禁止留空；表名和字段 key 必须来自上下文 schema。
"#;

pub fn chat(
    provider: &str,
    protocol: &str,
    endpoint: &str,
    model: &str,
    api_key: &str,
    question: &str,
    context: &str,
    page_context: &str,
    history: &[Value],
    system_override: Option<&str>,
    on_stream: Option<&dyn Fn(&str)>,
) -> Result<AgentReply, String> {
    if endpoint.trim().is_empty() || model.trim().is_empty() {
        return Err("请先完成第三方 AI API 配置".to_string());
    }
    // 免费服务（如 opencode Zen）无需 API Key；OpenAI 兼容协议允许空 Key（不携带鉴权头），
    // 仅 Anthropic/Gemini 等强制鉴权的协议要求必须提供。
    if api_key.trim().is_empty() && protocol != "openai" {
        return Err("该服务需要 API Key，请先在「AI 助手」配置中填写".to_string());
    }
    if question.trim().is_empty() {
        return Err("问题不能为空".to_string());
    }

    const DEFAULT_SYSTEM_PROMPT: &str = r#"你是模具管理系统内置助手。你只能根据提供的业务数据回答，不要编造不存在的记录。
上下文中的 data 只包含每张表的部分记录（按时间倒序优先展示最新记录），totals 给出每张表的真实记录总数；未列出的记录不代表不存在。查询库存时优先参考「库存汇总」表（实时全量数据，但库存汇总为只读，禁止修改）。当用户只是查询已有记录且记录未出现在已加载数据中时：先对照 totals 判断该表是否还有更多记录，若有则明确告知「该表共 N 条记录，已加载的部分中未找到」，并引导用户用更精确关键词重新查询；不要武断回答「不存在」。
【新品螺丝配模推荐模式】当用户表达「我要做/准备生产/开发/试做某个螺丝」「为新螺丝找牙板/冲头/模具」「某个系统中不存在的规格能用什么工具」时，严禁只回答「系统不存在」。必须使用 toolingRecommendation：
1. 解析需求：螺丝直径、长度、头型（如 B）、牙型（如 B牙）、线材/线径、机型；缺失信息列为待确认项，但仍先给可用的初步候选。
2. 先从 history 中找相似历史螺丝，优先级依次为：同直径+同牙型+同头型 > 同直径+同牙型 > 相近直径+同牙型/头型；长度通常影响牙板长度/工艺，但不应因长度不同直接否定冲头候选。
3. 沿相似螺丝的 verifiedPunches/verifiedDies 推荐工具；evidence=verified_link 只能表述为「已用于相似历史螺丝」，不能表述为「已验证适用于新品」。没有直接关联时才从 allPunches/allDies 按名称中的公称直径、牙型、机型、线径推测。
4. 每类候选最多列 3 个，必须给出：工具 ID、名称/规格、库存状态、匹配等级（高/中/低）、推荐依据、风险/待确认项；优先推荐有库存者，但规格匹配优先于库存。
5. 主模具只能按 wireMaterial/孔径做 wire_compatible 候选；若 mainMolds 为空，明确写「系统暂无主模具数据，无法推荐」，严禁编造。
6. 回答必须明确声明「这是基于历史关联与规格相似度的候选推荐，投产前需由工艺人员确认」。仅在用户明确要求新增记录/建立关联时才生成 changes，否则 changes 为空。
attachments 字段列出有附件的螺丝规格（id → 附件数量），未列出的规格表示没有附件；用户询问图纸/附件时据此回答。
memory 是近期对话摘要（含时间，列表按时间从新到旧排列，第一条是最近一次对话）；用户可能引用之前的对话，回答时可参考最近几条，但不要主动复述旧摘要，除非与当前问题相关。
【当前页面上下文】描述用户正在查看的页面与筛选条件；回答优先结合该范围，但用户明确提问其他范围时不受限制。
请始终只输出一个 JSON 对象，不要输出 Markdown，不要输出代码块，格式必须是：
{"answer":"给用户看的简洁中文回答","changes":[]}
answer 字段中如需引用术语或字段值，请使用「」中文引号包裹；严禁使用未转义的 ASCII 双引号包裹中文内容（如 "初始库存"），会破坏 JSON 解析。
changes 是数组，每个元素是一个需要系统执行的操作（没有操作时为空数组），支持以下类型：
1. 业务数据新增/修改/删除：{"operation":"add|update|delete","table":"业务表名","id":"更新或删除时必填","fields":{"字段key":"新值"}}
2. 系统设置变更：{"operation":"set_setting","table":"system_settings","fields":{...}}（可修改 theme、"backup_count"、"allow_delete"、"backup_path"；涉及路径的配置项需用户确认）
3. 导入操作：{"operation":"import","table":"目标业务表名","fields":{"file_path":"用户提供的文件完整路径"}}（仅当用户明确给出导入文件路径时生成；否则在回答中提示用户先提供路径）
用户要求批量修正/规范化多条记录时，可以把所有需要修正的记录一次性放入 changes 数组（每条一条变更）。用户没有明确要求执行以上操作时，changes 必须为空数组。
**修改已有记录（改名称、改规格、改字段值等）必须使用 update 操作，并且 id 必须是上下文数据中该记录已存在的 id；严禁用 add 操作创建与现有记录同 id 或同名称的新记录。** 上下文数据中每张业务表列出了部分现有记录（含 id），需要定位已有记录时先在其中查找 id。
**每条 change 的 fields 必须填写所有要修改的字段（含字段 key 和新值），禁止留空**——空 fields 会被系统拒绝执行。answer 文本描述的修改必须与 changes 数组里的 fields 完全一致，不能只写在 answer 而不出现在 changes 里。
**严禁在 answer 中声称「已修改/已更新/已执行/已设置」，除非对应操作真实出现在本次返回的 changes 数组中**。系统只会在收到 changes 后真实执行，不会执行 answer 文字。若用户要求修改但你没有生成对应 change，answer 必须如实说明「未执行任何修改」，并给出需要确认的内容，不能假装已完成。
表名和字段 key 必须来自上下文中的 schema。库存汇总表是系统计算表，禁止修改。
不要修改 id，不要在 fields 中输出未声明的字段。
删除、导入、以及涉及文件路径的操作属于高风险，需要用户二次确认；业务新增/修改和系统设置变更可直接执行。
"#;
    // 业务规则始终注入：即使前端传入自定义 system_override（如 Excel 分析），也保留领域规则。
    let base_prompt = system_override.unwrap_or(DEFAULT_SYSTEM_PROMPT);
    let system_prompt = format!("{}\n{}", BUSINESS_RULES_PROMPT, base_prompt);
    let user_prompt = format!(
        "【当前页面上下文】{}\n\n用户请求：{}\n\n当前系统数据上下文：\n{}",
        if page_context.trim().is_empty() {
            "（无）"
        } else {
            page_context.trim()
        },
        question.trim(),
        context
    );
    let client = Client::builder()
        .timeout(Duration::from_secs(150))
        .build()
        .map_err(|error| format!("创建 AI API 客户端失败: {}", error))?;

    let mut reasoning: Option<String> = None;
    let raw = match protocol {
        "responses" => {
            let (text, reason) = call_responses(
                &client,
                endpoint,
                model,
                api_key,
                &system_prompt,
                history,
                &user_prompt,
            )?;
            reasoning = reason;
            text
        }
        "openai" => {
            // GLM 有 thinking/response_format 特殊参数，流式兼容风险大，保持非流式；
            // 其余 OpenAI 兼容服务（opencode-zen/deepseek/openai/qwen 等）走 SSE 流式。
            if provider == "glm" {
                let (text, reason) = call_openai(
                    &client,
                    provider,
                    endpoint,
                    model,
                    api_key,
                    &system_prompt,
                    history,
                    &user_prompt,
                )?;
                reasoning = reason;
                text
            } else if let Some(callback) = on_stream {
                let (mut text, reason) = call_openai_stream(
                    &client,
                    provider,
                    endpoint,
                    model,
                    api_key,
                    &system_prompt,
                    history,
                    &user_prompt,
                    false,
                    callback,
                )?;
                reasoning = reason;
                // 部分推理模型在长 Excel 上会把全部 token 用于 reasoning_content，
                // 最终 content 为空。自动关闭 thinking 以非流式重试一次，确保拿到正式 JSON 回答。
                if text.trim().is_empty() && reasoning.is_some() {
                    let (retry_text, _) = call_openai(
                        &client,
                        "deepseek", // 触发 thinking:disabled；端点/model/key 仍用当前服务
                        endpoint,
                        model,
                        api_key,
                        &system_prompt,
                        history,
                        &user_prompt,
                    )?;
                    text = retry_text;
                }
                text
            } else {
                let (text, reason) = call_openai(
                    &client,
                    provider,
                    endpoint,
                    model,
                    api_key,
                    &system_prompt,
                    history,
                    &user_prompt,
                )?;
                reasoning = reason;
                text
            }
        }
        "anthropic" => {
            let (text, _) = call_anthropic(
                &client,
                provider,
                endpoint,
                model,
                api_key,
                &system_prompt,
                history,
                &user_prompt,
            )?;
            text
        }
        "gemini" => {
            let (text, _) = call_gemini(
                &client,
                provider,
                endpoint,
                model,
                api_key,
                &system_prompt,
                history,
                &user_prompt,
            )?;
            text
        }
        _ => return Err(format!("不支持的 AI API 协议：{}", protocol)),
    };

    let parsed = parse_json_output(&raw);
    let answer = parsed
        .as_ref()
        .and_then(|value| value.get("answer"))
        .and_then(Value::as_str)
        .unwrap_or(raw.trim())
        .to_string();
    // 优先取 changes 数组；兼容旧格式的单个 change（转为单元素数组）
    let changes = parsed
        .as_ref()
        .and_then(|value| value.get("changes"))
        .and_then(Value::as_array)
        .cloned()
        .map(|items| {
            items
                .into_iter()
                .filter(|item| !item.is_null())
                .collect::<Vec<_>>()
        })
        .or_else(|| {
            parsed
                .as_ref()
                .and_then(|value| value.get("change"))
                .filter(|value| !value.is_null())
                .cloned()
                .map(|change| vec![change])
        })
        .unwrap_or_default();
    Ok(AgentReply {
        answer,
        changes,
        reasoning,
        raw,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_protocol_urls() {
        assert_eq!(
            openai_url("https://api.example.com/v1"),
            "https://api.example.com/v1/chat/completions"
        );
        assert_eq!(
            anthropic_url("https://api.example.com"),
            "https://api.example.com/v1/messages"
        );
        assert_eq!(
            gemini_url("https://example.com/v1beta", "gemini-flash"),
            "https://example.com/v1beta/models/gemini-flash:generateContent"
        );
        assert_eq!(
            gemini_url("https://opencode.ai/zen/v1", "gemini-3.6-flash"),
            "https://opencode.ai/zen/v1/models/gemini-3.6-flash"
        );
    }

    #[test]
    fn extracts_all_supported_response_formats() {
        let responses = json!({"output":[{"content":[{"type":"output_text","text":"ok"}]}]});
        let openai = json!({"choices":[{"message":{"content":"ok"}}]});
        let anthropic = json!({"content":[{"type":"text","text":"ok"}]});
        let gemini = json!({"candidates":[{"content":{"parts":[{"text":"ok"}]}}]});
        assert_eq!(extract_responses_content(&responses).unwrap(), "ok");
        assert_eq!(extract_openai_content(&openai).unwrap(), "ok");
        assert_eq!(extract_anthropic_content(&anthropic).unwrap(), "ok");
        assert_eq!(extract_gemini_content(&gemini).unwrap(), "ok");
    }

    #[test]
    fn parses_well_formed_json_directly() {
        let raw = r#"{"answer":"ok","changes":[]}"#;
        assert_eq!(
            parse_json_output(raw).unwrap(),
            json!({"answer":"ok","changes":[]})
        );
    }

    #[test]
    fn repairs_unbalanced_quotes_in_answer_field() {
        // AI 在 answer 里用 ASCII 双引号包裹中文"初始库存"，破坏 JSON 边界
        let raw = r#"{"answer":"已清理备注（原"初始库存"字已清空），入库记录未动。","changes":[]}"#;
        let parsed = parse_json_output(raw).expect("应通过 repair 解析成功");
        assert_eq!(
            parsed["answer"],
            "已清理备注（原「初始库存」字已清空），入库记录未动。"
        );
        assert_eq!(parsed["changes"], json!([]));
    }

    #[test]
    fn repairs_multiple_unbalanced_quotes_in_answer_field() {
        // 多个误用引号，全部替换为成对中文引号
        let raw = r#"{"answer":"检查"螺丝规格表"和"冲头信息表"","changes":[]}"#;
        let parsed = parse_json_output(raw).expect("应通过 repair 解析成功");
        assert_eq!(parsed["answer"], "检查「螺丝规格表」和「冲头信息表」");
    }

    #[test]
    fn repairs_quotes_in_answer_even_when_answer_is_last_field() {
        // answer 是最后一个字段（其后是 }），也要能修复
        let raw = r#"{"answer":"备注"原"字已清","changes":[]}"#;
        let parsed = parse_json_output(raw).expect("应通过 repair 解析成功");
        assert_eq!(parsed["answer"], "备注「原」字已清");
    }
}

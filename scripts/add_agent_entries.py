# -*- coding: utf-8 -*-
"""给 7 个业务列表页加「问 AI」入口：header-right 按钮 + router + askAgent（2026-08-07）"""
import re
import sys

PAGES = [
    ("ScrewSpec.vue", "螺丝规格管理", "螺丝规格表", False),
    ("Punch.vue", "冲头管理", "冲头信息表", True),
    ("Die.vue", "牙板管理", "牙板信息表", True),
    ("Belt.vue", "皮带管理", "皮带信息表", True),
    ("MainMold.vue", "主模具管理", "主模具信息表", True),
    ("Scissor.vue", "剪刀管理", "剪刀信息表", True),
    ("UpperPunch.vue", "上冲管理", "上冲信息表", True),
]

BASE = r"C:\Users\Administrator\mold-management\src\views"

for filename, from_label, table, has_tab in PAGES:
    path = f"{BASE}\\{filename}"
    with open(path, encoding="utf-8") as f:
        content = f.read()

    # 1. header-right 内插入「问 AI」按钮（保持缩进推导）
    pattern = re.compile(r'(\n(\s*)<div class="header-right">\n)')
    m = pattern.search(content)
    if not m:
        print(f"[SKIP] {filename}: 未找到 header-right")
        continue
    indent = m.group(2) + "    "
    btn = f'{indent}<el-button @click="askAgent">问 AI</el-button>\n'
    content = pattern.sub(m.group(1) + btn, content, count=1)

    # 2. import useRouter
    if "useRouter" not in content:
        content = content.replace(
            "import { ElMessage, ElMessageBox } from 'element-plus'",
            "import { ElMessage, ElMessageBox } from 'element-plus'\nimport { useRouter } from 'vue-router'",
            1,
        )

    # 3. tableSearch 定义后加 router + askAgent
    if "const askAgent" not in content:
        tab_part = ", tab: activeTab.value" if has_tab else ""
        snippet = f"""

const router = useRouter()
const askAgent = () => {{
  router.push({{ path: '/agent', query: {{ from: '{from_label}', table: '{table}', filter: tableSearch.value{tab_part} }} }})
}}"""
        content = content.replace("const tableSearch = ref('')", "const tableSearch = ref('')" + snippet, 1)

    with open(path, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"[OK] {filename}")

print("=== 完成 ===")

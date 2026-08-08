# -*- coding: utf-8 -*-
"""修复牙板信息表重复数据 + 重建唯一索引 + 重算牙板库存汇总（2026-08-07）"""
import sqlite3
import re
import sys

DB = r'C:\Users\Administrator\Desktop\mold-data.db'
conn = sqlite3.connect(DB)
conn.execute('PRAGMA busy_timeout = 5000')

try:
    conn.execute('BEGIN')

    # ========== 1. 牙板信息表：删除 AI 新增行 + 历史重复 ==========
    c1 = conn.execute("DELETE FROM 牙板信息表 WHERE remark IS NULL")
    print('删除 AI 新增行(remark=NULL):', c1.rowcount)
    c2 = conn.execute('DELETE FROM 牙板信息表 WHERE rowid=74')  # YB0015 重复
    c3 = conn.execute('DELETE FROM 牙板信息表 WHERE rowid=55')  # YB0055 X16
    print('删除历史重复 YB0015(rowid74):', c2.rowcount, ', YB0055(rowid55):', c3.rowcount)

    cur = conn.execute('SELECT COUNT(*), COUNT(DISTINCT id) FROM 牙板信息表')
    print('修复后牙板信息表: 总数/唯一id =', cur.fetchone())
    for iid in ('YB0015', 'YB0055'):
        cur = conn.execute('SELECT rowid, id, name, remark FROM 牙板信息表 WHERE id=?', (iid,))
        print(iid, '保留行:', cur.fetchall())

    # ========== 2. 重建全部有 id 表的唯一索引 ==========
    cur = conn.execute("SELECT name FROM sqlite_master WHERE type='table' AND name != 'meta'")
    tables = [r[0] for r in cur.fetchall()]
    for t in tables:
        cur = conn.execute('PRAGMA table_info("{}")'.format(t))
        cols = [r[1] for r in cur.fetchall()]
        if 'id' not in cols:
            continue
        idx_name = 'idx_' + re.sub(r'[^0-9A-Za-z]', '_', t)
        conn.execute('CREATE UNIQUE INDEX IF NOT EXISTS "{}" ON "{}" ("id")'.format(idx_name, t))
        print('建唯一索引:', idx_name, '->', t)

    # ========== 3. 牙板库存汇总重算（复刻 excel::recalc_stock_summary 逻辑） ==========
    cur = conn.execute('SELECT id, name, safetyStock FROM 牙板信息表')
    info = cur.fetchall()
    cur = conn.execute("SELECT dieId, quantity FROM 牙板入库记录 WHERE status='已到货'")
    orders = {}
    for die_id, q in cur.fetchall():
        try:
            orders[die_id] = orders.get(die_id, 0) + int(q)
        except (TypeError, ValueError):
            pass
    cur = conn.execute('SELECT dieId, quantity FROM 牙板领用记录')
    uses = {}
    for die_id, q in cur.fetchall():
        try:
            uses[die_id] = uses.get(die_id, 0) + int(q)
        except (TypeError, ValueError):
            pass
    conn.execute('DELETE FROM 牙板库存汇总')
    for die_id, name, safety in info:
        safety_stock = int(safety) if str(safety).strip().isdigit() else 0
        current = orders.get(die_id, 0) - uses.get(die_id, 0)
        status = '需入库' if current < safety_stock else '安全'
        conn.execute(
            'INSERT INTO 牙板库存汇总 (dieId, name, currentStock, safetyStock, status) VALUES (?,?,?,?,?)',
            (die_id, name, str(current), str(safety_stock), status),
        )
    cur = conn.execute('SELECT COUNT(*) FROM 牙板库存汇总')
    print('重算后牙板库存汇总:', cur.fetchone()[0], '条')

    # ========== 4. 完整性检查 ==========
    cur = conn.execute('PRAGMA quick_check')
    print('quick_check:', cur.fetchone()[0])

    conn.execute('COMMIT')
    print('=== 修复完成，已提交 ===')
except Exception as e:
    conn.execute('ROLLBACK')
    print('!!! 失败，已回滚:', e)
    sys.exit(1)
finally:
    conn.close()

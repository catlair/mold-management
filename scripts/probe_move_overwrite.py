"""Probe: 坚果云 WebDAV MOVE 覆盖已存在目标的行为。测试后清理，不影响正式文件。"""
import base64, http.client, json, ssl, uuid
from urllib.parse import urlparse

# 读取 .dev 凭据
dev = {}
for line in open(r'C:\Users\Administrator\mold-management\.dev', encoding='utf-8'):
    line = line.strip()
    if not line or '=' not in line:
        continue
    k, _, v = line.partition('=')
    dev[k.strip()] = v.strip()

url = dev['WEBDAV_URL']
user = dev['WEBDAV_USERNAME']
pwd = dev['WEBDAV_PASSWORD']
u = urlparse(url)

ctx = ssl.create_default_context()
conn = http.client.HTTPSConnection(u.hostname, u.port or 443, timeout=20, context=ctx)
auth = 'Basic ' + base64.b64encode(f'{user}:{pwd}'.encode()).decode()

def req(method, path, body=None, headers=None):
    h = {'Authorization': auth}
    if headers:
        h.update(headers)
    try:
        conn.request(method, path, body=body, headers=h)
        r = conn.getresponse()
        data = r.read(500)
        return r.status, data
    except Exception as e:
        return -1, str(e).encode()

base = u.path.rstrip('/')  # e.g. /dav/%E6%88%91%E7%9A%84%E5%9D%9A%E6%9E%9C%E4%BA%91
suffix = uuid.uuid4().hex[:8]
src = f'{base}/probe-move-{suffix}.txt'
dst = f'{base}/probe-move-{suffix}-dest.txt'
dst_url = f'https://{u.hostname}{dst}'

print('1) PUT source:', src)
s, d = req('PUT', src, b'source-content', {'Content-Type': 'text/plain'})
print('   ->', s)

print('2) PUT dest (exists):', dst)
s, d = req('PUT', dst, b'dest-content', {'Content-Type': 'text/plain'})
print('   ->', s)

print('3) MOVE src -> dest with Overwrite: T')
s, d = req('MOVE', src, None, {'Destination': dst_url, 'Overwrite': 'T'})
print('   ->', s, d[:200])

print('4) MOVE dest2 (fresh) -> dest (no Overwrite)')
src2 = f'{base}/probe-move-{suffix}-2.txt'
req('PUT', src2, b'x', {'Content-Type': 'text/plain'})
s, d = req('MOVE', src2, None, {'Destination': dst_url})
print('   ->', s, d[:200])

print('5) MOVE fresh -> fresh (no Overwrite, target absent)')
src3 = f'{base}/probe-move-{suffix}-3.txt'
dst3 = f'{base}/probe-move-{suffix}-3-dest.txt'
req('PUT', src3, b'x', {'Content-Type': 'text/plain'})
s, d = req('MOVE', src3, None, {'Destination': f'https://{u.hostname}{dst3}'})
print('   ->', s, d[:200])

print('6) cleanup')
for p in [src, dst, src2, src3, dst3]:
    req('DELETE', p)
print('   done')
conn.close()

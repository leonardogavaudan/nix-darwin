---
name: chrome-cookies
description: Extract and set cookies from Chrome's encrypted cookie database on macOS.
---

# Chrome Cookies Skill

Extract and set cookies from Chrome's encrypted cookie database on macOS.

## Prerequisites

```bash
pip3 install browser-cookie3 pycryptodome
```

## How Chrome Cookie Encryption Works

1. **Storage**: Cookies are stored in SQLite at:
   ```
   ~/Library/Application Support/Google/Chrome/Default/Cookies
   ```

2. **Encryption**: Cookie values are encrypted with AES-128-CBC
   - Key is derived from a password stored in macOS Keychain under "Chrome Safe Storage"
   - Encrypted values start with `v10` (hex: `763130`)

3. **Key retrieval**:
   ```bash
   security find-generic-password -w -s "Chrome Safe Storage"
   ```

## Extracting Cookies

### Using browser-cookie3 (recommended)

```python
import browser_cookie3

# Get cookies for a specific domain
cj = browser_cookie3.chrome(domain_name='example.com')
for cookie in cj:
    print(f"{cookie.name}: {cookie.value}")
```

### One-liner to extract a specific cookie

```bash
python3 -c "import browser_cookie3; cj=browser_cookie3.chrome(domain_name='example.com'); print([c.value for c in cj if c.name=='session_id'][0])"
```

### Using sqlite3 to list cookies (values encrypted)

```bash
sqlite3 ~/Library/Application\ Support/Google/Chrome/Default/Cookies \
  "SELECT host_key, name, path FROM cookies WHERE host_key LIKE '%example%'"
```

## Setting/Using Cookies

### Method 1: Proxy Cookie Injection (Recommended for local dev)

**Best approach when running a local dev server that proxies to a remote backend.**

Instead of modifying browser cookies, inject the session cookie at the proxy level:
- No need to restart Chrome
- No browser cookie encryption issues
- Works immediately

**Example: Vite proxy configuration**

```typescript
// vite.config.mts
const sessionCookie = process.env.SESSION_COOKIE;

const proxyOptions: ProxyOptions = {
  target: backendHost,
  changeOrigin: true,
  configure: (proxy) => {
    if (sessionCookie) {
      proxy.on('proxyReq', (proxyReq) => {
        proxyReq.setHeader('Cookie', `session_id=${sessionCookie}`);
      });
    }
  },
};
```

**Usage:**
```bash
# Extract cookie and pass to dev server
SESSION_COOKIE=$(python3 -c "import browser_cookie3; cj=browser_cookie3.chrome(domain_name='example.com'); print([c.value for c in cj if c.name=='session_id'][0])") \
yarn vite
```

### Method 2: JavaScript in browser (if on the target domain)

```javascript
// Delete old cookie first
document.cookie = "cookie_name=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT";
// Set new cookie
document.cookie = "cookie_name=new_value; path=/";
```

### Method 3: Direct database modification (requires Chrome restart)

**Important**: Chrome caches cookies in memory. Changes require Chrome restart.

```python
import sqlite3
import os
from Crypto.Cipher import AES
from Crypto.Protocol.KDF import PBKDF2
import subprocess
import base64

def get_chrome_key():
    """Get Chrome encryption key from macOS Keychain"""
    result = subprocess.run(
        ['security', 'find-generic-password', '-w', '-s', 'Chrome Safe Storage'],
        capture_output=True, text=True
    )
    chrome_pass = result.stdout.strip()
    chrome_pass_bytes = base64.b64decode(chrome_pass)
    salt = b'saltysalt'
    return PBKDF2(chrome_pass_bytes, salt, dkLen=16, count=1003)

def encrypt_cookie(value: str, key: bytes) -> bytes:
    """Encrypt a cookie value for Chrome"""
    padding_len = 16 - (len(value) % 16)
    padded = value.encode() + bytes([padding_len] * padding_len)
    iv = b' ' * 16
    cipher = AES.new(key, AES.MODE_CBC, iv)
    encrypted = cipher.encrypt(padded)
    return b'v10' + encrypted

def set_chrome_cookie(domain: str, name: str, value: str, path: str = '/'):
    """Set a cookie in Chrome's database"""
    db_path = os.path.expanduser(
        "~/Library/Application Support/Google/Chrome/Default/Cookies"
    )
    key = get_chrome_key()
    encrypted_value = encrypt_cookie(value, key)

    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    cursor.execute("""
        UPDATE cookies
        SET encrypted_value = ?, value = ''
        WHERE host_key = ? AND name = ? AND path = ?
    """, (encrypted_value, domain, name, path))
    conn.commit()
    conn.close()

# Example: set_chrome_cookie('localhost', 'session_id', 'abc123')
```

### Method 4: Force Chrome to re-read cookies

After modifying the database:
1. **Close all Chrome windows** and reopen
2. **Clear site data** for the specific domain in Chrome DevTools
3. **Use incognito mode** (fresh cookie state)

## Troubleshooting

### "database is locked"
Chrome has the database open. Either:
- Close Chrome completely
- Use WAL mode checkpoint (risky while Chrome is running)

### Cookie not taking effect
Chrome caches cookies in memory. Use proxy injection method or restart Chrome.

### Permission denied on Keychain
Grant Terminal/iTerm access in System Preferences → Security & Privacy → Privacy → Full Disk Access.

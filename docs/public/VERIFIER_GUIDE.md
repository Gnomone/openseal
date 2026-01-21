# OpenSeal User Guide

**For**: API consumers who want to verify sealed responses

[🇰🇷 한국어 버전](./USER_GUIDE_KR.md)

---

## What You Need to Know

As an API user, you don't need to run OpenSeal yourself. You're verifying responses from sealed services.

**Key Concept**: Every response includes a cryptographic "Seal" that proves:
1. The result came from the claimed container
2. The result hasn't been tampered with
3. The result is fresh (not replayed)

---

## Making a Request

### Add Wax Header

```bash
curl -H "X-OpenSeal-Wax: your-challenge-here" \
  http://sealed-api.com/endpoint
```

**What is Wax?**
- A random string you provide (challenge)
- Proves the response is fresh (not a replay)
- Can be anything: `"my-request-123"`, `UUID`, timestamp, etc.

### Example

```bash
curl -X POST https://api.example.com/crypto/price \
  -H "Content-Type: application/json" \
  -H "X-OpenSeal-Wax: $(uuidgen)" \
  -d '{"symbol":"BTC"}'
```

---

## Understanding the Response

### Response Format

```json
{
  "openseal": {
    "a_hash": "18ddef79a8138634ce4ea0ce9a6e2377...",
    "b_hash": "493911b28d91e0ae8d8bb5a99690c919...",
    "signature": "c327c9ef05b62792b79e7dd1c8ec84b9...",
    "pub_key": "d30c05d163733bae3d24b1c189ca0a8c..."
  },
  "result": {
    "symbol": "BTC",
    "price": "89553.03",
    "currency": "USD",
    "timestamp": "2026-01-22T05:30:00Z"
  }
}
```

### Seal Components

| Field | Meaning | Verification |
|-------|---------|--------------|
| `a_hash` | Identity commitment | Binds result to specific container |
| `b_hash` | Result binding | Cryptographically ties A-hash to result |
| `signature` | Ed25519 signature | Mathematical proof of authenticity |
| `pub_key` | Ephemeral public key | Used to verify signature |

---

## Verifying a Response

### Option 1: Trust the Seal (Simple)

OpenSeal's cryptography ensures:
- ✅ If the signature is valid, the result is authentic
- ✅ Tampering breaks the signature
- ✅ Replay attacks are prevented by Wax

**You can trust any response with a valid seal.**

### Option 2: Using the CLI (Recommended)

You can verify the seal using the `openseal` CLI from v1.0.0-alpha.2+.

1. **Save the response**:
    ```bash
    curl -H "X-OpenSeal-Wax: test1234" \
      http://api.example.com/endpoint > response.json
    ```

2. **Verify**:
    ```bash
    openseal verify --response response.json --wax test1234
    ```

3. **Output**:
    ```
    🔍 Verifying seal...
       🔑 Public Key: 4fdab7f...
       🆔 A-hash:    086a49...
       ✅ Signature Verified!
    ```

4. **Optional: Verify Identity (Root Hash)**
    Ensure the result came from the specific Docker image you trust:
    ```bash
    openseal verify \
      --response response.json \
      --wax test1234 \
      --root-hash "sha256:abc123..."
    ```

---

## Security Guarantees

### What OpenSeal Proves

✅ **Result Integrity**
   - The result came from the sealed container
   - No tampering during transit

✅ **Identity Binding**
   - A-hash ties result to specific Docker image
   - You know exactly which code produced the result

✅ **Freshness**
   - Your Wax is included in the seal
   - Prevents replay attacks

### What OpenSeal Does NOT Prove

❌ **Correctness**
   - Seal proves "code X produced result Y"
   - Does NOT prove "result Y is correct"
   - Example: Buggy code produces verifiably wrong results

❌ **Data Source Truth**
   - If API fetches external data (e.g., Coinbase price)
   - Seal proves "container fetched and returned this"
   - Does NOT prove external data source is honest

❌ **Container Content**
   - Seal proves "result from container with digest ABC"
   - Does NOT prove what's inside the container
   - Provider could have sealed malicious code

**Bottom Line**: OpenSeal proves **verifiable execution**, not **correctness**.

---

## Use Cases

### 1. API Marketplace (HighStation)

```javascript
// MCP Tool
async function getCryptoPrice(symbol) {
  const wax = `request-${Date.now()}`;
  const response = await fetch('https://oracle.highstation.net/price', {
    headers: {
      'Content-Type': 'application/json',
      'X-OpenSeal-Wax': wax
    },
    body: JSON.stringify({ symbol })
  });
  
  const data = await response.json();
  
  // Seal is automatically verified by OpenSeal
  return data.result.price;
}
```

### 2. Audit Trail

```bash
# Every request is provably tied to a specific container version
curl -H "X-OpenSeal-Wax: audit-2026-01-22-001" \
  https://api.example.com/transaction

# Response includes cryptographic proof
# → Can be used for compliance/auditing
```

### 3. Trustless Integration

```python
# Don't trust the API provider?
# OpenSeal lets you verify every response

import requests

def query_sealed_api(endpoint, wax):
    response = requests.get(endpoint, headers={
        "X-OpenSeal-Wax": wax
    })
    data = response.json()
    
    # Seal proves:
    # 1. Result from specific container
    # 2. Not tampered
    # 3. Fresh (matches your wax)
    
    return data["result"]
```

---

## Best Practices

### 1. Always Use Unique Wax

```bash
# ❌ Bad: Reusing wax
curl -H "X-OpenSeal-Wax: static-value" ...

# ✅ Good: Unique per request
curl -H "X-OpenSeal-Wax: $(uuidgen)" ...
```

### 2. Store Seals for Audit

```bash
# Save full response with seal
curl -H "X-OpenSeal-Wax: audit-$(date +%s)" \
  https://api.example.com/critical-operation > audit-log.json
```

### 3. Check Provider's Root Hash

```bash
# Provider should publish their Image Digest
# Example: "Our BTC Oracle: sha256:abc123..."

# Compare with a_hash in responses
# Ensures you're querying the claimed service
```

---

## Troubleshooting

### Missing Seal in Response

**Problem**: Response has no `openseal` field

**Causes**:
1. API is not running through OpenSeal
2. Direct access to container (bypassing proxy)

**Solution**: Contact the provider

### Invalid Signature

**Problem**: "Signature verification failed"

**Causes**:
1. Response was tampered with
2. Network corruption

**Solution**: 
- Retry the request
- If persistent, contact provider
- **Do not trust the result**

### Wax Mismatch

**Problem**: Your Wax doesn't appear in the seal

**Causes**:
1. Forgot to send `X-OpenSeal-Wax` header
2. Header was stripped by proxy

**Solution**: Ensure header is sent correctly

---

## FAQ

**Q: Do I need to install OpenSeal to use sealed APIs?**  
A: No! You only need to send the `X-OpenSeal-Wax` header. The provider runs OpenSeal.

**Q: Can I verify seals programmatically?**  
A: Yes, use `openseal verify` (coming in beta). Or implement Ed25519 verification yourself.

**Q: What if I don't send Wax?**  
A: Most sealed APIs will return an error. Wax is required for security.

**Q: How do I know what container the provider is running?**  
A: Providers should publish their Image Digest (Root Hash). Check their documentation.

**Q: Can sealed APIs lie?**  
A: Seals prove "container X produced Y". If container X contains malicious code, the seal is still valid. Always verify the provider is running trusted code.

---

## Next Steps

- **[Provider Guide](./PROVIDER_GUIDE.md)**: Learn how to deploy sealed services
- **[Crypto Oracle Example](https://github.com/Gnomone/crypto-price-oracle)**: Live demo
- **[HighStation](https://www.highstation.net)**: Marketplace for sealed APIs

---

**Questions?** Open an issue on [GitHub](https://github.com/Gnomone/openseal/issues)

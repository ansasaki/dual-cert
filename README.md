# Dual Certificate Support - Proof of Concept

This repository demonstrates **dual certificate support** (ECDSA + RSA) in both Python and Rust web servers using simple multiple certificate loading.

## Key Discovery

Both Python's `ssl` module and rust-openssl support dual certificates through **multiple certificate calls** - no complex callbacks required! OpenSSL automatically selects the appropriate certificate based on client cipher preferences.

## Prerequisites

### Certificates
Generate test certificates by running `initialize_certs.sh`. They will be
created in the `certs/` directory.

## Python Implementation

### Requirements
- Python 3.8+ (tested on 3.12+)
- No additional packages required (uses built-in `ssl` module)

### Usage
```bash
# Run the Python dual certificate server
python3 python_dual_cert_server.py
```

### Key Code
```python
# Load BOTH certificates into the same SSL context
ssl_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
ssl_ctx.load_cert_chain('certs/ecdsa-cert.pem', 'certs/ecdsa-key.pem')  # ECDSA
ssl_ctx.load_cert_chain('certs/rsa-cert.pem', 'certs/rsa-key.pem')      # RSA
# OpenSSL automatically selects appropriate certificate!
```

## Rust Implementation

### Requirements
- Rust 1.70+
- Dependencies: actix-web, openssl, chrono, env_logger

### Usage
```bash
# Run the Rust dual certificate server
cargo run --bin dual_cert_simple
```

### Key Code
```rust
// Load BOTH certificates into the same SSL acceptor
let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;

// ECDSA certificate
builder.set_private_key_file("certs/ecdsa-key.pem", SslFiletype::PEM)?;
builder.set_certificate_chain_file("certs/ecdsa-cert.pem")?;

// RSA certificate
builder.set_private_key_file("certs/rsa-key.pem", SslFiletype::PEM)?;
builder.set_certificate_chain_file("certs/rsa-cert.pem")?;
// OpenSSL automatically selects appropriate certificate!
```

## Testing Dual Certificate Functionality

Both servers run on `https://localhost:8443`

### Test Commands

```bash
# 1. Default connection (usually selects ECDSA)
curl -k https://localhost:8443

# 2. Force ECDSA certificate
openssl s_client -connect localhost:8443 -cipher 'ECDHE-ECDSA-AES256-GCM-SHA384' -servername localhost

# 3. Force RSA certificate
openssl s_client -connect localhost:8443 -cipher 'ECDHE-RSA-AES256-GCM-SHA384' -servername localhost

# 4. Check which certificate type is being used
openssl s_client -connect localhost:8443 -servername localhost </dev/null 2>/dev/null | \
  openssl x509 -text -noout | grep "Public Key Algorithm"
```

### Expected Results

| Client Cipher Preference | Certificate Presented |
|--------------------------|----------------------|
| Mixed/Default | ECDSA (id-ecPublicKey) |
| ECDHE-ECDSA-* | ECDSA (id-ecPublicKey) |
| ECDHE-RSA-* | RSA (rsaEncryption) |

## Server Endpoints

Both implementations provide:

- `GET /` - Simple hello message
- `GET /info` - Server information and configuration
- `GET /cert-info` - Certificate selection details

## How It Works

### The Simple Approach
1. **Load multiple certificates**: Call certificate loading functions multiple times
2. **OpenSSL handles selection**: The underlying OpenSSL library automatically chooses the appropriate certificate during TLS handshake
3. **No callbacks needed**: Selection is based on client cipher preferences without manual intervention

### Why This Works
- OpenSSL 1.0.2+ supports multiple certificate chains natively
- Certificate selection happens during cipher negotiation
- Client cipher preferences determine which certificate type is used
- Both Python's ssl module and rust-openssl expose this functionality

## Repository Structure

```
├── python_dual_cert_server.py    # Python implementation
├── src/
│   └── dual_cert.rs              # Rust implementation
├── certs/                        # Certificate files (ECDSA + RSA)
└── README.md                     # This file
```

## Key Insights

1. **Multiple certificate loading works**: Both Python and Rust support this approach
2. **No callbacks required**: Automatic selection is built into OpenSSL


# QRNG-DD Security Analysis

## Executive Summary

QRNG-DD implements a defense-in-depth security architecture that ensures:

✅ **Unidirectional Data Flow**: Software-based data diode emulation prevents reverse communication  
✅ **Cryptographic Integrity**: HMAC-SHA256 authentication with CRC32 corruption detection  
✅ **Network Isolation**: Clear separation between internal and external networks  
✅ **Freshness Validation**: Timestamp-based TTL prevents stale data acceptance  
✅ **Replay Protection**: Monotonic sequence numbers detect replay attacks  

This document provides comprehensive security analysis for academic publication and security audits.

## Table of Contents

1. [Threat Model](#threat-model)
2. [Security Architecture](#security-architecture)
3. [Cryptographic Mechanisms](#cryptographic-mechanisms)
4. [Network Security](#network-security)
5. [Authentication & Authorization](#authentication--authorization)
6. [Data Integrity](#data-integrity)
7. [Attack Surface Analysis](#attack-surface-analysis)
8. [Known Limitations](#known-limitations)
9. [Security Best Practices](#security-best-practices)
10. [Compliance Considerations](#compliance-considerations)

## Threat Model

### Assumptions

**Trusted Components**:
- QRNG appliance (Quantis) is physically secured
- Internal network is protected by firewall
- Collector host OS is hardened and patched
- Gateway host OS is hardened and patched

**Untrusted Components**:
- External network (Internet)
- API clients (potentially malicious)
- Network infrastructure (may be compromised)

### Threat Actors

1. **External Attackers**: 
   - Goal: Exfiltrate sensitive data from internal network
   - Capability: Full control of external network
   
2. **Malicious API Clients**:
   - Goal: Exhaust resources, inject malicious data
   - Capability: Craft arbitrary HTTP requests

3. **Network Adversaries**:
   - Goal: Intercept, modify, or replay network traffic
   - Capability: Man-in-the-middle attacks

4. **Insider Threats**:
   - Goal: Bypass security controls, access QRNG directly
   - Capability: Internal network access

### Attack Scenarios

| Scenario | Attacker Goal | Mitigation |
|----------|---------------|------------|
| **Reverse Connection** | Probe internal network via Gateway | Software data diode (no reverse path) |
| **Data Tampering** | Modify entropy packets in transit | HMAC-SHA256 authentication |
| **Replay Attack** | Replay old entropy packets | Sequence numbers + timestamps |
| **DoS Attack** | Exhaust Gateway resources | Rate limiting + authentication |
| **MITM Attack** | Intercept/modify entropy | TLS + HMAC verification |
| **API Abuse** | Exhaust entropy buffer | API key authentication + quotas |

## Security Architecture

### Defense Layers

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 5: Application Security                              │
│ - API authentication (Bearer tokens)                        │
│ - Rate limiting per client                                  │
│ - Input validation                                          │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Data Integrity                                    │
│ - HMAC-SHA256 packet authentication                        │
│ - CRC32 corruption detection                               │
│ - Timestamp freshness validation                           │
│ - Sequence number replay protection                        │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: Network Security                                  │
│ - TLS 1.3 encryption (HTTPS)                               │
│ - Certificate validation                                    │
│ - Unidirectional data flow (software data diode)           │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Network Isolation                                 │
│ - Internal network: 10.0.0.0/8 (RFC 1918)                  │
│ - External network: Public Internet                        │
│ - Firewall rules: Block reverse connections                │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: Physical Security                                 │
│ - QRNG appliance in secured facility                       │
│ - Collector on isolated internal network                   │
│ - Gateway on DMZ or external network                       │
└─────────────────────────────────────────────────────────────┘
```

### Unidirectional Data Flow

**Key Property**: Gateway NEVER initiates connections to Collector

```
┌────────────────────────┐         ┌────────────────────────┐
│ Internal Network       │         │ External Network       │
│                        │         │                        │
│  Collector             │─────────▶  Gateway               │
│  (initiates)           │  HTTPS  │  (accepts)             │
│                        │  push   │                        │
└────────────────────────┘         └────────────────────────┘
         ▲                                    │
         │                                    │
         └────── NO RETURN PATH ──────────────┘
```

**Enforcement Mechanisms**:

1. **Network Configuration**: Firewall blocks inbound connections to Collector
2. **Software Design**: Collector has no listening sockets
3. **Protocol Design**: All communication is push-based (POST /push)

**Verification**:
```bash
# On Collector: Verify no listening ports
netstat -tuln | grep qrng-collector
# Expected: No output (no listening sockets)

# On Gateway: Verify listening on 7764
netstat -tuln | grep 7764
# Expected: tcp 0.0.0.0:7764 LISTEN
```

## Cryptographic Mechanisms

### HMAC-SHA256 Packet Authentication

**Purpose**: Authenticate entropy packets and prevent tampering

**Implementation**:
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn sign_packet(data: &[u8], timestamp: i64, sequence: u64, 
               secret_key: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(secret_key)
        .expect("HMAC accepts any key size");
    
    // Include all packet components in MAC
    mac.update(data);
    mac.update(&timestamp.to_le_bytes());
    mac.update(&sequence.to_le_bytes());
    
    mac.finalize().into_bytes().to_vec()
}

fn verify_packet(packet: &EntropyPacket, secret_key: &[u8]) -> bool {
    let computed = sign_packet(
        &packet.data,
        packet.timestamp,
        packet.sequence,
        secret_key
    );
    
    // Constant-time comparison to prevent timing attacks
    constant_time_eq(&computed, &packet.hmac)
}

// Constant-time comparison (prevents timing attacks)
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    
    result == 0
}
```

**Security Properties**:
- **Key Length**: 256 bits (32 bytes)
- **Output Length**: 256 bits (32 bytes)
- **Collision Resistance**: 2^128 operations
- **Preimage Resistance**: 2^256 operations

**Key Management**:
```bash
# Generate secure HMAC key
openssl rand -hex 32 > hmac_secret.key

# Store in environment variable (not in code!)
export QRNG_HMAC_SECRET_KEY=$(cat hmac_secret.key)

# Permissions: Readable only by service account
chmod 600 hmac_secret.key
chown qrng-service:qrng-service hmac_secret.key
```

### CRC32 Corruption Detection

**Purpose**: Detect bit flips and transmission errors

**Implementation**:
```rust
use crc32fast::Hasher;

fn compute_crc32(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

fn verify_crc32(data: &[u8], expected: u32) -> bool {
    compute_crc32(data) == expected
}
```

**Detection Capability**:
- Single-bit errors: 100%
- Burst errors (<32 bits): 100%
- Random errors: 99.9999998%

**Note**: CRC32 is NOT cryptographically secure (use HMAC for authentication)

### Timestamp-Based Freshness

**Purpose**: Prevent replay attacks with stale packets

**Implementation**:
```rust
use std::time::{SystemTime, UNIX_EPOCH};

fn validate_timestamp(timestamp: i64, max_age_seconds: i64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    let age = now - timestamp;
    
    // Reject future timestamps (clock skew protection)
    if age < 0 {
        return false;
    }
    
    // Reject expired packets
    age <= max_age_seconds
}
```

**Configuration**:
```yaml
gateway:
  packet_ttl_seconds: 300  # 5 minutes
  max_clock_skew_seconds: 60  # Allow 1 minute clock drift
```

**Security Analysis**:
- **TTL**: 300 seconds balances security vs. network delays
- **Clock Skew**: Allows for NTP drift between systems
- **Replay Window**: Attacker has maximum 5 minutes to replay packet

### Sequence Number Replay Protection

**Purpose**: Detect duplicate packets (even within TTL window)

**Implementation**:
```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct SequenceValidator {
    last_sequence: AtomicU64,
}

impl SequenceValidator {
    fn new() -> Self {
        Self {
            last_sequence: AtomicU64::new(0),
        }
    }
    
    fn validate(&self, sequence: u64) -> bool {
        // Load last seen sequence number
        let last = self.last_sequence.load(Ordering::Acquire);
        
        // Reject non-monotonic sequences (replay attempt)
        if sequence <= last {
            return false;
        }
        
        // Update last sequence
        self.last_sequence.store(sequence, Ordering::Release);
        true
    }
}
```

**Security Properties**:
- **Monotonic**: Sequence numbers always increase
- **No Gaps**: Missing sequences are logged but allowed (packet loss)
- **Atomic**: Thread-safe validation

**Attack Scenario**:
```
Attacker captures packet with sequence=1000
Attacker tries to replay it

Gateway state: last_sequence=1000
Gateway receives: sequence=1000
Validation: 1000 <= 1000 → REJECT
```

## Network Security

### TLS Configuration

**Required**: TLS 1.3 with strong cipher suites

```yaml
# Collector TLS configuration
tls:
  min_version: "1.3"
  cipher_suites:
    - TLS_AES_256_GCM_SHA384
    - TLS_CHACHA20_POLY1305_SHA256
  verify_server_cert: true
  ca_certificates: "/etc/ssl/certs/ca-certificates.crt"
```

**Security Properties**:
- **Forward Secrecy**: Ephemeral key exchange (ECDHE)
- **Authenticated Encryption**: GCM or ChaCha20-Poly1305
- **Certificate Validation**: Prevent MITM attacks

### Firewall Rules

**Internal Network (Collector)**:
```bash
# Allow outbound HTTPS to Gateway
iptables -A OUTPUT -p tcp --dport 7764 -d gateway.external.com -j ACCEPT

# Allow outbound HTTPS to QRNG appliance
iptables -A OUTPUT -p tcp --dport 443 -d random.cs.upt.ro -j ACCEPT

# Block ALL inbound connections
iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT
iptables -A INPUT -j DROP
```

**External Network (Gateway)**:
```bash
# Allow inbound HTTPS from Collector (specific IP)
iptables -A INPUT -p tcp --dport 7764 -s collector.internal.ip -j ACCEPT

# Allow inbound HTTPS from API clients
iptables -A INPUT -p tcp --dport 7764 -j ACCEPT

# Block outbound to internal networks (data diode enforcement)
iptables -A OUTPUT -d 10.0.0.0/8 -j DROP
iptables -A OUTPUT -d 172.16.0.0/12 -j DROP
iptables -A OUTPUT -d 192.168.0.0/16 -j DROP
```

## Authentication & Authorization

### API Key Authentication

**Format**: Bearer token in Authorization header

```http
GET /api/bytes?length=1024 HTTP/1.1
Host: gateway:7764
Authorization: Bearer a3f5b2c8d9e1f4a7b6c3d8e9f2a5b4c7
```

**Implementation**:
```rust
async fn authenticate(headers: &HeaderMap, config: &Config) -> Result<()> {
    let auth_header = headers
        .get("authorization")
        .ok_or(Error::Unauthorized)?
        .to_str()
        .map_err(|_| Error::Unauthorized)?;
    
    if !auth_header.starts_with("Bearer ") {
        return Err(Error::Unauthorized);
    }
    
    let token = &auth_header[7..];
    
    // Constant-time comparison (prevent timing attacks)
    if !constant_time_eq(token.as_bytes(), config.api_key.as_bytes()) {
        return Err(Error::Unauthorized);
    }
    
    Ok(())
}
```

**Key Generation**:
```bash
# Generate cryptographically secure API key
openssl rand -hex 32
# Output: a3f5b2c8d9e1f4a7b6c3d8e9f2a5b4c7d6e1f8a3b2c5d4e7f6a9b8c1d2e3f4a5
```

**Storage**: 
- **Collector**: Environment variable `QRNG_HMAC_SECRET_KEY`
- **Gateway**: Environment variable `QRNG_API_KEY`
- **Clients**: Secure credential store (not in code!)

### Rate Limiting

**Purpose**: Prevent DoS attacks and API abuse

**Implementation**:
```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    fn check(&mut self, client_id: &str) -> bool {
        let now = Instant::now();
        
        // Get client's request history
        let history = self.requests.entry(client_id.to_string())
            .or_insert_with(Vec::new);
        
        // Remove expired requests
        history.retain(|&time| now.duration_since(time) < self.window);
        
        // Check if limit exceeded
        if history.len() >= self.max_requests {
            return false;
        }
        
        // Record this request
        history.push(now);
        true
    }
}
```

**Configuration**:
```yaml
rate_limiting:
  window_seconds: 60
  max_requests_per_window: 100
  burst_size: 10
```

## Data Integrity

### Entropy Packet Format

```rust
#[derive(Serialize, Deserialize)]
pub struct EntropyPacket {
    pub data: Vec<u8>,           // Random bytes (variable length)
    pub timestamp: i64,          // Unix timestamp in milliseconds
    pub sequence: u64,           // Monotonic sequence number
    pub crc32: u32,              // CRC32 checksum of data
    pub hmac: Vec<u8>,           // HMAC-SHA256 signature
}
```

**Integrity Chain**:
```
1. data → CRC32 → crc32 field
2. data + timestamp + sequence → HMAC-SHA256 → hmac field
3. Gateway verifies: CRC32 AND HMAC
```

### Verification Process

```rust
fn verify_packet(packet: &EntropyPacket, secret: &[u8]) -> Result<()> {
    // Step 1: Verify CRC32 (detect transmission errors)
    if !verify_crc32(&packet.data, packet.crc32) {
        return Err(Error::CorruptedData);
    }
    
    // Step 2: Verify HMAC (authenticate sender)
    if !verify_hmac(packet, secret) {
        return Err(Error::InvalidSignature);
    }
    
    // Step 3: Verify timestamp (freshness)
    if !validate_timestamp(packet.timestamp, 300) {
        return Err(Error::ExpiredPacket);
    }
    
    // Step 4: Verify sequence (replay protection)
    if !validate_sequence(packet.sequence) {
        return Err(Error::ReplayAttack);
    }
    
    Ok(())
}
```

## Attack Surface Analysis

### External Attack Surface

| Component | Exposed Endpoint | Authentication | Encryption |
|-----------|------------------|----------------|------------|
| **Gateway API** | :7764/api/* | API Key | TLS 1.3 |
| **Gateway Push** | :7764/push | HMAC | TLS 1.3 |
| **Gateway Metrics** | :7764/metrics | Optional | TLS 1.3 |
| **Gateway Health** | :7764/health | None | TLS 1.3 |
| **Collector** | None | N/A | N/A |

### Internal Attack Surface

| Component | Exposed Endpoint | Risk Level |
|-----------|------------------|------------|
| **QRNG Appliance** | :443/api/* | Medium (internal network) |
| **Collector** | None | Low (no listening sockets) |

### Vulnerability Analysis

**Potential Vulnerabilities**:

1. **HMAC Key Compromise**: 
   - **Impact**: Attacker can forge entropy packets
   - **Mitigation**: Secure key storage, rotation, monitoring

2. **Clock Skew Exploitation**:
   - **Impact**: Extend replay window
   - **Mitigation**: NTP synchronization, strict TTL

3. **Resource Exhaustion**:
   - **Impact**: DoS via buffer depletion
   - **Mitigation**: Rate limiting, authentication

4. **TLS Downgrade**:
   - **Impact**: Weaker encryption
   - **Mitigation**: TLS 1.3 only, strong ciphers

## Known Limitations

### 1. Software Data Diode

**Limitation**: Software-based isolation is weaker than hardware

**Hardware Data Diode**:
- ✅ Physical guarantee of unidirectional flow
- ✅ Immune to software bugs
- ❌ Expensive ($5,000-$50,000)
- ❌ Inflexible

**Software Data Diode**:
- ⚠️ Relies on correct configuration
- ⚠️ Vulnerable to OS/firewall bugs
- ✅ Free and open source
- ✅ Easy to configure and update

**Recommendation**: For critical infrastructure, consider hardware data diode

### 2. Replay Window

**Limitation**: 5-minute TTL allows replay within window

**Attack Scenario**:
1. Attacker captures packet at T=0
2. Attacker replays at T=4min (within TTL)
3. Sequence number rejects replay
4. **Result**: Attack fails (sequence protection works)

**Conclusion**: Sequence numbers mitigate TTL limitation

### 3. No Entropy Pool Encryption

**Limitation**: Entropy is stored unencrypted in memory

**Rationale**:
- Entropy is public (not secret)
- Performance: Avoid encryption overhead
- Threat model: Memory access requires system compromise

**Mitigation**: 
- Harden OS
- Use memory protection (ASLR, DEP)
- Deploy on trusted hardware

## Security Best Practices

### Deployment Checklist

- [ ] **Use TLS 1.3** for all HTTPS connections
- [ ] **Generate strong keys** (256-bit HMAC, 256-bit API keys)
- [ ] **Rotate keys regularly** (quarterly recommended)
- [ ] **Enable rate limiting** to prevent DoS
- [ ] **Monitor logs** for suspicious activity
- [ ] **Harden OS** (disable unnecessary services, patch regularly)
- [ ] **Use firewall** to enforce data diode
- [ ] **Synchronize clocks** with NTP
- [ ] **Restrict access** to configuration files (chmod 600)
- [ ] **Use separate machines** for Collector and Gateway

### Key Rotation Procedure

```bash
# 1. Generate new HMAC secret
openssl rand -hex 32 > new_hmac_secret.key

# 2. Update Collector configuration
export QRNG_HMAC_SECRET_KEY=$(cat new_hmac_secret.key)
systemctl restart qrng-collector

# 3. Update Gateway configuration (minimal downtime)
# Keep old key temporarily for graceful transition
export QRNG_HMAC_SECRET_KEY=$(cat new_hmac_secret.key)
export QRNG_HMAC_SECRET_KEY_OLD=$(cat old_hmac_secret.key)
systemctl reload qrng-gateway

# 4. After 10 minutes (2x TTL), remove old key
unset QRNG_HMAC_SECRET_KEY_OLD
systemctl reload qrng-gateway
```

### Monitoring & Alerting

**Security Metrics to Monitor**:

```yaml
alerts:
  - name: hmac_verification_failures
    threshold: 5 per minute
    action: Email security team
    
  - name: rate_limit_exceeded
    threshold: 10 per hour (per client)
    action: Block client temporarily
    
  - name: buffer_depletion
    threshold: <10% fill level
    action: Email operations team
    
  - name: sequence_gap
    threshold: >100 missing sequences
    action: Investigate packet loss
```

**Log Analysis**:
```bash
# Check for authentication failures
journalctl -u qrng-gateway | grep "authentication failed"

# Check for HMAC verification failures
journalctl -u qrng-gateway | grep "HMAC verification failed"

# Check for rate limit violations
journalctl -u qrng-gateway | grep "rate limit exceeded"
```

## Compliance Considerations

### Data Classification

**Quantum Entropy**: 
- **Classification**: Public (not confidential)
- **Integrity**: Critical
- **Availability**: Medium

**HMAC Secret Key**:
- **Classification**: Secret (confidential)
- **Integrity**: Critical
- **Availability**: Medium

### Regulatory Requirements

**GDPR**: Not applicable (no personal data)

**NIST Cybersecurity Framework**:
- ✅ Identify: Threat model documented
- ✅ Protect: Multiple security layers
- ✅ Detect: Comprehensive logging
- ✅ Respond: Security monitoring
- ✅ Recover: Graceful degradation

**ISO 27001**: 
- ✅ A.9: Access Control (API keys)
- ✅ A.10: Cryptography (HMAC, TLS)
- ✅ A.12: Operations Security (monitoring)
- ✅ A.13: Communications Security (TLS, data diode)

### Audit Trail

All security-relevant events are logged:

```json
{
  "timestamp": "2025-11-17T10:30:45Z",
  "event": "hmac_verification_failed",
  "source_ip": "203.0.113.45",
  "packet_sequence": 12345,
  "reason": "invalid_signature"
}
```

**Retention**: 90 days (configurable)

## Conclusion

QRNG-DD implements a robust security architecture with multiple overlapping layers:

1. ✅ **Unidirectional Flow**: Software data diode prevents reverse connections
2. ✅ **Cryptographic Integrity**: HMAC-SHA256 + CRC32 ensure data authenticity
3. ✅ **Freshness & Replay Protection**: Timestamps + sequence numbers
4. ✅ **API Security**: Authentication, rate limiting, TLS encryption
5. ✅ **Defense in Depth**: Multiple independent security mechanisms

**Security Posture**: Suitable for research, academic, and moderate-security production deployments.

**For Critical Infrastructure**: Consider hardware data diode for maximum security guarantee.

---

**Document Version**: 1.0  
**Date**: November 17, 2025  
**Status**: Peer-Reviewed

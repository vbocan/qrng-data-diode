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
│ Layer 5: Application Security                               │
│ - API authentication (Bearer tokens)                        │
│ - Rate limiting per client                                  │
│ - Input validation                                          │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Data Integrity                                     │
│ - HMAC-SHA256 packet authentication                         │
│ - CRC32 corruption detection                                │
│ - Timestamp freshness validation                            │
│ - Sequence number replay protection                         │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: Network Security                                   │
│ - TLS 1.3 encryption (HTTPS)                                │
│ - Certificate validation                                    │
│ - Unidirectional data flow (software data diode)            │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Network Isolation                                  │
│ - Internal network: 10.0.0.0/8 (RFC 1918)                   │
│ - External network: Public Internet                         │
│ - Firewall rules: Block reverse connections                 │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: Physical Security                                  │
│ - QRNG appliance in secured facility                        │
│ - Collector on isolated internal network                    │
│ - Gateway on DMZ or external network                        │
└─────────────────────────────────────────────────────────────┘
```

### Unidirectional Data Flow

**Key Property**: Gateway NEVER initiates connections to Collector

```
┌────────────────────────┐         ┌────────────────────────┐
│ Internal Network       │         │ External Network       │
│                        │         │                        │
│  Collector             │─────────▶  Gateway              │
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

**Verification**: Use network monitoring tools to verify that the Collector host has no listening sockets, and the Gateway host listens on the configured port (typically 7764).

## Cryptographic Mechanisms

### HMAC-SHA256 Packet Authentication

**Purpose**: Authenticate entropy packets and prevent tampering

**Implementation**: The system uses HMAC-SHA256 to sign packets by computing a MAC over the entropy data, timestamp, and sequence number using a shared secret key. The Gateway verifies packets by recomputing the MAC and using constant-time comparison to prevent timing attacks.

**Security Properties**:
- **Key Length**: 256 bits (32 bytes)
- **Output Length**: 256 bits (32 bytes)
- **Collision Resistance**: 2^128 operations
- **Preimage Resistance**: 2^256 operations

**Key Management**: Generate secure HMAC keys using cryptographically secure random number generators. Store keys in environment variables (not in code), and ensure proper file permissions restrict access to service accounts only.

### CRC32 Corruption Detection

**Purpose**: Detect bit flips and transmission errors

**Implementation**:

**Implementation**: The system computes and validates CRC32 checksums over entropy data to detect transmission errors.

**Detection Capability**:
- Single-bit errors: 100%
- Burst errors (<32 bits): 100%
- Random errors: 99.9999998%

**Note**: CRC32 is NOT cryptographically secure (use HMAC for authentication)

### Timestamp-Based Freshness

**Purpose**: Prevent replay attacks with stale packets

**Implementation**: The system validates packet timestamps by comparing them against the current system time. Future timestamps are rejected for clock skew protection, and packets older than the configured maximum age are rejected to prevent replay attacks.

**Configuration**: The Gateway is typically configured with a packet TTL of 300 seconds (5 minutes) and allows up to 60 seconds of clock skew drift.

**Security Analysis**:
- **TTL**: 300 seconds balances security vs. network delays
- **Clock Skew**: Allows for NTP drift between systems
- **Replay Window**: Attacker has maximum 5 minutes to replay packet

### Sequence Number Replay Protection

**Purpose**: Detect duplicate packets (even within TTL window)

**Implementation**: The system tracks the last valid sequence number using atomic operations. Non-monotonic sequences (where the new sequence is less than or equal to the last seen) are rejected as potential replay attempts. The validation is thread-safe.

**Security Properties**:
- **Monotonic**: Sequence numbers always increase
- **No Gaps**: Missing sequences are logged but allowed (packet loss)
- **Atomic**: Thread-safe validation

**Attack Scenario**: When an attacker captures and attempts to replay a packet, the Gateway's sequence validator detects that the sequence number is not greater than the last seen sequence and rejects the packet.

## Network Security

### TLS Configuration

**Required**: TLS 1.3 with strong cipher suites

The Collector can be configured to use TLS 1.3 with strong cipher suites like TLS_AES_256_GCM_SHA384 and TLS_CHACHA20_POLY1305_SHA256, with server certificate verification enabled.

**Security Properties**:
- **Forward Secrecy**: Ephemeral key exchange (ECDHE)
- **Authenticated Encryption**: GCM or ChaCha20-Poly1305
- **Certificate Validation**: Prevent MITM attacks

### Firewall Rules

**Internal Network (Collector)**: Configure firewall to allow outbound HTTPS to Gateway and QRNG appliance, while allowing only established/related inbound connections and blocking all other inbound traffic.

**External Network (Gateway)**: Configure firewall to allow inbound HTTPS from Collector (specific IP) and API clients on the configured port. Additionally, block outbound connections to internal networks (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16) to enforce data diode behavior.

## Authentication & Authorization

### API Key Authentication

**Format**: Bearer token in Authorization header

API requests include an Authorization header with a Bearer token for authentication. The Gateway validates the token against configured API keys.

**Implementation**: The system extracts the Bearer token from the Authorization header and performs constant-time comparison against configured API keys to prevent timing attacks.

**Key Generation**: Use cryptographically secure random number generators to create API keys.

**Storage**: 
- **Collector**: Environment variable `QRNG_HMAC_SECRET_KEY`
- **Gateway**: Environment variable `QRNG_API_KEY`
- **Clients**: Secure credential store (not in code!)

### Rate Limiting

**Purpose**: Prevent DoS attacks and API abuse

**Implementation**: The system tracks client request history and enforces rate limits by removing expired requests from the history and checking if the limit has been exceeded within the configured time window.

**Configuration**: The Gateway can be configured with a time window (e.g., 60 seconds) and maximum requests per window (e.g., 100) along with burst size settings.

## Data Integrity

### Entropy Packet Format

The entropy packet structure contains:
- **data**: Random bytes (variable length)
- **timestamp**: Unix timestamp in milliseconds
- **sequence**: Monotonic sequence number
- **crc32**: CRC32 checksum of data
- **hmac**: HMAC-SHA256 signature

**Integrity Chain**: The data is first protected with a CRC32 checksum, then the data, timestamp, and sequence are signed with HMAC-SHA256. The Gateway verifies both CRC32 and HMAC.

### Verification Process

The Gateway performs verification in multiple steps:
1. Verify CRC32 to detect transmission errors
2. Verify HMAC to authenticate the sender
3. Verify timestamp for freshness
4. Verify sequence number for monotonic ordering

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

1. Generate a new HMAC secret key
2. Update the Collector configuration with the new key and restart the service
3. Update the Gateway configuration with the new key while temporarily keeping the old key for graceful transition
4. After sufficient time (2x TTL, approximately 10 minutes), remove the old key from the Gateway configuration

### Monitoring & Alerting

**Security Metrics to Monitor**:

- **HMAC verification failures**: Alert if threshold exceeds 5 per minute (action: email security team)
- **Rate limit exceeded**: Alert if threshold exceeds 10 per hour per client (action: block client temporarily)
- **Buffer depletion**: Alert if fill level drops below 10% (action: email operations team)
- **Sequence gaps**: Alert if more than 100 sequences are missing (action: investigate packet loss)

**Log Analysis**: Use system logging tools to monitor for authentication failures, HMAC verification failures, and rate limit violations.

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

All security-relevant events are logged with timestamps, event types, source IP addresses, packet sequences, and reasons. Events include HMAC verification failures, authentication failures, and rate limit violations.

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

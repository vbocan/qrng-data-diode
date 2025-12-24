# QRNG-DD: A High-Performance Rust Implementation of Software-Based Data Diode Architecture for Quantum Random Number Distribution with AI Agent Integration

**Valer Bocan, PhD, CSSLP**

_Department of Computer and Information Technology, Politehnica University of Timisoara, Timisoara, 300223, Romania_

_Email: valer.bocan@upt.ro_

_ORCID: 0009-0006-9084-4064_

---

## Abstract

QRNG-DD is an open-source research infrastructure for securely distributing quantum random numbers across network boundaries, specifically designed to enable quantum computing experiments, AI-assisted research workflows, and cryptographic studies requiring high-quality entropy. The system implements a software-based data diode architecture that enables researchers to access quantum randomness from protected internal networks while maintaining strict security isolation.

The Rust implementation achieves high-performance entropy delivery with throughput limited by QRNG hardware rather than software overhead. Multi-source entropy aggregation using XOR or HKDF-based mixing provides defense against single-vendor failures and potential backdoors—critical for reproducible research. The Model Context Protocol (MCP) integration enables AI agents to consume quantum randomness through standardized tools, supporting emerging AI-assisted scientific workflows in quantum computing, machine learning, and computational physics.

QRNG-DD addresses key research challenges including transparent entropy distribution for reproducible studies, AI-accessible quantum randomness for autonomous research agents, high-throughput low-latency delivery for Monte Carlo simulations, and cost-effective deployment for academic institutions. All source code, configurations, and benchmark artifacts are provided under MIT license to support reproducible research and community extension.

**Keywords:** quantum random number generator, QRNG, data diode, entropy distribution, Rust, Model Context Protocol, MCP, AI agents, cryptography, network security, high-performance computing

---

## 1. Motivation and Significance

### 1.1 Problem Statement

Quantum Random Number Generators (QRNGs) provide randomness derived from fundamental quantum processes, offering a qualitatively different security foundation than pseudo-random number generators that rely on deterministic algorithms [1]. This distinction is critical for cryptographic key generation, scientific simulations requiring unbiased sampling, and security protocols depending on unpredictable values.

Commercial QRNG appliances such as ID Quantique's Quantis are frequently deployed on isolated internal networks due to security policies [2]. This creates an accessibility paradox: quantum entropy is protected but difficult for researchers, AI systems, and external applications to access. Hardware data diodes enforce unidirectional data flow but cost $5,000–$50,000 and require specialized infrastructure [3], making them prohibitive for academic institutions.

Public QRNG services from ANU and NIST expose quantum entropy via Internet APIs [4][5] but impose rate limits, small response sizes, high latency (~450ms), and privacy concerns through centralized logging. The emerging field of AI-assisted research compounds these challenges: agents increasingly support scientific workflows but must access services through ad hoc HTTP clients rather than standardized tools, hindering systematic adoption of quantum entropy.

### 1.2 Innovation and Contribution

QRNG-DD introduces a software-based data diode architecture enabling secure quantum entropy distribution without hardware data diode costs. An Entropy Collector on the internal network fetches quantum data, signs and encapsulates it, and pushes to an Entropy Gateway on the external network. The Gateway never initiates reverse connections, emulating hardware data diode properties through software architecture [3].

The Rust implementation benefits from compile-time memory safety guarantees preventing buffer overflows and data races [6], while zero-copy buffers and asynchronous I/O achieve high performance. Model Context Protocol integration exposes quantum randomness through standardized tools [7], and multi-source aggregation using XOR or HKDF mixing addresses vendor backdoor concerns through information-theoretic security guarantees [8].

### 1.3 Target Research Domains

QRNG-DD addresses infrastructure needs across quantum computing (state initialization, gate testing, error correction validation), machine learning (weight initialization, dropout, data augmentation), cryptographic research (key generation, nonce generation, backdoor-resistant protocols), and computational physics (Monte Carlo methods, random walks, statistical mechanics). The emerging paradigm of AI-assisted research, where autonomous agents design experiments and iterate hypotheses, requires standardized quantum entropy access through tools like MCP [1].

### 1.4 Related Work and Research Gap

Hardware data diodes from vendors like Owl Cyber Defense provide physical unidirectional guarantees through fiber-optic transmission [3], offering maximum security but requiring substantial investment. QRNG-DD trades physical guarantees for practical software isolation at significantly reduced cost, suitable for research and moderate-security deployments.

Public QRNG services from ANU and NIST democratize access but impose limitations unsuitable for research-grade applications [4][5]: rate limits, small response sizes, high latency, and privacy concerns through centralized logging. QRNG-DD's self-hosted architecture eliminates these constraints while ensuring complete privacy.

Commercial QRNG appliances provide quantum hardware with basic APIs but lack data isolation, AI integration, and multi-source mixing [2]. Direct API implementations in C [13] enable embedded integration but introduce memory safety risks that affect approximately 70% of security vulnerabilities in major software projects [14][15]. QRNG-DD uses Rust where ownership rules enforce memory safety at compile-time, addressing distributed network infrastructure where security boundary enforcement justifies the language choice.

Academic literature on software data diodes remains limited [9], leaving QRNG-DD as an open-source system uniquely combining software data diode emulation, quantum entropy distribution, and AI integration through the Model Context Protocol [7].

---

## 2. Software Description

### 2.1 Architecture Overview

QRNG-DD implements a three-tier research infrastructure. The Entropy Collector operates on the internal network, fetching quantum entropy via HTTPS, signing packets with HMAC-SHA256, and pushing data outward. The Entropy Gateway on the external network verifies signatures, buffers entropy, and serves an authenticated REST API. The MCP server exposes quantum randomness to AI agents via JSON-RPC 2.0. The unidirectional data flow from Collector to Gateway with no reverse communication emulates hardware data diode properties through software architecture, enabling secure cross-boundary distribution for research environments.

Configuration via environment variables, Docker containerization for reproducible deployment, and comprehensive observability through Prometheus metrics and structured logging facilitate research adoption and experimental reproducibility.

### 2.2 Performance Characteristics

Rust's zero-cost abstractions and asynchronous I/O enable gateway processing with negligible overhead, suitable for interactive workflows and Monte Carlo simulations. Sustained throughput is constrained by QRNG appliance entropy generation rather than software processing. Multi-source configurations issue parallel requests, achieving aggregate latency approaching the maximum of individual sources. Detailed benchmarks are presented in Section 4.

### 2.3 Integrity Verification

Four independent integrity layers ensure entropy authenticity: HMAC-SHA256 signatures prevent forgery, CRC32 checksums detect transmission errors, timestamp validation prevents replay attacks within configurable TTL windows, and sequence numbers provide monotonic ordering. All checks must pass for packet acceptance, creating auditable entropy provenance for peer-reviewed publications.

### 2.4 AI-Assisted Research Through Model Context Protocol

The MCP server exposes quantum randomness through six standardized JSON-RPC 2.0 tools organized into two functional categories. Four data generation tools provide random bytes with configurable encoding for cryptographic experiments, random integers within specified ranges for algorithm testing, random floats in the unit interval for Monte Carlo simulations, and quantum-seeded UUIDs for experimental tracking. Two utility tools complement these: a status endpoint for monitoring gateway health and buffer levels, and a Monte Carlo π estimation tool for immediate quality validation achieving sub-0.0002% error rates.

Each tool returns structured JSON with source attribution marking entropy as quantum-derived, enabling AI agents to cite quantum sources in automated research reports and distinguish from pseudo-random baselines. This zero-configuration interface eliminates custom integration code, reducing AI agent quantum randomness access from hours of HTTP client development to conversational tool invocation [7].

### 2.5 Multi-Source Entropy Aggregation

Multi-source aggregation addresses vendor backdoor concerns and hardware failure risks. XOR combination provides information-theoretic security: if at least one source generates uniform random bits, the output inherits that uniformity [8]. HKDF-based mixing handles correlated sources through cryptographic extraction [10]. Automatic health monitoring implements fault isolation with configurable retry intervals, enabling heterogeneous QRNG arrays resilient to single-point failures.

### 2.6 Research Infrastructure Features

Prometheus metrics and structured logging provide observability for performance analysis: buffer levels, latency histograms, and per-source health tracking. Docker containerization ensures reproducible deployment while native binaries support Linux, macOS, and Windows. Automated testing includes unit tests, integration tests, and property-based testing for cryptographic functions via GitHub Actions.

---

## 3. Research Applications

### 3.1 Quantum Computing Research

QRNG-DD provides critical infrastructure for quantum computing experiments requiring genuine quantum randomness. Researchers can generate random quantum states for algorithm testing, create input vectors for validating quantum gate implementations, and produce error patterns for testing quantum error correction codes. Through MCP integration, AI agents can autonomously design experiments, generate test vectors, and analyze results—accelerating research cycles beyond manual workflows.

### 3.2 AI and Machine Learning Research

AI-assisted research workflows benefit from standardized quantum entropy access. Neural network training depends on weight initialization where quantum randomness avoids biases that pseudo-random generators can introduce. Stochastic optimization techniques, dropout regularization, and data augmentation pipelines all require high-quality randomness. AI agents access quantum entropy conversationally through MCP tools, eliminating integration barriers and enabling autonomous experimental iteration.

### 3.3 Computational Physics and Monte Carlo Methods

Monte Carlo simulations rely on vast quantities of unbiased entropy that QRNG-DD delivers efficiently. Random walk studies, statistical mechanics simulations, and Markov Chain Monte Carlo methods achieve their theoretical convergence properties only with sufficiently high-quality randomness. Built-in quality validation through Monte Carlo π estimation—achieving 0.0002% error with 10,000,000 iterations—demonstrates statistical quality suitable for research-grade simulations.

### 3.4 Cryptographic Research

Multi-source entropy aggregation makes QRNG-DD valuable for cryptographic studies requiring vendor-independent randomness. The XOR and HKDF mixing strategies demonstrate practical defense against vendor backdoors, providing reproducible infrastructure for security analysis. The standardized test infrastructure supports randomness quality comparisons using NIST SP 800-22 [11] and Dieharder [12] test suites.

### 3.5 Example Implementations

The repository includes 15 Rust applications demonstrating quantum randomness across research domains: Monte Carlo π estimation, genetic algorithms, simulated annealing, random walk simulations, cryptographic key generation, and statistical test suites. PowerShell scripts provide benchmarking and quality validation. All examples include source code, documentation, and sample outputs.

---

## 4. Performance and Comparison

### 4.1 Research Infrastructure Advantages

QRNG-DD provides cost-effective quantum entropy infrastructure through MIT-licensed open-source software requiring no hardware beyond existing QRNG appliances—critical for resource-constrained academic institutions. Benchmark testing confirms sub-4ms median end-to-end latency on local networks (100× faster than public services' 450ms), with gateway internal processing under 100μs regardless of client location. Megabyte-scale request support versus public service 1KB limits enables high-volume research workloads. MCP integration enables AI-assisted research workflows absent from commercial appliances or public services.

**Table 1: Comparison with Related Quantum Entropy Access Solutions**

| Feature | Public QRNG (ANU) | libqrng [13] | QRNG-DD |
|---------|-------------------|--------------|---------|
| **Network Access** | Internet required | Local network | Cross-boundary |
| **Rate Limits** | 5 req/sec | Appliance limit | Appliance limit |
| **Request Size** | 1024 bytes max | Configurable | Megabyte scale |
| **Latency** | ~450ms | <1ms (local) | <10ms P99 |
| **Privacy** | Centralized logs | Private | Private |
| **Security Model** | HTTPS | Direct appliance | Data diode |
| **Target Use Case** | Education | Embedded systems | Distributed infrastructure |
| **AI Integration** | None | None | MCP native |
| **Language** | N/A | C | Rust |
| **Memory Safety** | N/A | Manual | Compile-time verified |
| **Multi-source** | No | No | Yes (XOR/HKDF) |
| **Cost** | Free | $0 software | $0 software |

**Table 2: Security Properties Comparison**

| Security Feature | C-based Implementations | QRNG-DD (Rust) |
|------------------|-------------------------|----------------|
| **Memory Safety** | Developer responsibility | Compile-time ownership system |
| **TLS Verification** | Configuration-dependent | Enforced by default |
| **Buffer Overflow Protection** | Manual bounds checking | Type system guarantees |
| **Thread Safety** | Requires documentation | Ownership system enforced |
| **Input Validation** | Manual implementation | Type-level constraints |
| **Error Propagation** | Return codes | Result<T, E> type |
| **Null Pointer Safety** | Runtime checks | Option<T> type |
| **Data Race Prevention** | Synchronization primitives | Borrow checker verification |

### 4.2 Research-Grade Performance Validation

Monte Carlo π estimation with 10,000,000 iterations achieves 0.0002% error, confirming statistical quality for research applications. Local network testing demonstrates 28.74 req/s sustained throughput with P50=3.62ms, P95=6.89ms, P99=9.13ms latency and 100% success rate. Burst capability reaches 438 req/s, constrained by QRNG appliance entropy generation (~80 KB/s) rather than software processing. Remote access verification at `qrng.dataman.ro` confirmed gateway internal processing of ~100μs with end-to-end latency dominated by network round-trip time.

---

## 5. Extensibility for Research

The modular architecture supports research extensions: custom entropy sources integrate via trait implementations for additional QRNG vendors, custom mixing strategies implement specialized aggregation algorithms, and pluggable test suites extend validation beyond Monte Carlo π to NIST SP 800-22 [11] or Dieharder [12]. The shared core library provides reusable Rust components applicable to broader research software development.

---

## 6. Limitations and Future Research

Software data diode architecture provides adequate isolation for research but weaker guarantees than hardware solutions—critical infrastructure should employ physical data diodes. Throughput is constrained by QRNG appliance entropy generation (~80 KB/s); higher throughput requires multiple appliances with multi-source aggregation. Future development includes additional QRNG vendor integration and federated entropy distribution for multi-institution collaborations.

---

## 7. Conclusions

QRNG-DD provides open-source research infrastructure for quantum entropy distribution addressing needs in quantum computing, AI-assisted research, and computational physics. The software-based data diode architecture achieves sub-4ms median latency with negligible processing overhead, while MCP integration enables AI agents to access quantum randomness through standardized tools. Multi-source entropy aggregation addresses vendor backdoor concerns through information-theoretic mixing guarantees. The MIT-licensed implementation provides complete transparency, enabling researchers to validate entropy provenance and extend the system for domain-specific applications.

---

## Acknowledgments

The author gratefully acknowledges Politehnica University of Timisoara for providing access to the Quantis appliance used in the development and testing of this system.

---

## References

[1] M. Herrero-Collantes and J. C. Garcia-Escartin, "Quantum random number generators," Reviews of Modern Physics, vol. 89, no. 1, article 015004, 2017. doi: 10.1103/RevModPhys.89.015004

[2] ID Quantique, "Quantis QRNG Appliance," 2024. [Online]. Available: https://www.idquantique.com/random-number-generation/products/quantis-rng-appliance/. [Accessed: 03-Oct-2025].

[3] Owl Cyber Defense, "Data Diode Technology," 2024. [Online]. Available: https://owlcyberdefense.com/resource/data-diode-security-solutions/. [Accessed: 21-Oct-2025].

[4] Australian National University, "ANU QRNG API," 2024. [Online]. Available: https://qrng.anu.edu.au/. [Accessed: 28-Oct-2025].

[5] National Institute of Standards and Technology, "NIST Randomness Beacon," 2024. [Online]. Available: https://beacon.nist.gov/. [Accessed: 28-Oct-2025].

[6] R. Jung, J.-H. Jourdan, R. Krebbers, and D. Dreyer, "RustBelt: Securing the Foundations of the Rust Programming Language," Proceedings of the ACM on Programming Languages, vol. 2, no. POPL, article 66, pp. 1-34, 2018. doi: 10.1145/3158154

[7] Anthropic, "Model Context Protocol Specification," 2024. [Online]. Available: https://modelcontextprotocol.io/docs/getting-started/intro. [Accessed: 07-Dec-2024].

[8] C. H. Bennett, G. Brassard, and J.-M. Robert, "Privacy amplification by public discussion," SIAM Journal on Computing, vol. 17, no. 2, pp. 210-229, 1988. doi: 10.1137/0217014

[9] A. Ginter, "Unidirectional Security Gateways: Stronger Than Firewalls," in Proc. 14th Int. Conf. Accelerator and Large Experimental Physics Control Systems (ICALEPCS'13), San Francisco, CA, USA, Oct. 2013, paper THCOBA02. [Online]. Available: https://proceedings.jacow.org/ICALEPCS2013/papers/thcoba02.pdf [Accessed: 19-Nov-2025].

[10] H. Krawczyk and P. Eronen, "HMAC-based Extract-and-Expand Key Derivation Function (HKDF)," RFC 5869, Internet Engineering Task Force, May 2010. [Online]. Available: https://datatracker.ietf.org/doc/html/rfc5869 [Accessed: 10-Oct-2025].

[11] A. Rukhin et al., "A Statistical Test Suite for Random and Pseudorandom Number Generators for Cryptographic Applications," NIST Special Publication 800-22 Rev. 1a, National Institute of Standards and Technology, Apr. 2010. [Online]. Available: https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-22r1a.pdf [Accessed: 10-Oct-2025].

[12] R. G. Brown, "Dieharder: A Random Number Test Suite," 2024. [Online]. Available: https://webhome.phy.duke.edu/~rgb/General/dieharder.php. [Accessed: 25-Oct-2025].

[13] S. M. Ardelean, M. Udrescu, and V. Stangaciu, "Easy to integrate API for accessing true random numbers generated with IDQ's Quantis Appliance," SoftwareX, vol. 27, article 101841, 2024. doi: 10.1016/j.softx.2024.101841

[14] M. Miller, "Trends, challenges, and strategic shifts in the software vulnerability mitigation landscape," Microsoft Security Response Center, Feb. 2019. [Online]. Available: https://github.com/microsoft/MSRC-Security-Research/blob/master/presentations/2019_02_BlueHatIL/2019_01%20-%20BlueHatIL%20-%20Trends%2C%20challenge%2C%20and%20shifts%20in%20software%20vulnerability%20mitigation.pdf [Accessed: 19-Nov-2025].

[15] "Memory safety," The Chromium Projects, 2024. [Online]. Available: https://www.chromium.org/Home/chromium-security/memory-safety/ [Accessed: 19-Nov-2025].

---

## Code Metadata

| Metadata Item                         | Description                                                                                                                                                                                                                                               |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Current code version**              | v1.0.0                                                                                                                                                                                                                                                    |
| **Permanent link to code/repository** | https://github.com/vbocan/qrng-data-diode                                                                                                                                                                                                                 |
| **Legal Code License**                | MIT License                                                                                                                                                                                                                                               |
| **Code versioning system used**       | Git                                                                                                                                                                                                                                                       |
| **Software code languages**           | Rust 1.75+                                                                                                                                                                                                                                                |
| **Compilation requirements**          | Rust 1.75+ toolchain (no OpenSSL required - uses Rustls), Docker & Docker Compose (optional)                                                                                                                                                             |
| **Operating environments**            | Linux, macOS, Windows                                                                                                                                                                                                                                     |
| **Dependencies**                      | tokio 1.48 (async runtime), axum 0.8 (HTTP server), bytes 1.11 (zero-copy buffers), parking_lot 0.12 (locks), serde 1.0 (serialization), hmac 0.12 + sha2 0.10 (cryptography), crc32fast 1.5 (checksums), metrics 0.24 (observability), tracing 0.1 (logging) |
| **Link to developer documentation**   | [Developer Guide](https://github.com/vbocan/qrng-data-diode/blob/master/README.md)                                                                                                                                                                        |
| **Support email**                     | valer.bocan@upt.ro                                                                                                                                                                                                                                        |

---

## Software Availability

- **Repository**: https://github.com/vbocan/qrng-data-diode
- **Documentation**: https://github.com/vbocan/qrng-data-diode/tree/master/docs
- **Supplementary Materials**:
  - Architecture Documentation: [docs/ARCHITECTURE.md](https://github.com/vbocan/qrng-data-diode/blob/master/docs/ARCHITECTURE.md)
  - Performance Testing: [docs/BENCHMARK.md](https://github.com/vbocan/qrng-data-diode/blob/master/docs/BENCHMARK.md)
  - Security Analysis: [docs/SECURITY-ANALYSIS.md](https://github.com/vbocan/qrng-data-diode/blob/master/docs/SECURITY-ANALYSIS.md)
  - MCP Integration Guide: [docs/MCP-INTEGRATION.md](https://github.com/vbocan/qrng-data-diode/blob/master/docs/MCP-INTEGRATION.md)

---

**Funding**: This research received no specific grant from any funding agency in the public, commercial, or not-for-profit sectors.

**Conflict of Interest**: The author declares no competing interests.

**Data Availability**: All source code, configuration examples, test cases, and benchmark data are included in the GitHub repository under MIT license.

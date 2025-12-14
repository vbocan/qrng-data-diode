# QRNG-DD: A High-Performance Rust Implementation of Software-Based Data Diode Architecture for Quantum Random Number Distribution with AI Agent Integration

**Valer Bocan, PhD, CSSLP**

_Department of Computer and Information Technology, Politehnica University of Timisoara, Timisoara, 300223, Romania_

_Email: valer.bocan@upt.ro_

_ORCID: 0009-0006-9084-4064_

---

## Abstract

QRNG-DD is an open-source research infrastructure for securely distributing quantum random numbers across network boundaries, specifically designed to enable quantum computing experiments, AI-assisted research workflows, and cryptographic studies requiring high-quality entropy. The system implements a software-based data diode architecture that enables researchers to access quantum randomness from protected internal networks while maintaining strict security isolation.

The Rust implementation achieves sub-4ms median end-to-end latency on local networks (with gateway internal processing under 100μs), throughput limited by QRNG hardware rather than software overhead. Multi-source entropy aggregation using XOR or HKDF-based mixing provides defense against single-vendor failures and potential backdoors—critical for reproducible research. The Model Context Protocol (MCP) integration enables AI agents to consume quantum randomness through standardized tools, supporting emerging AI-assisted scientific workflows in quantum computing, machine learning, and computational physics.

QRNG-DD addresses key research challenges including transparent entropy distribution for reproducible studies, AI-accessible quantum randomness for autonomous research agents, high-throughput low-latency delivery for Monte Carlo simulations, and cost-effective deployment for academic institutions. All source code, configurations, and benchmark artifacts are provided under MIT license to support reproducible research and community extension.

**Keywords:** quantum random number generator, QRNG, data diode, entropy distribution, Rust, Model Context Protocol, MCP, AI agents, cryptography, network security, high-performance computing

---

## 1. Motivation and Significance

### 1.1 Problem Statement

Quantum Random Number Generators (QRNGs) provide randomness derived from fundamental quantum processes such as photon detection and vacuum fluctuations, offering a qualitatively different security foundation than pseudo-random number generators (PRNGs) that rely on deterministic algorithms and internal state [1]. This distinction is critical for applications demanding high-quality unpredictability, including cryptographic key generation where seed compromise undermines entire systems, scientific simulations that must avoid systematic sampling biases, and security protocols that depend on nonces and challenge–response values.

Commercial QRNG appliances such as ID Quantique's Quantis family are frequently deployed on internal organizational networks isolated from the Internet due to security policies governing sensitive research infrastructure. This creates an accessibility paradox: the quantum entropy is well protected from external threats but becomes difficult for researchers, AI systems, and external applications to use [2]. Organizations consequently face an uncomfortable choice between exposing QRNG appliances to external networks with increased risk of unauthorized access and denial-of-service attacks, or accepting restricted accessibility that limits the return on investment in quantum hardware.

Hardware data diodes offer a traditional solution for enforcing unidirectional data flow: optical transmission paths are physically configured so that information can only move from internal to external networks, with no reverse communication possible [3]. These specialized devices, however, typically cost between $5,000 and $50,000 and require dedicated rack space, specialized fiber-optic cabling, and inflexible configurations that are difficult to modify after installation. Academic institutions with limited budgets, small organizations without dedicated security teams, and exploratory projects often find hardware data diodes economically and operationally prohibitive.

Public QRNG services operated by institutions such as the Australian National University and the National Institute of Standards and Technology partially address accessibility challenges by exposing quantum entropy via Internet-accessible APIs [4][5]. These services are valuable for education and small experiments but impose strict rate limits (for example, ANU allows only 5 requests per second and NIST one pulse per minute), small response sizes (typically 500–1000 bytes), and substantial network latency around 450 ms for transcontinental traffic. They also require full Internet connectivity, which precludes air-gapped environments, and their centralized logging of all requests can raise privacy concerns for sensitive research.

The emerging field of AI-assisted research introduces additional complexity as artificial intelligence agents increasingly support scientific workflows, including experimental parameter selection, data analysis automation, and iterative hypothesis refinement. Today these agents typically access remote services through ad hoc HTTP clients rather than standardized tools. As a result, integrating quantum randomness often requires custom authentication, binary parsing, error handling, and retry logic for each platform, which hinders systematic adoption of quantum entropy in AI-driven research.

### 1.2 Innovation and Contribution

QRNG-DD introduces a software-based data diode architecture that enables secure quantum entropy distribution without the cost and deployment overhead of hardware data diodes. It implements a split design in which an Entropy Collector on the internal network fetches quantum data from the QRNG, signs and encapsulates it, and pushes it to an Entropy Gateway on the external network that serves API clients. A critical constraint of this design is that the Gateway never initiates reverse connections to the internal network, emulating hardware data diode properties through software architecture [3].

This architecture provides cost-effective security for academic research and moderate-security deployments compared to hardware data diodes, while retaining flexibility through configuration of operational parameters and offering comprehensive audit capabilities via structured logging and Prometheus metrics. Because QRNG-DD is implemented in Rust, it benefits from compile-time memory safety guarantees that prevent common vulnerability classes such as buffer overflows and data races [6]. In benchmarking, zero-copy buffers, efficient reader–writer locks, and asynchronous I/O yield gateway internal processing times under 100 μs (P50), with end-to-end latency on local networks under 4 ms median and 99th percentile below 10 ms. Sustained throughput is primarily limited by the QRNG appliance's entropy generation rate rather than software processing.

The Model Context Protocol integration exposes quantum randomness through standardized tool interfaces that eliminate custom HTTP client development and reduce integration from hours to zero-configuration deployment [7]. Built-in Monte Carlo π estimation provides immediate quality validation achieving sub-0.0002% error rates, while multi-source aggregation using XOR or HKDF mixing addresses vendor dependence and backdoor concerns through information-theoretic security guarantees [8].

### 1.3 Target Research Domains

QRNG-DD addresses critical infrastructure needs across multiple research domains. In quantum computing, experiments require genuine quantum randomness for quantum state initialization, quantum gate testing, quantum error correction validation, and quantum algorithm benchmarking where pseudo-random sequences introduce systematic biases that compromise experimental validity [1].

Machine learning research increasingly demands high-quality randomness for stochastic gradient descent initialization, dropout layer operation, data augmentation, and adversarial example generation. The emerging paradigm of AI-assisted research, where autonomous agents design experiments, analyze results, and iterate hypotheses, requires standardized access to quantum entropy through tools like MCP rather than custom integration code.

Cryptographic research benefits from multi-source entropy aggregation that demonstrates practical defense against vendor backdoors while providing reproducible infrastructure for comparing key generation protocols, nonce generation schemes, and cryptographic primitive testing. Computational physics simulations including Monte Carlo methods, random walk studies, and statistical mechanics experiments require vast quantities of high-quality randomness where QRNG-DD's throughput and latency characteristics enable practical research deployment.

### 1.4 Related Work and Research Gap

Hardware data diodes from vendors like Owl Cyber Defense provide physical unidirectional guarantees through fiber-optic transmission with removed receive capability [3], offering maximum security for critical infrastructure but requiring high investments and inflexible deployment. QRNG-DD trades physical guarantees for practical software isolation while maintaining adequate security for research and moderate-security deployments at significantly reduced cost.

Public QRNG services from ANU and NIST democratize quantum randomness access but impose limitations unsuitable for research-grade applications [4][5]: ANU limits requests to 5/sec. with 1024-byte maximums and 450ms latency, while NIST provides only 1 pulse per minute. These services require Internet connectivity, raising privacy concerns through centralized request logging. QRNG-DD's self-hosted architecture eliminates rate limits, supports megabyte requests, achieves single-digit millisecond latencies, and ensures complete privacy.

Commercial QRNG appliances like ID Quantique's Quantis provide quantum hardware with basic APIs but lack data isolation, AI integration, multi-source mixing, and quality validation [2]. Organizations with existing appliances can add QRNG-DD at zero hardware cost, gaining sophisticated distribution capabilities. The Model Context Protocol from Anthropic establishes AI tool integration standards [7], enabling researchers to access quantum randomness in AI-assisted workflows.

Direct API implementations in C for Quantis appliances [13] enable embedded system integration, yet highlight persistent challenges in systems programming where manual memory management and configuration complexity introduce security risks. Microsoft's analysis of security vulnerabilities reported that approximately 70% of all security issues in their products stemmed from memory safety bugs [14], while Chromium project data similarly identified memory safety issues as the dominant vulnerability class [15], motivating QRNG-DD's use of Rust where ownership rules enforce memory safety at compile-time and secure-by-default library APIs prevent common pitfalls including disabled certificate verification, buffer overflows, and race conditions. The trade-off between C's embedded systems compatibility and Rust's safety guarantees reflects differing deployment priorities: C-based solutions target resource-constrained devices with local network access, while QRNG-DD addresses distributed network infrastructure where security boundary enforcement justifies additional runtime dependencies.

Academic literature on software data diodes remains limited, with most work focusing on hardware or theoretical models [9], leaving QRNG-DD as an open-source system combining software data diode emulation, quantum entropy distribution, AI integration, and production-grade performance.

---

## 2. Software Description

### 2.1 Architecture Overview

QRNG-DD implements a three-tier research infrastructure: (1) Entropy Collector on the internal network fetches quantum entropy via HTTPS, signs packets with HMAC-SHA256, and pushes outward; (2) Entropy Gateway on the external network verifies signatures, buffers entropy, and serves authenticated REST API; (3) MCP server exposes quantum randomness to AI agents via JSON-RPC 2.0. The unidirectional data flow (Collector→Gateway) with no reverse communication emulates hardware data diode properties through software architecture, enabling secure cross-boundary distribution for research environments.

Configuration via YAML or environment variables, Docker containerization for reproducible deployment, and comprehensive observability through Prometheus metrics and structured logging facilitate research adoption and experimental reproducibility.

### 2.2 Performance Characteristics for Research Workloads

Rust's zero-cost abstractions and asynchronous I/O achieve gateway internal processing times of approximately 100μs (P50) to 2ms (P99), with end-to-end latency on local networks under 4ms median and below 10ms at the 99th percentile—critical for interactive research workflows and Monte Carlo simulations requiring millions of samples. Remote access over the internet introduces additional network latency (typically 50-200ms depending on geographic distance), though the gateway's internal processing overhead remains negligible. The Gateway demonstrates 438 req/s burst capability when buffers are full on local networks, while sustained throughput (28.74 req/s for 1KB requests) is constrained by QRNG appliance entropy generation rate (~80 KB/s) rather than software processing. Remote clients can mitigate network latency impact through parallel requests, achieving approximately 54 req/s with 50 concurrent connections. Multi-source configurations issue parallel requests, achieving aggregate latency approaching the maximum of individual sources rather than cumulative delays—essential for high-throughput research applications.

### 2.3 Integrity Verification

Four independent integrity layers ensure entropy authenticity for research reproducibility: HMAC-SHA256 signatures prevent forgery with 2^256 preimage resistance, CRC32 checksums detect transmission errors with sub-millisecond verification overhead, timestamp validation prevents replay attacks within configurable TTL windows (default 300s), and sequence numbers provide monotonic ordering guarantees. All checks must pass for packet acceptance, creating auditable entropy provenance essential for peer-reviewed publications.

### 2.4 AI-Assisted Research Through Model Context Protocol

The MCP server exposes quantum randomness through 5 standardized JSON-RPC 2.0 tools enabling AI-assisted research workflows: (1) random bytes (1-1048576 bytes, hex/base64) for cryptographic experiments, (2) random integers with configurable ranges for algorithm testing, (3) random floats [0.0, 1.0) for Monte Carlo simulations, (4) quantum-seeded UUIDs for experimental tracking, and (5) Monte Carlo π estimation (1000-100000000 iterations) for immediate quality validation achieving sub-0.0002% error rates.

Each tool returns structured JSON with source attribution marking entropy as quantum-derived, enabling AI agents to cite quantum sources in automated research reports and distinguish from pseudo-random baselines. This zero-configuration interface eliminates custom integration code, reducing AI agent quantum randomness access from hours of HTTP client development to conversational tool invocation [7].

### 2.5 Multi-Source Entropy Aggregation for Research Robustness

Multi-source aggregation addresses vendor backdoor concerns and hardware failure risks critical for high-assurance research. XOR combination provides information-theoretic security guarantees: if at least one independent source generates uniform random bits, the combined output necessarily inherits that uniformity [8]. HKDF-based mixing handles potentially correlated sources through cryptographic extraction maintaining uniformity despite complex correlation patterns [10].

Automatic health monitoring tracks per-source success rates and latencies, implementing fault isolation (default: 3 consecutive failures trigger exclusion) with automatic 60-second retry intervals. This enables researchers to deploy heterogeneous QRNG arrays combining multiple vendors or technologies for robust entropy generation resilient to single-point failures.

### 2.6 Research Infrastructure Features

Prometheus metrics and structured JSON logging provide comprehensive observability essential for research: buffer fill levels, latency histograms (P50/P95/P99), per-source health tracking, and distributed tracing enable performance analysis and experimental validation. Docker containerization ensures reproducible deployment across research environments while native binaries support Linux, macOS, and Windows for diverse laboratory configurations.

Automated testing includes unit tests, integration tests with container frameworks, property-based testing for cryptographic functions, and continuous integration via GitHub Actions—supporting reproducible builds and peer review validation.

---

## 3. Research Applications

### 3.1 Quantum Computing Research

QRNG-DD provides critical infrastructure for quantum computing experiments requiring genuine quantum randomness for:
- **Quantum state initialization**: Generating random quantum states for algorithm testing and quantum circuit benchmarking
- **Quantum gate testing**: Creating random input vectors for validating quantum gate implementations and error rates
- **Quantum error correction**: Generating error patterns for testing quantum error correction codes and fault-tolerant protocols
- **Algorithm benchmarking**: Providing unbiased test inputs for comparing quantum algorithms like Shor's and Grover's against classical baselines

The MCP integration enables AI agents to autonomously design quantum experiments, generate test vectors, execute simulations, and analyze results—accelerating quantum computing research cycles.

### 3.2 AI and Machine Learning Research

AI-assisted research workflows benefit from standardized quantum entropy access:
- **Neural network initialization**: Quantum randomness for weight initialization avoiding pseudo-random biases in deep learning
- **Stochastic optimization**: High-quality randomness for gradient descent, dropout, and Monte Carlo tree search
- **Data augmentation**: Quantum-derived transformations for training data enhancement in computer vision and NLP
- **Adversarial testing**: Generating attack vectors for robustness evaluation of ML models

AI agents access quantum entropy conversationally through MCP tools, eliminating integration barriers and enabling autonomous experimental iteration.

### 3.3 Computational Physics and Monte Carlo Methods

High-throughput low-latency delivery supports computationally intensive simulations:
- **Monte Carlo integration**: Quantum randomness for variance reduction in numerical integration and particle physics simulations
- **Random walk studies**: Unbiased entropy for Brownian motion, diffusion processes, and statistical mechanics
- **Statistical sampling**: High-quality randomness for Markov Chain Monte Carlo and importance sampling

Benchmark testing confirms 10,000,000-iteration Monte Carlo π estimation achieves 0.0002% error, demonstrating statistical quality suitable for research-grade simulations.

### 3.4 Cryptographic Research

Multi-source aggregation enables practical cryptographic studies:
- **Key generation protocols**: Comparing quantum vs. pseudo-random sources for symmetric and asymmetric key generation
- **Backdoor mitigation**: Demonstrating vendor-independent entropy mixing for defense against compromised RNG hardware
- **Nonce generation**: High-throughput quantum randomness for authentication protocols and challenge-response systems
- **Statistical testing**: Reproducible infrastructure for comparing randomness quality across sources using NIST SP 800-22 [11] and Dieharder [12]

### 3.5 Example Implementations

The repository includes 15 Rust applications demonstrating quantum randomness across research domains: cryptographic key generation, UUID generation, Monte Carlo π estimation, genetic algorithms, simulated annealing, random walk simulations, and statistical test suites. PowerShell scripts provide API integration examples for password generation, quality validation, and performance benchmarking. All examples include source code, documentation, and sample outputs supporting reproducible research and educational use.

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

Monte Carlo π estimation with 10,000,000 iterations achieves π = 3.141598 with 0.0002% error, confirming statistical quality for research applications. Local network testing over 600 seconds demonstrates 17,243 successful requests (28.74 req/s, 100% success rate) with P50=3.62ms, P95=6.89ms, P99=9.13ms end-to-end latency—suitable for interactive research workflows. Burst capability reaches 438 req/s when buffers are full, with throughput constrained by QRNG appliance entropy generation (~80 KB/s) rather than software processing. Internal metrics confirm sub-millisecond cryptographic overhead (HMAC 820μs, CRC32 290μs), demonstrating efficient research infrastructure.

Remote access verification against the production deployment at `qrng.dataman.ro` (December 2025) confirmed gateway internal processing of approximately 100μs (P50), with 100% success rate maintained over internet connections. End-to-end latency for remote clients is dominated by network round-trip time rather than gateway processing, with parallel request strategies effectively mitigating latency impact for throughput-sensitive applications.

---

## 5. Extensibility for Research

The modular architecture supports research extensions: custom entropy sources integrate via trait implementations enabling Whitewood netRandom, PicoQuant devices, or experimental QRNG prototypes. Custom mixing strategies implement specialized aggregation algorithms through strategy traits. Statistical validation extends beyond Monte Carlo π through pluggable test suites supporting NIST SP 800-22 [11] or Dieharder [12]. Custom MCP tools enable domain-specific randomness including prime generation for cryptographic research or distribution sampling for statistical studies.

The shared core library provides reusable Rust components (HMAC signing, CRC32 verification, concurrent buffers, MCP server patterns) applicable to broader research software development.

---

## 6. Limitations and Future Research

Software data diode architecture provides adequate isolation for research environments but weaker guarantees than hardware solutions—critical infrastructure deployments should employ physical data diodes. Sustained throughput (28.74 req/s for 1KB requests) is constrained by QRNG appliance entropy generation (~80 KB/s); researchers requiring higher throughput can deploy multiple appliances with multi-source aggregation. Future development includes integration with additional QRNG vendors (Whitewood, PicoQuant), enhanced statistical test suites, and federated entropy distribution for multi-institution research collaborations.

---

## 7. Conclusions

QRNG-DD provides open-source research infrastructure for quantum entropy distribution addressing critical needs in quantum computing, AI-assisted research, and computational physics. The software-based data diode architecture achieves gateway internal processing under 100μs with sub-4ms end-to-end latency on local networks, offering cost advantages over hardware solutions. MCP integration enables AI agents to access quantum randomness through standardized tools—supporting emerging autonomous research workflows.

Multi-source entropy aggregation addresses vendor backdoor concerns through information-theoretic mixing guarantees, while comprehensive observability supports reproducible research and peer review validation. The MIT-licensed implementation provides complete transparency from QRNG appliance to API delivery, enabling researchers to validate entropy provenance and extend the system for domain-specific applications. All source code, benchmarks, and configuration examples are publicly available to support reproducible research and community extension.

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

[9] A. Ginter and J. Tschersich, "Unidirectional Gateways and Industrial Network Security," in Proc. 14th Int. Conf. Accelerator and Large Experimental Physics Control Systems (ICALEPCS'13), San Francisco, CA, USA, Oct. 2013, paper THCOBA02.  [Online] Available: https://proceedings.jacow.org/ICALEPCS2013/papers/thcoba02.pdf [Accessed: 19-Nov-2025]

[10] H. Krawczyk and P. Eronen, "HMAC-based Extract-and-Expand Key Derivation Function (HKDF)," RFC 5869, Internet Engineering Task Force, May 2010. [Online]. Available: https://tools.ietf.org/html/rfc5869 [Accessed: 10-Oct-2025].

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
| **Dependencies**                      | tokio 1.40 (async runtime), axum 0.7/0.8 (HTTP server), bytes 1.7 (zero-copy buffers), parking_lot 0.12 (locks), serde 1.0 (serialization), hmac 0.12 + sha2 0.10 (cryptography), crc32fast 1.4 (checksums), metrics-exporter-prometheus 0.15 (metrics), tracing 0.1 (logging) |
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

---

_Manuscript prepared for submission to SoftwareX_

_December 2025_

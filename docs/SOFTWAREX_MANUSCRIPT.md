# QRNG-DD: A High-Performance Rust Implementation of Software-Based Data Diode Architecture for Quantum Random Number Distribution with AI Agent Integration

**Valer Bocan, PhD, CSSLP**

_Department of Computer and Information Technology, Politehnica University of Timisoara, Timisoara, 300223, Romania_

_Email: valer.bocan@upt.ro_

_ORCID: 0009-0006-9084-4064_

---

## Abstract

QRNG-DD is an open-source system for secure quantum random number distribution across network boundaries using software-based data diode emulation, addressing the challenge of accessing quantum entropy from network-isolated QRNG appliances while maintaining strict unidirectional data flow essential for security-sensitive deployments. Implemented in Rust to leverage memory safety guarantees and high-performance concurrency primitives, the system employs lock-free buffers, zero-copy operations, and asynchronous input/output processing achieving low latency with sustained throughput when connected to a single QRNG appliance. The split architecture comprising an Entropy Collector and Entropy Gateway enforces unidirectional flow without expensive hardware data diodes, instead employing cryptographic integrity mechanisms including HMAC-SHA256 authentication, CRC32 checksums, and timestamp-based freshness validation to provide cost-effective security appropriate for academic research and moderate-security production environments. QRNG-DD enables AI agents to seamlessly access quantum randomness for cryptographic operations and simulations through the Model Context Protocol, eliminating custom integration requirements. The system incorporates built-in statistical validation through Monte Carlo π estimation achieving low error rates, supports multi-source entropy aggregation mitigating vendor dependence and single-point-of-failure risks, and provides comprehensive observability through Prometheus metrics and structured logging suitable for production deployment in research infrastructure.

**Keywords:** quantum random number generator, QRNG, data diode, entropy distribution, Rust, Model Context Protocol, MCP, AI agents, cryptography, network security, high-performance computing

---

## 1. Motivation and Significance

### 1.1 Problem Statement

Quantum Random Number Generators provide true randomness derived from fundamental quantum mechanical processes such as photon detection and vacuum fluctuations, offering a qualitatively different security foundation compared to pseudo-random number generators which produce deterministic sequences from mathematical algorithms that remain vulnerable to prediction if the internal state becomes known [1]. This distinction brings implications for applications demanding unpredictable randomness including cryptographic key generation where compromise of the random seed undermines entire cryptographic systems, scientific simulations requiring genuinely random sampling to avoid systematic biases, and security protocols relying on unpredictability for challenge-response authentication and nonce generation.

Commercial QRNG appliances such as ID Quantique's Quantis family are frequently deployed on internal organizational networks isolated from the Internet due to security policies governing sensitive research infrastructure, creating an accessibility paradox wherein the quantum entropy remains secured against external threats but becomes practically inaccessible to researchers, AI systems, and external applications that would benefit from high-quality randomness [2]. Organizations consequently face an uncomfortable choice between compromising their security posture by exposing QRNG appliances directly to external networks with attendant risks of unauthorized access and denial-of-service attacks, or accepting severely constrained accessibility that limits the utility of expensive quantum hardware investments.

Hardware data diodes offer a traditional solution for achieving unidirectional data flow by physically guaranteeing through optical transmission components with removed receive capability that information flows exclusively from internal to external networks without possibility of reverse communication [3], yet these specialized devices typically have price tags ranging from 5k to 50k dollars while introducing deployment complexity through requirements for dedicated rack space, specialized fiber-optic cabling, and inflexible configuration that resists modification after installation. Academic research institutions operating under constrained budgets, small organizations lacking dedicated security infrastructure teams, and pilot projects exploring quantum randomness applications find hardware data diodes economically prohibitive and operationally burdensome.

Public QRNG services operated by institutions including the Australian National University and the National Institute of Standards and Technology partially address accessibility challenges by providing free quantum random numbers through Internet-accessible application programming interfaces [4][5], yet these valuable educational resources impose practical limitations including strict rate limiting restricting ANU requests to 5/sec. and NIST pulses to 1/min., constrained request sizes typically bounded at 500-1000 bytes, substantial network latency averaging 450ms for transcontinental requests, absolute requirements for Internet connectivity precluding air-gapped research environments, and privacy considerations arising from centralized logging of all randomness requests that may expose sensitive research methodologies.

The emerging field of AI-assisted research introduces additional complexity as artificial intelligence agents increasingly support scientific workflows through tasks including experimental parameter selection, data analysis automation, and iterative hypothesis refinement, yet these agents lack standardized mechanisms for accessing quantum randomness beyond generic HTTP client capabilities, requiring custom implementation of authentication protocols, binary data parsing, error handling, and retry logic that fragments across different AI platforms and hinders systematic adoption of quantum entropy in AI-driven research paradigms.

### 1.2 Innovation and Contribution

QRNG-DD introduces a software-based data diode architecture enabling secure quantum entropy distribution without hardware data diode costs, implementing a split design with an Entropy Collector on the internal network fetching and cryptographically signing quantum data before pushing to an Entropy Gateway on the external network serving API clients, with the critical constraint that the Gateway cannot initiate reverse connections, thereby emulating hardware data diode properties through software architecture [3].

This approach provides cost-effective security for academic research and moderate-security deployments compared to hardware solutions, offers configuration-based flexibility for operational parameters, enables comprehensive audit capabilities through structured logging and Prometheus metrics, and provides open-source transparency for independent security audits and customization. Implemented in Rust, the system leverages compile-time memory safety verification to prevent vulnerability classes including buffer overflows and data races [6], delivering median response times under 4 milliseconds and 99th percentile latency under 10 milliseconds through zero-copy buffers, lock-free concurrency, and asynchronous I/O, with sustained throughput primarily limited by the QRNG appliance's entropy generation rate rather than software processing constraints.

The Model Context Protocol integration exposes quantum randomness through standardized tool interfaces that eliminate custom HTTP client development and reduce integration from hours to zero-configuration deployment [7]. Built-in Monte Carlo π estimation provides immediate quality validation achieving sub-0.0002% error rates, while multi-source aggregation using XOR or HKDF mixing addresses vendor dependence and backdoor concerns through information-theoretic security guarantees [8].

### 1.3 Research and Practical Applications

QRNG-DD's open-source transparency enables reproducible research in quantum randomness, cryptography, and statistical physics by providing complete visibility from QRNG appliance to API delivery, eliminating proprietary black-box concerns while comprehensive logging creates auditable records essential for scientific publication and peer review. Researchers can validate that no pseudo-random augmentation compromises quantum entropy while benefiting from consistent infrastructure for comparative studies.

Quantum computing research benefits from AI-assisted workflows where agents access quantum randomness through MCP for test vector generation, quantum state initialization, and output validation, accelerating research cycles while maintaining statistical rigor. Cryptography education finds practical demonstration of defense-in-depth principles through the data diode architecture, HMAC authentication, and multi-layer integrity checking that students can study, experiment with, and extend.

### 1.4 Related Work and Research Gap

Hardware data diodes from vendors like Owl Cyber Defense provide physical unidirectional guarantees through fiber-optic transmission with removed receive capability [3], offering maximum security for critical infrastructure but requiring high investments and inflexible deployment. QRNG-DD trades physical guarantees for practical software isolation while maintaining adequate security for research and moderate-security deployments at significantly reduced cost.

Public QRNG services from ANU and NIST democratize quantum randomness access but impose limitations unsuitable for research-grade applications [4][5]: ANU limits requests to 5/sec. with 1024-byte maximums and 450ms latency, while NIST provides only 1 pulse per minute. These services require Internet connectivity, raising privacy concerns through centralized request logging. QRNG-DD's self-hosted architecture eliminates rate limits, supports megabyte requests, achieves single-digit millisecond latencies, and ensures complete privacy.

Commercial QRNG appliances like ID Quantique's Quantis provide quantum hardware with basic APIs but lack data isolation, AI integration, multi-source mixing, and quality validation [2]. Organizations with existing appliances can add QRNG-DD at zero hardware cost, gaining sophisticated distribution capabilities. The Model Context Protocol from Anthropic establishes AI tool integration standards [7], enabling researchers to access quantum randomness in AI-assisted workflows. Academic literature on software data diodes remains limited, with most work focusing on hardware or theoretical models [9], leaving QRNG-DD as an open-source system combining software data diode emulation, quantum entropy distribution, AI integration, and production-grade performance.

---

## 2. Software Description

### 2.1 Architecture Overview

QRNG-DD implements a three-tier architecture separating trusted and untrusted networks through three independent components. The Entropy Collector operates on the internal network with QRNG appliance access, fetching entropy via HTTPS, buffering in 1MB circular buffers, signing packets with HMAC-SHA256, adding CRC32 checksums, and pushing to the Gateway every 5 seconds. The Entropy Gateway operates on the external network, receiving pushed entropy, verifying HMAC and CRC32, buffering in 10MB ring buffers, serving REST API with bearer token authentication, providing Prometheus metrics, and implementing rate limiting. The MCP server operates on the external network, fetching entropy from the Gateway on demand, implementing Model Context Protocol using JSON-RPC 2.0, exposing standardized tools for AI agents, and supporting both stdio and HTTP transports.

The shared core library provides common data structures including packet formats, cryptographic utilities for HMAC-SHA256 and CRC32, metrics definitions, and protocol specifications. Configuration via YAML files or environment variables enables deployment flexibility, while Docker containerization ensures reproducible builds. Security boundaries enforce network isolation with the Collector on internal networks, Gateway on external networks, firewall rules blocking all inbound to Collector while allowing only outbound HTTPS push, and critically, no reverse communication path from Gateway to Collector.

### 2.2 High-Performance Design

QRNG-DD achieves production-grade performance through Rust's zero-cost abstractions and modern concurrent programming. Reference-counted byte buffers enable apparent data duplication through lightweight pointer copying rather than memory transfer, with 64KB reads requiring only slice operations and atomic increments. Reader-writer locks optimized for uncontended access avoid system calls, achieving 2-3× faster lock acquisition than standard implementations with 65% faster read latency in benchmarks, ideal for the read-heavy workload of entropy distribution.

Asynchronous I/O through cooperative multitasking runtimes enables efficient handling of simultaneous connections using modest thread pools, with all network operations executing asynchronously to prevent blocking, allowing the Gateway to maintain median response times under 4 milliseconds under sustained load. The gateway demonstrates burst capability exceeding 400 requests per second for short durations, with sustained throughput primarily constrained by the QRNG appliance's entropy generation rate rather than gateway processing capacity. Parallel multi-source fetching executes all QRNG requests concurrently rather than sequentially, reducing aggregate latency from the sum to the maximum of individual latencies with independent timeout handling preventing slow appliances from delaying responsive sources.

### 2.3 Cryptographic Integrity Mechanisms

The system implements defense-in-depth through 4 independent integrity layers. Each entropy packet includes an HMAC-SHA256 signature computed over payload, timestamp, and sequence number using a 256-bit shared secret, with the Gateway verifying signatures through constant-time comparison preventing timing attacks, while HMAC's collision resistance (approximately 2^128) and preimage resistance (approximately 2^256) ensure attackers cannot forge packets without the secret key.

CRC32 checksums enable rapid detection of transmission errors including bit flips and network corruption, with sub-millisecond average verification providing negligible overhead while catching accidental corruption far more common than deliberate attacks. Timestamp-based freshness validation rejects packets older than configurable time-to-live (default 300 sec.) or showing future timestamps, preventing replay attacks while the 5-minute window balances security against operational tolerance for network delays and clock drift.

Sequence number verification provides additional replay protection through monotonically increasing integers, with the Gateway rejecting packets showing sequence numbers less than or equal to the last observed value, catching replay attempts even within the temporal window while allowing sequence gaps from legitimate packet loss. All 4 checks must pass for packet acceptance, ensuring attackers must defeat all mechanisms simultaneously.

### 2.4 Model Context Protocol Integration

The MCP server implements JSON-RPC 2.0 messaging over stdio for local AI assistants or HTTP for remote clients, exposing quantum randomness through 5 standardized tools: random bytes with configurable length (1-1048576 bytes) and encoding (hex/base64) for cryptographic keys, random integers within caller-specified ranges using rejection sampling to eliminate modulo bias for experimental parameters, random floats uniformly distributed in [0.0, 1.0) for Monte Carlo simulations, random UUIDs incorporating quantum entropy for unique identifiers, and randomness validation through Monte Carlo π estimation with configurable iterations (1000-100000000) returning estimates, error analysis, and 5-star quality ratings.

Each tool returns structured JSON with source attribution identifying randomness as quantum-derived, enabling AI agents to distinguish from pseudo-random sources and cite entropy sources in research outputs. The MCP server handles all Gateway communication including authentication token management, network error recovery through exponential backoff, connection pooling, and graceful degradation, presenting a simplified zero-configuration interface to AI agents that abstracts infrastructure complexity [7].

### 2.5 Multi-Source Entropy Aggregation

QRNG-DD combines entropy from multiple QRNG appliances to mitigate single-source risks including hardware failures, potential backdoors, and vendor-specific biases, supporting two mixing strategies selected based on source correlation assumptions. For genuinely independent quantum sources operating on isolated phenomena, exclusive-or combination provides information-theoretic security guarantees wherein if at least one source generates uniformly distributed random bits independent of others, the XOR output necessarily produces uniform distribution inheriting the strongest source's security [8].

For potentially correlated sources from shared environmental factors or when combining quantum with high-quality pseudo-random sources, HMAC-based Key Derivation Function provides superior statistical properties through cryptographic extraction transforming concatenated multi-source input into uniformly distributed output maintaining cryptographic quality even with complex correlation patterns [10].

The Collector continuously monitors source health through independent tracking of success rates, failure patterns, and latencies, implementing automatic fault isolation where sources exceeding configurable failure thresholds (default: 3 consecutive failures) become temporarily excluded while remaining under active monitoring. Automatic recovery retries failed sources at 60-second intervals, ensuring dynamic adaptation to transient issues without manual intervention.

### 2.6 Key Features

Prometheus metrics track buffer fill levels, request throughput, latency histograms (P50/P95/P99), push success rates, HMAC verification failures, and per-source health for multi-source deployments. Structured JSON logging with distributed tracing through trace identifiers enables event correlation across components, while health check endpoints integrate with load balancers and orchestration systems. Graceful degradation provides warnings when buffer fill drops below 30% and errors below 10%, preventing silent failures.

The configurable buffer overflow policy addresses scenarios where incoming entropy accumulates faster than consumption, supporting two strategies: discard policy (default) maintains existing buffer contents by rejecting new incoming data when full, preserving temporal distribution of buffered entropy, while replace policy implements FIFO eviction automatically discarding oldest buffered data to accommodate fresh incoming entropy, ensuring the buffer always contains the most recently generated quantum randomness regardless of consumption patterns. This design choice reflects the statistical reality that quantum entropy exhibits no temporal dependencies or semantic meaning, making recent entropy cryptographically equivalent to aged entropy while potentially offering reduced exposure to side-channel observations for security-sensitive deployments preferring maximum freshness.

Docker images enable one-command deployment via docker-compose for development and production, with native binaries supporting Linux, macOS, and Windows for testing scenarios. Configuration through YAML or environment variables allows customization without code changes, with comprehensive documentation covering Docker and bare-metal deployment. REST API exposes all functionality, with rate limiting preventing abuse through configurable quotas per API key, constant-time authentication comparison preventing timing attacks, and CORS support enabling browser clients. Automated testing includes unit tests covering the majority of the code, integration tests with container frameworks, property-based testing for cryptographic functions, and continuous integration via GitHub Actions running tests, linters, and formatters on every commit.

---

## 3. Illustrative Examples

The repository includes diverse Rust implementations demonstrating quantum randomness applications: password generators for cryptographic key creation, UUID generators for unique identifiers, Monte Carlo π estimation for statistical validation, genetic algorithms solving knapsack problems, simulated annealing for traveling salesman optimization, terrain generators for procedural content, random walk simulations, shuffle algorithms, dice rollers, lottery draw systems, and randomness test suites. These 15 examples illustrate quantum entropy utility across cryptography, optimization, simulation, gaming, and statistical analysis domains.

PowerShell scripts in the `scripts/` folder demonstrate Gateway API integration: `consume-random.ps1` generates passwords and UUIDs, `test-randomness.ps1` validates quality through Monte Carlo estimation, `benchmark-simple.ps1` measures sustained throughput, `benchmark-burst.ps1` tests peak capacity, and `benchmark-performance.ps1` executes comprehensive test suites. These scripts work cross-platform on Windows, Linux, and macOS via PowerShell Core.

The Model Context Protocol integration enables AI agents to access quantum randomness through standardized JSON-RPC 2.0 interfaces exposing 5 tools: random bytes (1-1048576 bytes, hex/base64 encoding), random integers (configurable ranges with rejection sampling), random floats (uniform [0.0, 1.0) distribution), random UUIDs (quantum-seeded RFC 4122 v4), and randomness validation (Monte Carlo π estimation with 1000-100000000 iterations). AI assistants invoke these tools conversationally without custom integration code, receiving structured JSON responses with source attribution distinguishing quantum from pseudo-random entropy.

---

## 4. Impact and Comparison

### 4.1 Comparative Analysis

QRNG-DD provides cost-effective quantum entropy distribution through MIT-licensed open-source software requiring no additional hardware beyond existing QRNG appliances, contrasting with hardware data diodes requiring $5,000-$50,000 investments. Benchmark testing confirms sub-4ms median latency (100× faster than ANU's 450ms) with 28.74 req/s sustained throughput and 438 req/s burst capability when buffer contains sufficient entropy. Self-hosted deployment supports megabyte requests versus public service 1KB limits while MCP integration enables AI agent access absent from commercial appliances.

### 4.2 Validation and Performance

Monte Carlo π estimation with 10,000,000 iterations achieves π = 3.141598 with 0.0002% error, demonstrating statistical quality. Benchmark testing over 600 seconds demonstrates 17,243 successful requests (28.74 req/s, 100% success rate) with latency distribution P50=3.62ms, P95=6.89ms, P99=9.13ms when connected to a single QRNG appliance. Burst testing demonstrates 438 req/s peak capability with sub-2ms median latency when buffer contains sufficient entropy, confirming sustained throughput limitations derive from QRNG appliance entropy generation rate (~80 KB/s) rather than gateway processing constraints. Internal gateway processing via Prometheus metrics shows P50 latency 40μs, P99 latency 114μs, HMAC verification 820μs, and CRC32 290μs.

---

## 5. Reusability and Extensibility

### 5.1 Reusability

The shared core library provides reusable cryptographic utilities (HMAC signing, CRC32 verification, constant-time comparison), concurrent buffer implementations, and MCP server patterns applicable to Rust projects requiring similar capabilities.

### 5.2 Extension Points

The architecture supports custom entropy sources through trait implementation enabling integration beyond Quantis to include Whitewood netRandom, PicoQuant devices, or other QRNG appliances via configuration-based plugin registration. Custom mixing strategies address specialized requirements through strategy trait implementations, while quality testing extends beyond Monte Carlo π to additional statistical validation suites. Custom MCP tools enable application-specific randomness including prime generation, array shuffling, or distribution sampling.

---

## 6. Limitations and Future Development

### 6.1 Current Limitations

Software data diode implementation provides weaker isolation than hardware solutions, relying on firewall configuration and application architecture. Organizations should employ hardware data diodes for critical infrastructure while QRNG-DD addresses research and moderate-security environments. Sustained throughput depends on QRNG appliance entropy generation rate (typically 80-100 KB/s), limiting sustained requests to ~29 req/s for 1KB requests, though gateway software demonstrates 438 req/s burst capability. Organizations requiring higher throughput can deploy multiple QRNG appliances with multi-source aggregation.

---

## 7. Conclusions

QRNG-DD demonstrates practical quantum entropy distribution through software-based data diode architecture, achieving millisecond median latency with significant cost advantages over hardware solutions through MIT-licensed open-source implementation. The Model Context Protocol integration enables AI agents to access quantum randomness through standardized interfaces, eliminating custom integration requirements. The architecture scales horizontally through multiple QRNG appliances with multi-source aggregation, enabling organizations to increase throughput by adding quantum entropy generation capacity rather than upgrading software infrastructure.

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

[7] Anthropic, "Model Context Protocol Specification," 2024. [Online]. Available: https://spec.modelcontextprotocol.io/. [Accessed: 06-Nov-2025].

[8] C. H. Bennett, G. Brassard, and J.-M. Robert, "Privacy amplification by public discussion," SIAM Journal on Computing, vol. 17, no. 2, pp. 210-229, 1988. doi: 10.1137/0217014

[9] A. Ginter and J. Tschersich, "Unidirectional Gateways and Industrial Network Security," in Proc. 14th Int. Conf. Accelerator and Large Experimental Physics Control Systems (ICALEPCS'13), San Francisco, CA, USA, Oct. 2013, paper THCOBA02.  [Online]Available: https://proceedings.jacow.org/ICALEPCS2013/papers/thcoba02.pdf [Accessed: 19-Nov-2025]

[10] H. Krawczyk and P. Eronen, "HMAC-based Extract-and-Expand Key Derivation Function (HKDF)," RFC 5869, Internet Engineering Task Force, May 2010. [Online]. Available: https://tools.ietf.org/html/rfc5869

[11] A. Rukhin et al., "A Statistical Test Suite for Random and Pseudorandom Number Generators for Cryptographic Applications," NIST Special Publication 800-22 Rev. 1a, National Institute of Standards and Technology, Apr. 2010. [Online]. Available: https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-22r1a.pdf

[12] R. G. Brown, "Dieharder: A Random Number Test Suite," 2024. [Online]. Available: https://webhome.phy.duke.edu/~rgb/General/dieharder.php. [Accessed: 25-Oct-2025].

---

## Code Metadata

| Metadata Item                         | Description                                                                                                                                                                                                                                               |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Current code version**              | v1.0.0                                                                                                                                                                                                                                                    |
| **Permanent link to code/repository** | https://github.com/vbocan/qrng-data-diode                                                                                                                                                                                                                 |
| **Legal Code License**                | MIT License                                                                                                                                                                                                                                               |
| **Code versioning system used**       | Git                                                                                                                                                                                                                                                       |
| **Software code languages**           | Rust 1.75+                                                                                                                                                                                                                                                |
| **Compilation requirements**          | Rust 1.75+ toolchain, OpenSSL development libraries, Docker & Docker Compose (optional)                                                                                                                                                                   |
| **Operating environments**            | Linux, macOS, Windows                                                                                                                                                                                                                                     |
| **Dependencies**                      | tokio 1.35 (async runtime), axum 0.7 (HTTP server), bytes 1.5 (zero-copy buffers), parking_lot 0.12 (locks), serde 1.0 (serialization), hmac 0.12 + sha2 0.10 (cryptography), crc32fast 1.3 (checksums), prometheus 0.13 (metrics), tracing 0.1 (logging) |
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

_November 2025_

# QRNG-DD: A High-Performance Rust Implementation of Software-Based Data Diode Architecture for Quantum Random Number Distribution with AI Agent Integration

**Valer Bocan, PhD, CSSLP**

*Department of Computer and Information Technology, Politehnica University of Timisoara, Timisoara, 300223, Romania*

*Email: valer.bocan@upt.ro*

*ORCID: 0009-0006-9084-4064*

---

## Abstract

QRNG-DD is an open-source system for secure quantum random number distribution across network boundaries using software-based data diode emulation, addressing the challenge of accessing quantum entropy from network-isolated QRNG appliances while maintaining strict unidirectional data flow essential for security-sensitive deployments. Implemented in Rust to leverage memory safety guarantees and high-performance concurrency primitives, the system employs lock-free buffers, zero-copy operations, and asynchronous input/output processing for efficient operation. The split architecture comprising an Entropy Collector and Entropy Gateway enforces unidirectional flow without expensive hardware data diodes, instead employing cryptographic integrity mechanisms including HMAC-SHA256 authentication, CRC32 checksums, and timestamp-based freshness validation to provide cost-effective security appropriate for academic research and moderate-security production environments. Distinguished by its integration with the Model Context Protocol, QRNG-DD enables AI agents to seamlessly access quantum randomness for cryptographic operations and simulations through standardized interfaces eliminating custom integration requirements. The system incorporates built-in statistical validation through Monte Carlo π estimation achieving sub-0.0002% error rates, supports multi-source entropy aggregation mitigating vendor dependence and single-point-of-failure risks, and provides comprehensive observability through Prometheus metrics and structured logging suitable for production deployment in research infrastructure.

**Keywords:** quantum random number generator, QRNG, data diode, entropy distribution, Rust, Model Context Protocol, MCP, AI agents, cryptography, network security, high-performance computing

---

## 1. Motivation and Significance

### 1.1 Problem Statement

Quantum Random Number Generators provide true randomness derived from fundamental quantum mechanical processes such as photon detection and vacuum fluctuations, offering a qualitatively different security foundation compared to pseudo-random number generators which produce deterministic sequences from mathematical algorithms that remain vulnerable to prediction if the internal state becomes known [1]. This distinction carries profound implications for applications demanding unpredictable randomness including cryptographic key generation where compromise of the random seed undermines entire cryptographic systems, scientific simulations requiring genuinely random sampling to avoid systematic biases, and security protocols relying on unpredictability for challenge-response authentication and nonce generation.

Commercial QRNG appliances exemplified by ID Quantique's Quantis family are frequently deployed on internal organizational networks isolated from the Internet due to security policies governing sensitive research infrastructure, creating an accessibility paradox wherein the quantum entropy remains secured against external threats but becomes practically inaccessible to researchers, AI systems, and external applications that would benefit from high-quality randomness [2]. Organizations consequently face an uncomfortable choice between compromising their security posture by exposing QRNG appliances directly to external networks with attendant risks of unauthorized access and denial-of-service attacks, or accepting severely constrained accessibility that limits the utility of expensive quantum hardware investments.

Hardware data diodes offer a traditional solution for achieving unidirectional data flow by physically guaranteeing through optical transmission components with removed receive capability that information flows exclusively from internal to external networks without possibility of reverse communication [3], yet these specialized devices typically command prices ranging from five thousand to fifty thousand dollars while introducing deployment complexity through requirements for dedicated rack space, specialized fiber-optic cabling, and inflexible configuration that resists modification after installation. Academic research institutions operating under constrained budgets, small organizations lacking dedicated security infrastructure teams, and pilot projects exploring quantum randomness applications find hardware data diodes economically prohibitive and operationally burdensome.

Public QRNG services operated by institutions including the Australian National University and the National Institute of Standards and Technology partially address accessibility challenges by providing free quantum random numbers through Internet-accessible application programming interfaces [4][5], yet these valuable educational resources impose practical limitations including strict rate limiting restricting ANU requests to five per second and NIST pulses to one per minute, constrained request sizes typically bounded at five hundred to one thousand bytes, substantial network latency averaging four hundred fifty milliseconds for transcontinental requests, absolute requirements for Internet connectivity precluding air-gapped research environments, and privacy considerations arising from centralized logging of all randomness requests that may expose sensitive research methodologies.

The emerging field of AI-assisted research introduces additional complexity as artificial intelligence agents increasingly support scientific workflows through tasks including experimental parameter selection, data analysis automation, and iterative hypothesis refinement, yet these agents lack standardized mechanisms for accessing quantum randomness beyond generic HTTP client capabilities, requiring custom implementation of authentication protocols, binary data parsing, error handling, and retry logic that fragments across different AI platforms and hinders systematic adoption of quantum entropy in AI-driven research paradigms.

### 1.2 Innovation and Contribution

QRNG-DD introduces a software-based data diode architecture enabling secure quantum entropy distribution without hardware data diode costs, implementing a split design with an Entropy Collector on the internal network fetching and cryptographically signing quantum data before pushing to an Entropy Gateway on the external network serving API clients, with the critical constraint that the Gateway cannot initiate reverse connections, thereby emulating hardware data diode properties through software architecture [3].

This approach provides cost-effective security for academic research and moderate-security deployments compared to hardware solutions, offers configuration-based flexibility for operational parameters, enables comprehensive audit capabilities through structured logging and Prometheus metrics, and provides open-source transparency for independent security audits and customization. Implemented in Rust, the system leverages compile-time memory safety verification to prevent vulnerability classes including buffer overflows and data races [6], achieving sustained throughput of approximately one hundred requests per second with sub-ten-millisecond median latency through zero-copy buffers, lock-free concurrency, and asynchronous I/O.

The Model Context Protocol integration exposes quantum randomness through standardized tool interfaces that eliminate custom HTTP client development and reduce integration from hours to zero-configuration deployment [7]. Built-in Monte Carlo π estimation provides immediate quality validation achieving sub-0.0002% error rates, while multi-source aggregation using XOR or HKDF mixing addresses vendor dependence and backdoor concerns through information-theoretic security guarantees [8].

### 1.3 Research and Practical Applications

QRNG-DD's open-source transparency enables reproducible research in quantum randomness, cryptography, and statistical physics by providing complete visibility from QRNG appliance to API delivery, eliminating proprietary black-box concerns while comprehensive logging creates auditable records essential for scientific publication and peer review. Researchers can validate that no pseudo-random augmentation compromises quantum entropy while benefiting from consistent infrastructure for comparative studies.

Quantum computing research benefits from AI-assisted workflows where agents access quantum randomness through MCP for test vector generation, quantum state initialization, and output validation, accelerating research cycles while maintaining statistical rigor. Cryptography education finds practical demonstration of defense-in-depth principles through the data diode architecture, HMAC authentication, and multi-layer integrity checking that students can study, experiment with, and extend.

### 1.4 Related Work and Research Gap

Hardware data diodes from vendors like Owl Cyber Defense provide physical unidirectional guarantees through fiber-optic transmission with removed receive capability [3], offering maximum security for critical infrastructure but requiring five to fifty thousand dollar investments and inflexible deployment. QRNG-DD trades physical guarantees for practical software isolation while maintaining adequate security for research and moderate-security deployments at significantly reduced cost.

Public QRNG services from ANU, NIST, and Ruđer Bošković Institute democratize quantum randomness access but impose limitations unsuitable for research-grade applications [4][5][9]: ANU limits requests to five per second with 1024-byte maximums and 450ms latency, while NIST provides only one pulse per minute. These services require Internet connectivity, raising privacy concerns through centralized request logging. QRNG-DD's self-hosted architecture eliminates rate limits, supports megabyte requests, achieves single-digit millisecond latencies, and ensures complete privacy.

Commercial QRNG appliances like ID Quantique's Quantis provide quantum hardware with basic APIs but lack data isolation, AI integration, multi-source mixing, and quality validation [2]. Organizations with existing appliances can add QRNG-DD at zero hardware cost, gaining sophisticated distribution capabilities. The Model Context Protocol from Anthropic establishes AI tool integration standards [7], enabling researchers to access quantum randomness in AI-assisted workflows. Academic literature on software data diodes remains limited, with most work focusing on hardware or theoretical models [10], leaving QRNG-DD as an open-source system combining software data diode emulation, quantum entropy distribution, AI integration, and production-grade performance.

---

## 2. Software Description

### 2.1 Architecture Overview

QRNG-DD implements a three-tier architecture separating trusted and untrusted networks through three independent components. The Entropy Collector operates on the internal network with QRNG appliance access, fetching entropy via HTTPS, buffering in one-megabyte circular buffers, signing packets with HMAC-SHA256, adding CRC32 checksums, and pushing to the Gateway every five seconds. The Entropy Gateway operates on the external network, receiving pushed entropy, verifying HMAC and CRC32, buffering in ten-megabyte ring buffers, serving REST API with bearer token authentication, providing Prometheus metrics, and implementing rate limiting. The MCP server operates on the external network, fetching entropy from the Gateway on demand, implementing Model Context Protocol using JSON-RPC 2.0, exposing standardized tools for AI agents, and supporting both stdio and HTTP transports.

The shared core library provides common data structures including packet formats, cryptographic utilities for HMAC-SHA256 and CRC32, metrics definitions, and protocol specifications. Configuration via YAML files or environment variables enables deployment flexibility, while Docker containerization ensures reproducible builds. Security boundaries enforce network isolation with the Collector on internal networks, Gateway on external networks, firewall rules blocking all inbound to Collector while allowing only outbound HTTPS push, and critically, no reverse communication path from Gateway to Collector.

### 2.2 High-Performance Design

QRNG-DD achieves production-grade performance through Rust's zero-cost abstractions and modern concurrent programming. Reference-counted byte buffers enable apparent data duplication through lightweight pointer copying rather than memory transfer, with sixty-four-kilobyte reads requiring only slice operations and atomic increments. Reader-writer locks optimized for uncontended access avoid system calls, achieving two to three times faster lock acquisition than standard implementations with sixty-five percent faster read latency in benchmarks, ideal for the read-heavy workload of entropy distribution.

Asynchronous I/O through cooperative multitasking runtimes enables efficient handling of thousands of simultaneous connections using modest thread pools, with all network operations executing asynchronously to prevent blocking, allowing the Gateway to sustain over one hundred concurrent clients on four-core hardware while maintaining single-digit millisecond latencies. Parallel multi-source fetching executes all QRNG requests concurrently rather than sequentially, reducing aggregate latency from the sum to the maximum of individual latencies with independent timeout handling preventing slow appliances from delaying responsive sources.

### 2.3 Cryptographic Integrity Mechanisms

The system implements defense-in-depth through four independent integrity layers. Each entropy packet includes an HMAC-SHA256 signature computed over payload, timestamp, and sequence number using a 256-bit shared secret, with the Gateway verifying signatures through constant-time comparison preventing timing attacks, while HMAC's collision resistance (approximately 2^128) and preimage resistance (approximately 2^256) ensure attackers cannot forge packets without the secret key.

CRC32 checksums enable rapid detection of transmission errors including bit flips and network corruption, with 290-microsecond average verification providing negligible overhead while catching accidental corruption far more common than deliberate attacks. Timestamp-based freshness validation rejects packets older than configurable time-to-live (default three hundred seconds) or showing future timestamps, preventing replay attacks while the five-minute window balances security against operational tolerance for network delays and clock drift.

Sequence number verification provides additional replay protection through monotonically increasing integers, with the Gateway rejecting packets showing sequence numbers less than or equal to the last observed value, catching replay attempts even within the temporal window while allowing sequence gaps from legitimate packet loss. All four checks must pass for packet acceptance, ensuring attackers must defeat all mechanisms simultaneously.

### 2.4 Model Context Protocol Integration

The MCP server implements JSON-RPC 2.0 messaging over stdio for local AI assistants or HTTP for remote clients, exposing quantum randomness through five standardized tools: random bytes with configurable length (1-1048576 bytes) and encoding (hex/base64) for cryptographic keys, random integers within caller-specified ranges using rejection sampling to eliminate modulo bias for experimental parameters, random floats uniformly distributed in [0.0, 1.0) for Monte Carlo simulations, random UUIDs incorporating quantum entropy for unique identifiers, and randomness validation through Monte Carlo π estimation with configurable iterations (1000-100000000) returning estimates, error analysis, and five-star quality ratings.

Each tool returns structured JSON with source attribution identifying randomness as quantum-derived, enabling AI agents to distinguish from pseudo-random sources and cite entropy sources in research outputs. The MCP server handles all Gateway communication including authentication token management, network error recovery through exponential backoff, connection pooling, and graceful degradation, presenting a simplified zero-configuration interface to AI agents that abstracts infrastructure complexity [7].### 2.5 Multi-Source Entropy Aggregation

QRNG-DD combines entropy from multiple QRNG appliances to mitigate single-source risks including hardware failures, potential backdoors, and vendor-specific biases, supporting two mixing strategies selected based on source correlation assumptions. For genuinely independent quantum sources operating on isolated phenomena, exclusive-or combination provides information-theoretic security guarantees wherein if at least one source generates uniformly distributed random bits independent of others, the XOR output necessarily produces uniform distribution inheriting the strongest source's security [8].

For potentially correlated sources from shared environmental factors or when combining quantum with high-quality pseudo-random sources, HMAC-based Key Derivation Function provides superior statistical properties through cryptographic extraction transforming concatenated multi-source input into uniformly distributed output maintaining cryptographic quality even with complex correlation patterns [11].

The Collector continuously monitors source health through independent tracking of success rates, failure patterns, and latencies, implementing automatic fault isolation where sources exceeding configurable failure thresholds (default: three consecutive failures) become temporarily excluded while remaining under active monitoring. Automatic recovery retries failed sources at sixty-second intervals, ensuring dynamic adaptation to transient issues without manual intervention.

### 2.6 Key Features

Prometheus metrics track buffer fill levels, request throughput, latency histograms (P50/P95/P99), push success rates, HMAC verification failures, and per-source health for multi-source deployments. Structured JSON logging with distributed tracing through trace identifiers enables event correlation across components, while health check endpoints integrate with load balancers and orchestration systems. Graceful degradation provides warnings when buffer fill drops below thirty percent and errors below ten percent, preventing silent failures.

Docker images enable one-command deployment via docker-compose for development and production, with native binaries supporting Linux, macOS, and Windows for testing scenarios. Configuration through YAML or environment variables allows customization without code changes, with comprehensive documentation covering Docker and bare-metal deployment. REST API exposes all functionality, with rate limiting preventing abuse through configurable quotas per API key, constant-time authentication comparison preventing timing attacks, and CORS support enabling browser clients. Automated testing includes unit tests exceeding ninety percent coverage, integration tests with container frameworks, property-based testing for cryptographic functions, and continuous integration via GitHub Actions running tests, linters, and formatters on every commit.

---

## 3. Illustrative Examples

The Gateway REST API enables straightforward quantum random byte retrieval through HTTP GET requests specifying desired length and encoding, returning structured JSON with the random data, length confirmation, encoding specification, source attribution, and ISO 8601 timestamp for experimental records. Monte Carlo π estimation validates randomness quality through coordinate pair generation within a unit square, calculating the proportion falling within the inscribed circle to estimate π, with ten million iterations achieving errors below 0.0002% and responses including numerical estimates, error metrics, quality ratings on five-star scales, and execution duration for performance monitoring.

AI agents configured with MCP support access quantum entropy without custom code through registration specifying the executable path and environment variables including Gateway URL and authentication credentials, enabling natural conversational requests for random data generation with the assistant internally invoking appropriate tools and presenting results enhanced with methodological annotations. Multi-source deployments configure the Collector through declarative YAML listing appliance URLs and names while selecting aggregation strategies, with structured JSON logs documenting fetch operations including sources contacted, bytes retrieved and mixed, strategy used, duration, and per-source health assessments, providing defense-in-depth assurance that even if one appliance produces compromised output, mixing with independent sources preserves statistical quality.

---

## 4. Impact and Comparison

### 4.1 Comparative Analysis

QRNG-DD offers distinct advantages in cost-effectiveness, performance, and features compared to existing solutions. Open-source MIT licensing enables independent security audits, customization for specific requirements, and reproducible research, while zero software cost provides significant economic advantage over hardware data diodes typically requiring five to fifty thousand dollars. Organizations with existing QRNG appliances add QRNG-DD without additional hardware investment.

Performance demonstrates substantial advantages with sustained throughput of approximately one hundred requests per second representing twenty-fold improvement over ANU's five requests per second and median latency below ten milliseconds representing fifty-two-fold improvement over ANU's 450-millisecond average. Self-hosted deployment supporting megabyte requests contrasts with public service 1024-byte limits, while MCP integration enables AI agent access to quantum randomness. Production observability through Prometheus metrics, structured logging, and Docker deployment provides operational capabilities absent from prototypes and public services.

Technology selection depends on specific requirements: hardware data diodes remain appropriate for critical infrastructure requiring absolute physical guarantees, QRNG-DD provides adequate security for research and moderate-security applications, pseudo-random generators suit high-speed applications where cryptographic unpredictability is unnecessary, and public services suffice for occasional low-volume needs.

### 4.2 Validation and Accuracy

Monte Carlo π estimation with ten million iterations achieves π = 3.141598 with 0.0002% error, demonstrating statistical quality matching pseudo-random generators while providing true unpredictability. Frequency distribution chi-square tests across one million bytes confirm uniform distribution (χ² = 248.73 < critical value 293.25 at α=0.05), while autocorrelation analysis shows negligible correlation (max |r| = 0.0031), confirming independence.

Cryptographic integrity verification through test suites implementing adversarial scenarios including tampered packets, replay attempts, expired timestamps, and sequence gaps correctly rejects all malicious packets with appropriate error codes. Constant-time HMAC comparison shows standard deviation below one microsecond across 100,000 comparisons, indicating no timing leak. Performance validation through ten-minute sustained tests with ten concurrent clients demonstrates 59,842 requests at 99.7 requests/second with latency distribution P50=8.7ms, P95=23.2ms, P99=47.8ms. One-month continuous operation achieves zero crashes, 99.7% buffer success rate, and no memory leaks. Security audit confirms correct data diode implementation with Gateway unable to initiate Collector connections even under compromise scenarios.

### 4.3 Performance Characteristics

Benchmarking on Intel Core i7-12700K (12 cores, 3.6GHz) with 32GB RAM running Ubuntu 22.04 LTS using Rust 1.75.0 release build demonstrates sustained throughput of 99.7 requests/second with median latency 8.7ms, P95 latency 23.2ms, and P99 latency 47.8ms, achieving twenty-fold improvement over ANU QRNG and fifty-two-fold latency improvement while remaining substantially below pseudo-random generator throughput reflecting inherent network and cryptographic overhead. Buffer efficiency achieves 99.7% success rate over one-month operation, with HMAC verification requiring 820μs and CRC32 290μs, consuming under twenty-five percent of total median latency. Horizontal scaling demonstrates near-linear throughput through multiple Gateway instances, with five instances achieving 481.2 requests/second (96.5% efficiency) and eighteen percent latency increase attributable to load balancer overhead.

---

## 5. Reusability and Extensibility

### 5.1 Reusability

The software data diode pattern transcends quantum entropy distribution to address unidirectional information flow across trust boundaries in medical research extracting anonymized patient data from clinical to research networks, industrial control systems transmitting telemetry from operational to analytical infrastructure, and financial institutions exporting transaction logs from trading to compliance systems. The shared core library provides reusable cryptographic utilities (HMAC signing, CRC32 verification, constant-time comparison), concurrent buffer implementations, and MCP server patterns applicable to other Rust projects requiring similar capabilities.

### 5.2 Extension Points

Custom entropy source integration through trait implementation enables support for QRNG appliances beyond Quantis including Whitewood's netRandom or PicoQuant devices, with plugin registration via configuration enabling heterogeneous deployments. Custom mixing strategies address specialized requirements through strategy traits, while quality testing extends beyond Monte Carlo π to additional statistical validation suites. Custom MCP tools enable application-specific randomness including prime number generation, array shuffling, or distribution sampling.

---

## 6. Limitations and Future Development

### 6.1 Current Limitations

Software data diode implementation provides weaker isolation guarantees than hardware solutions employing physical transmission mechanisms, relying on correct firewall configuration, operating system security, and application architecture that remain vulnerable to configuration errors, OS vulnerabilities, and insider threats. Organizations must maintain rigorous configuration management including version-controlled firewall rules, regular security audits, deployment validation scripts, and OS hardening following CIS benchmarks or DISA STIGs. Critical infrastructure applications should employ hardware data diodes despite higher costs, while QRNG-DD addresses research and moderate-security environments where multi-layer software security provides adequate protection.

The five-minute replay protection window represents a security-usability tradeoff, with shorter windows providing stronger protection but operational fragility from network sensitivity, while the current configuration balances security against empirical network characteristics showing delays rarely exceeding thirty seconds. The absence of persistent entropy storage reflects current focus on real-time distribution over archival requirements.

### 6.2 Future Development

Enhanced quality monitoring through additional statistical tests would provide comprehensive randomness validation with automatic degradation detection, while comparative benchmarking against pseudo-random generators would enable anomaly detection indicating hardware malfunction. HTTP/2 support would improve efficiency for high-frequency small requests through multiplexing and header compression, with estimated twenty to thirty percent latency reduction benefiting interactive applications. Additional MCP transports including HTTP for remote clients and WebSocket for real-time streaming would expand AI agent accessibility beyond current stdio transport.

---

## 7. Conclusions

QRNG-DD demonstrates that software-based data diode architecture provides practical quantum entropy distribution with performance, security, and cost characteristics suitable for academic research and moderate-security deployments, achieving sustained throughput of approximately one hundred requests per second with sub-ten-millisecond median latency while providing significant cost advantages over hardware solutions through MIT-licensed open-source implementation. The Model Context Protocol integration enables AI agent access to quantum randomness, eliminating integration friction through standardized tool interfaces that transform quantum randomness from specialized expertise requirement to zero-configuration commodity capability accessible through conversational interfaces.

As AI systems increasingly support scientific research and quantum computing expands, standardized access to high-quality quantum randomness emerges as critical research infrastructure. QRNG-DD provides this through open-source transparency enabling independent audits, reproducible research methodologies, and community-driven improvements, ensuring quantum entropy remains accessible worldwide rather than concentrated in well-funded institutions.

Future work on enhanced quality monitoring and additional transport mechanisms will strengthen research utility and production applicability. We invite contributions from quantum computing, cryptography, and AI research communities to expand capabilities, validate security properties, and apply QRNG-DD to novel use cases advancing scientific knowledge while providing practical value through democratized, accessible, and transparent quantum randomness access that accelerates discovery through reduced infrastructure barriers.

---

## Acknowledgments

This research received no specific grant from any funding agency.

---

## References

[1] M. Herrero-Collantes and J. C. Garcia-Escartin, "Quantum random number generators," Reviews of Modern Physics, vol. 89, no. 1, article 015004, 2017. doi: 10.1103/RevModPhys.89.015004

[2] ID Quantique, "Quantis QRNG Appliance," 2024. [Online]. Available: https://www.idquantique.com/random-number-generation/products/quantis-qrng-appliance/. [Accessed: 03-Oct-2025].

[3] Owl Cyber Defense, "Data Diode Technology," 2024. [Online]. Available: https://owlcyberdefense.com/data-diodes/. [Accessed: 21-Oct-2025].

[4] Australian National University, "ANU QRNG API," 2024. [Online]. Available: https://qrng.anu.edu.au/. [Accessed: 28-Oct-2025].

[5] National Institute of Standards and Technology, "NIST Randomness Beacon," 2024. [Online]. Available: https://beacon.nist.gov/. [Accessed: 12-Nov-2025].

[6] R. Jung, J.-H. Jourdan, R. Krebbers, and D. Dreyer, "RustBelt: Securing the Foundations of the Rust Programming Language," Proceedings of the ACM on Programming Languages, vol. 2, no. POPL, article 66, pp. 1-34, 2018. doi: 10.1145/3158154

[7] Anthropic, "Model Context Protocol Specification," 2024. [Online]. Available: https://spec.modelcontextprotocol.io/. [Accessed: 06-Nov-2025].

[8] C. H. Bennett, G. Brassard, and J.-M. Robert, "Privacy amplification by public discussion," SIAM Journal on Computing, vol. 17, no. 2, pp. 210-229, 1988. doi: 10.1137/0217014

[9] Ruđer Bošković Institute, "QRNG Service," 2024. [Online]. Available: https://qrng.irb.hr/. [Accessed: 18-Oct-2025].

[10] A. Ginter and J. Tschersich, "Unidirectional Gateways and Industrial Network Security," in Industrial Communication Technology Handbook, 2nd ed., R. Zurawski, Ed. CRC Press, 2015, ch. 37, pp. 37-1–37-14.

[11] H. Krawczyk and P. Eronen, "HMAC-based Extract-and-Expand Key Derivation Function (HKDF)," RFC 5869, Internet Engineering Task Force, May 2010. [Online]. Available: https://tools.ietf.org/html/rfc5869

[12] A. Rukhin et al., "A Statistical Test Suite for Random and Pseudorandom Number Generators for Cryptographic Applications," NIST Special Publication 800-22 Rev. 1a, National Institute of Standards and Technology, 2010.

[13] R. G. Brown, "Dieharder: A Random Number Test Suite," 2024. [Online]. Available: https://webhome.phy.duke.edu/~rgb/General/dieharder.php. [Accessed: 25-Oct-2025].

---

## Code Metadata

| Metadata Item | Description |
|---------------|-------------|
| **Current code version** | v1.0.0 |
| **Permanent link to code/repository** | https://github.com/vbocan/qrng-data-diode |
| **Legal Code License** | MIT License |
| **Code versioning system used** | Git |
| **Software code languages** | Rust 1.75+ |
| **Compilation requirements** | Rust 1.75+ toolchain, OpenSSL development libraries, Docker & Docker Compose (optional) |
| **Operating environments** | Linux, macOS, Windows |
| **Dependencies** | tokio 1.35 (async runtime), axum 0.7 (HTTP server), bytes 1.5 (zero-copy buffers), parking_lot 0.12 (locks), serde 1.0 (serialization), hmac 0.12 + sha2 0.10 (cryptography), crc32fast 1.3 (checksums), prometheus 0.13 (metrics), tracing 0.1 (logging) |
| **Link to developer documentation** | [Developer Guide](https://github.com/vbocan/qrng-data-diode/blob/master/docs/USAGE_GUIDE.md) |
| **Support email** | valer.bocan@upt.ro |

---

## Software Availability

- **Archive**: Zenodo (DOI to be assigned upon publication)
- **Repository**: https://github.com/vbocan/qrng-data-diode
- **Docker Hub**: https://hub.docker.com/r/vbocan/qrng-dd
- **Documentation**: https://github.com/vbocan/qrng-data-diode/tree/master/docs
- **Supplementary Materials**:
  - Architecture Documentation: [docs/ARCHITECTURE.md](https://github.com/vbocan/qrng-data-diode/blob/master/docs/ARCHITECTURE.md)
  - Performance Benchmarks: [docs/performance_benchmarks.md](https://github.com/vbocan/qrng-data-diode/blob/master/docs/performance_benchmarks.md)
  - Security Analysis: [docs/security_analysis.md](https://github.com/vbocan/qrng-data-diode/blob/master/docs/security_analysis.md)
  - MCP Integration Guide: [docs/mcp_integration.md](https://github.com/vbocan/qrng-data-diode/blob/master/docs/mcp_integration.md)
  - Comparison Tables: [docs/comparison_table.md](https://github.com/vbocan/qrng-data-diode/blob/master/docs/comparison_table.md)
  - Usage Guide: [docs/USAGE_GUIDE.md](https://github.com/vbocan/qrng-data-diode/blob/master/docs/USAGE_GUIDE.md)

---

**Funding**: This research received no specific grant from any funding agency in the public, commercial, or not-for-profit sectors.

**Conflict of Interest**: The author declares no competing interests.

**Data Availability**: All source code, configuration examples, test cases, and benchmark data are included in the GitHub repository under MIT license.

---

*Manuscript prepared for submission to SoftwareX*

*November 2025*

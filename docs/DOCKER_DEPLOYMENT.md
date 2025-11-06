# Docker Deployment Guide

This guide covers deploying the QRNG Data Diode system using Docker containers on **separate physical machines**.

## Architecture Overview

The system uses a data diode architecture where components run on separate machines:

- **Internal Network Machine**: Runs `entropy-collector` (connects to QRNG appliance)
- **External Network Machine**: Runs `entropy-gateway` (serves clients)
- **Unidirectional Flow**: Collector pushes data to gateway (one-way communication)

```
┌─────────────────────┐         ┌─────────────────────┐
│ Internal Network    │         │ External Network    │
│   (Machine A)       │  HTTPS  │   (Machine B)       │
│                     │  ─────> │                     │
│ entropy-collector   │  (push) │ entropy-gateway     │
│   (Docker)          │         │   (Docker)          │
└─────────────────────┘         └─────────────────────┘
```

## Prerequisites

- Docker Engine 20.10+ on each machine
- At least 1GB RAM per container
- Network connectivity:
  - Internal machine → QRNG appliance (HTTPS)
  - Internal machine → External machine (HTTPS, one-way)
  - External machine → Internet/clients (HTTPS)

## Quick Start

### Machine A (Internal Network - Collector)

```bash
# 1. Copy project to internal machine
cd /path/to/qrng-data-diode

# 2. Configure collector
# Edit config/collector.yaml:
#   - appliance_url: Your QRNG appliance URL
#   - push_url: Gateway URL (Machine B)
#   - hmac_secret_key: Generate with: openssl rand -hex 32

# 3. Build image
docker build -f Dockerfile.collector -t qrng-entropy-collector:latest .

# 4. Run container
docker run -d \
  --name entropy-collector \
  --restart unless-stopped \
  -v $(pwd)/config/collector.yaml:/etc/qrng/collector.yaml:ro \
  -e RUST_LOG=info \
  qrng-entropy-collector:latest

# 5. Verify
docker logs entropy-collector -f
```

### Machine B (External Network - Gateway)

```bash
# 1. Copy project to external machine
cd /path/to/qrng-data-diode

# 2. Configure gateway
# Edit config/gateway-push.yaml:
#   - listen_address: 0.0.0.0:8080
#   - hmac_secret_key: Same key as collector
#   - api_keys: Generate secure keys for clients

# 3. Build image
docker build -f Dockerfile.gateway -t qrng-entropy-gateway:latest .

# 4. Run container
docker run -d \
  --name entropy-gateway \
  --restart unless-stopped \
  -p 8080:8080 \
  -v $(pwd)/config/gateway-push.yaml:/etc/qrng/gateway.yaml:ro \
  -e RUST_LOG=info \
  qrng-entropy-gateway:latest \
  --config /etc/qrng/gateway.yaml

# 5. Verify
docker logs entropy-gateway -f
curl http://localhost:8080/health
```

## Using Helper Scripts

### PowerShell (Windows)

```powershell
# On internal machine
.\deploy.ps1 build-collector
.\deploy.ps1 run-collector config\collector.yaml

# On external machine
.\deploy.ps1 build-gateway
.\deploy.ps1 run-gateway config\gateway-push.yaml

# View logs
.\deploy.ps1 logs

# Check status
.\deploy.ps1 status
```

### Bash (Linux/macOS)

```bash
# On internal machine
./deploy.sh build-collector
./deploy.sh run-collector config/collector.yaml

# On external machine
./deploy.sh build-gateway
./deploy.sh run-gateway config/gateway-push.yaml

# View logs
./deploy.sh logs

# Check status
./deploy.sh status
```

### Makefile

```bash
# On internal machine
make build-collector
make run-collector

# On external machine
make build-gateway
make run-gateway

# Management
make logs    # View logs
make status  # Check status
make stop    # Stop container
make clean   # Remove everything
```

## Configuration

### Generate HMAC Secret Key

Both machines need the same HMAC secret key:

```bash
openssl rand -hex 32
```

### Machine A (Collector) - config/collector.yaml

```yaml
appliance_url: "https://your-qrng-appliance:443/random"
push_url: "https://machine-b-ip:8080/push"
hmac_secret_key: "<your-generated-256-bit-key>"
fetch_chunk_size: 4096
push_interval_ms: 1000
buffer_size: 1048576  # 1MB
```

### Machine B (Gateway) - config/gateway-push.yaml

```yaml
mode: "push"
listen_address: "0.0.0.0:8080"
hmac_secret_key: "<same-key-as-collector>"
api_keys:
  - "your-secure-api-key-1"
  - "your-secure-api-key-2"
buffer_size: 10485760  # 10MB
buffer_ttl_seconds: 3600
rate_limit_per_second: 100
```

## Container Management

### Health Checks

Both containers include built-in health checks:

```bash
# Check gateway health
curl http://localhost:8080/health

# View container health status
docker inspect --format='{{.State.Health.Status}}' entropy-gateway

# Check from inside container
docker exec entropy-gateway wget --quiet --tries=1 --spider http://localhost:8080/health
```

### Viewing Logs

```bash
# Follow logs in real-time
docker logs entropy-collector -f
docker logs entropy-gateway -f

# Last 100 lines
docker logs entropy-collector --tail 100

# With timestamps
docker logs entropy-gateway --timestamps

# Since specific time
docker logs entropy-gateway --since 2025-01-06T10:00:00
```

### Resource Monitoring

```bash
# View resource usage
docker stats entropy-gateway

# Detailed container info
docker inspect entropy-collector

# Process list inside container
docker top entropy-gateway
```

### Updating Configuration

```bash
# Edit configuration file
vim config/gateway-push.yaml

# Restart container to apply changes
docker restart entropy-gateway

# Verify new configuration is loaded
docker logs entropy-gateway --tail 20
```

### Container Lifecycle

```bash
# Stop container
docker stop entropy-gateway

# Start stopped container
docker start entropy-gateway

# Restart container
docker restart entropy-gateway

# Remove container (must be stopped first)
docker stop entropy-gateway && docker rm entropy-gateway

# Remove and recreate
docker stop entropy-gateway && docker rm entropy-gateway
docker run -d ... # Run command as shown in Quick Start
```

## Application Monitoring

### Metrics Endpoint

```bash
# Prometheus-compatible metrics
curl http://localhost:8080/metrics

# Application status with API key
curl -H "Authorization: Bearer your-api-key" \
  http://localhost:8080/api/status
```

### Testing the System

```bash
# Test random number generation
curl -H "Authorization: Bearer your-api-key" \
  "http://localhost:8080/api/random?bytes=32&encoding=hex"

# Run Monte Carlo π test
curl -X POST -H "Authorization: Bearer your-api-key" \
  "http://localhost:8080/api/test/monte-carlo?iterations=1000000"
```

## Troubleshooting

### Container Won't Start

```bash
# View detailed logs
docker logs entropy-gateway --tail 100

# Check configuration
docker exec entropy-gateway cat /etc/qrng/gateway-push.yaml

# Verify binary
docker exec entropy-gateway /usr/local/bin/entropy-gateway --version
```

### Network Connectivity Issues

```bash
# Test internal connectivity
docker exec entropy-collector ping entropy-gateway

# Check network configuration
docker network inspect qrng-data-diode_bridge-network

# Test external connectivity
docker exec entropy-gateway wget --spider https://your-qrng-appliance
```

### Permission Issues

```bash
# Containers run as non-root user (qrng:1000)
# Check file permissions
ls -la config/

# Fix permissions if needed
chmod 644 config/*.yaml
```

## Security Best Practices

### 1. Configuration Secrets

```bash
# Use Docker secrets for production
echo "your-hmac-key" | docker secret create hmac_secret -
echo "your-api-key" | docker secret create api_key -
```

Update docker-compose.yml:
```yaml
secrets:
  hmac_secret:
    external: true
  api_key:
    external: true

services:
  entropy-gateway:
    secrets:
      - hmac_secret
      - api_key
```

### 2. Network Isolation

```bash
# In production, use separate physical hosts or VLANs
# Map docker networks to actual network interfaces

# Example: bind to specific interface
docker-compose up -d --network host
```

### 3. TLS/HTTPS

For production, place containers behind reverse proxy (nginx, Traefik):

```yaml
# Example nginx configuration
upstream qrng_gateway {
    server entropy-gateway:8080;
}

server {
    listen 443 ssl http2;
    server_name qrng.example.com;
    
    ssl_certificate /etc/ssl/certs/cert.pem;
    ssl_certificate_key /etc/ssl/private/key.pem;
    
    location / {
        proxy_pass http://qrng_gateway;
    }
}
```

### 4. Resource Limits

Add resource constraints:

```yaml
services:
  entropy-gateway:
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
```

## Backup and Recovery

### Configuration Backup

```bash
# Backup configurations
tar -czf qrng-config-backup-$(date +%Y%m%d).tar.gz config/

# Restore
tar -xzf qrng-config-backup-20250106.tar.gz
```

### Container State

```bash
# Export container (includes runtime state)
docker commit entropy-gateway qrng-gateway-backup:20250106
docker save -o qrng-gateway-backup.tar qrng-gateway-backup:20250106

# Restore
docker load -i qrng-gateway-backup.tar
```

## Scaling

### Horizontal Scaling (Gateway)

```yaml
services:
  entropy-gateway:
    deploy:
      replicas: 3
```

Then use load balancer:

```bash
# Using nginx
upstream qrng_cluster {
    server gateway-1:8080;
    server gateway-2:8080;
    server gateway-3:8080;
}
```

## Production Deployment Checklist

- [ ] Generate strong HMAC secret key (256-bit)
- [ ] Configure unique API keys per client
- [ ] Enable TLS/HTTPS via reverse proxy
- [ ] Set resource limits and reservations
- [ ] Configure log rotation
- [ ] Set up monitoring and alerting
- [ ] Test health checks and failover
- [ ] Document recovery procedures
- [ ] Enable automatic container restart policies
- [ ] Secure configuration files (restrict access)
- [ ] Implement backup strategy
- [ ] Review network isolation
- [ ] Test disaster recovery procedures

## Support

For issues or questions:
- Check logs: `make logs` or `docker-compose logs -f`
- Review configuration files in `config/`
- Consult main README.md for API documentation
- Open issue on GitHub repository

---

**Note**: This deployment is designed for both development and production use. Always review security settings before production deployment.

# Spring Boot Observability Patterns

Patterns for implementing observability in Spring Boot applications — Micrometer metrics, distributed tracing, structured logging, health monitoring, SLI/SLO, and alerting.

---

## Metrics — Micrometer

### Registries

| Registry | Use Case |
|---|---|
| Prometheus | Most common — Kubernetes deployments |
| InfluxDB | Time-series database |
| CloudWatch | AWS environments |
| Datadog | Datadog monitoring |
| New Relic | New Relic APM |

### Metric Types

| Type | Description | Example |
|---|---|---|
| Counter | Monotonically increasing value | Request count |
| Gauge | Current value (up or down) | Queue size |
| Timer | Duration of operations | Request latency |
| Summary | Distribution of values | Response sizes |

### Custom Metrics

```java
@Service
public class OrderService {
    private final Counter ordersCreated;
    private final Timer orderProcessingTime;

    public OrderService(MeterRegistry registry) {
        this.ordersCreated = registry.counter("orders.created");
        this.orderProcessingTime = registry.timer("order.processing.time");
    }

    public Order create(CreateOrderRequest request) {
        return orderProcessingTime.record(() -> {
            Order order = processOrder(request);
            ordersCreated.increment();
            return order;
        });
    }
}
```

### Custom Metric Categories

| Category | Examples |
|---|---|
| Business metrics | `orders.created`, `revenue.total` |
| Performance metrics | `product.query.duration`, `order.processing.time` |
| Error metrics | `errors.rate`, `errors.by.type` |
| Resource metrics | `active.users.count`, `queue.size` |

### Built-in Spring Boot Metrics

| Metric | Description |
|---|---|
| `http.server.requests` | HTTP request metrics |
| `jvm.memory` | JVM memory usage |
| `jvm.gc` | Garbage collection metrics |
| `jvm.threads` | Thread metrics |
| `process.cpu` | CPU usage |
| `hikaricp.connections` | Database connection pool metrics |

### Configuration

```properties
management.metrics.export.prometheus.enabled=true
management.endpoints.web.exposure.include=health,info,metrics,prometheus
```

### Best Practices

- Use Micrometer for all metrics
- Use appropriate metric types
- Add meaningful tags/labels
- Avoid high-cardinality tags
- Export to Prometheus for Kubernetes
- Monitor business metrics, not just technical

---

## Distributed Tracing

### Implementations

| Technology | Tracer | Use Case |
|---|---|---|
| Micrometer Tracing | Brave, OpenTelemetry, Wavefront | Spring-native tracing |
| Zipkin | — | Request tracing visualization, dependency graph |
| Jaeger | — | High-performance tracing, OpenTelemetry support |

### Trace Concepts

| Concept | Description |
|---|---|
| Trace | Complete request path across services |
| Span | Individual operation within a trace |
| Span Kind | `SERVER`, `CLIENT`, `PRODUCER`, `CONSUMER` |
| Correlation ID | Unique ID propagated across services |
| Baggage | Context data propagated with trace |
| Sampling | Reduce trace volume in high-throughput systems |

### Configuration

```properties
management.tracing.sampling.probability=1.0
management.zipkin.tracing.endpoint=http://localhost:9411/api/v2/spans
```

### Best Practices

- Use Micrometer Tracing with Spring Boot
- Propagate trace context across service calls
- Add custom spans for business operations
- Use correlation IDs in logs
- Configure appropriate sampling rates
- Monitor trace volume and performance
- Add trace context to error messages
- Use baggage for cross-service context

---

## Structured Logging

### Logback + Logstash Encoder

```xml
<!-- logback-spring.xml -->
<configuration>
    <appender name="JSON" class="ch.qos.logback.core.ConsoleAppender">
        <encoder class="net.logstash.logback.encoder.LogstashEncoder"/>
    </appender>

    <root level="INFO">
        <appender-ref ref="JSON"/>
    </root>
</configuration>
```

### MDC (Mapped Diagnostic Context)

Use cases: Correlation IDs, User IDs, Request IDs, Trace IDs.

```java
MDC.put("traceId", traceId);
MDC.put("userId", userId);
try {
    // Business logic...
} finally {
    MDC.clear();
}
```

### Log Levels

| Level | Use |
|---|---|
| `TRACE` | Very detailed debugging |
| `DEBUG` | Debugging information |
| `INFO` | General information |
| `WARN` | Warning messages |
| `ERROR` | Error messages |

### Best Practices

- Use structured logging (JSON) in production
- Add correlation IDs via MDC
- Use appropriate log levels
- Don't log sensitive information
- Configure different formats per environment
- Monitor log volume
- Use log aggregation (ELK, Loki)

---

## Spring Boot Actuator

### Endpoints

| Endpoint | Purpose |
|---|---|
| `/actuator/health` | Application health status |
| `/actuator/info` | Application information |
| `/actuator/metrics` | Application metrics |
| `/actuator/prometheus` | Prometheus metrics endpoint |
| `/actuator/env` | Environment properties |
| `/actuator/loggers` | Logger configuration |
| `/actuator/threaddump` | Thread dump |
| `/actuator/heapdump` | Heap dump |

### Health Probes (Kubernetes)

| Probe | Endpoint | Purpose |
|---|---|---|
| Liveness | `/actuator/health/liveness` | Container is alive |
| Readiness | `/actuator/health/readiness` | Container is ready for traffic |
| Startup | `/actuator/health/startup` | Container has started |

```properties
management.health.probes.enabled=true
management.endpoints.web.exposure.include=health,info,metrics,prometheus
management.endpoint.health.show-details=when-authorized
```

### Custom Health Indicators

Built-in: `DatabaseHealthIndicator`, `DiskSpaceHealthIndicator`, `RedisHealthIndicator`.

Implement `HealthIndicator` interface for custom checks (database connectivity, external service availability, business health).

### Best Practices

- Expose only necessary endpoints
- Secure actuator endpoints
- Use health probes for Kubernetes
- Create custom health indicators
- Monitor actuator metrics
- Use info endpoint for version info

---

## Prometheus

### Scraping

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'spring-boot-app'
    scrape_interval: 15s
    metrics_path: /actuator/prometheus
    static_configs:
      - targets: ['localhost:8080']
```

### PromQL Examples

| Query | Purpose |
|---|---|
| `rate(http_server_requests_seconds_count[5m])` | Request rate |
| `histogram_quantile(0.95, rate(http_server_requests_seconds_bucket[5m]))` | P95 latency |
| `sum(rate(http_server_requests_seconds_count[5m])) by (status)` | Rate by status |

### Best Practices

- Use Prometheus for Kubernetes deployments
- Configure appropriate scrape intervals
- Use service discovery for scraping
- Set up alerting rules
- Monitor Prometheus itself
- Use recording rules for complex queries

---

## Grafana

### Data Sources

| Source | Purpose |
|---|---|
| Prometheus | Metrics |
| Loki | Log aggregation |
| Jaeger | Distributed tracing |
| Elasticsearch | Log storage |

### Dashboard Panels

| Panel | Visualization |
|---|---|
| Graph | Time series graphs |
| Stat | Single stat panels |
| Table | Table visualizations |
| Heatmap | Heatmap visualizations |

### Alerting Channels

Supported: Email, Slack, PagerDuty, Webhook.

### Best Practices

- Create service-specific dashboards
- Use consistent naming
- Add alerts to dashboards
- Use variables for flexibility
- Avoid alert fatigue
- Document alert runbooks

---

## SLI / SLO / SLA

### Service Level Indicators (SLI)

| SLI | Example |
|---|---|
| Availability | Uptime percentage |
| Latency | Response time (p50, p95, p99) |
| Error rate | Percentage of failed requests |
| Throughput | Requests per second |

### Service Level Objectives (SLO)

| SLO | Target |
|---|---|
| Availability | 99.9% uptime |
| Latency | 95% of requests < 200ms |
| Error rate | < 0.1% errors |
| Throughput | > 1000 req/s |

### SLA

SLA is a contract with users — typically less strict than SLO.

### Best Practices

- Define SLIs based on user experience
- Set realistic SLOs
- Monitor SLI continuously
- Alert when SLO is at risk
- Review and adjust SLOs regularly
- Document SLOs and SLIs

---

## Alerting

### Alert Types

| Type | Urgency |
|---|---|
| Critical | Immediate attention required |
| Warning | Attention needed soon |
| Info | Informational alerts |

### Alert Rule Examples

| Rule | Condition |
|---|---|
| High error rate | Error rate > 1% for 5 minutes |
| High latency | P95 latency > 500ms for 5 minutes |
| Low availability | Availability < 99% for 1 minute |
| Resource exhaustion | Memory usage > 90% |

### Best Practices

- Alert on symptoms, not causes
- Avoid alert fatigue
- Use alert grouping
- Set appropriate thresholds
- Test alert channels
- Document runbooks
- Review and tune alerts regularly

---

## Correlation IDs

### Implementation

| Channel | Method |
|---|---|
| MDC | Add to MDC for logging |
| HTTP headers | Propagate via HTTP headers |
| Trace context | Include in trace context |
| Messaging | Include in message headers |

### Best Practices

- Generate at API Gateway
- Propagate across all services
- Include in all logs
- Add to error messages
- Use for debugging
- Store in trace context

---

## Complete Observability Stack

| Pillar | Tools |
|---|---|
| Metrics | Prometheus + Grafana |
| Logs | ELK Stack (Elasticsearch, Logstash, Kibana) or Loki |
| Traces | Zipkin or Jaeger |
| APM | Application Performance Monitoring (optional) |

### Integrations

| Platform | Integration |
|---|---|
| Spring Boot | Micrometer for metrics and tracing |
| Kubernetes | Prometheus Operator, ServiceMonitor |
| CI/CD | Observability in deployment pipelines |

### Best Practices

- Use consistent naming conventions
- Correlate metrics, logs, and traces
- Set up dashboards for each service
- Implement alerting
- Monitor the monitoring system
- Regularly review and optimize

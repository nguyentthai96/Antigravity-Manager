# Cloud Native — Spring Cloud 2025 / Spring Boot 4.x

## Built-in Resilience (Spring Framework 7 — No External Library)

> ⚡ **Spring Boot 4.x**: `@Retryable` and `@ConcurrencyLimit` are now part of the core framework. Remove `spring-retry` and `resilience4j` dependencies.

```java
@Configuration
@EnableResilientMethods  // ⚡ Activates built-in resilience
public class ResilienceConfig {}
```

### @Retryable — Built-in retry

```java
@Service
public class ExternalApiService {
    private final RestClient restClient;

    private static final Logger log = LoggerFactory.getLogger(ExternalApiService.class);

    public ExternalApiService(RestClient.Builder builder) {
        this.restClient = builder.baseUrl("https://api.example.com").build();
    }

    @Retryable(maxAttempts = 3, backoff = @Backoff(delay = 1000, multiplier = 2.0))
    public ExternalData fetchData(String id) {
        return restClient.get()
            .uri("/data/{id}", id)
            .retrieve()
            .body(ExternalData.class);
    }

    @Retryable(maxAttempts = 3, retryFor = {RestClientException.class, TimeoutException.class})
    public OrderStatus checkOrderStatus(String orderId) {
        return restClient.get()
            .uri("/orders/{id}/status", orderId)
            .retrieve()
            .body(OrderStatus.class);
    }

    // Fallback method (same signature + Exception parameter)
    public ExternalData fetchDataFallback(String id, Exception ex) {
        log.warn("Fallback for fetchData({}): {}", id, ex.getMessage());
        return new ExternalData(id, "Fallback data", LocalDateTime.now());
    }
}
```

### @ConcurrencyLimit — Built-in concurrency protection

```java
@Service
public class ReportService {

    @ConcurrencyLimit(10)  // ⚡ Max 10 concurrent executions
    public Report generateHeavyReport(ReportRequest request) {
        // CPU-intensive computation...
        return report;
    }
}
```

### Reactive @Retryable

```java
@Service
public class ReactiveExternalService {
    private final WebClient webClient;

    public ReactiveExternalService(WebClient.Builder builder) {
        this.webClient = builder.baseUrl("https://api.example.com").build();
    }

    @Retryable(maxAttempts = 3, backoff = @Backoff(delay = 500))
    public Mono<ExternalData> fetchDataReactive(String id) {
        return webClient.get()
            .uri("/data/{id}", id)
            .retrieve()
            .bodyToMono(ExternalData.class);
    }
}
```

## Spring Cloud Config Server

```java
// Config Server
@SpringBootApplication
@EnableConfigServer
public class ConfigServerApplication {
    public static void main(String[] args) {
        SpringApplication.run(ConfigServerApplication.class, args);
    }
}
```

```properties
# application.properties (Config Server)
server.port=8888
spring.cloud.config.server.git.uri=https://github.com/example/config-repo
spring.cloud.config.server.git.default-label=main
spring.cloud.config.server.git.search-paths={application}
spring.cloud.config.server.git.username=${GIT_USERNAME}
spring.cloud.config.server.git.password=${GIT_PASSWORD}
spring.security.user.name=config-user
spring.security.user.password=${CONFIG_PASSWORD}
```

```properties
# application.properties (Config Client)
spring.application.name=user-service
spring.config.import=configserver:http://localhost:8888
spring.cloud.config.username=config-user
spring.cloud.config.password=${CONFIG_PASSWORD}
spring.cloud.config.fail-fast=true
spring.cloud.config.retry.max-attempts=6
spring.cloud.config.retry.initial-interval=1000
```

## Dynamic Configuration Refresh

```java
@RestController
@RefreshScope
public class ConfigController {
    @Value("${app.feature.enabled:false}")
    private boolean featureEnabled;

    @Value("${app.max-connections:100}")
    private int maxConnections;

    @GetMapping("/config")
    public Map<String, Object> getConfig() {
        return Map.of(
            "featureEnabled", featureEnabled,
            "maxConnections", maxConnections
        );
    }
}

// Refresh configuration via Actuator endpoint:
// POST /actuator/refresh
```

## Service Discovery — Eureka

```java
// Eureka Server
@SpringBootApplication
@EnableEurekaServer
public class EurekaServerApplication {
    public static void main(String[] args) {
        SpringApplication.run(EurekaServerApplication.class, args);
    }
}

// Eureka Client
@SpringBootApplication
@EnableDiscoveryClient
public class UserServiceApplication {
    public static void main(String[] args) {
        SpringApplication.run(UserServiceApplication.class, args);
    }
}
```

## Spring Cloud Gateway

```java
@SpringBootApplication
public class GatewayApplication {
    public static void main(String[] args) {
        SpringApplication.run(GatewayApplication.class, args);
    }

    @Bean
    public RouteLocator customRouteLocator(RouteLocatorBuilder builder) {
        return builder.routes()
            .route("user-service", r -> r
                .path("/api/users/**")
                .filters(f -> f
                    .rewritePath("/api/users/(?<segment>.*)", "/users/${segment}")
                    .addRequestHeader("X-Gateway", "Spring-Cloud-Gateway")
                    .circuitBreaker(config -> config
                        .setName("userServiceCircuitBreaker")
                        .setFallbackUri("forward:/fallback/users")
                    )
                    .retry(config -> config
                        .setRetries(3)
                        .setStatuses(HttpStatus.SERVICE_UNAVAILABLE)
                    )
                )
                .uri("lb://user-service")
            )
            .build();
    }
}
```

## Declarative HTTP Interface Client (replaces OpenFeign)

> ⚡ Spring Boot 4.x promotes HTTP Interface Clients as the standard way for service-to-service communication.

```java
public interface OrderServiceClient {
    @GetExchange("/orders/{id}")
    OrderDto getOrder(@PathVariable String id);

    @PostExchange("/orders")
    OrderDto createOrder(@RequestBody CreateOrderRequest request);
}

@Configuration
public class HttpClientConfig {
    @Bean
    public OrderServiceClient orderServiceClient(RestClient.Builder builder) {
        RestClient restClient = builder
            .baseUrl("http://order-service:8080/api/v1")
            .build();

        return HttpServiceProxyFactory
            .builderFor(RestClientAdapter.create(restClient))
            .build()
            .createClient(OrderServiceClient.class);
    }
}
```

## OpenTelemetry Observability (Spring Boot 4.x)

> ⚡ Spring Boot 4.x provides native OpenTelemetry integration alongside Micrometer.

```properties
# application.properties
management.tracing.sampling.probability=1.0
management.otlp.tracing.endpoint=http://localhost:4318/v1/traces
management.otlp.metrics.export.enabled=true
management.otlp.metrics.export.url=http://localhost:4318/v1/metrics

logging.pattern.level=%5p [${spring.application.name:},%X{traceId:-},%X{spanId:-}]
```

```java
// Custom spans with Micrometer Observation API
@Service
public class OrderService {
    private final ObservationRegistry observationRegistry;
    private final OrderRepository orderRepository;

    public OrderService(ObservationRegistry observationRegistry, OrderRepository orderRepository) {
        this.observationRegistry = observationRegistry;
        this.orderRepository = orderRepository;
    }

    public Order processOrder(OrderRequest request) {
        return Observation.createNotStarted("processOrder", observationRegistry)
            .lowCardinalityKeyValue("order.type", request.type())
            .highCardinalityKeyValue("order.items", String.valueOf(request.items().size()))
            .observe(() -> {
                // Business logic
                return createOrder(request);
            });
    }
}
```

## Health Checks & Actuator

```java
@Component
public class CustomHealthIndicator implements HealthIndicator {

    @Override
    public Health health() {
        boolean serviceUp = checkExternalService();

        if (serviceUp) {
            return Health.up()
                .withDetail("externalService", "Available")
                .withDetail("timestamp", LocalDateTime.now())
                .build();
        } else {
            return Health.down()
                .withDetail("externalService", "Unavailable")
                .withDetail("error", "Connection timeout")
                .build();
        }
    }

    private boolean checkExternalService() {
        // Check external dependency
        return true;
    }
}
```

```properties
# application.properties
management.endpoints.web.exposure.include=health,info,metrics,prometheus
management.endpoint.health.show-details=always
management.endpoint.health.probes.enabled=true
management.health.livenessstate.enabled=true
management.health.readinessstate.enabled=true
management.metrics.tags.application=${spring.application.name}
```

## Kubernetes Deployment (Java 25)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: user-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: user-service
  template:
    metadata:
      labels:
        app: user-service
    spec:
      containers:
      - name: user-service
        image: user-service:1.0.0
        ports:
        - containerPort: 8080
        env:
        - name: SPRING_PROFILES_ACTIVE
          value: "kubernetes"
        - name: JAVA_OPTS
          value: "-Xmx512m -Xms256m"
        - name: SPRING_THREADS_VIRTUAL_ENABLED
          value: "true"  # ⚡ Virtual threads
        livenessProbe:
          httpGet:
            path: /actuator/health/liveness
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /actuator/health/readiness
            port: 8080
          initialDelaySeconds: 15
          periodSeconds: 5
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
---
apiVersion: v1
kind: Service
metadata:
  name: user-service
spec:
  selector:
    app: user-service
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```

## Docker Configuration (Java 25)

```dockerfile
# Multi-stage build — Spring Boot 4.x with Java 25
FROM eclipse-temurin:25-jdk-alpine AS build
WORKDIR /workspace/app

COPY mvnw .
COPY .mvn .mvn
COPY pom.xml .
COPY src src

RUN ./mvnw install -DskipTests
RUN mkdir -p target/dependency && (cd target/dependency; jar -xf ../*.jar)

FROM eclipse-temurin:25-jre-alpine
VOLUME /tmp
ARG DEPENDENCY=/workspace/app/target/dependency
COPY --from=build ${DEPENDENCY}/BOOT-INF/lib /app/lib
COPY --from=build ${DEPENDENCY}/META-INF /app/META-INF
COPY --from=build ${DEPENDENCY}/BOOT-INF/classes /app

ENTRYPOINT ["java","-cp","app:app/lib/*","com.example.Application"]
```

## Quick Reference

| Component | Purpose |
|-----------|---------|
| **Config Server** | Centralized configuration management |
| **Eureka** | Service discovery and registration |
| **Gateway** | API gateway with routing, filtering, load balancing |
| **Built-in Resilience** | ⚡ `@Retryable`, `@ConcurrencyLimit` (no spring-retry/resilience4j) |
| **HTTP Interface Client** | ⚡ Declarative service-to-service calls (replaces OpenFeign) |
| **OpenTelemetry** | ⚡ Native distributed tracing and metrics |
| **Load Balancer** | Client-side load balancing |
| **Actuator** | Production-ready monitoring and management |
| **Kubernetes** | Container orchestration and deployment |

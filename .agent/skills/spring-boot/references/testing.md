# Testing — Spring Boot 4.x

> ⚠️ **Spring Boot 4.x Breaking Changes**:
> - `@MockBean` → `@MockitoBean`, `@SpyBean` → `@MockitoSpyBean`
> - `@WebMvcTest` moved to `org.springframework.boot.webmvc.test.autoconfigure` — requires `spring-boot-starter-webmvc-test`
> - `ObjectMapper` → `JsonMapper` (Jackson 3)
> - TestContainers 2.0+ required (artifact/package renames)

---

## Testing Dependencies

```xml
<dependencies>
    <!-- Spring Boot Test Starter (includes JUnit 5, Mockito, AssertJ) -->
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-test</artifactId>
        <scope>test</scope>
    </dependency>

    <!-- WebMvc Test Starter (provides @WebMvcTest, @AutoConfigureMockMvc) -->
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-webmvc-test</artifactId>
        <scope>test</scope>
    </dependency>

    <!-- TestContainers 2.0+ for integration tests -->
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-testcontainers</artifactId>
        <scope>test</scope>
    </dependency>

    <dependency>
        <groupId>org.testcontainers</groupId>
        <artifactId>testcontainers-postgresql</artifactId>
        <scope>test</scope>
    </dependency>

    <!-- Security Testing -->
    <dependency>
        <groupId>org.springframework.security</groupId>
        <artifactId>spring-security-test</artifactId>
        <scope>test</scope>
    </dependency>
</dependencies>
```

---

## Test Naming Conventions

| Suffix | Type | Runner | Example |
|--------|------|--------|---------|
| `*Test` | Unit test | Maven Surefire | `UserServiceTest.java` |
| `*IT` | Integration test | Maven Failsafe | `UserIntegrationIT.java` |

---

## Unit Testing with JUnit 5

```java
@ExtendWith(MockitoExtension.class)
class UserServiceTest {

    @Mock
    private UserRepository userRepository;

    @Mock
    private PasswordEncoder passwordEncoder;

    @InjectMocks
    private UserService userService;

    @Test
    @DisplayName("Should create user successfully")
    void shouldCreateUser() {
        // Given
        UserCreateRequest request = new UserCreateRequest(
            "test@example.com",
            "Password123",
            "testuser",
            25
        );

        User user = new User();
        user.setId(1L);
        user.setEmail(request.email());
        user.setUsername(request.username());

        when(userRepository.existsByEmail(request.email())).thenReturn(false);
        when(passwordEncoder.encode(request.password())).thenReturn("encodedPassword");
        when(userRepository.save(any(User.class))).thenReturn(user);

        // When
        UserResponse response = userService.create(request);

        // Then
        assertThat(response).isNotNull();
        assertThat(response.email()).isEqualTo(request.email());

        verify(userRepository).existsByEmail(request.email());
        verify(passwordEncoder).encode(request.password());
        verify(userRepository).save(any(User.class));
    }

    @Test
    @DisplayName("Should throw exception when email already exists")
    void shouldThrowExceptionWhenEmailExists() {
        // Given
        UserCreateRequest request = new UserCreateRequest(
            "test@example.com",
            "Password123",
            "testuser",
            25
        );

        when(userRepository.existsByEmail(request.email())).thenReturn(true);

        // When & Then
        assertThatThrownBy(() -> userService.create(request))
            .isInstanceOf(DuplicateResourceException.class)
            .hasMessageContaining("Email already registered");

        verify(userRepository, never()).save(any(User.class));
    }
}
```

---

## Web Layer Testing with MockMvc — `@MockitoBean`

> ✅ **Spring Boot 4 import:** `org.springframework.boot.webmvc.test.autoconfigure.WebMvcTest`
> ✅ **Dependency:** `spring-boot-starter-webmvc-test` (required)

```java
@WebMvcTest(UserController.class)  // ✅ org.springframework.boot.webmvc.test.autoconfigure.WebMvcTest
@Import(SecurityConfig.class)
class UserControllerTest {

    @Autowired
    private MockMvc mockMvc;

    @MockitoBean  // ⚡ Spring Boot 4.x — replaces @MockBean
    private UserService userService;

    @Autowired
    private JsonMapper jsonMapper;  // ⚡ Jackson 3 — replaces ObjectMapper

    @Test
    @WithMockUser(roles = "ADMIN")
    @DisplayName("Should get all users")
    void shouldGetAllUsers() throws Exception {
        // Given
        Page<UserResponse> users = new PageImpl<>(List.of(
            new UserResponse(1L, "user1@example.com", "user1", 25, true, null, null),
            new UserResponse(2L, "user2@example.com", "user2", 30, true, null, null)
        ));

        when(userService.findAll(any(Pageable.class))).thenReturn(users);

        // When & Then
        mockMvc.perform(get("/api/v1/users")
                .contentType(MediaType.APPLICATION_JSON))
            .andExpect(status().isOk())
            .andExpect(jsonPath("$.content").isArray())
            .andExpect(jsonPath("$.content.length()").value(2))
            .andExpect(jsonPath("$.content[0].email").value("user1@example.com"))
            .andDo(print());
    }

    @Test
    @WithMockUser(roles = "ADMIN")
    @DisplayName("Should create user")
    void shouldCreateUser() throws Exception {
        // Given
        UserCreateRequest request = new UserCreateRequest(
            "test@example.com",
            "Password123",
            "testuser",
            25
        );

        UserResponse response = new UserResponse(
            1L,
            request.email(),
            request.username(),
            request.age(),
            true,
            LocalDateTime.now(),
            LocalDateTime.now()
        );

        when(userService.create(any(UserCreateRequest.class))).thenReturn(response);

        // When & Then
        mockMvc.perform(post("/api/v1/users")
                .contentType(MediaType.APPLICATION_JSON)
                .content(jsonMapper.writeValueAsString(request)))
            .andExpect(status().isCreated())
            .andExpect(header().exists("Location"))
            .andExpect(jsonPath("$.email").value(request.email()))
            .andExpect(jsonPath("$.username").value(request.username()))
            .andDo(print());
    }

    @Test
    @WithMockUser(roles = "USER")
    @DisplayName("Should return 403 for non-admin user")
    void shouldReturn403ForNonAdmin() throws Exception {
        mockMvc.perform(get("/api/v1/users")
                .contentType(MediaType.APPLICATION_JSON))
            .andExpect(status().isForbidden());
    }
}
```

---

## Integration Testing with @SpringBootTest

```java
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
@ActiveProfiles("test")
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
class UserIntegrationTest {

    @Autowired
    private TestRestTemplate restTemplate;

    @Autowired
    private UserRepository userRepository;

    @BeforeEach
    void setUp() {
        userRepository.deleteAll();
    }

    @Test
    @Order(1)
    @DisplayName("Should create user via API")
    void shouldCreateUserViaApi() {
        // Given
        UserCreateRequest request = new UserCreateRequest(
            "test@example.com",
            "Password123",
            "testuser",
            25
        );

        // When
        ResponseEntity<UserResponse> response = restTemplate.postForEntity(
            "/api/v1/users",
            request,
            UserResponse.class
        );

        // Then
        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CREATED);
        assertThat(response.getBody()).isNotNull();
        assertThat(response.getBody().email()).isEqualTo(request.email());
        assertThat(response.getHeaders().getLocation()).isNotNull();
    }
}
```

---

## Data JPA Testing

```java
@DataJpaTest
@AutoConfigureTestDatabase(replace = AutoConfigureTestDatabase.Replace.NONE)
@ActiveProfiles("test")
class UserRepositoryTest {

    @Autowired
    private UserRepository userRepository;

    @Autowired
    private TestEntityManager entityManager;

    @Test
    @DisplayName("Should find user by email")
    void shouldFindUserByEmail() {
        // Given
        User user = User.builder()
            .email("test@example.com")
            .password("password")
            .username("testuser")
            .active(true)
            .build();

        entityManager.persistAndFlush(user);

        // When
        Optional<User> found = userRepository.findByEmail("test@example.com");

        // Then
        assertThat(found).isPresent();
        assertThat(found.get().getEmail()).isEqualTo("test@example.com");
    }

    @Test
    @DisplayName("Should fetch user with roles")
    void shouldFetchUserWithRoles() {
        // Given
        Role adminRole = Role.builder().name("ADMIN").build();
        entityManager.persist(adminRole);

        User user = User.builder()
            .email("admin@example.com")
            .password("password")
            .username("admin")
            .active(true)
            .roles(Set.of(adminRole))
            .build();

        entityManager.persistAndFlush(user);
        entityManager.clear();

        // When
        Optional<User> found = userRepository.findByEmailWithRoles("admin@example.com");

        // Then
        assertThat(found).isPresent();
        assertThat(found.get().getRoles()).hasSize(1);
        assertThat(found.get().getRoles()).extracting(Role::getName).contains("ADMIN");
    }
}
```

---

## Testcontainers for Database

### Configuration (package-private!)

```java
// src/test/java/.../TestcontainersConfiguration.java
import org.testcontainers.postgresql.PostgreSQLContainer; // ✅ TC 2.x package

@TestConfiguration(proxyBeanMethods = false)
class TestcontainersConfiguration {  // ✅ package-private (no public modifier)
    @Bean
    @ServiceConnection
    PostgreSQLContainer postgresContainer() {
        return new PostgreSQLContainer("postgres:17-alpine");
    }
}
```

> **Why package-private?**
> - Test configuration should not be exported outside the test package
> - Integration tests using `@Import(TestcontainersConfiguration.class)` must be in the **same package**

### Usage in Tests

```java
@SpringBootTest
@Testcontainers
@ActiveProfiles("test")
class UserServiceIntegrationTest {

    @Container
    @ServiceConnection  // ⚡ Auto-configures datasource — no @DynamicPropertySource needed
    static PostgreSQLContainer<?> postgres = new PostgreSQLContainer<>("postgres:17-alpine");

    @Autowired
    private UserService userService;

    @Autowired
    private UserRepository userRepository;

    @BeforeEach
    void setUp() {
        userRepository.deleteAll();
    }

    @Test
    @DisplayName("Should create and find user in real database")
    void shouldCreateAndFindUser() {
        // Given
        UserCreateRequest request = new UserCreateRequest(
            "test@example.com",
            "Password123",
            "testuser",
            25
        );

        // When
        UserResponse created = userService.create(request);
        UserResponse found = userService.findById(created.id());

        // Then
        assertThat(found).isNotNull();
        assertThat(found.email()).isEqualTo(request.email());
    }
}
```

---

## REST API Integration Test

```java
@SpringBootTest
@AutoConfigureMockMvc  // ✅ org.springframework.boot.webmvc.test.autoconfigure.AutoConfigureMockMvc
@Import(TestcontainersConfiguration.class)
class UserIntegrationIT {

    @Autowired
    private MockMvc mockMvc;

    @Autowired
    private JsonMapper jsonMapper;  // ⚡ Jackson 3

    @Autowired
    private UserRepository userRepository;

    @BeforeEach
    void setUp() {
        userRepository.deleteAll();
    }

    @Test
    void shouldCreateAndRetrieveUser() throws Exception {
        // Given
        User newUser = new User(null, "integrationuser", "integration@example.com");

        // When — Create user
        String response = mockMvc.perform(post("/api/users")
                .contentType(MediaType.APPLICATION_JSON)
                .content(jsonMapper.writeValueAsString(newUser)))
            .andExpect(status().isCreated())
            .andExpect(jsonPath("$.username").value("integrationuser"))
            .andReturn()
            .getResponse()
            .getContentAsString();

        User createdUser = jsonMapper.readValue(response, User.class);

        // Then — Verify user exists in database
        assertThat(userRepository.findById(createdUser.getId())).isPresent();

        // When — Retrieve user
        mockMvc.perform(get("/api/users/" + createdUser.getId()))
            .andExpect(status().isOk())
            .andExpect(jsonPath("$.username").value("integrationuser"))
            .andExpect(jsonPath("$.email").value("integration@example.com"));
    }
}
```

---

## REST API Integration Test with RestTestClient

> `RestTestClient` provides a `WebTestClient`-style fluent API for testing REST APIs against a running server.
> Use when testing with a real HTTP connection (vs. MockMvc's in-memory approach).

### Base Integration Test Class

```java
import org.springframework.boot.resttestclient.autoconfigure.AutoConfigureRestTestClient;
import org.springframework.test.web.servlet.client.RestTestClient;
import tools.jackson.databind.json.JsonMapper;

@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
@Import(TestcontainersConfiguration.class)
@AutoConfigureRestTestClient
@Sql("/test-data.sql")
public abstract class BaseIT {

    @Autowired
    protected RestTestClient restTestClient;

    @Autowired
    protected JsonMapper jsonMapper;  // ⚡ Jackson 3

    protected String getAuthToken(String email, String password) {
        // Logic to generate JWT token
        return "jwt-token";
    }
}
```

### Controller Test Example

```java
class UserControllerTests extends BaseIT {

    @Test
    void shouldRegisterUserSuccessfully() {
        RegisterUserResponse response = restTestClient
                .post()
                .uri("/api/users")
                .contentType(MediaType.APPLICATION_JSON)
                .body("""
                        {
                          "fullName":"User123",
                          "email":"user123@gmail.com",
                          "password":"Secret@121212"
                        }
                        """)
                .exchange()
                .expectStatus()
                .isCreated()
                .returnResult(RegisterUserResponse.class)
                .getResponseBody();

        assertThat(response).isNotNull();
        assertThat(response.email()).isEqualTo("user123@gmail.com");
    }

    @ParameterizedTest
    @CsvSource({
        ",user1@gmail.com,password123,FullName",
        "user1,,password123,Email",
        "user1,user1@gmail.com,,Password",
    })
    void shouldNotRegisterWithoutRequiredFields(
            String fullName, String email, String password, String errorField) {
        record ReqBody(String fullName, String email, String password) {}

        ExchangeResult result = restTestClient
                .post()
                .uri("/api/users")
                .contentType(MediaType.APPLICATION_JSON)
                .body(new ReqBody(fullName, email, password))
                .exchange()
                .expectStatus()
                .isBadRequest()
                .returnResult();

        String responseJson = new String(result.getResponseBodyContent());
        assertThat(responseJson).contains("%s is required".formatted(errorField));
    }

    @Test
    void shouldNotUpdateUserWithoutAuthentication() {
        restTestClient
                .put()
                .uri("/api/users/me")
                .contentType(MediaType.APPLICATION_JSON)
                .body("""
                        { "fullName": "Updated Name" }
                        """)
                .exchange()
                .expectStatus()
                .isUnauthorized();
    }
}
```

---

## Testing Reactive Endpoints with WebTestClient

```java
@WebFluxTest(UserReactiveController.class)
class UserReactiveControllerTest {

    @Autowired
    private WebTestClient webTestClient;

    @MockitoBean  // ⚡ Spring Boot 4.x
    private UserReactiveService userService;

    @Test
    @DisplayName("Should get user reactively")
    void shouldGetUserReactively() {
        // Given
        UserResponse user = new UserResponse(
            1L, "test@example.com", "testuser", 25, true,
            LocalDateTime.now(), LocalDateTime.now()
        );

        when(userService.findById(1L)).thenReturn(Mono.just(user));

        // When & Then
        webTestClient.get()
            .uri("/api/v1/users/{id}", 1L)
            .accept(MediaType.APPLICATION_JSON)
            .exchange()
            .expectStatus().isOk()
            .expectBody(UserResponse.class)
            .value(response -> {
                assertThat(response.id()).isEqualTo(1L);
                assertThat(response.email()).isEqualTo("test@example.com");
            });
    }
}
```

---

## Testing HTTP Interface Clients

```java
@ExtendWith(MockitoExtension.class)
class UserServiceClientTest {

    private MockWebServer mockServer;
    private UserServiceClient client;

    @BeforeEach
    void setUp() throws IOException {
        mockServer = new MockWebServer();
        mockServer.start();

        RestClient restClient = RestClient.builder()
            .baseUrl(mockServer.url("/api/v1").toString())
            .build();

        client = HttpServiceProxyFactory
            .builderFor(RestClientAdapter.create(restClient))
            .build()
            .createClient(UserServiceClient.class);
    }

    @AfterEach
    void tearDown() throws IOException {
        mockServer.shutdown();
    }

    @Test
    @DisplayName("Should call external user service")
    void shouldCallExternalService() {
        // Given
        mockServer.enqueue(new MockResponse()
            .setBody("""{"id":1,"name":"John"}""")
            .addHeader("Content-Type", "application/json"));

        // When
        UserDto user = client.getUser(1L);

        // Then
        assertThat(user.id()).isEqualTo(1L);
        assertThat(user.name()).isEqualTo("John");
    }
}
```

---

## Testing Built-in Resilience

```java
@SpringBootTest
@ActiveProfiles("test")
class ResilientServiceTest {

    @MockitoBean  // ⚡ Spring Boot 4.x
    private ExternalApiClient externalApiClient;

    @Autowired
    private ExternalApiService externalApiService;

    @Test
    @DisplayName("Should retry on transient failure")
    void shouldRetryOnTransientFailure() {
        // Given — fail twice, succeed third time
        when(externalApiClient.fetchData("123"))
            .thenThrow(new RestClientException("Connection reset"))
            .thenThrow(new RestClientException("Connection reset"))
            .thenReturn(new ExternalData("123", "Success"));

        // When
        ExternalData result = externalApiService.fetchData("123");

        // Then
        assertThat(result.value()).isEqualTo("Success");
        verify(externalApiClient, times(3)).fetchData("123");
    }
}
```

---

## Security Scanning & SBOM (CI-ready)

Add to `pom.xml` (plugins section):
```xml
<plugin>
  <groupId>org.cyclonedx</groupId>
  <artifactId>cyclonedx-maven-plugin</artifactId>
  <version>2.8.0</version>
  <executions>
    <execution>
      <phase>verify</phase>
      <goals><goal>makeAggregateBom</goal></goals>
    </execution>
  </executions>
</plugin>
<plugin>
  <groupId>org.owasp</groupId>
  <artifactId>dependency-check-maven</artifactId>
  <version>9.2.0</version>
  <configuration>
    <failBuildOnCVSS>7</failBuildOnCVSS>
  </configuration>
</plugin>
```

CI commands:
```bash
mvn -B -DskipTests=false verify cyclonedx:makeAggregateBom dependency-check:check
```

---

## Test Configuration (Spring Boot 4.x)

```properties
# application-test.properties (prefer .properties over YAML)
spring.datasource.url=jdbc:tc:postgresql:17-alpine:///testdb
spring.datasource.driver-class-name=org.testcontainers.jdbc.ContainerDatabaseDriver
spring.jpa.hibernate.ddl-auto=create-drop
spring.jpa.show-sql=true
spring.jpa.properties.hibernate.format_sql=true

logging.level.org.hibernate.SQL=DEBUG
logging.level.org.hibernate.orm.jdbc.bind=TRACE
```

```java
@TestConfiguration
public class TestConfig {

    @Bean
    @Primary
    public PasswordEncoder passwordEncoder() {
        return new BCryptPasswordEncoder(4); // Faster for tests
    }

    @Bean
    public Clock fixedClock() {
        return Clock.fixed(
            Instant.parse("2025-01-01T00:00:00Z"),
            ZoneId.of("UTC")
        );
    }
}
```

---

## @MockitoBean Migration from @MockBean

**Old (Deprecated in Spring Boot 4):**
```java
import org.springframework.boot.test.mock.mockito.MockBean;  // ❌ Deprecated

@SpringBootTest
class MyTest {
    @MockBean
    private UserService userService;
}
```

**New (Spring Boot 4):**
```java
import org.springframework.test.context.bean.override.mockito.MockitoBean;  // ✅ Correct

@SpringBootTest
class MyTest {
    @MockitoBean
    private UserService userService;
}
```

### Shared Mocks Pattern

```java
// Custom annotation for shared mocks
@Target(ElementType.TYPE)
@Retention(RetentionPolicy.RUNTIME)
@MockitoBean(types = {UserService.class, OrderService.class})
@MockitoBean(name = "emailService", types = EmailService.class)
public @interface SharedMocks {
}

@SpringBootTest
@SharedMocks
class ApplicationTests {
    // Clean test class
}
```

### TestContainers 2.0 Migration

- **Artifact rename:** `org.testcontainers:postgresql` → `org.testcontainers:testcontainers-postgresql`
- **Package rename:** `org.testcontainers.containers.PostgreSQLContainer` → `org.testcontainers.postgresql.PostgreSQLContainer`
- **`PostgreSQLContainer` is no longer generic** — use `PostgreSQLContainer` (not `PostgreSQLContainer<?>`)
- `junit-jupiter` artifact removed — TC 2.x integrates with JUnit 5 directly

---

## Running Tests

```bash
# Run all tests (unit + integration)
./mvnw verify

# Run only unit tests (fast)
./mvnw test

# Run only integration tests
./mvnw failsafe:integration-test

# Run specific test class
./mvnw test -Dtest=UserServiceTest

# Run specific integration test
./mvnw verify -Dit.test=UserIntegrationIT

# Run with coverage report
./mvnw verify jacoco:report

# Skip tests during build
./mvnw package -DskipTests
```

---

## Example Test Directory Structure

```
src/test/java/
└── com/example/app/
    ├── TestcontainersConfiguration.java     # TestContainers config (package-private!)
    ├── UserIntegrationIT.java               # Integration test (same package as TC config)
    ├── UserRepositoryIT.java                # Integration test (same package as TC config)
    ├── controller/
    │   └── UserControllerTest.java          # Unit test with mocks (@WebMvcTest)
    └── service/
        └── UserServiceTest.java             # Unit test with mocks
```

> Integration tests (`*IT.java`) must live in the **same package** as `TestcontainersConfiguration` because it is package-private.

---

## Quick Reference

| Annotation | Purpose |
|------------|---------|
| `@SpringBootTest` | Full application context integration test |
| `@WebMvcTest` | Test MVC controllers with mocked services |
| `@WebFluxTest` | Test reactive controllers |
| `@DataJpaTest` | Test JPA repositories with in-memory database |
| `@MockitoBean` | ⚡ Add mock bean to Spring context (replaces `@MockBean`) |
| `@MockitoSpyBean` | ⚡ Add spy bean to Spring context (replaces `@SpyBean`) |
| `@ServiceConnection` | ⚡ Auto-configure Testcontainers connection |
| `@WithMockUser` | Mock authenticated user for security tests |
| `@Testcontainers` | Enable Testcontainers support |
| `@ActiveProfiles` | Activate specific Spring profiles for test |

---

## Testing Best Practices

- Use `@MockitoBean` (NOT `@MockBean`) — deprecated in Spring Boot 4.x
- Use `@ServiceConnection` with Testcontainers — eliminates `@DynamicPropertySource` boilerplate
- Use `JsonMapper` (NOT `ObjectMapper`) for Jackson 3 serialization in tests
- Write tests following AAA pattern (Arrange, Act, Assert)
- Use descriptive test names with `@DisplayName`
- Mock external dependencies, use real DB with Testcontainers
- Test built-in resilience (`@Retryable`, `@ConcurrencyLimit`) behavior
- Test HTTP Interface Clients with MockWebServer
- Achieve 85%+ code coverage
- Test happy path and edge cases
- Use `@Transactional` for test data cleanup
- Separate unit tests from integration tests
- Use parameterized tests for multiple scenarios
- Test security rules and validation
- Keep tests fast and independent

## References

- [Spring Boot Testing Guide](https://spring.io/guides/gs/testing-web/)
- [TestContainers Documentation](https://testcontainers.com/)
- [Mockito Documentation](https://javadoc.io/doc/org.mockito/mockito-core/latest/org/mockito/Mockito.html)
- [AssertJ Documentation](https://assertj.github.io/doc/)

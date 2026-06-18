# Spring Boot + Kotlin Patterns

Spring Boot with Kotlin — coroutine controllers, WebFlux integration, Kotlin DSL configuration, MockK testing, functional routing, and idiomatic data access.

---

## Coroutine Controllers

Use `suspend` functions in `@RestController` for non-blocking request handling.

### When to Use

- Reactive/async APIs
- High concurrency
- WebFlux applications

### Example

```kotlin
@RestController
class UserController(private val userService: UserService) {
    @GetMapping("/users/{id}")
    suspend fun getUser(@PathVariable id: Long): User =
        userService.findById(id)

    @PostMapping("/users")
    suspend fun createUser(@Valid @RequestBody request: CreateUserRequest): User =
        userService.create(request)
}
```

### Best Practices

- Use `suspend` for I/O in controllers
- Inject `Dispatchers` for tests
- Handle exceptions with `@ControllerAdvice`

---

## WebFlux + Kotlin Flow

Spring WebFlux with Kotlin `Flow` and coroutines for streaming responses.

### When to Use

- Streaming responses
- Server-Sent Events (SSE)
- Reactive backends

### Example

```kotlin
@GetMapping("/users", produces = [MediaType.APPLICATION_NDJSON_VALUE])
fun streamUsers(): Flow<User> = userService.findAllAsFlow()
```

### Best Practices

- Use `Flow` for streaming
- Configure `WebClient` for reactive clients
- Handle backpressure

---

## Kotlin DSL Configuration

Use `beans {}` and `router {}` DSL for Spring configuration.

### When to Use

- Bean registration
- Routing configuration
- Functional config

### Example

```kotlin
@Configuration
class AppConfig {
    @Bean
    fun userService(): UserService = UserServiceImpl()
}
```

### Best Practices

- Use DSL for router config
- Keep bean definitions readable
- Leverage type inference

---

## MockK Testing

MockK for mocking in Kotlin tests — works with coroutines.

### When to Use

- Unit tests with mocks
- Coroutine testing

### Example

```kotlin
@Test
fun `getUser returns user`() = runTest {
    coEvery { userRepository.findById(1L) } returns User(1, "Alice")
    val result = userService.getUser(1L)
    assertThat(result.name).isEqualTo("Alice")
}
```

### Best Practices

- Use `coEvery` for suspend functions
- Use `runTest` for coroutine tests
- Use relaxed mocks for optional calls

---

## Functional Routing

`RouterFunction` with Kotlin DSL for route definitions.

### When to Use

- Functional WebFlux
- Clean routing without annotations

### Example

```kotlin
@Configuration
class RouterConfig {
    @Bean
    fun routes(handler: UserHandler) = router {
        GET("/users", handler::list)
        GET("/users/{id}", handler::get)
        POST("/users", handler::create)
    }
}
```

### Best Practices

- Group routes by feature
- Use `handler::reference` for methods

---

## Nullable Repositories

Spring Data JPA with Kotlin nullable return types.

### When to Use

- Optional finders
- Kotlin nullable semantics

### Example

```kotlin
interface UserRepository : JpaRepository<User, Long> {
    fun findByName(name: String): User?
}
```

### Best Practices

- Use `?` for optional returns
- Prefer `findByIdOrNull()` over `getById()` when possible

---

## DSL Builders

Kotlin DSL for Gradle, Spring Security, and custom config.

### When to Use

- Gradle build scripts (`build.gradle.kts`)
- Security config
- Declarative setup

### Example

```kotlin
// Spring Security DSL
http {
    authorizeHttpRequests {
        authorize("/api/public/**", permitAll)
        authorize(anyRequest, authenticated)
    }
    csrf { disable() }
    sessionManagement {
        sessionCreationPolicy = SessionCreationPolicy.STATELESS
    }
}
```

### Best Practices

- Use `@DslMarker` for nested DSLs
- Provide sensible defaults

---

## General Best Practices

- Use `suspend` for I/O in controllers
- Inject `Dispatchers` for tests
- Handle exceptions with `@ControllerAdvice`
- Use `Flow` for streaming
- Configure `WebClient` for reactive clients
- Handle backpressure
- Use DSL for router config
- Keep bean definitions readable

## Anti-Patterns

- Applying patterns without understanding the underlying concepts
- Copy-pasting solutions without adapting to specific context
- Over-engineering simple problems with complex patterns

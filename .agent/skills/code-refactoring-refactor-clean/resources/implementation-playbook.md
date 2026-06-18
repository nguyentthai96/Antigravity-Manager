# Refactor and Clean Code Implementation Playbook

All examples use **Java / Kotlin + Spring Boot 4.x** stack.

## Instructions

### 1. Code Analysis

Analyze current code for:

- **Code Smells**
  - Long methods (>20 lines)
  - Large classes (>200 lines)
  - Duplicate code blocks
  - Dead code and unused variables
  - Complex conditionals and nested loops
  - Magic numbers and hardcoded values
  - Poor naming conventions
  - Tight coupling between components
  - Missing abstractions

- **SOLID Violations**
  - Single Responsibility Principle violations
  - Open/Closed Principle issues
  - Liskov Substitution problems
  - Interface Segregation concerns
  - Dependency Inversion violations

- **Performance Issues**
  - Inefficient algorithms (O(n²) or worse)
  - Unnecessary object creation
  - Memory leaks potential
  - Blocking operations
  - Missing caching opportunities

### 2. Refactoring Strategy

**Immediate Fixes (High Impact, Low Effort)**
- Extract magic numbers to constants
- Improve variable and method names
- Remove dead code
- Simplify boolean expressions
- Extract duplicate code to methods

**Method Extraction**

```java
// Before — 120-line monolithic method
public class OrderProcessor {
    public OrderResult processOrder(Order order) {
        // 50 lines of validation
        // 30 lines of calculation
        // 40 lines of notification
    }
}

// After — each method does one thing
public class OrderProcessor {

    private final OrderValidator validator;
    private final PricingCalculator calculator;
    private final NotificationService notifier;

    public OrderProcessor(OrderValidator validator,
                          PricingCalculator calculator,
                          NotificationService notifier) {
        this.validator = validator;
        this.calculator = calculator;
        this.notifier = notifier;
    }

    public OrderResult processOrder(Order order) {
        validator.validate(order);
        BigDecimal total = calculator.calculateTotal(order);
        notifier.sendConfirmation(order, total);
        return new OrderResult(order.getId(), total);
    }
}
```

---

### 3. SOLID Principles in Action

**Single Responsibility Principle (SRP)**

```java
// BEFORE: Multiple responsibilities in one class
@Service
public class UserManager {
    public User createUser(CreateUserRequest data) {
        // Validate data
        // Save to database
        // Send welcome email
        // Log activity
        // Update cache
    }
}

// AFTER: Each class has one responsibility
public class UserValidator {
    public void validate(CreateUserRequest data) {
        if (data.email() == null || data.email().isBlank()) {
            throw new ValidationException("email", "Email is required");
        }
    }
}

public interface UserRepository extends JpaRepository<User, Long> {
    boolean existsByEmail(String email);
}

@Service
public class EmailService {
    public void sendWelcomeEmail(User user) { /* ... */ }
}

@Service
public class UserService {

    private final UserValidator validator;
    private final UserRepository repository;
    private final EmailService emailService;

    // Constructor injection — mandatory, no @Autowired
    public UserService(UserValidator validator,
                       UserRepository repository,
                       EmailService emailService) {
        this.validator = validator;
        this.repository = repository;
        this.emailService = emailService;
    }

    @Transactional
    public User createUser(CreateUserRequest data) {
        validator.validate(data);
        User user = new User(data.name(), data.email());
        User saved = repository.save(user);
        emailService.sendWelcomeEmail(saved);
        return saved;
    }
}
```

**Open/Closed Principle (OCP)**

```java
// BEFORE: Modification required for new discount types
@Service
public class DiscountCalculator {
    public BigDecimal calculate(Order order, String discountType) {
        if ("percentage".equals(discountType)) {
            return order.getTotal().multiply(BigDecimal.valueOf(0.1));
        } else if ("fixed".equals(discountType)) {
            return BigDecimal.TEN;
        } else if ("tiered".equals(discountType)) {
            // more logic...
        }
        return BigDecimal.ZERO;
    }
}

// AFTER: Open for extension, closed for modification
public interface DiscountStrategy {
    BigDecimal calculate(Order order);
}

@Component("percentageDiscount")
public class PercentageDiscount implements DiscountStrategy {

    private final BigDecimal percentage;

    public PercentageDiscount(@Value("${discount.percentage:0.10}") BigDecimal percentage) {
        this.percentage = percentage;
    }

    @Override
    public BigDecimal calculate(Order order) {
        return order.getTotal().multiply(percentage);
    }
}

@Component("fixedDiscount")
public class FixedDiscount implements DiscountStrategy {

    @Override
    public BigDecimal calculate(Order order) {
        return BigDecimal.TEN;
    }
}

@Component("tieredDiscount")
public class TieredDiscount implements DiscountStrategy {

    @Override
    public BigDecimal calculate(Order order) {
        BigDecimal total = order.getTotal();
        if (total.compareTo(BigDecimal.valueOf(1000)) > 0) {
            return total.multiply(BigDecimal.valueOf(0.15));
        }
        if (total.compareTo(BigDecimal.valueOf(500)) > 0) {
            return total.multiply(BigDecimal.valueOf(0.10));
        }
        return total.multiply(BigDecimal.valueOf(0.05));
    }
}

// Spring auto-injects all implementations via Map
@Service
public class DiscountService {

    private final Map<String, DiscountStrategy> strategies;

    public DiscountService(Map<String, DiscountStrategy> strategies) {
        this.strategies = strategies;
    }

    public BigDecimal calculate(Order order, String strategyName) {
        DiscountStrategy strategy = strategies.get(strategyName);
        if (strategy == null) {
            throw new IllegalArgumentException("Unknown discount: " + strategyName);
        }
        return strategy.calculate(order);
    }
}
```

**Liskov Substitution Principle (LSP)**

```java
// BEFORE: Violates LSP — Square changes Rectangle behavior
public class Rectangle {
    protected int width;
    protected int height;

    public void setWidth(int width) { this.width = width; }
    public void setHeight(int height) { this.height = height; }
    public int area() { return width * height; }
}

public class Square extends Rectangle {
    @Override
    public void setWidth(int width) {
        this.width = width;
        this.height = width; // Breaks LSP
    }
}

// AFTER: Proper abstraction respects LSP
public sealed interface Shape permits Rectangle, Square, Circle {
    int area();
}

public record Rectangle(int width, int height) implements Shape {
    @Override
    public int area() { return width * height; }
}

public record Square(int side) implements Shape {
    @Override
    public int area() { return side * side; }
}

public record Circle(int radius) implements Shape {
    @Override
    public int area() { return (int) (Math.PI * radius * radius); }
}
```

**Interface Segregation Principle (ISP)**

```java
// BEFORE: Fat interface forces unnecessary implementations
public interface Worker {
    void work();
    void eat();
    void sleep();
}

public class Robot implements Worker {
    public void work() { /* ok */ }
    public void eat() { /* robots don't eat! */ }
    public void sleep() { /* robots don't sleep! */ }
}

// AFTER: Segregated interfaces
public interface Workable {
    void work();
}

public interface Feedable {
    void eat();
}

public interface Restable {
    void sleep();
}

public class HumanWorker implements Workable, Feedable, Restable {
    public void work() { /* work */ }
    public void eat() { /* eat */ }
    public void sleep() { /* sleep */ }
}

public class RobotWorker implements Workable {
    public void work() { /* work */ }
}
```

**Dependency Inversion Principle (DIP)**

```java
// BEFORE: High-level module depends on low-level module
@Service
public class UserService {
    private final MySQLUserRepository db = new MySQLUserRepository(); // Tight coupling

    public void createUser(String name) {
        db.save(name);
    }
}

// AFTER: Both depend on abstraction (Spring DI)
public interface UserRepository extends JpaRepository<User, Long> {
    // Spring Data provides implementation automatically
}

@Service
public class UserService {

    private final UserRepository repository; // Depends on abstraction

    public UserService(UserRepository repository) {
        this.repository = repository;
    }

    @Transactional
    public User createUser(CreateUserRequest request) {
        User user = new User(request.name(), request.email());
        return repository.save(user);
    }
}
```

---

### 4. Code Smell Resolution Catalog

```java
// SMELL: Long Parameter List
// BEFORE
public User createUser(String firstName, String lastName, String email,
                        String phone, String street, String city,
                        String state, String zipCode) { /* ... */ }

// AFTER: Parameter Object (Java Record)
public record CreateUserRequest(
        @NotBlank String firstName,
        @NotBlank String lastName,
        @Email String email,
        String phone,
        @Valid Address address
) {}

public record Address(
        @NotBlank String street,
        @NotBlank String city,
        @NotBlank String state,
        @Pattern(regexp = "\\d{5}") String zipCode
) {}

public User createUser(@Valid CreateUserRequest request) { /* ... */ }

// SMELL: Feature Envy
// BEFORE — Order uses Customer's data more than its own
public class Order {
    public BigDecimal calculateShipping(Customer customer) {
        if (customer.isPremium()) {
            return customer.getAddress().isInternational()
                    ? BigDecimal.ZERO : BigDecimal.valueOf(5);
        }
        return customer.getAddress().isInternational()
                ? BigDecimal.valueOf(20) : BigDecimal.valueOf(10);
    }
}

// AFTER — Move method to the class it envies
public class Customer {
    public BigDecimal calculateShippingCost() {
        if (this.premium) {
            return this.address.isInternational()
                    ? BigDecimal.ZERO : BigDecimal.valueOf(5);
        }
        return this.address.isInternational()
                ? BigDecimal.valueOf(20) : BigDecimal.valueOf(10);
    }
}

public class Order {
    public BigDecimal calculateShipping(Customer customer) {
        return customer.calculateShippingCost();
    }
}

// SMELL: Primitive Obsession
// BEFORE
public boolean validateEmail(String email) {
    return email.matches("^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$");
}

// AFTER: Value Object
public record Email(String value) {
    public Email {
        if (value == null || !value.matches("^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$")) {
            throw new IllegalArgumentException("Invalid email: " + value);
        }
    }

    @Override
    public String toString() { return value; }
}
```

---

### 5. Decision Frameworks

**Code Quality Metrics**

| Metric | Good | Warning | Critical | Action |
|--------|------|---------|----------|--------|
| Cyclomatic Complexity | <10 | 10-15 | >15 | Split into smaller methods |
| Method Lines | <20 | 20-50 | >50 | Extract methods, apply SRP |
| Class Lines | <200 | 200-500 | >500 | Decompose into multiple classes |
| Test Coverage | >80% | 60-80% | <60% | Add JUnit 5 tests immediately |
| Code Duplication | <3% | 3-5% | >5% | Extract common code |
| Dependency Count | <5 | 5-10 | >10 | Apply DIP, use facades |

---

### 6. Static Analysis Toolchain (Java / Spring Boot)

**Checkstyle Configuration**

```xml
<!-- checkstyle.xml -->
<module name="Checker">
    <module name="TreeWalker">
        <module name="MethodLength"><property name="max" value="20"/></module>
        <module name="ParameterNumber"><property name="max" value="4"/></module>
        <module name="CyclomaticComplexity"><property name="max" value="10"/></module>
        <module name="NeedBraces"/>
        <module name="AvoidStarImport"/>
        <module name="UnusedImports"/>
    </module>
</module>
```

**SpotBugs + SonarQube (build.gradle.kts)**

```kotlin
plugins {
    id("com.github.spotbugs") version "6.x"
    id("org.sonarqube") version "5.x"
}

spotbugs {
    effort = com.github.spotbugs.snom.Effort.MAX
    reportLevel = com.github.spotbugs.snom.Confidence.MEDIUM
}

sonarqube {
    properties {
        property("sonar.projectKey", "base-core")
        property("sonar.coverage.jacoco.xmlReportPaths", "build/reports/jacoco/test/jacocoTestReport.xml")
        property("sonar.coverage.exclusions", "**/config/**,**/dto/**")
    }
}
```

**ArchUnit (Architecture Tests)**

```java
@AnalyzeClasses(packages = "com.example.app")
class ArchitectureTest {

    @ArchTest
    static final ArchRule noFieldInjection = noClasses()
            .should().beAnnotatedWith(Autowired.class);

    @ArchTest
    static final ArchRule servicesShouldNotDependOnControllers = noClasses()
            .that().resideInAPackage("..service..")
            .should().dependOnClassesThat().resideInAPackage("..controller..");

    @ArchTest
    static final ArchRule repositoriesShouldOnlyBeAccessedByServices = classes()
            .that().resideInAPackage("..repository..")
            .should().onlyBeAccessed().byClassesThat()
            .resideInAnyPackage("..service..", "..repository..");
}
```

---

### 7. Error Handling

```java
// Use specific exceptions
public class OrderValidationException extends RuntimeException {
    public OrderValidationException(String message) { super(message); }
}

public class InsufficientInventoryException extends RuntimeException {
    private final Long productId;
    private final int requested;
    private final int available;

    public InsufficientInventoryException(Long productId, int requested, int available) {
        super("Insufficient inventory for product %d: requested %d, available %d"
                .formatted(productId, requested, available));
        this.productId = productId;
        this.requested = requested;
        this.available = available;
    }
}

// Fail fast with clear messages
public class OrderValidator {
    public void validate(Order order) {
        if (order.getItems().isEmpty()) {
            throw new OrderValidationException("Order must contain at least one item");
        }
        for (OrderItem item : order.getItems()) {
            if (item.getQuantity() <= 0) {
                throw new OrderValidationException(
                        "Invalid quantity for " + item.getName());
            }
        }
    }
}
```

---

### 8. Testing Strategy (JUnit 5 + Mockito)

```java
@ExtendWith(MockitoExtension.class)
class OrderServiceTest {

    @Mock
    private OrderRepository orderRepository;

    @Mock
    private OrderValidator orderValidator;

    @InjectMocks
    private OrderService orderService;

    @Test
    void shouldRejectEmptyOrder() {
        Order emptyOrder = new Order(List.of());
        doThrow(new OrderValidationException("Order must contain items"))
                .when(orderValidator).validate(emptyOrder);

        assertThatThrownBy(() -> orderService.processOrder(emptyOrder))
                .isInstanceOf(OrderValidationException.class)
                .hasMessageContaining("must contain items");
    }

    @Test
    void shouldCalculateVipDiscount() {
        Order order = OrderFixtures.createWithTotal(new BigDecimal("1000"));
        Customer customer = CustomerFixtures.vip();

        BigDecimal discount = discountService.calculate(order, "tieredDiscount");

        assertThat(discount).isEqualByComparingTo(new BigDecimal("150.00"));
    }
}

// Integration test (Spring Boot 4.x)
@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
class OrderControllerIT {

    @Autowired
    private TestRestTemplate restTemplate;

    @MockitoBean // Spring Boot 4.x — NOT @MockBean
    private OrderService orderService;

    @Test
    void shouldReturn201WhenOrderCreated() {
        var request = new CreateOrderRequest(/* ... */);
        when(orderService.create(any())).thenReturn(OrderFixtures.sample());

        var response = restTemplate.postForEntity("/api/orders", request, OrderResponse.class);

        assertThat(response.getStatusCode()).isEqualTo(HttpStatus.CREATED);
        assertThat(response.getHeaders().getLocation()).isNotNull();
    }
}
```

---

### 9. Before/After Comparison

```
Before:
- processData(): 150 lines, complexity: 25
- 0% test coverage
- 3 responsibilities mixed
- Field injection @Autowired

After:
- validateInput():   20 lines, complexity: 4
- transformData():   25 lines, complexity: 5
- persistResults():  15 lines, complexity: 3
- 95% test coverage
- Clear separation of concerns
- Constructor injection only
```

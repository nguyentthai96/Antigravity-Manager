# Architecture Patterns Implementation Playbook

This file contains detailed patterns, checklists, and code samples referenced by the skill.

## Core Concepts

### 1. Clean Architecture (Uncle Bob)

**Layers (dependency flows inward):**

- **Entities**: Core business models
- **Use Cases**: Application business rules
- **Interface Adapters**: Controllers, presenters, gateways
- **Frameworks & Drivers**: UI, database, external services

**Key Principles:**

- Dependencies point inward
- Inner layers know nothing about outer layers
- Business logic independent of frameworks
- Testable without UI, database, or external services

### 2. Hexagonal Architecture (Ports and Adapters)

**Components:**

- **Domain Core**: Business logic
- **Ports**: Interfaces defining interactions
- **Adapters**: Implementations of ports (database, REST, message queue)

**Benefits:**

- Swap implementations easily (mock for testing)
- Technology-agnostic core
- Clear separation of concerns

### 3. Domain-Driven Design (DDD)

**Strategic Patterns:**

- **Bounded Contexts**: Separate models for different domains
- **Context Mapping**: How contexts relate
- **Ubiquitous Language**: Shared terminology

**Tactical Patterns:**

- **Entities**: Objects with identity
- **Value Objects**: Immutable objects defined by attributes
- **Aggregates**: Consistency boundaries
- **Repositories**: Data access abstraction
- **Domain Events**: Things that happened

## Clean Architecture Pattern

### Directory Structure (Spring Boot)

```
src/main/java/com/example/app/
├── domain/                  # Entities & business rules
│   ├── model/
│   │   ├── User.java
│   │   └── Order.java
│   ├── valueobject/
│   │   ├── Email.java
│   │   └── Money.java
│   └── port/                # Abstract interfaces (Ports)
│       ├── UserRepository.java
│       └── PaymentGateway.java
├── application/             # Use cases / Application services
│   ├── usecase/
│   │   ├── CreateUserUseCase.java
│   │   └── ProcessOrderUseCase.java
│   └── dto/
│       ├── CreateUserRequest.java
│       └── UserResponse.java
├── adapter/                 # Interface implementations
│   ├── persistence/
│   │   └── JpaUserRepository.java
│   ├── web/
│   │   └── UserController.java
│   └── gateway/
│       └── StripePaymentGateway.java
└── infrastructure/          # Framework & external concerns
    ├── config/
    └── security/
```

### Implementation Example

```java
// --- domain/model/User.java ---
// Core entity — no framework dependencies

public class User {

    private Long id;
    private String email;
    private String name;
    private Instant createdAt;
    private boolean active = true;

    public User(String email, String name) {
        this.email = email;
        this.name = name;
        this.createdAt = Instant.now();
    }

    /** Business rule: deactivating user. */
    public void deactivate() {
        this.active = false;
    }

    /** Business rule: active users can place orders. */
    public boolean canPlaceOrder() {
        return this.active;
    }

    // Getters and setters...
}

// --- domain/port/UserRepository.java ---
// Port: defines contract, no implementation

public interface UserRepository {
    Optional<User> findById(Long id);
    Optional<User> findByEmail(String email);
    User save(User user);
    void deleteById(Long id);
    boolean existsByEmail(String email);
}

// --- application/dto/CreateUserRequest.java ---

public record CreateUserRequest(
        @NotBlank @Email String email,
        @NotBlank String name
) {}

public record UserResponse(
        Long id, String email, String name,
        boolean active, Instant createdAt
) {}

// --- application/usecase/CreateUserUseCase.java ---
// Use case: orchestrates business logic

@Service
public class CreateUserUseCase {

    private final UserRepository userRepository;

    public CreateUserUseCase(UserRepository userRepository) {
        this.userRepository = userRepository;
    }

    @Transactional
    public UserResponse execute(CreateUserRequest request) {
        // Business validation
        if (userRepository.existsByEmail(request.email())) {
            throw new DuplicateResourceException(
                    "Email already exists: " + request.email());
        }

        // Create entity
        User user = new User(request.email(), request.name());

        // Persist
        User saved = userRepository.save(user);

        return new UserResponse(
                saved.getId(), saved.getEmail(), saved.getName(),
                saved.isActive(), saved.getCreatedAt());
    }
}

// --- adapter/persistence/JpaUserRepository.java ---
// Adapter: Spring Data JPA implementation

public interface JpaUserRepository
        extends JpaRepository<User, Long>, UserRepository {
    // Spring Data auto-implements methods
}

// --- adapter/web/UserController.java ---
// Controller: handles HTTP concerns only

@RestController
@RequestMapping("/api/users")
public class UserController {

    private final CreateUserUseCase createUserUseCase;

    public UserController(CreateUserUseCase createUserUseCase) {
        this.createUserUseCase = createUserUseCase;
    }

    @PostMapping
    public ResponseEntity<UserResponse> createUser(
            @Valid @RequestBody CreateUserRequest request) {
        UserResponse response = createUserUseCase.execute(request);
        URI location = ServletUriComponentsBuilder.fromCurrentRequest()
                .path("/{id}").buildAndExpand(response.id()).toUri();
        return ResponseEntity.created(location).body(response);
    }
}
```

## Hexagonal Architecture Pattern

```java
// --- Core domain (hexagon center) ---

@Service
public class OrderService {
    // Domain service — no infrastructure dependencies

    private final OrderRepositoryPort orderRepository;
    private final PaymentGatewayPort paymentGateway;
    private final NotificationPort notificationService;

    public OrderService(OrderRepositoryPort orderRepository,
                        PaymentGatewayPort paymentGateway,
                        NotificationPort notificationService) {
        this.orderRepository = orderRepository;
        this.paymentGateway = paymentGateway;
        this.notificationService = notificationService;
    }

    @Transactional
    public OrderResult placeOrder(Order order) {
        // Business logic
        if (!order.isValid()) {
            return OrderResult.failure("Invalid order");
        }

        // Use ports (interfaces)
        PaymentResult payment = paymentGateway.charge(
                order.getTotal(), order.getCustomerId());

        if (!payment.isSuccess()) {
            return OrderResult.failure("Payment failed");
        }

        order.markAsPaid();
        Order saved = orderRepository.save(order);

        notificationService.send(
                order.getCustomerEmail(),
                "Order confirmed",
                "Order " + order.getId() + " confirmed");

        return OrderResult.success(saved);
    }
}

// --- Ports (interfaces) ---

public interface OrderRepositoryPort {
    Order save(Order order);
    Optional<Order> findById(Long id);
}

public interface PaymentGatewayPort {
    PaymentResult charge(Money amount, String customerId);
}

public interface NotificationPort {
    void send(String to, String subject, String body);
}

// --- Adapters (implementations) ---

@Component
public class StripePaymentAdapter implements PaymentGatewayPort {
    // Primary adapter: connects to Stripe API

    private final StripeClient stripeClient;

    public StripePaymentAdapter(StripeClient stripeClient) {
        this.stripeClient = stripeClient;
    }

    @Override
    public PaymentResult charge(Money amount, String customerId) {
        try {
            var charge = stripeClient.charges().create(
                    ChargeCreateParams.builder()
                            .setAmount(amount.cents())
                            .setCurrency(amount.currency())
                            .setCustomer(customerId)
                            .build());
            return PaymentResult.success(charge.getId());
        } catch (StripeException e) {
            return PaymentResult.failure(e.getMessage());
        }
    }
}

// Test adapter: no external dependencies
public class MockPaymentAdapter implements PaymentGatewayPort {
    @Override
    public PaymentResult charge(Money amount, String customerId) {
        return PaymentResult.success("mock-123");
    }
}
```

## Domain-Driven Design Pattern

```java
// --- Value Objects (immutable — Java Records) ---

public record Email(String value) {
    public Email {
        if (value == null || !value.contains("@")) {
            throw new IllegalArgumentException("Invalid email: " + value);
        }
    }
}

public record Money(long cents, String currency) {
    public Money add(Money other) {
        if (!this.currency.equals(other.currency)) {
            throw new IllegalArgumentException("Currency mismatch");
        }
        return new Money(this.cents + other.cents, this.currency);
    }
}

// --- Entities (with identity) ---

public class Order {
    // Entity: has identity, mutable state

    private Long id;
    private Customer customer;
    private List<OrderItem> items = new ArrayList<>();
    private OrderStatus status = OrderStatus.PENDING;
    private final List<DomainEvent> events = new ArrayList<>();

    public Order(Long id, Customer customer) {
        this.id = id;
        this.customer = customer;
    }

    /** Business logic in entity. */
    public void addItem(Product product, int quantity) {
        OrderItem item = new OrderItem(product, quantity);
        this.items.add(item);
        this.events.add(new ItemAddedEvent(this.id, item));
    }

    /** Calculated property. */
    public Money total() {
        return items.stream()
                .map(OrderItem::subtotal)
                .reduce(new Money(0, "USD"), Money::add);
    }

    /** State transition with business rules. */
    public void submit() {
        if (items.isEmpty()) {
            throw new IllegalStateException("Cannot submit empty order");
        }
        if (status != OrderStatus.PENDING) {
            throw new IllegalStateException("Order already submitted");
        }
        this.status = OrderStatus.SUBMITTED;
        this.events.add(new OrderSubmittedEvent(this.id));
    }

    // Getters...
}

// --- Aggregates (consistency boundary) ---

public class Customer {
    // Aggregate root: controls access to entities

    private Long id;
    private Email email;
    private final List<Address> addresses = new ArrayList<>();
    private final List<Long> orderIds = new ArrayList<>(); // IDs only

    /** Aggregate enforces invariants. */
    public void addAddress(Address address) {
        if (addresses.size() >= 5) {
            throw new IllegalStateException("Maximum 5 addresses allowed");
        }
        addresses.add(address);
    }

    public Optional<Address> primaryAddress() {
        return addresses.stream()
                .filter(Address::isPrimary)
                .findFirst();
    }
}

// --- Domain Events ---

public record OrderSubmittedEvent(
        Long orderId,
        Instant occurredAt
) implements DomainEvent {
    public OrderSubmittedEvent(Long orderId) {
        this(orderId, Instant.now());
    }
}

// --- Repository (aggregate persistence) ---

public interface OrderRepository {
    Optional<Order> findById(Long orderId);
    Order save(Order order);
}
```

## Resources

- **references/clean-architecture-guide.md**: Detailed layer breakdown
- **references/hexagonal-architecture-guide.md**: Ports and adapters patterns
- **references/ddd-tactical-patterns.md**: Entities, value objects, aggregates
- **assets/clean-architecture-template/**: Complete project structure
- **assets/ddd-examples/**: Domain modeling examples

## Best Practices

1. **Dependency Rule**: Dependencies always point inward
2. **Interface Segregation**: Small, focused interfaces
3. **Business Logic in Domain**: Keep frameworks out of core
4. **Test Independence**: Core testable without infrastructure
5. **Bounded Contexts**: Clear domain boundaries
6. **Ubiquitous Language**: Consistent terminology
7. **Thin Controllers**: Delegate to use cases
8. **Rich Domain Models**: Behavior with data

## Common Pitfalls

- **Anemic Domain**: Entities with only data, no behavior
- **Framework Coupling**: Business logic depends on frameworks
- **Fat Controllers**: Business logic in controllers
- **Repository Leakage**: Exposing ORM objects
- **Missing Abstractions**: Concrete dependencies in core
- **Over-Engineering**: Clean architecture for simple CRUD

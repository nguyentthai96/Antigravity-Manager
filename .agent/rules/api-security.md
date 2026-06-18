---
trigger: model_decision
description: "API security rules covering OAuth2, JWT token management, refresh/revoke strategies, and E2EE best practices for production systems."
---

# API Security Rules

> Security rules for API Gateway, OAuth2/JWT token management, and encrypted communication in base-core.

## Principles

- Defense in depth — multiple security layers
- Least privilege — minimal required permissions
- Zero trust — verify every request
- Fail secure — deny by default on errors
- Short-lived credentials — minimize token lifetime

## Rules

### OAuth2 & JWT

- Use **Authorization Code + PKCE** for user-facing clients
- Use **Client Credentials** for service-to-service communication
- JWT must include mandatory claims: `iss`, `aud`, `exp`, `nbf`, `jti`, `scope`
- Access token TTL: **5–15 minutes** (short-lived, stateless validation)
- Refresh token TTL: **longer** with rotation strategy
- Bind refresh tokens to device / client / IP when possible

### Token Refresh & Session

- Implement **refresh token rotation** (new refresh token on each use)
- Detect and reject **reuse of rotated tokens** (compromised token detection)
- Choose between sliding session vs absolute expiry based on use case
- Clearly separate access token renewal from session prolongation

### Token Revocation

- Implement `jti`-based token blacklist for immediate revocation
- Support versioned tokens (`token_version`) for bulk invalidation
- Implement back-channel logout for SSO scenarios
- Revoke tokens on: logout, password change, compromised device detection
- Trade-off: immediate revoke (Redis lookup) vs eventual consistency (short TTL)

### Encryption & Communication

- Enforce **mTLS** between Gateway ↔ Microservices
- Implement certificate rotation with automated renewal
- Use **AES-GCM** for application-layer payload encryption
- Use **RSA / ECDH** for key exchange
- Manage keys via KMS / HSM with envelope encryption
- Distinguish between transport encryption (TLS) and true E2EE

### API Gateway (Spring Cloud Gateway)

- Validate JWT at the gateway level (stateless)
- Enforce scope and policy rules at gateway filters
- Apply rate limiting and abuse prevention at the edge
- Log and trace all token usage for audit

### Resource Server Security

- Use method-level security (`@PreAuthorize`)
- Implement claim-based authorization
- Configure CORS and CSRF appropriately
- Set security headers (CSP, HSTS, X-Frame-Options)

### Threat Mitigation (OWASP / STRIDE)

| Threat | Mitigation |
|--------|------------|
| Token theft | Short TTL + refresh rotation + device binding |
| Replay attack | `jti` claim + nonce + timestamp validation |
| CSRF | CSRF tokens + SameSite cookies |
| XSS | CSP headers + input sanitization |
| Man-in-the-middle | mTLS + certificate pinning |
| Privilege escalation | Scope-based validation + least privilege |

### Observability

- Audit log all authentication/authorization events
- Trace token usage across services
- Implement anomaly detection on login patterns
- Alert on unusual token refresh/revoke patterns

## Anti-Patterns

- ❌ Long-lived access tokens (> 30 minutes)
- ❌ Storing JWT in localStorage (XSS vulnerable)
- ❌ Refresh tokens without rotation
- ❌ Skipping `jti` validation on token reuse
- ❌ Hardcoded secrets or keys in source code
- ❌ Disabling CSRF protection without justification
- ❌ Trust-on-first-use without verification

## References

- Security architecture: [agents/security-engineer.md](../agents/security-engineer.md)
- Architecture review: [skills/architect-review/](../skills/architect-review/)
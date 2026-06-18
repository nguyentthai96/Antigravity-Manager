---
name: senior-architect
description: "Complete toolkit for senior architect with modern tools and best practices."
risk: critical
source: community
date_added: "2026-02-27"
---

# Senior Architect

Complete toolkit for senior architect with modern tools and best practices.

## Quick Start

### Main Capabilities

This skill provides three core capabilities through analysis workflows:

```bash
# 1: Architecture Diagram Generator (Mermaid / PlantUML)
# Generate from codebase using ArchUnit or jqAssistant
./gradlew archUnit --scan

# 2: Project Structure Analysis
./gradlew dependencies --configuration runtimeClasspath

# 3: Dependency Analyzer
./gradlew dependencyInsight --dependency spring-boot
```

## Core Capabilities

### 1. Architecture Diagram Generator

Automated tool for architecture diagram generator tasks.

**Features:**
- Automated scaffolding
- Best practices built-in
- Configurable templates
- Quality checks

**Usage:**
```bash
./gradlew archUnit --scan
```

### 2. Project Architect

Comprehensive analysis and optimization tool.

**Features:**
- Deep analysis
- Performance metrics
- Recommendations
- Automated fixes

**Usage:**
```bash
./gradlew check --info
```

### 3. Dependency Analyzer

Advanced tooling for specialized tasks.

**Features:**
- Expert-level automation
- Custom configurations
- Integration ready
- Production-grade output

**Usage:**
```bash
./gradlew dependencies --configuration runtimeClasspath
```

## Reference Documentation

### Architecture Patterns

Comprehensive guide available in `references/architecture_patterns.md`:

- Detailed patterns and practices
- Code examples
- Best practices
- Anti-patterns to avoid
- Real-world scenarios

### System Design Workflows

Complete workflow documentation in `references/system_design_workflows.md`:

- Step-by-step processes
- Optimization strategies
- Tool integrations
- Performance tuning
- Troubleshooting guide

### Tech Decision Guide

Technical reference guide in `references/tech_decision_guide.md`:

- Technology stack details
- Configuration examples
- Integration patterns
- Security considerations
- Scalability guidelines

## Tech Stack

**Languages:** Java 25+, Kotlin 2.2+
**Framework:** Spring Boot 4.x, Spring Framework 7.0, Jakarta EE 11
**Data:** Spring Data JPA, Hibernate, PostgreSQL, Redis
**Build:** Gradle 9, Maven
**DevOps:** Docker, Kubernetes, Terraform, GitHub Actions
**Cloud:** AWS, GCP, Azure

## Development Workflow

### 1. Setup and Configuration

```bash
# Build project
./gradlew build

# Configure environment
cp src/main/resources/application-dev.yml src/main/resources/application-local.yml
```

### 2. Run Quality Checks

```bash
# Run all quality checks
./gradlew check spotbugsMain checkstyleMain

# Review recommendations and apply fixes
```

### 3. Implement Best Practices

Follow the patterns and practices documented in:
- `references/architecture_patterns.md`
- `references/system_design_workflows.md`
- `references/tech_decision_guide.md`

## Best Practices Summary

### Code Quality
- Follow established patterns
- Write comprehensive tests
- Document decisions
- Review regularly

### Performance
- Measure before optimizing
- Use appropriate caching
- Optimize critical paths
- Monitor in production

### Security
- Validate all inputs
- Use parameterized queries
- Implement proper authentication
- Keep dependencies updated

### Maintainability
- Write clear code
- Use consistent naming
- Add helpful comments
- Keep it simple

## Common Commands

```bash
# Development
./gradlew bootRun
./gradlew build
./gradlew test
./gradlew check

# Analysis
./gradlew dependencies
./gradlew spotbugsMain checkstyleMain

# Deployment
./gradlew bootBuildImage
docker compose up -d
kubectl apply -f k8s/
```

## Troubleshooting

### Common Issues

Check the comprehensive troubleshooting section in `references/tech_decision_guide.md`.

### Getting Help

- Review reference documentation
- Check script output messages
- Consult tech stack documentation
- Review error logs

## Resources

- Pattern Reference: `references/architecture_patterns.md`
- Workflow Guide: `references/system_design_workflows.md`
- Technical Guide: `references/tech_decision_guide.md`
- Tool Scripts: `scripts/` directory

## When to Use
This skill is applicable to execute the workflow or actions described in the overview.

## Limitations
- Use this skill only when the task clearly matches the scope described above.
- Do not treat the output as a substitute for environment-specific validation, testing, or expert review.
- Stop and ask for clarification if required inputs, permissions, safety boundaries, or success criteria are missing.

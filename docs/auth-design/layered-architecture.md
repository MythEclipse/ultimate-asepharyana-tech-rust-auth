# Layered Architecture (Expanded)
- Edge / API Gateway: SSL termination, global rate limiting, WAF.
- Middleware Pipeline: filter requests before they reach controllers.
- Controller: endpoint handlers, input schema validation, service orchestration.
- Service (Use Case / Domain): complex business logic, transactions, cache management.
- Cache Layer: fast access to tokens, sessions, computed permissions.
- Repository: relational database abstraction (PostgreSQL/MySQL) with connection pooling.
- Event Bus / Message Broker: asynchronous event publishing.
- View / Presenter: response formatting (DTO) and standard error handling (RFC 7807).

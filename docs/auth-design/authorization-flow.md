# Complex Authorization Flow (Sub-millisecond SLA)
1. Request enters through TracingMiddleware then RateLimitMiddleware.
2. AuthMiddleware validates JWT using cached JWKS.
3. AuthMiddleware checks SessionCache to ensure the session is not revoked.
4. Request is forwarded to the target controller with user context.
5. Controller calls AuthorizationService.
6. AuthorizationService checks PermissionCache.
7. If hit, use the computed permissions.
8. If miss, load from repository (role, override, policy), compute, then store in cache.
9. If allowed, the controller executes the main action.
10. AuditService sends AuditLogCreated to the broker in a non-blocking way.
11. Async workers consume events and persist to AuditLogRepository.

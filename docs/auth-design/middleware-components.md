# Middleware Components
- TracingMiddleware: generates and injects a correlation ID for each request.
- RateLimitMiddleware: limits requests per IP or user ID (Redis token bucket).
- AuthMiddleware: validates JWT/opaque tokens, extracts identity, populates request context.
- DeviceFingerprintMiddleware: validates device headers to detect session hijacking.

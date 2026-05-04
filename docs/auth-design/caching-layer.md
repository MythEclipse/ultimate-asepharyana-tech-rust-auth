# Caching Layer (Redis)
- SessionCache: active tokens and revocation status (revocation list / blacklist).
- PermissionCache: computed effective permissions per user ID, async invalidation on role/permission changes.
- RateLimitCache: counters for access limiting algorithms.

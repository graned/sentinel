/**
 * Express middleware for Sentinel Auth.
 *
 * Validates the Bearer token on every request using Sentinel's `authenticate`
 * and `checkAuthorization` endpoints. When a policy-test token is detected
 * (`scope: "policy_test"`), the middleware short-circuits before the route
 * handler runs and returns a synthetic JSON response — this is how the
 * live probe mode works without executing real business logic.
 *
 * Usage:
 * ```ts
 * import express from 'express';
 * import { SentinelAuthClient } from '@sentinel/auth-sdk';
 * import { sentinelExpressMiddleware } from '@sentinel/auth-sdk/middleware/express';
 *
 * const sentinel = new SentinelAuthClient({ baseUrl: 'https://auth.example.com' });
 * const app = express();
 *
 * app.use(sentinelExpressMiddleware({ client: sentinel }));
 * ```
 */

import type { SentinelAuthClient } from '../client';
import type { AuthContextData } from '../types';

// ---------------------------------------------------------------------------
// Minimal Express interface declarations so the SDK stays framework-agnostic
// without a hard dependency on @types/express.
// ---------------------------------------------------------------------------

interface Request {
  method: string;
  path: string;
  headers: Record<string, string | string[] | undefined>;
  sentinelAuth?: AuthContextData;
}

interface Response {
  status(code: number): Response;
  json(body: unknown): void;
}

type NextFunction = () => void;
type RequestHandler = (req: Request, res: Response, next: NextFunction) => void | Promise<void>;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function extractBearer(req: Request): string | null {
  const header = req.headers['authorization'];
  const raw = Array.isArray(header) ? header[0] : header;
  if (!raw || !raw.startsWith('Bearer ')) return null;
  return raw.slice(7);
}

// ---------------------------------------------------------------------------
// Middleware factory
// ---------------------------------------------------------------------------

export interface SentinelMiddlewareOptions {
  /** Configured `SentinelAuthClient` instance. */
  client: SentinelAuthClient;
  /**
   * Override the policy ID used for authorization checks. Omit to use the
   * server's default active policy. For probe calls the token's embedded
   * `policy_test_id` is always used regardless of this option.
   */
  policyId?: string;
}

/**
 * Returns an Express-compatible middleware that:
 * 1. Extracts the Bearer token from `Authorization`.
 * 2. Calls `sentinel.authenticate()` to validate it.
 * 3. Calls `sentinel.checkAuthorization()` to evaluate the active policy.
 * 4. For policy-test tokens (`scope: "policy_test"`): short-circuits with a
 *    synthetic JSON response so the live probe sees accurate allow/deny results
 *    without executing real route handlers.
 * 5. For normal tokens: attaches `auth` to `req.sentinelAuth` and calls `next()`.
 */
export function sentinelExpressMiddleware(options: SentinelMiddlewareOptions): RequestHandler {
  return async (req: Request, res: Response, next: NextFunction) => {
    const token = extractBearer(req);
    if (!token) {
      next();
      return;
    }

    let auth: AuthContextData;
    try {
      auth = await options.client.authenticate({ access_token: token });
    } catch {
      res.status(401).json({ error: 'UNAUTHORIZED' });
      return;
    }

    // Determine which policy to evaluate against.
    // For probe tokens use the embedded policy ID; otherwise use the configured
    // override or the server's default active policy.
    const policyId =
      auth.scope === 'policy_test' ? auth.policy_test_id : (options.policyId ?? undefined);

    let authz: Awaited<ReturnType<typeof options.client.checkAuthorization>>;
    try {
      authz = await options.client.checkAuthorization({
        method: req.method,
        path: req.path,
        policy_id: policyId,
        roles: auth.roles,
      });
    } catch {
      res.status(403).json({ error: 'FORBIDDEN' });
      return;
    }

    if (!authz.allowed) {
      if (auth.scope === 'policy_test') {
        res.status(403).json({
          sentinel_probe: true,
          allowed: false,
          method: req.method,
          path: req.path,
          roles: auth.roles,
          policy_id: policyId,
          active_version: authz.active_version,
        });
      } else {
        res.status(403).json({ error: 'FORBIDDEN' });
      }
      return;
    }

    // Allowed path.
    if (auth.scope === 'policy_test') {
      // Short-circuit: never run the real handler for probe calls.
      res.status(200).json({
        sentinel_probe: true,
        allowed: true,
        method: req.method,
        path: req.path,
        roles: auth.roles,
        policy_id: policyId,
        active_version: authz.active_version,
      });
      return;
    }

    req.sentinelAuth = auth;
    next();
  };
}

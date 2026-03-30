import { randomUUID } from 'crypto';
import pg from 'pg';

export const API_BASE = process.env['API_BASE_URL'] ?? 'http://localhost:9000';
const DB_URL =
  process.env['DATABASE_URL'] ?? 'postgresql://postgres:password@localhost:5432/sentinel_auth';

export const TEST_PASSWORD = 'T3stP@ssw0rd#Sec';

/**
 * Returns a unique email address for the given prefix.
 * Using a UUID suffix ensures no two test runs collide in the DB.
 */
export function uniqueEmail(prefix = 'sdk-test'): string {
  return `${prefix}-${randomUUID()}@example.com`;
}

/**
 * Derives a unique `10.x.x.x` IP from a fresh UUID so every call lands in
 * its own rate-limit bucket. Pass as `X-Forwarded-For` on requests that hit
 * strict-tier endpoints (login, register).
 */
export function uniqueIp(): string {
  const hex = randomUUID().replace(/-/g, '');
  const a = Number.parseInt(hex.slice(0, 2), 16) % 256;
  const b = Number.parseInt(hex.slice(2, 4), 16) % 256;
  const c = Number.parseInt(hex.slice(4, 6), 16) % 256;
  return `10.${a}.${b}.${c}`;
}

/**
 * Registers a new user via `POST /v1/api/auth/register`.
 * Injects a unique IP so the call never shares a rate-limit bucket.
 */
export async function registerUser(email: string, password = TEST_PASSWORD): Promise<void> {
  const res = await fetch(`${API_BASE}/v1/api/auth/register`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Forwarded-For': uniqueIp(),
    },
    body: JSON.stringify({
      first_name: 'Test',
      last_name: 'User',
      email,
      avatar_url: null,
      password,
    }),
  });

  if (!res.ok) {
    const body = await res.json();
    throw new Error(`Registration failed (HTTP ${res.status}): ${JSON.stringify(body)}`);
  }
}

/**
 * Directly marks a user's email as verified in the DB.
 *
 * Required because:
 * - The `ev` claim is baked into PASETO tokens at login time.
 * - Tokens issued before email verification permanently carry `ev: false`.
 * - Must be called **before** the first login, not after.
 */
export async function markEmailVerified(email: string): Promise<void> {
  const client = new pg.Client(DB_URL);
  await client.connect();
  try {
    await client.query('UPDATE user_identities SET email_verified = true WHERE email = $1', [
      email,
    ]);
  } finally {
    await client.end();
  }
}

/**
 * Convenience wrapper: registers a user then immediately verifies their email
 * so they can log in without going through the email flow.
 */
export async function registerAndVerify(email: string, password = TEST_PASSWORD): Promise<void> {
  await registerUser(email, password);
  await markEmailVerified(email);
}

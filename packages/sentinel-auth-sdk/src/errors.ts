/**
 * Base class for all errors thrown by the Sentinel Auth SDK.
 *
 * Every error carries:
 * - `code`       — machine-readable code matching the server's `ApiErrorBody.code`
 * - `statusCode` — HTTP status (0 for network/local errors)
 * - `requestId`  — the `request_id` from the response envelope, useful for tracing
 * - `details`    — optional extra context from the server
 */
export class SentinelError extends Error {
  readonly code: string;
  readonly statusCode: number;
  readonly requestId: string | undefined;
  readonly details: unknown;

  constructor(
    code: string,
    message: string,
    statusCode: number,
    requestId?: string,
    details?: unknown,
  ) {
    super(message);
    this.name = 'SentinelError';
    this.code = code;
    this.statusCode = statusCode;
    this.requestId = requestId;
    this.details = details;

    // Maintains proper prototype chain in transpiled ES5 output.
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

/** Invalid credentials (wrong email / password). HTTP 401. */
export class AuthenticationError extends SentinelError {
  constructor(message: string, requestId?: string) {
    super('AUTH_ERROR', message, 401, requestId);
    this.name = 'AuthenticationError';
  }
}

/** Request body failed validation (e.g. invalid email format). HTTP 400. */
export class ValidationError extends SentinelError {
  constructor(message: string, requestId?: string, details?: unknown) {
    super('VALIDATION_ERROR', message, 400, requestId, details);
    this.name = 'ValidationError';
  }
}

/** The supplied token is malformed or cannot be decrypted. HTTP 401. */
export class InvalidTokenError extends SentinelError {
  constructor(message: string, requestId?: string) {
    super('INVALID_TOKEN', message, 401, requestId);
    this.name = 'InvalidTokenError';
  }
}

/** The supplied token has passed its TTL. HTTP 401. */
export class ExpiredTokenError extends SentinelError {
  constructor(message: string, requestId?: string) {
    super('EXPIRED_TOKEN', message, 401, requestId);
    this.name = 'ExpiredTokenError';
  }
}

/** No Bearer token was included in the request. HTTP 401. */
export class MissingTokenError extends SentinelError {
  constructor(message: string, requestId?: string) {
    super('MISSING_TOKEN', message, 401, requestId);
    this.name = 'MissingTokenError';
  }
}

/** Too many requests — the IP-level or MFA attempt rate limit was hit. HTTP 429. */
export class RateLimitError extends SentinelError {
  constructor(message: string, requestId?: string) {
    super('RATE_LIMIT_EXCEEDED', message, 429, requestId);
    this.name = 'RateLimitError';
  }
}

/**
 * The user's email address has not been verified yet.
 * They must click the verification link before accessing protected resources.
 * HTTP 403.
 */
export class EmailNotVerifiedError extends SentinelError {
  constructor(message: string, requestId?: string) {
    super('EMAIL_NOT_VERIFIED', message, 403, requestId);
    this.name = 'EmailNotVerifiedError';
  }
}

/** An unexpected server-side error occurred. HTTP 500. */
export class InternalServerError extends SentinelError {
  constructor(message: string, requestId?: string) {
    super('INTERNAL_ERROR', message, 500, requestId);
    this.name = 'InternalServerError';
  }
}

/**
 * The network request could not be completed (DNS failure, connection refused,
 * etc.). `statusCode` is 0.
 */
export class NetworkError extends SentinelError {
  constructor(message: string, cause?: unknown) {
    super('NETWORK_ERROR', message, 0, undefined, cause);
    this.name = 'NetworkError';
  }
}

/**
 * No cached session was found for the given `userId`.
 * Thrown by `refreshSession()` when called for an unknown user.
 */
export class SessionNotFoundError extends SentinelError {
  constructor(userId: string) {
    super('SESSION_NOT_FOUND', `No cached session for user "${userId}"`, 0);
    this.name = 'SessionNotFoundError';
  }
}

/** The submitted MFA code (TOTP or recovery) was incorrect. HTTP 401. */
export class MfaInvalidCodeError extends SentinelError {
  constructor(message: string, requestId?: string) {
    super('INVALID_MFA_CODE', message, 401, requestId);
    this.name = 'MfaInvalidCodeError';
  }
}

/**
 * Too many failed MFA attempts — the per-token attempt counter was exceeded.
 * Distinct from the IP-level `RateLimitError`. HTTP 429.
 */
export class MfaAttemptLimitError extends SentinelError {
  constructor(message: string, requestId?: string) {
    super('MFA_ATTEMPT_LIMIT_EXCEEDED', message, 429, requestId);
    this.name = 'MfaAttemptLimitError';
  }
}

/** The requested API token does not exist or has already been revoked. HTTP 404. */
export class ApiTokenNotFoundError extends SentinelError {
  constructor(message: string, requestId?: string) {
    super('API_TOKEN_NOT_FOUND', message, 404, requestId);
    this.name = 'ApiTokenNotFoundError';
  }
}

/** The caller does not have the required role (e.g. admin) to perform this operation. HTTP 403. */
export class ForbiddenError extends SentinelError {
  constructor(message: string, requestId?: string) {
    super('FORBIDDEN', message, 403, requestId);
    this.name = 'ForbiddenError';
  }
}

// ---------------------------------------------------------------------------
// Factory — maps a server error code to the right typed class
// ---------------------------------------------------------------------------

/**
 * Creates the most specific `SentinelError` subclass for a given API error
 * code. Falls back to the base `SentinelError` for unknown codes.
 */
export function createErrorFromCode(
  code: string,
  message: string,
  statusCode: number,
  requestId?: string,
  details?: unknown,
): SentinelError {
  switch (code) {
    case 'AUTH_ERROR':
      return new AuthenticationError(message, requestId);
    case 'VALIDATION_ERROR':
      return new ValidationError(message, requestId, details);
    case 'INVALID_TOKEN':
      return new InvalidTokenError(message, requestId);
    case 'EXPIRED_TOKEN':
      return new ExpiredTokenError(message, requestId);
    case 'MISSING_TOKEN':
      return new MissingTokenError(message, requestId);
    case 'RATE_LIMIT_EXCEEDED':
      return new RateLimitError(message, requestId);
    case 'EMAIL_NOT_VERIFIED':
      return new EmailNotVerifiedError(message, requestId);
    case 'INTERNAL_ERROR':
      return new InternalServerError(message, requestId);
    case 'INVALID_MFA_CODE':
      return new MfaInvalidCodeError(message, requestId);
    case 'MFA_ATTEMPT_LIMIT_EXCEEDED':
      return new MfaAttemptLimitError(message, requestId);
    case 'API_TOKEN_NOT_FOUND':
      return new ApiTokenNotFoundError(message, requestId);
    case 'FORBIDDEN':
      return new ForbiddenError(message, requestId);
    default:
      return new SentinelError(code, message, statusCode, requestId, details);
  }
}

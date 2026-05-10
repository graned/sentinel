import { expect } from 'chai';
import { before, describe, it } from 'mocha';
import { AuthenticationError, SentinelAuthClient, ValidationError } from '../src/index';
import { API_BASE, TEST_PASSWORD, registerAndVerify, uniqueEmail, uniqueIp } from './helpers';

// ---------------------------------------------------------------------------
// Each test gets its own SentinelAuthClient with a unique X-Forwarded-For so
// it never shares a rate-limit bucket (login is capped at 5 req / 15 min).
// ---------------------------------------------------------------------------

function makeClient(): SentinelAuthClient {
  return new SentinelAuthClient({
    baseUrl: API_BASE,
    headers: { 'X-Forwarded-For': uniqueIp() },
  });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('SentinelAuthClient — login (integration)', () => {
  // -------------------------------------------------------------------------
  // Validation errors
  // -------------------------------------------------------------------------

  describe('validation errors', () => {
    it('throws ValidationError for an invalid email format', async () => {
      const client = makeClient();
      try {
        await client.login({ email: 'not-an-email', password: TEST_PASSWORD });
        expect.fail('expected error to be thrown');
      } catch (err) {
        expect(err).to.be.instanceOf(ValidationError);
        expect((err as ValidationError).code).to.equal('VALIDATION_ERROR');
        expect((err as ValidationError).statusCode).to.equal(400);
      }
    });

    it('throws ValidationError for an empty password', async () => {
      const client = makeClient();
      try {
        await client.login({ email: uniqueEmail(), password: '' });
        expect.fail('expected error to be thrown');
      } catch (err) {
        expect(err).to.be.instanceOf(ValidationError);
        expect((err as ValidationError).code).to.equal('VALIDATION_ERROR');
        expect((err as ValidationError).statusCode).to.equal(400);
      }
    });
  });

  // -------------------------------------------------------------------------
  // Authentication errors
  // -------------------------------------------------------------------------

  describe('authentication errors', () => {
    it('throws AuthenticationError for a non-existent user', async () => {
      const client = makeClient();
      try {
        await client.login({ email: uniqueEmail('ghost'), password: TEST_PASSWORD });
        expect.fail('expected error to be thrown');
      } catch (err) {
        expect(err).to.be.instanceOf(AuthenticationError);
        expect((err as AuthenticationError).code).to.equal('AUTH_ERROR');
        expect((err as AuthenticationError).statusCode).to.equal(401);
      }
    });

    it('throws AuthenticationError for a wrong password', async () => {
      const client = makeClient();
      const email = uniqueEmail('wrongpw');
      await registerAndVerify(email);

      try {
        await client.login({ email, password: 'WrongP@ssword1!' });
        expect.fail('expected error to be thrown');
      } catch (err) {
        expect(err).to.be.instanceOf(AuthenticationError);
        expect((err as AuthenticationError).code).to.equal('AUTH_ERROR');
        expect((err as AuthenticationError).statusCode).to.equal(401);
      }
    });
  });

  // -------------------------------------------------------------------------
  // Successful login
  // -------------------------------------------------------------------------

  describe('successful login', () => {
    let client: SentinelAuthClient;
    let email: string;

    before(async () => {
      client = makeClient();
      email = uniqueEmail('success');
      await registerAndVerify(email);
    });

    it('returns a session result', async () => {
      const result = await client.login({ email, password: TEST_PASSWORD });
      expect(result.type).to.equal('session');
    });

    it('session has a valid userId, tokens, and future expiry', async () => {
      const result = await client.login({ email, password: TEST_PASSWORD });
      if (result.type !== 'session') return expect.fail('expected session');

      const { session } = result;
      expect(session.userId).to.be.a('string').and.not.empty;
      expect(session.accessToken).to.be.a('string').and.not.empty;
      expect(session.refreshToken).to.be.a('string').and.not.empty;
      expect(session.expiresAt).to.be.instanceOf(Date);
      expect(session.expiresAt.getTime()).to.be.greaterThan(Date.now());
    });

    it('stores the session in the cache keyed by userId', async () => {
      const result = await client.login({ email, password: TEST_PASSWORD });
      if (result.type !== 'session') return expect.fail('expected session');

      const cached = client.getSession(result.session.userId);
      expect(cached).to.deep.equal(result.session);
    });

    it('overwrites a stale cached session when the user logs in again', async () => {
      const first = await client.login({ email, password: TEST_PASSWORD });
      const second = await client.login({ email, password: TEST_PASSWORD });

      if (first.type !== 'session' || second.type !== 'session') {
        return expect.fail('expected session results');
      }

      const cached = client.getSession(first.session.userId);
      // The cache should hold the most-recent session
      expect(cached?.accessToken).to.equal(second.session.accessToken);
    });
  });

  // -------------------------------------------------------------------------
  // Logout
  // -------------------------------------------------------------------------

  describe('logout', () => {
    it('removes the session from the cache', async () => {
      const client = makeClient();
      const email = uniqueEmail('logout');
      await registerAndVerify(email);

      const result = await client.login({ email, password: TEST_PASSWORD });
      if (result.type !== 'session') return expect.fail('expected session');

      const { userId } = result.session;
      expect(client.getSession(userId)).to.not.be.undefined;

      await client.logout(userId);
      expect(client.getSession(userId)).to.be.undefined;
    });

    it('is a no-op when no session is cached for the userId', async () => {
      const client = makeClient();
      await client.logout('non-existent-user-id');
      // No error thrown — test passes
    });
  });

  // -------------------------------------------------------------------------
  // getValidSession
  // -------------------------------------------------------------------------

  describe('getValidSession()', () => {
    it('returns null for an unknown userId', async () => {
      const client = makeClient();
      const result = await client.getValidSession('unknown-user');
      expect(result).to.be.null;
    });

    it('returns null after the session has been removed', async () => {
      const client = makeClient();
      const email = uniqueEmail('evicted');
      await registerAndVerify(email);

      const loginResult = await client.login({ email, password: TEST_PASSWORD });
      if (loginResult.type !== 'session') return expect.fail('expected session');

      const { userId } = loginResult.session;
      client.clearSession(userId);

      expect(await client.getValidSession(userId)).to.be.null;
    });

    it('returns a live session for a freshly-logged-in user', async () => {
      const client = makeClient();
      const email = uniqueEmail('fresh-session');
      await registerAndVerify(email);

      const loginResult = await client.login({ email, password: TEST_PASSWORD });
      if (loginResult.type !== 'session') return expect.fail('expected session');

      const session = await client.getValidSession(loginResult.session.userId);
      expect(session).to.not.be.null;
      expect(session?.userId).to.equal(loginResult.session.userId);
      expect(session?.accessToken).to.be.a('string').and.not.empty;
    });
  });

  // -------------------------------------------------------------------------
  // clearAllSessions / evictExpiredSessions
  // -------------------------------------------------------------------------

  describe('cache management', () => {
    it('clearAllSessions() empties the cache', async () => {
      const client = makeClient();
      const email = uniqueEmail('clear-all');
      await registerAndVerify(email);

      const result = await client.login({ email, password: TEST_PASSWORD });
      if (result.type !== 'session') return expect.fail('expected session');

      client.clearAllSessions();
      expect(client.getSession(result.session.userId)).to.be.undefined;
    });

    it('evictExpiredSessions() keeps fresh sessions intact', async () => {
      const client = makeClient();
      const email = uniqueEmail('evict');
      await registerAndVerify(email);

      const result = await client.login({ email, password: TEST_PASSWORD });
      if (result.type !== 'session') return expect.fail('expected session');

      const removed = client.evictExpiredSessions();
      expect(removed).to.equal(0);
      expect(client.getSession(result.session.userId)).to.not.be.undefined;
    });
  });
});

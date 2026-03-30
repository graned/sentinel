import { expect } from 'chai';
import { describe, it } from 'mocha';
import { SentinelAuthClient, ValidationError } from '../src/index';
import { API_BASE, TEST_PASSWORD, uniqueEmail, uniqueIp } from './helpers';

function makeClient(): SentinelAuthClient {
  return new SentinelAuthClient({
    baseUrl: API_BASE,
    headers: { 'X-Forwarded-For': uniqueIp() },
  });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('SentinelAuthClient.register() (integration)', () => {
  describe('validation errors', () => {
    it('throws ValidationError for an invalid email format', async () => {
      const client = makeClient();
      try {
        await client.register({
          first_name: 'Test',
          last_name: 'User',
          email: 'not-an-email',
          password: TEST_PASSWORD,
        });
        expect.fail('expected error to be thrown');
      } catch (err) {
        expect(err).to.be.instanceOf(ValidationError);
        expect((err as ValidationError).code).to.equal('VALIDATION_ERROR');
        expect((err as ValidationError).statusCode).to.equal(400);
      }
    });

    it('throws ValidationError for a weak password', async () => {
      const client = makeClient();
      try {
        await client.register({
          first_name: 'Test',
          last_name: 'User',
          email: uniqueEmail(),
          password: 'weak',
        });
        expect.fail('expected error to be thrown');
      } catch (err) {
        expect(err).to.be.instanceOf(ValidationError);
        expect((err as ValidationError).code).to.equal('VALIDATION_ERROR');
      }
    });

    it('throws ValidationError for an empty first_name', async () => {
      const client = makeClient();
      try {
        await client.register({
          first_name: '',
          last_name: 'User',
          email: uniqueEmail(),
          password: TEST_PASSWORD,
        });
        expect.fail('expected error to be thrown');
      } catch (err) {
        expect(err).to.be.instanceOf(ValidationError);
        expect((err as ValidationError).code).to.equal('VALIDATION_ERROR');
      }
    });
  });

  describe('successful registration', () => {
    it('returns user data with PendingVerification status', async () => {
      const client = makeClient();
      const email = uniqueEmail('register');

      const result = await client.register({
        first_name: 'Alice',
        last_name: 'Smith',
        email,
        password: TEST_PASSWORD,
      });

      expect(result.user_id).to.be.a('string').and.not.empty;
      expect(result.first_name).to.equal('Alice');
      expect(result.last_name).to.equal('Smith');
      expect(result.status).to.equal('PendingVerification');
    });

    it('accepts an optional avatar_url', async () => {
      const client = makeClient();

      const result = await client.register({
        first_name: 'Bob',
        last_name: 'Jones',
        email: uniqueEmail('avatar'),
        avatar_url: 'https://example.com/avatar.png',
        password: TEST_PASSWORD,
      });

      expect(result.avatar_url).to.equal('https://example.com/avatar.png');
    });
  });
});

describe('SentinelAuthClient.getAuthMethods() (integration)', () => {
  it('returns auth method capabilities', async () => {
    const client = makeClient();
    const methods = await client.getAuthMethods();

    expect(methods.password_enabled).to.be.a('boolean');
    expect(methods.mfa_totp_available).to.be.a('boolean');
    expect(methods.api_tokens_available).to.be.a('boolean');
    expect(methods.email_verification_required).to.be.a('boolean');
    expect(methods.oidc_clients).to.be.an('array');
  });
});

describe('SentinelAuthClient.authenticate() (integration)', () => {
  it('returns auth context for a valid access token', async () => {
    const client = makeClient();
    const email = uniqueEmail('authenticate');

    // Register and verify, then login to get a token
    const { registerAndVerify } = await import('./helpers');
    await registerAndVerify(email);
    const loginResult = await client.login({ email, password: TEST_PASSWORD });
    if (loginResult.type !== 'session') return expect.fail('expected session');

    const ctx = await client.authenticate({
      access_token: loginResult.session.accessToken,
    });

    expect(ctx.user_id).to.equal(loginResult.session.userId);
    expect(ctx.roles).to.be.an('array');
    expect(ctx.email_verified).to.be.true;
  });
});

describe('SentinelAuthClient.forgotPassword() (integration)', () => {
  it('returns 200 for any email address (anti-enumeration)', async () => {
    const client = makeClient();
    // Should not throw for unknown email (server silently no-ops)
    await client.forgotPassword({ email: 'nonexistent@example.com' });
  });

  it('returns 200 for a known email address', async () => {
    const client = makeClient();
    const email = uniqueEmail('forgotpw');
    await client.register({
      first_name: 'Test',
      last_name: 'User',
      email,
      password: TEST_PASSWORD,
    });
    // Should not throw regardless of whether SMTP is configured
    await client.forgotPassword({ email });
  });
});

describe('SentinelAuthClient.logoutAll() (integration)', () => {
  it('removes the session from cache and revokes all server-side sessions', async () => {
    const client = makeClient();
    const email = uniqueEmail('logoutall');

    const { registerAndVerify } = await import('./helpers');
    await registerAndVerify(email);

    const loginResult = await client.login({ email, password: TEST_PASSWORD });
    if (loginResult.type !== 'session') return expect.fail('expected session');

    const { userId } = loginResult.session;
    await client.logoutAll(userId);

    expect(client.getSession(userId)).to.be.undefined;
  });

  it('is a no-op when no session is cached', async () => {
    const client = makeClient();
    await client.logoutAll('unknown-user-id');
  });
});

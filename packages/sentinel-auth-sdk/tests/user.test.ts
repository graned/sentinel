import { expect } from 'chai';
import { before, describe, it } from 'mocha';
import { SentinelAuthClient, ValidationError } from '../src/index';
import type { Session } from '../src/index';
import { API_BASE, TEST_PASSWORD, registerAndVerify, uniqueEmail, uniqueIp } from './helpers';

function makeClient(): SentinelAuthClient {
  return new SentinelAuthClient({
    baseUrl: API_BASE,
    headers: { 'X-Forwarded-For': uniqueIp() },
  });
}

async function loginAs(email: string): Promise<{ client: SentinelAuthClient; session: Session }> {
  const client = makeClient();
  const result = await client.login({ email, password: TEST_PASSWORD });
  if (result.type !== 'session') throw new Error('expected session, got mfa_challenge');
  return { client, session: result.session };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('client.user.getMe() (integration)', () => {
  let client: SentinelAuthClient;
  let session: Session;
  let email: string;

  before(async () => {
    email = uniqueEmail('getme');
    await registerAndVerify(email);
    ({ client, session } = await loginAs(email));
  });

  it('returns the user profile', async () => {
    const profile = await client.user.getMe(session.accessToken);

    expect(profile.user_id).to.equal(session.userId);
    expect(profile.email).to.equal(email);
    expect(profile.email_verified).to.be.true;
    expect(profile.first_name).to.be.a('string');
    expect(profile.last_name).to.be.a('string');
    expect(profile.status).to.be.a('string');
  });
});

describe('client.user.getSessions() (integration)', () => {
  let client: SentinelAuthClient;
  let session: Session;

  before(async () => {
    const email = uniqueEmail('sessions');
    await registerAndVerify(email);
    ({ client, session } = await loginAs(email));
  });

  it('returns an array containing at least the current session', async () => {
    const sessions = await client.user.getSessions(session.accessToken);

    expect(sessions).to.be.an('array').with.length.greaterThan(0);
    const current = sessions.find((s) => s.is_current);
    expect(current).to.not.be.undefined;
    expect(current?.session_id).to.be.a('string');
    expect(current?.expires_at).to.be.a('string');
  });
});

describe('client.user.getSession() (integration)', () => {
  let client: SentinelAuthClient;
  let session: Session;

  before(async () => {
    const email = uniqueEmail('getsession');
    await registerAndVerify(email);
    ({ client, session } = await loginAs(email));
  });

  it('returns detail for a specific session ID', async () => {
    const all = await client.user.getSessions(session.accessToken);
    const current = all.find((s) => s.is_current);
    if (!current) return expect.fail('no current session found in list');

    const detail = await client.user.getSession(session.accessToken, current.session_id);

    expect(detail.session_id).to.equal(current.session_id);
    expect(detail.is_active).to.be.a('boolean');
    expect(detail).to.have.property('revoked_at');
  });
});

describe('client.user.getPermissions() (integration)', () => {
  let client: SentinelAuthClient;
  let session: Session;

  before(async () => {
    const email = uniqueEmail('permissions');
    await registerAndVerify(email);
    ({ client, session } = await loginAs(email));
  });

  it('returns permissions with user_id and roles array', async () => {
    const perms = await client.user.getPermissions(session.accessToken);

    expect(perms.user_id).to.equal(session.userId);
    expect(perms.roles).to.be.an('array');
  });
});

describe('client.user.changePassword() (integration)', () => {
  it('throws ValidationError for a weak new password', async () => {
    const email = uniqueEmail('changepw-weak');
    await registerAndVerify(email);
    const { client, session } = await loginAs(email);

    try {
      await client.user.changePassword(session.accessToken, {
        current_password: TEST_PASSWORD,
        new_password: 'weak',
      });
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(ValidationError);
      expect((err as ValidationError).code).to.equal('VALIDATION_ERROR');
    }
  });

  it('succeeds and allows login with the new password', async () => {
    const newPassword = 'NewS3cur3P@ssw0rd!';
    const email = uniqueEmail('changepw-ok');
    await registerAndVerify(email);
    const { client, session } = await loginAs(email);

    // Change the password
    await client.user.changePassword(session.accessToken, {
      current_password: TEST_PASSWORD,
      new_password: newPassword,
    });

    // Login with new password (new client / fresh IP)
    const client2 = makeClient();
    const result = await client2.login({ email, password: newPassword });
    expect(result.type).to.equal('session');
  });
});

describe('client.system.health() (integration)', () => {
  it('returns a status string', async () => {
    const client = makeClient();
    const health = await client.system.health();
    expect(health.status).to.be.a('string');
  });
});

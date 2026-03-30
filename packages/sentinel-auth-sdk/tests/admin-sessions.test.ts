import { expect } from 'chai';
import { before, describe, it } from 'mocha';
import { SentinelAuthClient, SentinelError } from '../src/index';
import type { Session } from '../src/index';
import {
  API_BASE,
  TEST_PASSWORD,
  registerAndVerify,
  uniqueEmail,
  uniqueIp,
} from './helpers';

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
// Security tests — unauthenticated calls
// ---------------------------------------------------------------------------

describe('client.admin.listActiveSessions() — unauthenticated', () => {
  it('throws a SentinelError when called without a token', async () => {
    const client = makeClient();
    try {
      await client.admin.listActiveSessions('');
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(SentinelError);
    }
  });
});

describe('client.admin.revokeSession() — unauthenticated', () => {
  it('throws a SentinelError when called without a token', async () => {
    const client = makeClient();
    try {
      await client.admin.revokeSession('', '00000000-0000-0000-0000-000000000000');
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(SentinelError);
    }
  });
});

describe('client.admin.revokeSessionsBulk() — unauthenticated', () => {
  it('throws a SentinelError when called without a token', async () => {
    const client = makeClient();
    try {
      await client.admin.revokeSessionsBulk('', {
        session_ids: ['00000000-0000-0000-0000-000000000000'],
      });
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(SentinelError);
    }
  });
});

// ---------------------------------------------------------------------------
// Security tests — non-admin user
// ---------------------------------------------------------------------------

describe('client.admin.listActiveSessions() — non-admin user', () => {
  let session: Session;
  let client: SentinelAuthClient;

  before(async () => {
    const email = uniqueEmail('admin-sessions-nonadmin');
    await registerAndVerify(email);
    ({ client, session } = await loginAs(email));
  });

  it('throws a SentinelError (403 FORBIDDEN) for a regular user', async () => {
    try {
      await client.admin.listActiveSessions(session.accessToken);
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(SentinelError);
      expect((err as SentinelError).statusCode).to.equal(403);
    }
  });
});

describe('client.admin.revokeSession() — non-admin user', () => {
  let session: Session;
  let client: SentinelAuthClient;

  before(async () => {
    const email = uniqueEmail('admin-revoke-nonadmin');
    await registerAndVerify(email);
    ({ client, session } = await loginAs(email));
  });

  it('throws a SentinelError (403 FORBIDDEN) for a regular user', async () => {
    try {
      await client.admin.revokeSession(
        session.accessToken,
        '00000000-0000-0000-0000-000000000000',
      );
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(SentinelError);
      expect((err as SentinelError).statusCode).to.equal(403);
    }
  });
});

describe('client.admin.revokeSessionsBulk() — non-admin user', () => {
  let session: Session;
  let client: SentinelAuthClient;

  before(async () => {
    const email = uniqueEmail('admin-bulk-nonadmin');
    await registerAndVerify(email);
    ({ client, session } = await loginAs(email));
  });

  it('throws a SentinelError (403 FORBIDDEN) for a regular user', async () => {
    try {
      await client.admin.revokeSessionsBulk(session.accessToken, {
        session_ids: ['00000000-0000-0000-0000-000000000000'],
      });
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(SentinelError);
      expect((err as SentinelError).statusCode).to.equal(403);
    }
  });
});

// ---------------------------------------------------------------------------
// Admin happy-path tests
// ---------------------------------------------------------------------------
// These require a user with the "admin" role. There is currently no public API
// to elevate a user to admin. Skip these tests until an admin seed endpoint is
// available — at that point replace the `this.skip()` calls with real
// assertions following the registerAndVerify + makeClient pattern used above.

describe('client.admin.listActiveSessions() — admin happy-path', () => {
  it('returns an array of sessions with user_email and session_id (requires admin)', async function () {
    this.skip();
    // Seed an admin user, log in, then:
    // const sessions = await client.admin.listActiveSessions(adminToken);
    // expect(sessions).to.be.an('array');
    // expect(sessions[0]).to.have.property('session_id');
    // expect(sessions[0]).to.have.property('user_email');
    // expect(sessions[0]).to.have.property('expires_at');
  });
});

describe('client.admin.revokeSession() — admin happy-path', () => {
  it('revokes a specific session and it no longer appears in the list (requires admin)', async function () {
    this.skip();
    // Seed admin user + target user, then:
    // await client.admin.revokeSession(adminToken, targetSessionId);
    // const sessions = await client.admin.listActiveSessions(adminToken);
    // expect(sessions.find(s => s.session_id === targetSessionId)).to.be.undefined;
  });
});

describe('client.admin.revokeSessionsBulk() — admin happy-path', () => {
  it('returns revoked_count equal to the number of supplied IDs (requires admin)', async function () {
    this.skip();
    // Seed admin user + two target sessions, then:
    // const result = await client.admin.revokeSessionsBulk(adminToken, { session_ids: [id1, id2] });
    // expect(result.revoked_count).to.equal(2);
  });
});

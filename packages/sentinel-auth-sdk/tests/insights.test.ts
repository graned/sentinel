import { expect } from 'chai';
import { describe, it } from 'mocha';
import { ForbiddenError, MissingTokenError, SentinelAuthClient } from '../src/index';
import { API_BASE, TEST_PASSWORD, registerAndVerify, uniqueEmail, uniqueIp } from './helpers';

// ---------------------------------------------------------------------------
// Each test gets its own SentinelAuthClient with a unique X-Forwarded-For so
// it never shares a rate-limit bucket.
// ---------------------------------------------------------------------------

function makeClient(): SentinelAuthClient {
  return new SentinelAuthClient({
    baseUrl: API_BASE,
    headers: { 'X-Forwarded-For': uniqueIp() },
  });
}

// ---------------------------------------------------------------------------
// Shared helper — register + verify + login, return access token
// ---------------------------------------------------------------------------

async function getRegularUserToken(): Promise<string> {
  const client = makeClient();
  const email = uniqueEmail('insights-sdk');
  await registerAndVerify(email);
  const result = await client.login({ email, password: TEST_PASSWORD });
  if (result.type !== 'session') throw new Error('Expected a session, got MFA challenge');
  return result.session.accessToken;
}

// ---------------------------------------------------------------------------
// GET /v1/api/system/stats
// ---------------------------------------------------------------------------

describe('SystemClient.getInsightsSummary — auth checks (integration)', () => {
  it('throws MissingTokenError when no access token is provided', async () => {
    const client = makeClient();
    try {
      await client.system.getInsightsSummary('');
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(MissingTokenError);
    }
  });

  it('throws ForbiddenError for a non-admin user', async () => {
    const accessToken = await getRegularUserToken();
    const client = makeClient();
    try {
      await client.system.getInsightsSummary(accessToken);
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(ForbiddenError);
    }
  });
});

// ---------------------------------------------------------------------------
// GET /v1/api/system/analytics/user-growth
// ---------------------------------------------------------------------------

describe('SystemClient.getUserGrowth — auth checks (integration)', () => {
  it('throws MissingTokenError when no access token is provided', async () => {
    const client = makeClient();
    try {
      await client.system.getUserGrowth('');
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(MissingTokenError);
    }
  });

  it('throws ForbiddenError for a non-admin user', async () => {
    const accessToken = await getRegularUserToken();
    const client = makeClient();
    try {
      await client.system.getUserGrowth(accessToken, 7);
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(ForbiddenError);
    }
  });
});

// ---------------------------------------------------------------------------
// GET /v1/api/system/analytics/sessions
// ---------------------------------------------------------------------------

describe('SystemClient.getSessionActivity — auth checks (integration)', () => {
  it('throws MissingTokenError when no access token is provided', async () => {
    const client = makeClient();
    try {
      await client.system.getSessionActivity('');
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(MissingTokenError);
    }
  });

  it('throws ForbiddenError for a non-admin user', async () => {
    const accessToken = await getRegularUserToken();
    const client = makeClient();
    try {
      await client.system.getSessionActivity(accessToken, 7);
      expect.fail('expected error to be thrown');
    } catch (err) {
      expect(err).to.be.instanceOf(ForbiddenError);
    }
  });
});

// ---------------------------------------------------------------------------
// Admin happy-path tests
// Require a pre-seeded admin user. Run with:
//   npm run test:integration -- --grep "admin insights"
// ---------------------------------------------------------------------------

describe('SystemClient insights — admin happy path (integration, skipped)', () => {
  it.skip('getInsightsSummary returns the correct shape for an admin user', async () => {
    // Seed an admin user, log in with makeClient(), then:
    // const data = await client.system.getInsightsSummary(accessToken);
    // expect(data).to.have.keys([
    //   'total_users', 'new_users_week', 'new_users_month',
    //   'active_users_week', 'active_users_month', 'active_sessions',
    //   'mfa_adoption_pct', 'email_verified_pct',
    // ]);
    // expect(data.total_users).to.be.a('number').and.be.at.least(0);
    // expect(data.mfa_adoption_pct).to.be.within(0, 100);
    // expect(data.email_verified_pct).to.be.within(0, 100);
  });

  it.skip('getUserGrowth returns an array of UserGrowthPoint for an admin user', async () => {
    // const points = await client.system.getUserGrowth(accessToken, 7);
    // expect(points).to.be.an('array');
    // if (points.length > 0) {
    //   expect(points[0]).to.have.keys(['date', 'total_users', 'new_users']);
    //   expect(points[0].date).to.match(/^\d{4}-\d{2}-\d{2}$/);
    // }
  });

  it.skip('getSessionActivity returns an array of SessionActivityPoint for an admin user', async () => {
    // const points = await client.system.getSessionActivity(accessToken, 7);
    // expect(points).to.be.an('array');
    // if (points.length > 0) {
    //   expect(points[0]).to.have.keys(['date', 'sessions_created', 'unique_users']);
    // }
  });
});

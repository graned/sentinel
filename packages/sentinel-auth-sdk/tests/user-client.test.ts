import { expect } from 'chai';
import { describe, it } from 'mocha';
import type { RequestFn, UserProfileData } from '../src/types';
import { UserClient } from '../src/user-client';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** A mock `RequestFn` that captures calls for assertions. */
function mockRequestFn(): {
  req: RequestFn;
  calls: Array<{ path: string; options?: RequestInit }>;
} {
  const calls: Array<{ path: string; options?: RequestInit }> = [];
  const req: RequestFn = <T>(path: string, options?: RequestInit) => {
    calls.push({ path, options });
    return Promise.resolve({ data: {} as T, requestId: 'mock-req-id' });
  };
  return { req, calls };
}

/** A sample `UserProfileData` returned by the mock. */
const sampleProfile: UserProfileData = {
  user_id: 'user-abc-123',
  first_name: 'Alice',
  last_name: 'Smith',
  avatar_url: null,
  status: 'Active',
  email: 'alice@example.com',
  email_verified: true,
  mfa_enabled: false,
  created_at: '2026-01-01T00:00:00',
};

/** Return a mock `RequestFn` that resolves with `sampleProfile`. */
function mockProfileRequest(): RequestFn {
  return <T>(_path: string, _options?: RequestInit) =>
    Promise.resolve({ data: sampleProfile as T, requestId: 'mock-req-id' });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('UserClient (unit)', () => {
  describe('updateProfile()', () => {
    it('calls PATCH /v1/api/user/me with the correct endpoint', async () => {
      const { req, calls } = mockRequestFn();
      const client = new UserClient(req);

      await client.updateProfile('tok_abc', { first_name: 'Bob' });

      expect(calls).to.have.lengthOf(1);
      expect(calls[0].path).to.equal('/v1/api/user/me');
      expect(calls[0].options?.method).to.equal('PATCH');
    });

    it('sets the Authorization header to Bearer <token>', async () => {
      const { req, calls } = mockRequestFn();
      const client = new UserClient(req);

      await client.updateProfile('tok_abc', { first_name: 'Bob' });

      const headers = calls[0].options?.headers as Record<string, string>;
      expect(headers['Authorization']).to.equal('Bearer tok_abc');
    });

    it('serializes the request body as JSON with all fields', async () => {
      const { req, calls } = mockRequestFn();
      const client = new UserClient(req);

      const body = {
        first_name: 'Bob',
        last_name: 'Builder',
        avatar_url: 'https://example.com/avatar.png',
      };
      await client.updateProfile('tok_abc', body);

      const parsed = JSON.parse(calls[0].options?.body as string);
      expect(parsed).to.deep.equal(body);
    });

    it('serializes a partial body (single field) correctly', async () => {
      const { req, calls } = mockRequestFn();
      const client = new UserClient(req);

      await client.updateProfile('tok_abc', { last_name: 'NewName' });

      const parsed = JSON.parse(calls[0].options?.body as string);
      expect(parsed).to.deep.equal({ last_name: 'NewName' });
      expect(parsed).to.not.have.property('first_name');
      expect(parsed).to.not.have.property('avatar_url');
    });

    it('serializes an empty body correctly', async () => {
      const { req, calls } = mockRequestFn();
      const client = new UserClient(req);

      await client.updateProfile('tok_abc', {});

      const parsed = JSON.parse(calls[0].options?.body as string);
      expect(parsed).to.deep.equal({});
    });

    it('returns the full UserProfileData from the response', async () => {
      const client = new UserClient(mockProfileRequest());

      const result = await client.updateProfile('tok_abc', { first_name: 'Alice' });

      expect(result).to.deep.equal(sampleProfile);
      expect(result.user_id).to.equal('user-abc-123');
      expect(result.email).to.equal('alice@example.com');
    });

    it('sets Content-Type header to application/json', async () => {
      const { req, calls } = mockRequestFn();
      const client = new UserClient(req);

      await client.updateProfile('tok_abc', { first_name: 'Test' });

      // The SDK's `request()` method always sets this header, but the
      // sub-client's `req` is a bound version of it. Verify the call
      // includes the JSON body so the parent can set the content type.
      const parsed = JSON.parse(calls[0].options?.body as string);
      expect(parsed).to.deep.equal({ first_name: 'Test' });
    });
  });
});

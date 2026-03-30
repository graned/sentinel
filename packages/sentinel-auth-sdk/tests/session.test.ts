import { expect } from 'chai';
import { beforeEach, describe, it } from 'mocha';
import { SessionCache } from '../src/session';
import type { Session } from '../src/types';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeSession(overrides: Partial<Session> = {}): Session {
  return {
    userId: 'user-123',
    accessToken: 'tok_abc',
    refreshToken: 'ref_abc',
    expiresAt: new Date(Date.now() + 60 * 60 * 1000), // 1 hour from now
    ...overrides,
  };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('SessionCache (unit)', () => {
  let cache: SessionCache;

  beforeEach(() => {
    cache = new SessionCache();
  });

  // -------------------------------------------------------------------------
  // Basic CRUD
  // -------------------------------------------------------------------------

  describe('get / set', () => {
    it('returns undefined for an unknown userId', () => {
      expect(cache.get('unknown')).to.be.undefined;
    });

    it('stores and retrieves a session', () => {
      const session = makeSession();
      cache.set('user-123', session);
      expect(cache.get('user-123')).to.deep.equal(session);
    });

    it('overwrites an existing session', () => {
      cache.set('user-123', makeSession({ accessToken: 'first' }));
      cache.set('user-123', makeSession({ accessToken: 'second' }));
      expect(cache.get('user-123')?.accessToken).to.equal('second');
    });
  });

  describe('delete', () => {
    it('removes a stored session', () => {
      cache.set('user-123', makeSession());
      cache.delete('user-123');
      expect(cache.get('user-123')).to.be.undefined;
    });

    it('is a no-op for an unknown userId', () => {
      expect(() => cache.delete('ghost')).to.not.throw();
    });
  });

  describe('has', () => {
    it('returns false when the userId is absent', () => {
      expect(cache.has('user-123')).to.be.false;
    });

    it('returns true after a session is set', () => {
      cache.set('user-123', makeSession());
      expect(cache.has('user-123')).to.be.true;
    });

    it('returns false after the session is deleted', () => {
      cache.set('user-123', makeSession());
      cache.delete('user-123');
      expect(cache.has('user-123')).to.be.false;
    });
  });

  describe('size', () => {
    it('starts at 0', () => {
      expect(cache.size).to.equal(0);
    });

    it('reflects the number of stored sessions', () => {
      cache.set('a', makeSession({ userId: 'a' }));
      cache.set('b', makeSession({ userId: 'b' }));
      expect(cache.size).to.equal(2);
    });

    it('decrements after a delete', () => {
      cache.set('a', makeSession({ userId: 'a' }));
      cache.set('b', makeSession({ userId: 'b' }));
      cache.delete('a');
      expect(cache.size).to.equal(1);
    });
  });

  describe('clear', () => {
    it('removes all sessions', () => {
      cache.set('a', makeSession({ userId: 'a' }));
      cache.set('b', makeSession({ userId: 'b' }));
      cache.clear();
      expect(cache.size).to.equal(0);
    });
  });

  // -------------------------------------------------------------------------
  // Expiry helpers
  // -------------------------------------------------------------------------

  describe('isExpired()', () => {
    it('returns false when the expiry is in the future', () => {
      const session = makeSession({ expiresAt: new Date(Date.now() + 1_000) });
      expect(cache.isExpired(session)).to.be.false;
    });

    it('returns true when the expiry has passed', () => {
      const session = makeSession({ expiresAt: new Date(Date.now() - 1) });
      expect(cache.isExpired(session)).to.be.true;
    });
  });

  describe('isExpiringSoon()', () => {
    it('returns false when the expiry is outside the buffer window', () => {
      // 10 min remaining, 5 min buffer — not expiring soon
      const session = makeSession({ expiresAt: new Date(Date.now() + 10 * 60 * 1_000) });
      expect(cache.isExpiringSoon(session, 5 * 60 * 1_000)).to.be.false;
    });

    it('returns true when the expiry falls within the buffer window', () => {
      // 2 min remaining, 5 min buffer — expiring soon
      const session = makeSession({ expiresAt: new Date(Date.now() + 2 * 60 * 1_000) });
      expect(cache.isExpiringSoon(session, 5 * 60 * 1_000)).to.be.true;
    });

    it('returns true for an already-expired session', () => {
      const session = makeSession({ expiresAt: new Date(Date.now() - 1) });
      expect(cache.isExpiringSoon(session, 5 * 60 * 1_000)).to.be.true;
    });
  });

  // -------------------------------------------------------------------------
  // evictExpired
  // -------------------------------------------------------------------------

  describe('evictExpired()', () => {
    it('returns 0 when the cache is empty', () => {
      expect(cache.evictExpired()).to.equal(0);
    });

    it('removes expired sessions and returns the count', () => {
      cache.set('fresh', makeSession({ userId: 'fresh', expiresAt: new Date(Date.now() + 9_999) }));
      cache.set('stale', makeSession({ userId: 'stale', expiresAt: new Date(Date.now() - 1) }));

      const removed = cache.evictExpired();

      expect(removed).to.equal(1);
      expect(cache.get('stale')).to.be.undefined;
      expect(cache.get('fresh')).to.not.be.undefined;
    });

    it('does not remove non-expired sessions', () => {
      cache.set('a', makeSession({ userId: 'a' }));
      cache.set('b', makeSession({ userId: 'b' }));

      const removed = cache.evictExpired();

      expect(removed).to.equal(0);
      expect(cache.size).to.equal(2);
    });
  });
});

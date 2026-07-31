import claims from '../data/claims.json';
import type { StatusId } from './status';
import { assertStatus } from './status';

export interface ClaimRecord {
  id: string;
  text: string;
  status: StatusId;
  scope: string;
  source: string[];
  verified_for: string;
  last_verified: string;
}

const byId = new Map(
  (claims as ClaimRecord[]).map((c) => {
    assertStatus(c.status);
    return [c.id, c];
  }),
);

export function getClaim(id: string): ClaimRecord {
  const claim = byId.get(id);
  if (!claim) {
    throw new Error(`Unknown claim id: ${id}`);
  }
  return claim;
}

export function allClaims(): ClaimRecord[] {
  return [...byId.values()];
}

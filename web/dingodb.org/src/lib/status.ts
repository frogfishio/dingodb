import vocabulary from '../data/status-vocabulary.json';

export type StatusId =
  | 'available'
  | 'experimental'
  | 'development-only'
  | 'scaffold'
  | 'design'
  | 'deferred';

const allowed = new Set(vocabulary.allowed as StatusId[]);

export function assertStatus(status: string): StatusId {
  if (!allowed.has(status as StatusId)) {
    throw new Error(
      `Unknown status "${status}". Allowed: ${[...allowed].join(', ')}`,
    );
  }
  return status as StatusId;
}

export function statusDisplay(status: string): string {
  const id = assertStatus(status);
  return vocabulary.labels[id].display;
}

export function statusMeaning(status: string): string {
  const id = assertStatus(status);
  return vocabulary.labels[id].meaning;
}

export const statusLabels = vocabulary.labels;

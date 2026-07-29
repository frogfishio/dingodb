import vocabulary from '../data/status-vocabulary.json';

export type StatusId =
  | 'available'
  | 'experimental'
  | 'development-only'
  | 'scaffold'
  | 'design'
  | 'deferred';

const allowed = new Set(vocabulary.allowed as string[]);

export function assertStatus(status: string): StatusId {
  if (!allowed.has(status)) {
    throw new Error(`Unknown status "${status}"`);
  }
  return status as StatusId;
}

export function statusDisplay(status: string): string {
  const id = assertStatus(status);
  return (vocabulary.labels as Record<string, { display: string }>)[id].display;
}

export const surfaces = vocabulary.surfaces as string[];
export const docClasses = vocabulary.docClasses as string[];
export const specStates = vocabulary.specStates as Record<string, string>;

import { describe, expect, it } from 'vitest';
import { projectRulebenchStatus } from './index';

describe('projectRulebenchStatus', () => {
  it('projects ready status', () => {
    expect(projectRulebenchStatus({ status: 'ready', label: 'Ready' })).toEqual({ isReady: true, label: 'Ready' });
  });
});

import { describe, expect, it } from 'vitest';
import { projectTemplateStatus } from './index';

describe('projectTemplateStatus', () => {
  it('projects ready status', () => {
    expect(projectTemplateStatus({ status: 'ready', label: 'Ready' })).toEqual({ isReady: true, label: 'Ready' });
  });
});

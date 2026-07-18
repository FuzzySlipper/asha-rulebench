import assert from 'node:assert/strict';
import test from 'node:test';
import {
  buildVerifyChangePlan,
  parseVerifyChangeArguments,
} from './verify-change.mjs';

test('requires an explicit closed profile', () => {
  assert.throws(
    () => parseVerifyChangeArguments([]),
    /At least one --profile is required/,
  );
  assert.throws(
    () => parseVerifyChangeArguments(['--profile', 'rust-owner']),
    /Unknown verify:change profile/,
  );
});

test('takes the ordered union and deduplicates commands', () => {
  const selection = parseVerifyChangeArguments([
    '--profile',
    'browser',
    '--profile',
    'frontend',
    '--profile',
    'browser',
  ]);
  assert.deepEqual(selection.profiles, ['frontend', 'browser']);
  assert.deepEqual(
    buildVerifyChangePlan(selection).map((entry) => entry.id),
    [
      'pnpm:check:pattern',
      'pnpm:check:typescript-authority',
      'pnpm:lint',
      'pnpm:typecheck',
      'pnpm:test',
      'pnpm:build',
      'pnpm:e2e:gate',
    ],
  );
});

test('rejects arguments from retired authority profiles', () => {
  assert.throws(
    () =>
      parseVerifyChangeArguments([
        '--profile',
        'frontend',
        '--crate',
        'removed-owner',
      ]),
    /Unknown verify:change argument/,
  );
});

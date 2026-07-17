import { spawnSync } from 'node:child_process';
import { join } from 'node:path';

import type {
  RulebenchRuleModuleDeclarationDto,
  RulebenchRulesetDefinitionDto,
} from '@asha-rulebench/protocol';
import { describe, expect, it } from 'vitest';

import {
  authorActionResolutionModule,
  authorRulesetDefinition,
  authorTurnControlModule,
} from './ruleset-authoring';

describe('ruleset authoring helpers', () => {
  it('builds generated ruleset contracts without TypeScript validation', () => {
    const ruleset = authorRulesetDefinition({
      id: 'test.authored.ruleset',
      name: 'Test Authored Ruleset',
      version: '1.0.0',
      summary: 'A data-only authoring helper test.',
      modules: [authorActionResolutionModule({ version: '2' }), authorTurnControlModule()],
    });

    expect(ruleset.modules[0]?.version).toBe('2');
    expect(ruleset.modules).toHaveLength(2);
  });

  it('round-trips authored module declarations through Rust validation', () => {
    const valid = authorRulesetDefinition({
      id: 'test.authored.ruleset',
      name: 'Test Authored Ruleset',
      version: '1.0.0',
      summary: 'A data-only authoring helper test.',
      modules: [authorActionResolutionModule(), authorTurnControlModule()],
    });
    const invalid = authorRulesetDefinition({
      id: 'test.invalid.ruleset',
      name: 'Invalid Authored Ruleset',
      version: '1.0.0',
      summary: 'Rust must diagnose the module version.',
      modules: [authorActionResolutionModule({ version: '2' })],
    });

    expect(validateWithRust(valid)).toEqual({ exitCode: 0, output: 'accepted' });
    expect(validateWithRust(invalid)).toEqual({
      exitCode: 1,
      output: 'error:incompatibleRuleModuleVersion',
    });
  }, 30_000);
});

function validateWithRust(ruleset: RulebenchRulesetDefinitionDto): {
  readonly exitCode: number | null;
  readonly output: string;
} {
  const result = spawnSync(
    'cargo',
    [
      'run',
      '--quiet',
      '--manifest-path',
      join(process.cwd(), 'rulebench-rs', 'Cargo.toml'),
      '-p',
      'rulebench-protocol',
      '--bin',
      'validate_ruleset_authoring',
      '--',
      ruleset.id,
      ruleset.name,
      ruleset.version,
      ruleset.summary,
      ...ruleset.modules.map(moduleArgument),
    ],
    { encoding: 'utf8' },
  );
  return {
    exitCode: result.status,
    output: result.stdout.trim(),
  };
}

function moduleArgument(module: RulebenchRuleModuleDeclarationDto): string {
  const configuration = module.configuration;
  const configurationValue =
    configuration.module === 'actionResolution'
      ? configuration.targetingPolicy
      : configuration.turnOrderPolicy;
  return `${module.module}:${module.version}:${configurationValue}`;
}

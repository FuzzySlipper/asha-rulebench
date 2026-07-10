import type {
  RulebenchActionResolutionModuleConfigurationDto,
  RulebenchActionResolutionTargetingPolicyDto,
  RulebenchCheckHandlerKindDto,
  RulebenchCombatEndPolicyKindDto,
  RulebenchCombatSideIdDto,
  RulebenchRuleModuleDeclarationDto,
  RulebenchRulesetDefinitionDto,
  RulebenchTurnControlModuleConfigurationDto,
  RulebenchTurnOrderPolicyDto,
} from '@asha-rulebench/protocol';

export interface RulesetDefinitionOptions {
  readonly id: string;
  readonly name: string;
  readonly version: string;
  readonly summary: string;
  readonly modules: readonly RulebenchRuleModuleDeclarationDto[];
}

export interface ActionResolutionModuleOptions {
  readonly version?: string;
  readonly targetingPolicy?: RulebenchActionResolutionTargetingPolicyDto;
  readonly supportedCheckHandlers?: readonly RulebenchCheckHandlerKindDto[];
}

export interface TurnControlModuleOptions {
  readonly version?: string;
  readonly turnOrderPolicy?: RulebenchTurnOrderPolicyDto;
  readonly combatEndPolicy?: RulebenchCombatEndPolicyKindDto;
  readonly objectiveSide?: RulebenchCombatSideIdDto | null;
}

export function authorRulesetDefinition(options: RulesetDefinitionOptions): RulebenchRulesetDefinitionDto {
  return {
    id: options.id,
    name: options.name,
    version: options.version,
    summary: options.summary,
    modules: options.modules,
  };
}

export function authorActionResolutionModule(
  options: ActionResolutionModuleOptions = {},
): RulebenchRuleModuleDeclarationDto {
  const configuration: RulebenchActionResolutionModuleConfigurationDto = {
    module: 'actionResolution',
    targetingPolicy: options.targetingPolicy ?? 'declaredTargetsAndLineOfSight',
    supportedCheckHandlers: options.supportedCheckHandlers ?? ['attackVsDefense'],
  };
  return {
    module: 'actionResolution',
    version: options.version ?? '1',
    configuration,
  };
}

export function authorTurnControlModule(
  options: TurnControlModuleOptions = {},
): RulebenchRuleModuleDeclarationDto {
  const configuration: RulebenchTurnControlModuleConfigurationDto = {
    module: 'turnControl',
    turnOrderPolicy: options.turnOrderPolicy ?? 'explicit',
    combatEndPolicy: options.combatEndPolicy ?? 'lastSideStanding',
    objectiveSide: options.objectiveSide ?? null,
  };
  return {
    module: 'turnControl',
    version: options.version ?? '1',
    configuration,
  };
}

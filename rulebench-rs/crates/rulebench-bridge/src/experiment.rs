use std::collections::{BTreeMap, BTreeSet};

use rulebench_protocol::{
    AutomaticRunRequestDto, AutomationPolicyCatalogEntryDto, CombatAutomationPolicyDto,
    CombatControlCommandDto, CombatControlCommandKindDto, CombatSessionCreateRequestDto,
    CombatSessionHandleDto, CommandRollModeDto, ExperimentComparisonReadoutDto,
    ExperimentComparisonRequestDto, ExperimentDecisionEvidenceDto, ExperimentMatrixRequestDto,
    ExperimentMetricsDto, ExperimentReadoutDto, ExperimentTrialReadoutDto,
    PolicyRulesetCompatibilityDto, ProtocolRequestContextDto,
};
use rulebench_rules::{
    record_replay_package, verify_replay_package, CombatSessionAutomaticStepSpec,
    CombatSessionState, HitEffectOperation, COMBAT_AUTOMATION_POLICY_REGISTRY,
};

use crate::{BridgeError, BridgeErrorKind, RulebenchBridge};

const MAX_EXPERIMENT_TRIALS: usize = 16;
const MAX_EXPERIMENT_STEPS_PER_TRIAL: u32 = 64;

#[derive(Debug, Clone)]
pub(crate) struct PlannedExperimentTrial {
    scenario_id: String,
    policy: CombatAutomationPolicyDto,
    seed: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct ExperimentRecord {
    id: String,
    status: String,
    max_steps: u32,
    planned: Vec<PlannedExperimentTrial>,
    trials: Vec<ExperimentTrialReadoutDto>,
    reason: String,
}

impl ExperimentRecord {
    fn readout(&self) -> ExperimentReadoutDto {
        ExperimentReadoutDto {
            id: self.id.clone(),
            status: self.status.clone(),
            planned_trial_count: self.planned.len(),
            completed_trial_count: self.trials.len(),
            max_steps_per_trial: self.max_steps,
            trials: self.trials.clone(),
            reason: self.reason.clone(),
        }
    }
}

impl RulebenchBridge {
    pub fn automation_policy_catalog(
        &self,
        context: &ProtocolRequestContextDto,
    ) -> Result<Vec<AutomationPolicyCatalogEntryDto>, BridgeError> {
        self.check_version(context)?;
        let mut seen_rulesets = BTreeSet::new();
        let rulesets = self
            .scenarios
            .values()
            .filter_map(|scenario| {
                let ruleset = scenario.scenario.selected_ruleset()?;
                let identity = (ruleset.id.clone(), ruleset.version.clone());
                seen_rulesets.insert(identity.clone()).then_some((
                    &scenario.scenario,
                    identity.0,
                    identity.1,
                ))
            })
            .collect::<Vec<_>>();

        Ok(COMBAT_AUTOMATION_POLICY_REGISTRY
            .iter()
            .map(|registration| {
                let policy = CombatAutomationPolicyDto {
                    id: registration.id.to_string(),
                    version: registration.version,
                    no_candidate_behavior:
                        rulebench_protocol::CombatAutomationNoCandidateBehaviorDto::AdvanceTurn,
                };
                let compatibility = rulesets
                    .iter()
                    .map(|(scenario, ruleset_id, ruleset_version)| {
                        let state =
                            CombatSessionState::new("policy-catalog-probe", (*scenario).clone());
                        let plan = state.plan_automatic_step(
                            CombatSessionAutomaticStepSpec::new(
                                "policy-catalog-probe",
                                "Policy catalog probe",
                                "Read-only compatibility validation.",
                                Vec::new(),
                            )
                            .with_policy(policy.to_authority()),
                        );
                        PolicyRulesetCompatibilityDto {
                            ruleset_id: ruleset_id.clone(),
                            ruleset_version: ruleset_version.clone(),
                            compatible: plan.policy_validation.accepted,
                            code: plan.policy_validation.code.code().to_string(),
                            reason: plan.policy_validation.reason,
                        }
                    })
                    .collect();
                AutomationPolicyCatalogEntryDto {
                    id: registration.id.to_string(),
                    version: registration.version,
                    title: registration.title.to_string(),
                    summary: registration.summary.to_string(),
                    selector: registration.selector.code().to_string(),
                    requirement: registration.requirement.code().to_string(),
                    compatibility,
                }
            })
            .collect())
    }

    pub fn create_experiment(
        &mut self,
        context: &ProtocolRequestContextDto,
        request: &ExperimentMatrixRequestDto,
    ) -> Result<ExperimentReadoutDto, BridgeError> {
        self.check_version(context)?;
        validate_matrix_request(request)?;
        if self.experiments.contains_key(&request.id) {
            return Err(BridgeError::new(
                BridgeErrorKind::InvalidRequest,
                format!("Experiment id already exists: {}.", request.id),
            ));
        }

        let trial_count = request.scenario_ids.len() * request.policies.len() * request.seeds.len();
        if trial_count > MAX_EXPERIMENT_TRIALS {
            return Err(BridgeError::new(
                BridgeErrorKind::InvalidRequest,
                format!(
                    "Experiment matrix expands to {trial_count} trials; the maximum is {MAX_EXPERIMENT_TRIALS}."
                ),
            ));
        }

        let mut planned = Vec::with_capacity(trial_count);
        for scenario_id in &request.scenario_ids {
            let scenario = self.scenarios.get(scenario_id).ok_or_else(|| {
                BridgeError::new(
                    BridgeErrorKind::UnknownScenario,
                    format!("Experiment scenario does not exist: {scenario_id}."),
                )
            })?;
            if scenario.scenario.actions.iter().any(|action| {
                action
                    .hit
                    .operations
                    .iter()
                    .any(|operation| matches!(operation, HitEffectOperation::OpenReactionWindow(_)))
            }) {
                return Err(BridgeError::new(
                    BridgeErrorKind::InvalidRequest,
                    format!(
                        "Experiment scenario {scenario_id} can open a reaction window and requires the explicit manual reaction workflow."
                    ),
                ));
            }
            for policy in &request.policies {
                let state = CombatSessionState::new(
                    format!("{}-validation", request.id),
                    scenario.scenario.clone(),
                );
                let plan = state.plan_automatic_step(
                    CombatSessionAutomaticStepSpec::new(
                        "experiment-policy-validation",
                        "Experiment policy validation",
                        "Validate matrix compatibility before creating trial state.",
                        Vec::new(),
                    )
                    .with_policy(policy.to_authority()),
                );
                if !plan.policy_validation.accepted {
                    return Err(BridgeError::new(
                        BridgeErrorKind::InvalidRequest,
                        format!(
                            "Experiment policy {} is incompatible with scenario {}: {}",
                            policy.id, scenario_id, plan.policy_validation.reason
                        ),
                    ));
                }
                for seed in &request.seeds {
                    planned.push(PlannedExperimentTrial {
                        scenario_id: scenario_id.clone(),
                        policy: policy.clone(),
                        seed: *seed,
                    });
                }
            }
        }

        let record = ExperimentRecord {
            id: request.id.clone(),
            status: "planned".to_string(),
            max_steps: request.max_steps,
            planned,
            trials: Vec::new(),
            reason: "Experiment matrix is validated and ready for bounded execution.".to_string(),
        };
        let readout = record.readout();
        self.experiments.insert(request.id.clone(), record);
        Ok(readout)
    }

    pub fn list_experiments(
        &self,
        context: &ProtocolRequestContextDto,
    ) -> Result<Vec<ExperimentReadoutDto>, BridgeError> {
        self.check_version(context)?;
        Ok(self
            .experiments
            .values()
            .map(ExperimentRecord::readout)
            .collect())
    }

    pub fn get_experiment(
        &self,
        context: &ProtocolRequestContextDto,
        experiment_id: &str,
    ) -> Result<ExperimentReadoutDto, BridgeError> {
        self.check_version(context)?;
        self.experiments
            .get(experiment_id)
            .map(ExperimentRecord::readout)
            .ok_or_else(|| unknown_experiment(experiment_id))
    }

    pub fn advance_experiment(
        &mut self,
        context: &ProtocolRequestContextDto,
        experiment_id: &str,
    ) -> Result<ExperimentReadoutDto, BridgeError> {
        self.check_version(context)?;
        let mut record = self
            .experiments
            .remove(experiment_id)
            .ok_or_else(|| unknown_experiment(experiment_id))?;
        let result = if record.status == "cancelled" || record.status == "completed" {
            Ok(record.readout())
        } else {
            let index = record.trials.len();
            let planned = record.planned.get(index).cloned().ok_or_else(|| {
                BridgeError::new(
                    BridgeErrorKind::InvalidRequest,
                    "Experiment has no remaining planned trial.",
                )
            });
            match planned {
                Ok(planned) => match self
                    .execute_experiment_trial(context, &record, index, &planned)
                {
                    Ok(trial) => {
                        record.trials.push(trial);
                        if record.trials.len() == record.planned.len() {
                            record.status = "completed".to_string();
                            record.reason = "All bounded experiment trials completed.".to_string();
                        } else {
                            record.status = "running".to_string();
                            record.reason = format!(
                                "Completed {} of {} bounded trials.",
                                record.trials.len(),
                                record.planned.len()
                            );
                        }
                        Ok(record.readout())
                    }
                    Err(error) => Err(error),
                },
                Err(error) => Err(error),
            }
        };
        self.experiments.insert(experiment_id.to_string(), record);
        result
    }

    pub fn cancel_experiment(
        &mut self,
        context: &ProtocolRequestContextDto,
        experiment_id: &str,
    ) -> Result<ExperimentReadoutDto, BridgeError> {
        self.check_version(context)?;
        let record = self
            .experiments
            .get_mut(experiment_id)
            .ok_or_else(|| unknown_experiment(experiment_id))?;
        if record.status != "completed" {
            record.status = "cancelled".to_string();
            record.reason = format!(
                "Experiment cancelled after {} of {} trials.",
                record.trials.len(),
                record.planned.len()
            );
        }
        Ok(record.readout())
    }

    pub fn compare_experiment_trials(
        &self,
        context: &ProtocolRequestContextDto,
        request: &ExperimentComparisonRequestDto,
    ) -> Result<ExperimentComparisonReadoutDto, BridgeError> {
        self.check_version(context)?;
        let expected =
            self.find_trial(&request.expected_experiment_id, &request.expected_trial_id)?;
        let actual = self.find_trial(&request.actual_experiment_id, &request.actual_trial_id)?;
        let shared = expected.decisions.len().min(actual.decisions.len());
        let first_divergence_index = (0..shared)
            .find(|index| expected.decisions[*index] != actual.decisions[*index])
            .or_else(|| {
                (expected.decisions.len() != actual.decisions.len()
                    || expected.final_state_fingerprint != actual.final_state_fingerprint)
                    .then_some(shared)
            });
        let identical = first_divergence_index.is_none();
        Ok(ExperimentComparisonReadoutDto {
            identical,
            first_divergence_index,
            expected_trial_id: expected.id.clone(),
            actual_trial_id: actual.id.clone(),
            expected_evidence: first_divergence_index
                .and_then(|index| expected.decisions.get(index).cloned()),
            actual_evidence: first_divergence_index
                .and_then(|index| actual.decisions.get(index).cloned()),
            reason: if identical {
                "Trials have identical deterministic decision and final-state evidence.".to_string()
            } else {
                format!(
                    "Trials first diverge at decision evidence index {}.",
                    first_divergence_index.unwrap_or(shared)
                )
            },
        })
    }

    fn execute_experiment_trial(
        &mut self,
        context: &ProtocolRequestContextDto,
        record: &ExperimentRecord,
        index: usize,
        planned: &PlannedExperimentTrial,
    ) -> Result<ExperimentTrialReadoutDto, BridgeError> {
        let trial_id = format!("{}-trial-{index:03}", record.id);
        let session_id = format!("experiment-session-{trial_id}");
        let created = self.create_session(
            context,
            &CombatSessionCreateRequestDto {
                session_id: session_id.clone(),
                scenario_id: planned.scenario_id.clone(),
                participant_order: Vec::new(),
                content_pack: None,
            },
        )?;
        let run = self.automatic_run(
            context,
            &CombatSessionHandleDto {
                id: session_id.clone(),
            },
            &AutomaticRunRequestDto {
                id: format!("{trial_id}-automatic-run"),
                title: format!("Experiment trial {index}"),
                summary: "Bounded deterministic policy laboratory trial.".to_string(),
                max_steps: record.max_steps,
                roll_stream: Vec::new(),
                policy: planned.policy.clone(),
                roll_mode: CommandRollModeDto::AuthorityGenerated,
                generated_seed: Some(planned.seed),
            },
        )?;
        let session_handle = CombatSessionHandleDto {
            id: session_id.clone(),
        };
        if run.final_snapshot.finalization.is_none() {
            self.submit_control(
                context,
                &session_handle,
                &CombatControlCommandDto {
                    kind: CombatControlCommandKindDto::ExplicitEnd,
                },
            )?;
        }
        let final_snapshot = self.get_session(context, &session_handle)?;

        let recording = self.recordings.get(&session_id).cloned().ok_or_else(|| {
            BridgeError::new(
                BridgeErrorKind::InvalidRequest,
                "Experiment session recording does not exist.",
            )
        })?;
        let ruleset = recording
            .initial_session
            .scenario
            .selected_ruleset()
            .ok_or_else(|| {
                BridgeError::new(
                    BridgeErrorKind::InvalidScenario,
                    "Experiment scenario ruleset does not exist.",
                )
            })?
            .artifact_provenance();
        let replay_package_id = format!("experiment-{trial_id}");
        let package = record_replay_package(
            &replay_package_id,
            recording.initial_session.clone(),
            ruleset,
            recording.commands.clone(),
        );
        let verification = verify_replay_package(&package);
        self.replays
            .save(package, format!("experiment:{}", record.id))
            .map_err(BridgeError::from_replay_error)?;
        self.recovery
            .delete(&session_id)
            .map_err(BridgeError::from_recovery_storage_error)?;
        self.sessions
            .close_session(&rulebench_rules::CombatSessionHandle::new(&session_id))
            .map_err(BridgeError::from_session_error)?;
        self.recordings.remove(&session_id);

        let scenario = self.scenarios.get(&planned.scenario_id).ok_or_else(|| {
            BridgeError::new(
                BridgeErrorKind::UnknownScenario,
                "Experiment scenario disappeared.",
            )
        })?;
        let selected_ruleset = scenario.scenario.selected_ruleset().ok_or_else(|| {
            BridgeError::new(
                BridgeErrorKind::InvalidScenario,
                "Experiment ruleset disappeared.",
            )
        })?;
        let decisions = run
            .policy_decisions
            .iter()
            .enumerate()
            .map(|(decision_index, decision)| ExperimentDecisionEvidenceDto {
                index: decision_index,
                state_before_fingerprint: decision.state_before_fingerprint.value.clone(),
                operation_kind: decision.operation_kind.map(|kind| kind.code().to_string()),
                selected_action_id: decision.selected_action_id.clone(),
                selected_target_id: decision.selected_target_id.clone(),
                selected_candidate_index: decision.selected_candidate_index,
                candidate_count: decision.candidate_count,
                accepted_candidate_count: decision.accepted_candidate_count,
                reason: decision.reason.clone(),
            })
            .collect();
        let initial_total_hit_points = created
            .snapshot
            .current_state
            .combatants
            .iter()
            .map(|combatant| combatant.hit_points.current)
            .sum::<i32>();
        let final_total_hit_points = final_snapshot
            .current_state
            .combatants
            .iter()
            .map(|combatant| combatant.hit_points.current)
            .sum::<i32>();
        let accepted_command_count = run
            .steps
            .iter()
            .filter(|step| {
                step.auto_candidate
                    .as_ref()
                    .and_then(|execution| execution.submitted_step.as_ref())
                    .is_some_and(|submitted| submitted.receipt.accepted)
            })
            .count();
        let materialized_rolls = run
            .steps
            .iter()
            .filter_map(|step| {
                step.auto_candidate
                    .as_ref()
                    .and_then(|execution| execution.submitted_step.as_ref())
            })
            .flat_map(|submitted| submitted.command.roll_stream.iter().copied())
            .collect();
        Ok(ExperimentTrialReadoutDto {
            id: trial_id,
            scenario_id: planned.scenario_id.clone(),
            ruleset_id: selected_ruleset.id.clone(),
            ruleset_version: selected_ruleset.version.clone(),
            content_pack_id: scenario.option.content_pack_id.clone(),
            content_pack_version: scenario.option.content_pack_version.clone(),
            policy_id: planned.policy.id.clone(),
            policy_version: planned.policy.version,
            policy_no_candidate_behavior: match planned.policy.no_candidate_behavior {
                rulebench_protocol::CombatAutomationNoCandidateBehaviorDto::AdvanceTurn => {
                    "advanceTurn".to_string()
                }
                rulebench_protocol::CombatAutomationNoCandidateBehaviorDto::StopRun => {
                    "stopRun".to_string()
                }
            },
            seed: planned.seed,
            max_steps: record.max_steps,
            accepted: run.accepted,
            stop_reason: run.decision_kind.code().to_string(),
            finalization_outcome: final_snapshot
                .finalization
                .as_ref()
                .map(|finalization| finalization.outcome_kind.code().to_string()),
            initial_state_fingerprint: created.snapshot.current_state_fingerprint.value,
            final_state_fingerprint: final_snapshot.current_state_fingerprint.value.clone(),
            materialized_rolls,
            decisions,
            metrics: ExperimentMetricsDto {
                executed_step_count: run.executed_step_count,
                accepted_command_count,
                initial_total_hit_points,
                final_total_hit_points,
                observed_hit_point_delta: initial_total_hit_points - final_total_hit_points,
                audit_entry_count: final_snapshot.audit_log.len(),
                combat_log_entry_count: final_snapshot.combat_log.len(),
            },
            replay_package_id,
            replay_verified: verification.accepted,
        })
    }

    fn find_trial(
        &self,
        experiment_id: &str,
        trial_id: &str,
    ) -> Result<&ExperimentTrialReadoutDto, BridgeError> {
        self.experiments
            .get(experiment_id)
            .ok_or_else(|| unknown_experiment(experiment_id))?
            .trials
            .iter()
            .find(|trial| trial.id == trial_id)
            .ok_or_else(|| {
                BridgeError::new(
                    BridgeErrorKind::InvalidRequest,
                    format!("Experiment trial does not exist: {trial_id}."),
                )
            })
    }
}

fn validate_matrix_request(request: &ExperimentMatrixRequestDto) -> Result<(), BridgeError> {
    if request.id.is_empty()
        || request.scenario_ids.is_empty()
        || request.policies.is_empty()
        || request.seeds.is_empty()
    {
        return Err(BridgeError::new(
            BridgeErrorKind::InvalidRequest,
            "Experiment id, scenarios, policies, and seeds must not be empty.",
        ));
    }
    if request.max_steps == 0 || request.max_steps > MAX_EXPERIMENT_STEPS_PER_TRIAL {
        return Err(BridgeError::new(
            BridgeErrorKind::InvalidRequest,
            format!("Experiment max steps must be between 1 and {MAX_EXPERIMENT_STEPS_PER_TRIAL}."),
        ));
    }
    Ok(())
}

fn unknown_experiment(experiment_id: &str) -> BridgeError {
    BridgeError::new(
        BridgeErrorKind::InvalidRequest,
        format!("Experiment does not exist: {experiment_id}."),
    )
}

pub(crate) fn empty_experiment_registry() -> BTreeMap<String, ExperimentRecord> {
    BTreeMap::new()
}

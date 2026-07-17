//! Bounded, typed reaction-window lifecycle and deterministic resumption.

use super::*;

impl CombatSessionState {
    pub fn current_reaction_window(&self) -> Option<&ReactionWindowReadout> {
        self.reaction_window_stack
            .last()
            .map(|window| &window.readout)
    }

    pub fn reaction_window_lifecycle_log(&self) -> &[ReactionWindowLifecycleEntry] {
        &self.reaction_window_lifecycle_log
    }

    pub fn reaction_audit_log(&self) -> &[ReactionAuditEntry] {
        &self.reaction_audit_log
    }

    pub fn submit_reaction_command(
        &mut self,
        command: ReactionCommandSpec,
    ) -> ReactionCommandReadout {
        let previous_window = self.current_reaction_window().cloned();
        let rejection = self.validate_reaction_command(&command);
        if let Some((decision_kind, reason)) = rejection {
            return self.rejected_reaction_command(command, previous_window, decision_kind, reason);
        }

        let option = command.option_id.as_ref().and_then(|option_id| {
            self.current_reaction_window().and_then(|window| {
                window
                    .options
                    .iter()
                    .find(|option| option.option_id == *option_id)
                    .cloned()
            })
        });
        let response = ReactionResponseEntry {
            sequence: self
                .current_reaction_window()
                .map_or(0, |window| window.responses.len() as u32),
            reactor_id: command.reactor_id.clone(),
            response_kind: command.response_kind,
            option_id: command.option_id.clone(),
        };
        let mut trace = vec![
            TraceEntry::new(
                0,
                TracePhase::Proposal,
                TraceStatus::Accepted,
                "Reaction response proposed.",
                format!(
                    "Window {} received {} from {}.",
                    command.window_id,
                    command.response_kind.code(),
                    command.reactor_id
                ),
            ),
            TraceEntry::new(
                1,
                TracePhase::Validation,
                TraceStatus::Accepted,
                "Reaction response validated.",
                "Window identity, ordering, and option were current.",
            ),
        ];
        let window = self
            .reaction_window_stack
            .last_mut()
            .expect("validated reaction window");
        window.readout.responses.push(response);
        let lifecycle_window = window.readout.clone();
        self.record_reaction_lifecycle(
            ReactionWindowLifecycleKind::ResponseAccepted,
            &lifecycle_window,
            Some(command.reactor_id.clone()),
            command.option_id.clone(),
            "Reaction response accepted in deterministic reactor order.",
        );
        self.advance_current_reaction_window();

        let opened_nested_window = option
            .filter(|option| option.opens_nested_window)
            .map(|_| self.open_nested_reaction_window(&lifecycle_window));
        let resumed_pending_resolution = if opened_nested_window.is_none() {
            self.close_resolved_reaction_windows()
        } else {
            false
        };
        let next_window = self.current_reaction_window().cloned();
        let reason = if opened_nested_window.is_some() {
            "Reaction accepted and a nested reaction window opened."
        } else if resumed_pending_resolution {
            "Reaction response resolved the root window and resumed pending resolution."
        } else {
            "Reaction response accepted; the next eligible reactor may respond."
        }
        .to_string();
        trace.push(TraceEntry::new(
            2,
            TracePhase::Resolution,
            TraceStatus::Accepted,
            "Reaction response resolved.",
            reason.clone(),
        ));
        trace.push(TraceEntry::new(
            3,
            TracePhase::Commit,
            TraceStatus::Accepted,
            "Reaction response committed.",
            "Reaction lifecycle, audit, and pending-resolution state were updated.",
        ));
        self.reaction_audit_log.push(ReactionAuditEntry {
            sequence: self.reaction_audit_log.len() as u32,
            window_id: command.window_id.clone(),
            reactor_id: command.reactor_id.clone(),
            response_kind: command.response_kind,
            option_id: command.option_id.clone(),
            accepted: true,
            decision_kind: ReactionDecisionKind::Accepted,
            trace: trace.clone(),
            reason: reason.clone(),
        });

        ReactionCommandReadout {
            command,
            accepted: true,
            decision_kind: ReactionDecisionKind::Accepted,
            previous_window,
            next_window,
            opened_nested_window,
            resumed_pending_resolution,
            trace,
            reason,
        }
    }

    pub(super) fn open_reaction_window(
        &mut self,
        hook: &ReactionHookEffectOperation,
        step: &CombatSessionStepSummary,
        action_id: &str,
    ) -> Option<ReactionWindowReadout> {
        let eligible_reactor_ids = self
            .turn_order
            .participant_order
            .iter()
            .filter(|reactor_id| hook.eligible_reactor_ids.contains(reactor_id))
            .cloned()
            .collect::<Vec<_>>();
        if eligible_reactor_ids.is_empty() {
            return None;
        }
        let readout = ReactionWindowReadout {
            id: format!(
                "reaction-window-{}",
                self.reaction_window_lifecycle_log.len()
            ),
            hook_id: hook.hook_id.clone(),
            timing: hook.window,
            depth: 0,
            maximum_nested_depth: hook.maximum_nested_depth,
            parent_window_id: None,
            trigger_step_id: step.id.clone(),
            trigger_action_id: action_id.to_string(),
            current_reactor_id: eligible_reactor_ids.first().cloned(),
            eligible_reactor_ids,
            options: hook
                .options
                .iter()
                .map(|option| ReactionOptionReadout {
                    option_id: option.id.clone(),
                    reactor_id: option.reactor_id.clone(),
                    opens_nested_window: option.opens_nested_window,
                })
                .collect(),
            responses: Vec::new(),
            status: ReactionWindowStatus::Open,
        };
        self.reaction_window_stack.push(ActiveReactionWindow {
            readout: readout.clone(),
            current_reactor_index: 0,
        });
        self.record_reaction_lifecycle(
            ReactionWindowLifecycleKind::Opened,
            &readout,
            None,
            None,
            "Reaction window opened from an authored action hook.",
        );
        Some(readout)
    }

    fn validate_reaction_command(
        &self,
        command: &ReactionCommandSpec,
    ) -> Option<(ReactionDecisionKind, String)> {
        let Some(window) = self.current_reaction_window() else {
            return Some((
                ReactionDecisionKind::RejectedNoOpenWindow,
                "No reaction window is open.".to_string(),
            ));
        };
        if window.id != command.window_id {
            return Some((
                ReactionDecisionKind::RejectedStaleWindow,
                "Reaction command targets a stale or paused window.".to_string(),
            ));
        }
        if window.current_reactor_id.as_deref() != Some(command.reactor_id.as_str()) {
            return Some((
                ReactionDecisionKind::RejectedOutOfOrder,
                "Reaction command is not from the current eligible reactor.".to_string(),
            ));
        }
        match command.response_kind {
            ReactionResponseKind::Pass if command.option_id.is_some() => Some((
                ReactionDecisionKind::RejectedInvalidOption,
                "Pass responses cannot include an option id.".to_string(),
            )),
            ReactionResponseKind::Accept => {
                let option = command.option_id.as_ref().and_then(|option_id| {
                    window.options.iter().find(|option| {
                        option.option_id == *option_id && option.reactor_id == command.reactor_id
                    })
                });
                let Some(option) = option else {
                    return Some((
                        ReactionDecisionKind::RejectedInvalidOption,
                        "Accepted reaction option is not eligible for the current reactor."
                            .to_string(),
                    ));
                };
                if option.opens_nested_window && window.depth >= window.maximum_nested_depth {
                    return Some((
                        ReactionDecisionKind::RejectedNestedLimit,
                        "Reaction option would exceed the authored nested-window limit."
                            .to_string(),
                    ));
                }
                None
            }
            ReactionResponseKind::Pass => None,
        }
    }

    fn advance_current_reaction_window(&mut self) {
        let window = self
            .reaction_window_stack
            .last_mut()
            .expect("open reaction window");
        window.current_reactor_index += 1;
        window.readout.current_reactor_id = window
            .readout
            .eligible_reactor_ids
            .get(window.current_reactor_index)
            .cloned();
        if window.readout.current_reactor_id.is_none() {
            window.readout.status = ReactionWindowStatus::Resolved;
        }
    }

    fn open_nested_reaction_window(
        &mut self,
        parent: &ReactionWindowReadout,
    ) -> ReactionWindowReadout {
        let readout = ReactionWindowReadout {
            id: format!(
                "reaction-window-{}",
                self.reaction_window_lifecycle_log.len()
            ),
            hook_id: format!("{}.nested", parent.hook_id),
            timing: parent.timing,
            depth: parent.depth + 1,
            maximum_nested_depth: parent.maximum_nested_depth,
            parent_window_id: Some(parent.id.clone()),
            trigger_step_id: parent.trigger_step_id.clone(),
            trigger_action_id: parent.trigger_action_id.clone(),
            eligible_reactor_ids: parent.eligible_reactor_ids.clone(),
            current_reactor_id: parent.eligible_reactor_ids.first().cloned(),
            options: parent.options.clone(),
            responses: Vec::new(),
            status: ReactionWindowStatus::Open,
        };
        self.reaction_window_stack.push(ActiveReactionWindow {
            readout: readout.clone(),
            current_reactor_index: 0,
        });
        self.record_reaction_lifecycle(
            ReactionWindowLifecycleKind::NestedOpened,
            &readout,
            None,
            None,
            "Nested reaction window opened within the authored depth bound.",
        );
        readout
    }

    fn close_resolved_reaction_windows(&mut self) -> bool {
        let mut resolved_root = None;
        while self
            .reaction_window_stack
            .last()
            .is_some_and(|window| window.readout.status == ReactionWindowStatus::Resolved)
        {
            let resolved = self
                .reaction_window_stack
                .pop()
                .expect("resolved reaction window")
                .readout;
            if resolved.parent_window_id.is_none() {
                resolved_root = Some(resolved.clone());
            }
            self.record_reaction_lifecycle(
                ReactionWindowLifecycleKind::Resolved,
                &resolved,
                None,
                None,
                "All eligible reactors responded; reaction window resolved.",
            );
        }
        if self.reaction_window_stack.is_empty() && self.pending_reaction_resolution.is_some() {
            self.resume_pending_reaction_resolution(
                resolved_root
                    .as_ref()
                    .expect("pending resolution has a resolved root reaction window"),
            );
            true
        } else {
            if self.reaction_window_stack.is_empty() {
                self.finalize_if_condition_met();
            }
            false
        }
    }

    fn resume_pending_reaction_resolution(&mut self, resolved_root: &ReactionWindowReadout) {
        let mut pending = self
            .pending_reaction_resolution
            .take()
            .expect("pending reaction resolution");
        let accepted_response = resolved_root
            .responses
            .iter()
            .find(|response| response.response_kind == ReactionResponseKind::Accept);
        let accepted = accepted_response.is_some();
        let option_id = accepted_response.and_then(|response| response.option_id.clone());
        {
            let mut owner = CombatPreEffectOwner {
                state: &mut self.state,
                receipt: &mut pending.receipt,
                actor_id: &pending.actor_id,
                action_id: &resolved_root.trigger_action_id,
            };
            self.rpg_authority
                .resolve_before_effect(
                    &pending.gameplay_continuation,
                    accepted,
                    option_id,
                    &mut owner,
                )
                .expect("static gameplay continuation resolves")
        };
        for target in &pending.receipt.target_results {
            for resource in &target.resource_changes {
                self.record_effect_resource_transition(&pending.step, resource);
            }
        }
        for resource in rpg_resource_outcomes(&self.state, &pending.receipt) {
            self.state.apply_resource_change(&resource);
            self.record_effect_resource_transition(&pending.step, &resource);
        }
        for cost in &pending.resource_costs {
            let spend =
                self.state
                    .spend_action_resource(&pending.actor_id, &cost.resource_id, cost.amount);
            self.record_action_resource_spend_transition(&pending.step, &spend);
        }
        self.record_reaction_lifecycle(
            ReactionWindowLifecycleKind::ResolutionResumed,
            resolved_root,
            None,
            None,
            "Pending action effects and resource costs resumed after the root reaction window.",
        );
        self.finalize_if_condition_met();
    }

    fn rejected_reaction_command(
        &mut self,
        command: ReactionCommandSpec,
        previous_window: Option<ReactionWindowReadout>,
        decision_kind: ReactionDecisionKind,
        reason: String,
    ) -> ReactionCommandReadout {
        let trace = vec![TraceEntry::new(
            0,
            TracePhase::Validation,
            TraceStatus::Rejected,
            "Reaction response rejected.",
            reason.clone(),
        )];
        if self.lifecycle.phase != CombatLifecyclePhase::Ended {
            self.reaction_audit_log.push(ReactionAuditEntry {
                sequence: self.reaction_audit_log.len() as u32,
                window_id: command.window_id.clone(),
                reactor_id: command.reactor_id.clone(),
                response_kind: command.response_kind,
                option_id: command.option_id.clone(),
                accepted: false,
                decision_kind,
                trace: trace.clone(),
                reason: reason.clone(),
            });
        }
        ReactionCommandReadout {
            command,
            accepted: false,
            decision_kind,
            previous_window: previous_window.clone(),
            next_window: previous_window,
            opened_nested_window: None,
            resumed_pending_resolution: false,
            trace,
            reason,
        }
    }

    fn record_reaction_lifecycle(
        &mut self,
        lifecycle_kind: ReactionWindowLifecycleKind,
        window: &ReactionWindowReadout,
        reactor_id: Option<String>,
        option_id: Option<String>,
        reason: &str,
    ) {
        self.reaction_window_lifecycle_log
            .push(ReactionWindowLifecycleEntry {
                sequence: self.reaction_window_lifecycle_log.len() as u32,
                lifecycle_kind,
                window_id: window.id.clone(),
                parent_window_id: window.parent_window_id.clone(),
                depth: window.depth,
                reactor_id,
                option_id,
                reason: reason.to_string(),
            });
    }
}

struct CombatPreEffectOwner<'a> {
    state: &'a mut CombatState,
    receipt: &'a mut RulebenchReceipt,
    actor_id: &'a str,
    action_id: &'a str,
}

impl RpgPreEffectOwner for CombatPreEffectOwner<'_> {
    fn revision_hash(&self) -> String {
        let projection = self.state.project("Gameplay pre-effect owner revision.");
        let fingerprint = fingerprint_projected_state(&projection);
        format!("{}:{}", fingerprint.algorithm, fingerprint.value)
    }

    fn validate_commit(&self, workspace: &PreEffectWorkspace) -> Result<(), Vec<String>> {
        let Some(damage) = self.receipt.damage.as_ref() else {
            return Err(vec!["missingPendingDamage".to_owned()]);
        };
        if workspace.actor_id != self.actor_id
            || workspace.action_id != self.action_id
            || workspace.target_id != damage.target_id
            || workspace.damage_type != damage.damage_type
        {
            return Err(vec!["preEffectWorkspaceIdentityMismatch".to_owned()]);
        }
        let amount = i32::try_from(workspace.damage_amount)
            .map_err(|_| vec!["preEffectDamageOutOfRange".to_owned()])?;
        if amount > damage.amount {
            return Err(vec!["preEffectDamageIncreaseRejected".to_owned()]);
        }
        Ok(())
    }

    fn commit(&mut self, workspace: &PreEffectWorkspace) -> Vec<String> {
        let mut damage = self
            .receipt
            .damage
            .clone()
            .expect("validated pre-effect commit retains damage evidence");
        let amount = i32::try_from(workspace.damage_amount)
            .expect("validated pre-effect damage remains in range");
        damage.amount = amount;
        damage.after.current = damage
            .before
            .current
            .saturating_sub(amount)
            .clamp(0, damage.before.max);
        self.receipt.damage = Some(damage.clone());
        if let Some(target) = self
            .receipt
            .target_results
            .iter_mut()
            .find(|target| target.target_id == damage.target_id)
        {
            target.damage = Some(damage);
        }
        if self.receipt.target_results.is_empty() {
            let damage = self
                .receipt
                .damage
                .as_ref()
                .expect("pending legacy damage remains present");
            self.state.apply_hit(damage, self.receipt.modifier.as_ref());
        } else {
            apply_target_results_to_state(self.state, self.receipt);
        }
        for resource in rpg_resource_outcomes(self.state, self.receipt) {
            self.state.apply_resource_change(&resource);
        }
        let fingerprint =
            fingerprint_projected_state(&self.state.project("Gameplay pre-effect owner commit."));
        vec![format!("{}:{}", fingerprint.algorithm, fingerprint.value)]
    }
}

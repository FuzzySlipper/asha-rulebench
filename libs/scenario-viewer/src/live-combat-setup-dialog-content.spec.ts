import { Injector, runInInjectionContext, signal } from "@angular/core";
import { describe, expect, it } from "vitest";
import type { RulebenchContentActionBindingCandidateView } from "@asha-rulebench/domain";
import type {
  RulebenchAuthoredActionBindingRequestDto,
  RulebenchContentPackReferenceDto,
  RulebenchLiveTransportErrorDto,
} from "@asha-rulebench/protocol";
import {
  ContentWorkbenchStore,
  LiveCombatStore,
  type AsyncState,
} from "@asha-rulebench/store";
import { LiveCombatSetupDialogContentComponent } from "./live-combat-setup-dialog-content";

const contentPack: RulebenchContentPackReferenceDto = {
  id: "pack.authored.v3",
  version: "1.0.0",
  fingerprint: { algorithm: "fnv1a64", value: "exact-pack" },
};

const binding: RulebenchContentActionBindingCandidateView = {
  key: "exact-pack:action.binding-glyph",
  contentPack,
  packLabel: "pack.authored.v3@1.0.0",
  actionId: "action.binding-glyph",
  actionName: "Binding Glyph",
  actionLabel: "Binding Glyph · action.binding-glyph",
  abilityId: "ability.binding-glyph",
  scenarios: [
    {
      id: "scenario",
      title: "Scenario",
      actors: [
        { id: "entity-warden", name: "Warden", label: "Warden · entity-warden" },
      ],
    },
  ],
};

class LiveCombatSetupHarness extends LiveCombatSetupDialogContentComponent {
  connectForTest(): void {
    this.connect();
  }

  selectAuthoredActionForTest(key: string): void {
    this.selectAuthoredAction(key);
  }

  canCreateSessionForTest(): boolean {
    return this.canCreateSession();
  }

  createSessionForTest(): void {
    this.createSession();
  }
}

describe("LiveCombatSetupDialogContentComponent", () => {
  it("never drops an explicit authored binding during a pending or failed Connect refresh", async () => {
    let resolveReload:
      | ((error: RulebenchLiveTransportErrorDto) => void)
      | null = null;
    const pendingReload = new Promise<RulebenchLiveTransportErrorDto>((resolve) => {
      resolveReload = resolve;
    });
    const connection = signal<
      AsyncState<
        {
          protocolId: string;
          protocolVersion: number;
          authoritySurface: string;
        },
        RulebenchLiveTransportErrorDto
      >
    >({ kind: "idle" });
    const scenarios = signal({
      kind: "data" as const,
      value: [
        {
          id: "scenario",
          title: "Scenario",
          summary: "Test scenario.",
          rulesetId: "rules.test",
          rulesetVersion: "1.0.0",
          contentPackId: null,
          contentPackVersion: null,
          participants: [
            {
              id: "entity-warden",
              name: "Warden",
              sideId: "allies",
              initiative: 10,
            },
          ],
        },
      ],
    });
    const selectedScenarioId = signal<string | null>("scenario");
    const createdBindings: (RulebenchAuthoredActionBindingRequestDto | null)[] = [];
    const liveStore = {
      connection: connection.asReadonly(),
      scenarios: scenarios.asReadonly(),
      sessions: signal({ kind: "data" as const, value: [] }).asReadonly(),
      recovery: signal({
        kind: "data" as const,
        value: { sessions: [], issues: [] },
      }).asReadonly(),
      snapshot: signal({ kind: "idle" as const }).asReadonly(),
      selectedScenarioId: selectedScenarioId.asReadonly(),
      selectedSessionId: signal<string | null>(null).asReadonly(),
      connect: async () => {
        connection.set({
          kind: "data",
          value: {
            protocolId: "asha-rulebench.protocol",
            protocolVersion: 9,
            authoritySurface: "test-authority",
          },
        });
      },
      loadScenarios: async () => undefined,
      loadSessions: async () => undefined,
      loadRecovery: async () => undefined,
      createSession: async (
        _sessionId: string,
        _scenarioId: string,
        _participantOrder: readonly string[],
        _selectedContentPack: RulebenchContentPackReferenceDto | null,
        authoredActionBinding: RulebenchAuthoredActionBindingRequestDto | null,
      ) => {
        createdBindings.push(authoredActionBinding);
      },
    };
    const bindingCatalog = signal<
      AsyncState<
        readonly RulebenchContentActionBindingCandidateView[],
        RulebenchLiveTransportErrorDto
      >
    >({ kind: "idle" });
    let bindingRequestCount = 0;
    const contentStore = {
      workspace: signal({
        kind: "data" as const,
        value: { packs: [], audit: [] },
      }).asReadonly(),
      bindingCatalog: bindingCatalog.asReadonly(),
      loadWorkspace: async () => undefined,
      loadBindingCatalog: async () => {
        bindingRequestCount += 1;
        if (bindingRequestCount === 1) {
          bindingCatalog.set({ kind: "data", value: [binding] });
          return;
        }
        bindingCatalog.set({ kind: "loading" });
        const error = await pendingReload;
        bindingCatalog.set({ kind: "error", error });
      },
    };
    const injector = Injector.create({
      providers: [
        { provide: LiveCombatStore, useValue: liveStore },
        { provide: ContentWorkbenchStore, useValue: contentStore },
      ],
    });
    const component = runInInjectionContext(
      injector,
      () => new LiveCombatSetupHarness(),
    );

    component.ngOnInit();
    await waitFor(() => bindingCatalog().kind === "data");
    component.selectAuthoredActionForTest(binding.key);
    expect(component.canCreateSessionForTest()).toBe(true);

    component.connectForTest();
    await waitFor(() => bindingCatalog().kind === "loading");
    expect(component.canCreateSessionForTest()).toBe(false);
    component.createSessionForTest();
    expect(createdBindings).toEqual([]);

    const completeReload = resolveReload;
    if (completeReload === null) throw new Error("catalog reload was not requested");
    completeReload(
      transportError(
        "bindingCatalogUnavailable",
        "Binding catalog refresh failed.",
      ),
    );
    await waitFor(() => bindingCatalog().kind === "error");
    expect(component.canCreateSessionForTest()).toBe(false);
    component.createSessionForTest();
    expect(createdBindings).toEqual([]);
  });
});

function transportError(
  code: string,
  message: string,
): RulebenchLiveTransportErrorDto {
  return { kind: "fake", code, message, retryable: false };
}

async function waitFor(predicate: () => boolean): Promise<void> {
  for (let attempt = 0; attempt < 20; attempt += 1) {
    if (predicate()) return;
    await new Promise<void>((resolve) => {
      globalThis.setTimeout(resolve, 0);
    });
  }
  throw new Error("Timed out waiting for asynchronous component state.");
}

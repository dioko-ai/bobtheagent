# Functional Audit Report (Numbered)
Date: 2026-02-17
Scope: Functional and logical correctness only (no security review)

## 1. Final audit output is not evaluated
Severity: Critical

Evidence:
- `src/workflow.rs:639` marks final audit done whenever process exit is successful.
- Unlike implementor/test-writer auditors, final audit completion does not inspect transcript content for `AUDIT_RESULT` or issue keywords.

Remedy:
1. Evaluate final-audit transcript with the same token-first protocol used for other auditors.
2. If transcript indicates failure (`AUDIT_RESULT: FAIL` or equivalent), keep final audit in `NeedsChanges`.
3. Only mark final audit `Done` when transcript explicitly passes.

Test gaps to fill:
1. Add a regression test where final-audit process exits 0 but transcript includes `AUDIT_RESULT: FAIL`; verify final audit is not marked done.
2. Add a pass test requiring `AUDIT_RESULT: PASS` to complete final audit.

## 2. Final audit retries are unbounded
Severity: Critical

Evidence:
- `src/workflow.rs:646` to `src/workflow.rs:657` always requeues failed final audit with `pass + 1`.
- No max retry cap exists for final audit (other loops have `MAX_AUDIT_RETRIES` or `MAX_TEST_RETRIES`).

Remedy:
1. Add a retry cap for final audit (for example `MAX_FINAL_AUDIT_RETRIES`).
2. On exhaustion, record a failure entry and stop requeueing.
3. Emit clear user/system messaging indicating exhaustion and next action.

Test gaps to fill:
1. Add a regression test showing repeated final-audit failures stop at cap.
2. Assert failure is recorded in `recent_failures` and no additional final-audit jobs are queued.

## 3. Top task completion checks only the first implementor/test-writer branch
Severity: High

Evidence:
- `src/workflow.rs:1391` and `src/workflow.rs:1397` use `.find(...)`, not an all-branches check.
- Structure validation does not enforce uniqueness of implementor/test-writer siblings (`src/workflow.rs:1653`).

Remedy:
1. Change completion logic to require all implementor children to be `Done`.
2. If test-writer branches exist, require all test-writer children to be `Done`.
3. Optionally enforce single implementor-per-top constraint during validation if that is intended.

Test gaps to fill:
1. Add regression test with one top task containing two test-writer siblings; complete one and verify top task is still not done.
2. Add regression test with multiple implementor branches and verify completion waits for all branches.

## 4. Final-audit add/remove commands can overwrite tasks on read/parse failure
Severity: High

Evidence:
- `src/main.rs:1579` and `src/main.rs:1598` call `read_tasks().unwrap_or_default()`.
- Command then writes resulting vector back to disk (`src/main.rs:1583`, `src/main.rs:1603`), which can clobber valid data if read failed.

Remedy:
1. Replace `unwrap_or_default()` with explicit error handling.
2. Abort command when `tasks.json` cannot be read/parsed and surface a user-visible error.
3. Never write a mutated fallback vector when source data is unavailable.

Test gaps to fill:
1. Add regression test where `read_tasks()` fails; assert command returns without writing and reports error.
2. Add regression test where malformed `tasks.json` does not get replaced by empty/default content.

## 5. Master/master-report dispatch is not serialized (possible transcript interleaving)
Severity: High

Evidence:
- `CodexAdapter::send_prompt` always spawns a thread/process (`src/agent.rs:94`, `src/agent.rs:98`).
- Main loop appends outputs into shared transcripts (`src/main.rs:256`, `src/main.rs:302`) without in-flight guard per adapter.
- Input blocking does not gate on `master_in_progress` (`src/main.rs:1876`), allowing concurrent sends.

Remedy:
1. Introduce per-adapter in-flight state and block or queue additional sends until completion.
2. Refuse or queue user submissions while master is active.
3. Isolate transcript state by request id to avoid mixed completion parsing.

Test gaps to fill:
1. Add regression test that simulates multiple master sends before completion and verifies no mixed transcript state.
2. Add test proving second send is blocked or queued until first completion.

## 6. Session directory name collision risk within one-second window
Severity: Medium

Evidence:
- `src/session_store.rs:162` names directories as `{epoch_seconds}-{workspace}`.
- `fs::create_dir_all` at `src/session_store.rs:163` permits existing dir reuse.

Remedy:
1. Add uniqueness suffix (millis, monotonic counter, or random token) to session dir naming.
2. Alternatively, retry with incremented suffix on existing path.

Test gaps to fill:
1. Add regression test that creates two sessions with same timestamp/workspace and verifies distinct directories.

## 7. Missing `test_command` is treated as successful deterministic test run
Severity: Medium

Evidence:
- `src/deterministic.rs:66` to `src/deterministic.rs:73` emits success completion when command is missing.
- Workflow then treats branch as passing path in test-runner completion handlers.

Remedy:
1. Treat missing `test_command` as explicit non-pass state for branches that require test execution.
2. Route to `NeedsChanges` or a setup-required status with actionable feedback.
3. Reserve "skip with success" for branches explicitly configured to ignore tests.

Test gaps to fill:
1. Add regression test where a required test-runner branch with missing command does not mark branch/task done.
2. Add test for explicit ignore-tests branch to preserve intended skip behavior.

## 8. `/remove-final-audit` status message can be incorrect
Severity: Low

Evidence:
- `src/main.rs:1606` checks `before == 0`, but `before` is total task count, not removed final-audit count.

Remedy:
1. Count final-audit tasks before removal.
2. Emit "not present" message only when that count is zero.

Test gaps to fill:
1. Add regression test where tasks exist but none are `final_audit`; verify message says no final audit was present.

## 9. Initial `/start` path does not persist runtime snapshot immediately
Severity: Low to Medium

Evidence:
- In `submit_user_message`, first job dispatch after `/start` does not call `persist_runtime_tasks_snapshot` (`src/main.rs:1434` onward).
- Snapshot persistence occurs in helper path (`src/main.rs:221`), but not here.

Remedy:
1. Persist tasks snapshot immediately after first `/start` dispatch (same as helper path behavior).
2. Keep status persistence consistent across all dispatch entry points.

Test gaps to fill:
1. Add regression test proving `/start` immediately updates persisted task status before next loop iteration.

## Prioritized Fix Order
1. Fix final-audit correctness and retry cap (Items 1-2).
2. Fix branch-completion correctness for multi-branch tasks (Item 3).
3. Remove destructive fallback writes in final-audit commands (Item 4).
4. Add dispatch serialization for master/master-report (Item 5).
5. Address medium/low consistency issues (Items 6-9).

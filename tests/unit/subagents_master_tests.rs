use super::*;

const REMOVED_TEST_DECISION_QUESTIONS: [&str; 5] = [
    "Testing-decision flow before initial planning in a session:",
    "If project info indicates tests are absent/unknown, ask user whether to set up a testing system first.",
    "If tests exist, ask whether to write new tests as part of this work.",
    "If tests exist, also ask whether to enforce existing tests (do not break) or ignore tests entirely.",
    "If user testing choices are still missing, ask concise questions first and wait; do not write tasks.json yet.",
];

fn assert_prompt_omits_removed_test_decision_questions(prompt: &str) {
    for removed in REMOVED_TEST_DECISION_QUESTIONS {
        assert!(!prompt.contains(removed), "prompt should not contain: {removed}");
    }
}

#[test]
fn build_master_prompt_in_tests_mode_off_uses_off_policy_without_test_decision_questions() {
    let prompt = build_master_prompt("/tmp/tasks.json", "Workflow context", false);

    assert!(
        prompt.contains("Tests mode is OFF. Testing is globally disabled for this planning run.")
    );
    assert!(prompt.contains("do not request, create, modify, or schedule any testing work"));
    assert!(prompt.contains("Do not ask testing-decision questions; treat testing as explicitly out of scope."));
    assert!(prompt.contains("Do not create or update test_writer/test_runner tasks."));
    assert!(!prompt.contains("Include test_writer branch only when user wants new tests written."));
    assert!(!prompt.contains("If user wants existing tests enforced, include test_runner under implementor so failures report back to implementor."));
    assert!(!prompt.contains("Tests mode is ON. Test requirements are enabled for planning."));
    assert_prompt_omits_removed_test_decision_questions(&prompt);
}

#[test]
fn build_master_prompt_in_tests_mode_on_includes_test_writer_and_test_runner_guidance() {
    let prompt = build_master_prompt("/tmp/tasks.json", "Workflow context", true);

    assert!(prompt.contains("Tests mode is ON. Test requirements are enabled for planning."));
    assert!(prompt.contains(
        "Do not ask the user testing-decision questions; apply defaults automatically."
    ));
    assert!(prompt.contains(
        "Include test_writer branch for top-level tasks that change behavior/code."
    ));
    assert!(prompt.contains(
        "Include a direct implementor test_runner branch so existing-test failures report back to implementor."
    ));
    assert!(prompt.contains(
        "Every test_writer must include at least one test_runner subtask."
    ));
    assert!(prompt.contains(
        "If tests are absent/unknown, add a dedicated testing-setup top-level task before dependent feature work."
    ));
    assert!(!prompt.contains(
        "Tests mode is OFF. Testing is globally disabled for this planning run."
    ));
    assert!(!prompt.contains(
        "Do not create or update test_writer/test_runner tasks."
    ));
    assert_prompt_omits_removed_test_decision_questions(&prompt);
}

#[test]
fn build_master_prompt_requires_self_contained_isolated_context_details() {
    let prompt = build_master_prompt("/tmp/tasks.json", "Workflow context", true);
    assert!(prompt.contains(
        "Every details field must be self-contained for isolated-context execution"
    ));
    assert!(prompt.contains(
        "explicit isolated-context rationale stating why the assigned sub-agent can execute using only the task record and referenced artifacts"
    ));
}

#[test]
fn audit_and_test_edit_command_prompts_require_isolated_context_details() {
    let split_audits = split_audits_command_prompt();
    let merge_audits = merge_audits_command_prompt();
    let split_tests = split_tests_command_prompt();
    let merge_tests = merge_tests_command_prompt();

    assert!(split_audits.contains(
        "ensure details are self-contained and include files/modules, behavior expectations, constraints/non-goals, verification approach, and an explicit isolated-context rationale"
    ));
    assert!(merge_audits.contains(
        "keep details self-contained with files/modules, behavior expectations, constraints/non-goals, verification approach, and an explicit isolated-context rationale"
    ));
    assert!(split_tests.contains(
        "ensure details are self-contained and include files/modules, behavior expectations, constraints/non-goals, verification approach, and an explicit isolated-context rationale"
    ));
    assert!(merge_tests.contains(
        "keep details self-contained with files/modules, behavior expectations, constraints/non-goals, verification approach, and an explicit isolated-context rationale"
    ));
}

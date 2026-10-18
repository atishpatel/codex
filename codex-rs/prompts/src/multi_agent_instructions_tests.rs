//! Covers role composition and attribution for captured model instructions.

use super::*;
use codex_context_fragments::AnnotatedContent;
use codex_context_fragments::RenderedFragment;
use codex_protocol::models::ContentItemMetadata;
use codex_protocol_test_support::ContentItemTestExt;
use pretty_assertions::assert_eq;

#[test]
fn role_segment_filters_base_and_appends_bundled_guidance() {
    let shared = DEFAULT_MULTI_AGENT_V2_SHARED_USAGE_HINT_TEXT;
    let wait = DEFAULT_MULTI_AGENT_V2_WAIT_AGENT_USAGE_HINT_TEXT;
    let model_override = DEFAULT_MULTI_AGENT_V2_MODEL_OVERRIDE_USAGE_HINT_TEXT;
    let expected_body = format!(
        "Role.\n## Work\nContinue.\n{shared}\n{wait}\n\nThere are 2 available concurrency slots, meaning that up to 2 agents can be active at once, including you.\n\n{model_override}"
    );
    for marked in [false, true] {
        let instructions = MultiAgentRoleInstructions::Composed {
            base: "Role.\n## Plan tool\nOmit role checklist guidance.\n## Work\nContinue."
                .to_string(),
            marked,
            omit_update_plan_instructions: true,
            max_concurrency: 2,
            wait_agent_enabled: true,
            expose_model_overrides: true,
        };
        let expected_text = if marked {
            format!("<multi_agent_role>{expected_body}</multi_agent_role>")
        } else {
            expected_body.clone()
        };
        assert_eq!(
            instructions.render_fragment(),
            RenderedFragment::new(
                "developer",
                AnnotatedContent::text(
                    expected_text,
                    ContentItemKind("multi_agent.role_instructions".to_string()),
                    ContentItemMetadata::harness(),
                ),
            ),
        );
    }
}

#[test]
fn role_segment_model_override_guidance_is_history_independent() {
    for expose_model_overrides in [false, true] {
        let instructions = MultiAgentRoleInstructions::Composed {
            base: "Role.".to_string(),
            marked: true,
            omit_update_plan_instructions: false,
            max_concurrency: 2,
            wait_agent_enabled: true,
            expose_model_overrides,
        };
        let text = instructions.body().expect_text();

        for guidance in [
            "Full-history forks (`fork_turns` omitted or `\"all\"`) can use `model` and `reasoning_effort` overrides without dropping parent history.",
            "Omitted fields use configured subagent defaults or fall back to the parent settings; agent-type configuration still applies.",
            "Only set `model` or `reasoning_effort` when explicitly requested by the user, applicable `AGENTS.md` instructions, or skill instructions.",
        ] {
            assert_eq!(text.contains(guidance), expose_model_overrides);
        }
        assert!(!text.contains("do not accept overrides"));
        assert!(!text.contains("set `fork_turns` to `\"none\"`"));
    }
}

//! Guard both release paths and the OIDC-to-secret fallback without publishing.

use serde_yaml_ng::Value;

fn workflow() -> Value {
    serde_yaml_ng::from_str(include_str!("../.github/workflows/release.yml"))
        .expect("release workflow must be valid YAML")
}

#[test]
fn publishing_uses_oidc_then_legacy_tokens_in_both_release_jobs() {
    let workflow = workflow();
    for job_name in ["auto-release", "manual-release"] {
        let job = &workflow["jobs"][job_name];
        assert_eq!(job["permissions"]["id-token"], "write");
        assert_eq!(job["permissions"]["contents"], "write");
        let steps = job["steps"].as_sequence().unwrap();
        let auth_index = steps
            .iter()
            .position(|step| step["id"] == "prm_auth")
            .expect("release must request an OIDC token");
        let publish_index = steps
            .iter()
            .position(|step| step["id"] == "publish-crate")
            .expect("release must publish the crate");
        let auth = &steps[auth_index];
        let publish = &steps[publish_index];
        assert_eq!(
            auth["uses"],
            "rust-lang/crates-io-auth-action@c6f97d42243bad5fab37ca0427f495c86d5b1a18"
        );
        assert_eq!(auth["continue-on-error"], true);
        assert!(auth_index < publish_index);
        assert_eq!(auth["if"], publish["if"]);
        assert!(publish["if"].is_string());
        assert_eq!(publish["run"], "rust-script scripts/publish-crate.rs");
        assert_eq!(
            publish["env"]["CARGO_REGISTRY_TOKEN"],
            "${{ steps.prm_auth.outputs.token || secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}"
        );
        assert!(publish["continue-on-error"].is_null());
        assert!(steps[..auth_index]
            .iter()
            .any(|step| step["run"] == "cargo build --release"));
    }
}

#[test]
fn release_jobs_require_ci_and_keep_existing_triggers() {
    let workflow = workflow();
    assert_eq!(workflow["on"]["push"]["branches"][0], "main");
    assert!(workflow["on"]["pull_request"].is_mapping());
    assert!(workflow["on"]["workflow_dispatch"].is_mapping());
    for job_name in ["auto-release", "manual-release"] {
        let job = &workflow["jobs"][job_name];
        let needs = job["needs"].as_sequence().unwrap();
        for dependency in ["lint", "test", "build"] {
            assert!(needs.iter().any(|value| value == dependency));
        }
        let condition = job["if"].as_str().unwrap();
        assert!(condition.contains("!cancelled()"));
        assert!(condition.contains("needs.build.result == 'success'"));
        if job_name == "auto-release" {
            assert!(condition.contains("github.event_name == 'push'"));
            assert!(condition.contains("github.ref == 'refs/heads/main'"));
        } else {
            assert!(condition.contains("github.event_name == 'workflow_dispatch'"));
            assert!(condition.contains("github.event.inputs.release_mode == 'instant'"));
        }
    }
    for (name, job) in workflow["jobs"].as_mapping().unwrap() {
        if name != "auto-release" && name != "manual-release" {
            assert!(job["permissions"]["id-token"].is_null());
        }
    }
}

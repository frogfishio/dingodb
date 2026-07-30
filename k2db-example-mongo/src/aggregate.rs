// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

#[path = "aggregate/rewrite.rs"]
mod rewrite;
#[path = "aggregate/validate.rs"]
mod validate;

pub use rewrite::{enforce_no_deleted_in_pipeline, enforce_scope_in_pipeline, sanitize_pipeline_matches};
pub use validate::{assert_no_secure_field_refs_in_pipeline, collect_stage_ops, validate_pipeline};

#[cfg(test)]
mod tests {
    use mongodb::bson::doc;

    use crate::config::{AggregationMode, OwnershipMode};
    use crate::scope::Scope;

    use super::{
        assert_no_secure_field_refs_in_pipeline, collect_stage_ops, enforce_no_deleted_in_pipeline,
        enforce_scope_in_pipeline, validate_pipeline,
    };

    #[test]
    fn inserts_non_deleted_after_reserved_stage() {
        let pipeline = vec![
            doc! { "$search": { "text": { "query": "x", "path": "name" } } },
            doc! { "$match": { "a": 1 } },
        ];
        let rewritten = enforce_no_deleted_in_pipeline(&pipeline);
        assert_eq!(rewritten[1], doc! { "$match": { "_deleted": { "$ne": true } } });
    }

    #[test]
    fn scope_is_injected_at_root() {
        let pipeline = vec![doc! { "$match": { "a": 1 } }];
        let rewritten = enforce_scope_in_pipeline(
            &pipeline,
            Some(&Scope::owner("user1")),
            OwnershipMode::Strict,
        )
        .unwrap();
        assert_eq!(rewritten[0], doc! { "$match": { "_owner": "user1" } });
    }

    #[test]
    fn guarded_mode_denies_out() {
        let error = validate_pipeline(AggregationMode::Guarded, &[doc! { "$out": "x" }], 10).unwrap_err();
        assert_eq!(error.key.as_deref(), Some("sys_mdb_ag_stage_denied"));
    }

    #[test]
    fn strict_mode_only_allows_small_subset() {
        let error = validate_pipeline(AggregationMode::Strict, &[doc! { "$lookup": { "from": "x" } }], 10).unwrap_err();
        assert_eq!(error.key.as_deref(), Some("sys_mdb_ag_stage_denied"));
    }

    #[test]
    fn secure_field_refs_are_rejected() {
        let error = assert_no_secure_field_refs_in_pipeline(
            &[doc! { "$project": { "x": "$#secret" } }],
            &["#".to_owned()],
        )
        .unwrap_err();
        assert_eq!(error.key.as_deref(), Some("sys_mdb_ag_secure_field_ref"));
    }

    #[test]
    fn collects_stage_ops() {
        let ops = collect_stage_ops(&[doc! { "$match": {} }, doc! { "$sort": { "a": 1 } }]);
        assert_eq!(ops, vec!["$match".to_owned(), "$sort".to_owned()]);
    }
}
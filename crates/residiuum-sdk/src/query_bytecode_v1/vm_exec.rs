//! Query VM dispatch (**RQL-VM1R**) — one `run_vm` for Core + Full.
//!
//! Profile: **`residiuum-query-vm-v1`** (see [`super::vm`]).
//! Normative: [QUERY_VM_V1.md](../../../../../doc/todo/rql/QUERY_VM_V1.md)
//!
//! Product execute enters here after ISA decode + lower (+ QVM materialize).
//! Core pipeline opcodes call [`super::core_phases::CoreFrame`] (**RQL-VM2/VM3/VM3b**).
//! Scan establishes `PendingKeys`; Filter owns where (+ key-stream get/early-stop).
//! Full attach continues in the **same** loop after `ProjectPaths` (**RQL-VM4**);
//! `Within` imm is a shell (carrier + alias only).
//!
//! Decision 0 remains OPEN; **RQL-C1 must not be accepted.**

use crate::app_v1::{QueryBudget, QueryPage, QueryRunOptions};
use crate::error::Error;
use crate::plan_v1::RqlPlanV1;
use crate::predicate::{Path, Predicate};
use residiuum_heap::{CollectionId, HeapId};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

use super::core_page::DocScan;
use super::core_phases::CoreFrame;
use super::full_attach::{
    apply_project_rows, attach_enrich_rows, ensure_foreign_docs, filter_rows,
    load_foreign_docs_for_root_enrich, within_enter, within_leave, EnrichAttachMode,
    EnrichLoadEvidence, EnrichStepV1, FullPipelineStepV1, ProjectItemV1, WithinStepV1,
};
use super::vm::{Instruction, OpCode, VM_PROFILE};
use super::HostCapabilities;

/// Bound-collection [`DocScan`] over [`HostCapabilities`] for Core opcodes.
struct HostScan<'a, H: HostCapabilities> {
    host: &'a mut H,
    collection_id: CollectionId,
}

impl<H: HostCapabilities> DocScan for HostScan<'_, H> {
    fn list_keys(
        &mut self,
        limit: Option<usize>,
        after_key: Option<&str>,
    ) -> Result<Vec<String>, Error> {
        self.host
            .list_keys(self.collection_id, limit, after_key)
    }

    fn get_json(&mut self, key: &str) -> Result<Option<JsonValue>, Error> {
        self.host.get_json(self.collection_id, key)
    }

    fn try_equality_index_keys(
        &mut self,
        equalities: &[(String, JsonValue)],
    ) -> Result<Option<Vec<String>>, Error> {
        self.host
            .lookup_index_keys(self.collection_id, equalities)
    }
}

/// Result of one [`run_vm`] pass (Core page + optional attach rows).
#[derive(Debug, Clone)]
pub(crate) struct VmOutcome {
    /// Core page (Bind…ProjectPaths).
    pub page: QueryPage,
    /// Working rows after attach (equals `page.rows` when no Full opcodes ran).
    pub rows: Vec<(String, JsonValue)>,
    /// Root enrich load evidence (empty for Core-only programs).
    pub enrich_loads: Vec<EnrichLoadEvidence>,
}

/// Open Within scope: parents saved; working set is carrier elements.
struct WithinScope {
    carrier: Path,
    parents: Vec<(String, JsonValue)>,
}

/// Typed immediate for one VM instruction (operands live here or in [`VmPool`]).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum VmImm {
    /// Bind / verify collection id.
    Collection(CollectionId),
    /// Core pipeline op: meaning comes from [`VmPool::core`] (bound at Bind).
    Core,
    /// Enrich step (root or nested inside Within…WithinEnd).
    Enrich(EnrichStepV1),
    /// Within shell: carrier + alias only; body is stream ops until WithinEnd.
    Within(WithinStepV1),
    /// Delimiter after Within body (no payload).
    None,
    /// Post-attach filter (root or nested).
    FilterAttach(Predicate),
    /// Brace project.
    ProjectBrace(Vec<ProjectItemV1>),
}

/// Constant pool for Core plan operands (**RQL-QVM1**).
///
/// Executable meaning for Core opcodes is recovered from this pool after Bind —
/// not from a parallel `VmProgram::core` sidecar.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VmPool {
    /// Application Core plan (owned; also durable in QVM bytes).
    pub core: RqlPlanV1,
}

/// One typed VM instruction.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VmInstr {
    /// Opcode.
    pub op: OpCode,
    /// Immediate.
    pub imm: VmImm,
}

/// Lowered Query VM program (Core or Full) — **no plan/pipeline/project sidecars**.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VmProgram {
    /// Profile stamp.
    pub profile: &'static str,
    /// Instruction stream (executable with [`VmPool`]).
    pub ops: Vec<VmInstr>,
    /// Constant pool (Core plan).
    pub pool: VmPool,
    /// Optional execute budget from ISA/QVM.
    pub budget: Option<QueryBudget>,
}

impl VmProgram {
    /// Flatten to opcode+raw-imm instructions (diagnostics / evidence).
    pub fn to_instructions(&self) -> Vec<Instruction> {
        self.ops
            .iter()
            .map(|i| Instruction {
                op: i.op,
                imm: match &i.imm {
                    VmImm::Collection(id) => id.as_bytes().to_vec(),
                    VmImm::None | VmImm::Core => Vec::new(),
                    VmImm::Enrich(e) => e.using_id.as_bytes().to_vec(),
                    VmImm::Within(_) => Vec::new(),
                    VmImm::FilterAttach(_) => Vec::new(),
                    VmImm::ProjectBrace(_) => Vec::new(),
                },
            })
            .collect()
    }
}

/// Lower a Core plan into the canonical opcode sequence.
pub(crate) fn lower_core(core: RqlPlanV1, budget: Option<QueryBudget>) -> VmProgram {
    let id = core.from.collection_id;
    let ops = vec![
        VmInstr {
            op: OpCode::BindCollection,
            imm: VmImm::Collection(id),
        },
        VmInstr {
            op: OpCode::IndexEq,
            imm: VmImm::Core,
        },
        VmInstr {
            op: OpCode::Scan,
            imm: VmImm::Core,
        },
        VmInstr {
            op: OpCode::Filter,
            imm: VmImm::Core,
        },
        VmInstr {
            op: OpCode::Order,
            imm: VmImm::Core,
        },
        VmInstr {
            op: OpCode::Page,
            imm: VmImm::Core,
        },
        VmInstr {
            op: OpCode::ProjectPaths,
            imm: VmImm::Core,
        },
        VmInstr {
            op: OpCode::Halt,
            imm: VmImm::None,
        },
    ];
    VmProgram {
        profile: VM_PROFILE,
        ops,
        pool: VmPool { core },
        budget,
    }
}

/// Emit Full pipeline steps onto the opcode stream (**RQL-VM4**).
///
/// Nested Within bodies expand between `Within` (shell imm) and `WithinEnd`.
fn emit_attach_pipeline(ops: &mut Vec<VmInstr>, pipeline: &[FullPipelineStepV1]) {
    for step in pipeline {
        match step {
            FullPipelineStepV1::Enrich(e) => ops.push(VmInstr {
                op: OpCode::Enrich,
                imm: VmImm::Enrich(e.clone()),
            }),
            FullPipelineStepV1::Within(w) => {
                ops.push(VmInstr {
                    op: OpCode::Within,
                    imm: VmImm::Within(WithinStepV1 {
                        carrier: w.carrier.clone(),
                        element_alias: w.element_alias.clone(),
                        steps: Vec::new(),
                    }),
                });
                emit_attach_pipeline(ops, &w.steps);
                ops.push(VmInstr {
                    op: OpCode::WithinEnd,
                    imm: VmImm::None,
                });
            }
            FullPipelineStepV1::Filter(p) => ops.push(VmInstr {
                op: OpCode::FilterAttach,
                imm: VmImm::FilterAttach(p.clone()),
            }),
        }
    }
}

/// Lower Core + Full attach section into one program.
pub(crate) fn lower_full(
    core: RqlPlanV1,
    budget: Option<QueryBudget>,
    pipeline: Vec<FullPipelineStepV1>,
    project: Option<Vec<ProjectItemV1>>,
) -> VmProgram {
    let id = core.from.collection_id;
    let mut ops = vec![
        VmInstr {
            op: OpCode::BindCollection,
            imm: VmImm::Collection(id),
        },
        VmInstr {
            op: OpCode::IndexEq,
            imm: VmImm::Core,
        },
        VmInstr {
            op: OpCode::Scan,
            imm: VmImm::Core,
        },
        VmInstr {
            op: OpCode::Filter,
            imm: VmImm::Core,
        },
        VmInstr {
            op: OpCode::Order,
            imm: VmImm::Core,
        },
        VmInstr {
            op: OpCode::Page,
            imm: VmImm::Core,
        },
        VmInstr {
            op: OpCode::ProjectPaths,
            imm: VmImm::Core,
        },
    ];
    emit_attach_pipeline(&mut ops, &pipeline);
    if let Some(ref fields) = project {
        ops.push(VmInstr {
            op: OpCode::ProjectBrace,
            imm: VmImm::ProjectBrace(fields.clone()),
        });
    }
    ops.push(VmInstr {
        op: OpCode::Halt,
        imm: VmImm::None,
    });
    VmProgram {
        profile: VM_PROFILE,
        ops,
        pool: VmPool { core },
        budget,
    }
}

/// One Query VM dispatch for Core and Full programs (**RQL-VM1R**).
///
/// Core opcodes call [`CoreFrame`] phase helpers; after `ProjectPaths`, Full
/// opcodes continue in the same loop (no second dispatcher / Core-prefix skip).
pub(crate) fn run_vm<H: HostCapabilities>(
    host: &mut H,
    prog: &VmProgram,
    params: &BTreeMap<String, JsonValue>,
    options: &QueryRunOptions,
    heap_id: HeapId,
    collection_id: CollectionId,
    force_enrich_scan: bool,
) -> Result<VmOutcome, Error> {
    if prog.profile != VM_PROFILE {
        return Err(Error::QueryInvalid(format!(
            "run_vm: profile mismatch: got {:?}, want {VM_PROFILE}",
            prog.profile
        )));
    }
    let mut pc = 0usize;
    let mut frame: Option<CoreFrame<'_>> = None;
    let mut page: Option<QueryPage> = None;
    let mut rows: Vec<(String, JsonValue)> = Vec::new();
    let mut enrich_loads: Vec<EnrichLoadEvidence> = Vec::new();
    let mut foreign_cache: BTreeMap<CollectionId, Vec<(String, JsonValue)>> = BTreeMap::new();
    let mut within_stack: Vec<WithinScope> = Vec::new();

    while pc < prog.ops.len() {
        let instr = &prog.ops[pc];
        match instr.op {
            OpCode::BindCollection => {
                let VmImm::Collection(id) = &instr.imm else {
                    return Err(Error::QueryInvalid(
                        "run_vm: BindCollection immediate mismatch".into(),
                    ));
                };
                if *id != collection_id || *id != prog.pool.core.from.collection_id {
                    return Err(Error::QueryInvalid(
                        "run_vm: BindCollection id mismatch".into(),
                    ));
                }
                if page.is_some() {
                    return Err(Error::QueryInvalid(
                        "run_vm: BindCollection after Core page".into(),
                    ));
                }
                frame = Some(CoreFrame::begin(
                    &prog.pool.core,
                    params,
                    options,
                    heap_id,
                    collection_id,
                    prog.budget,
                )?);
                pc += 1;
            }
            OpCode::IndexEq => {
                let f = frame.as_mut().ok_or_else(|| {
                    Error::QueryInvalid("run_vm: IndexEq before BindCollection".into())
                })?;
                let mut scan = HostScan {
                    host,
                    collection_id,
                };
                f.index_eq(&mut scan)?;
                pc += 1;
            }
            OpCode::Scan => {
                let f = frame.as_mut().ok_or_else(|| {
                    Error::QueryInvalid("run_vm: Scan before BindCollection".into())
                })?;
                let mut scan = HostScan {
                    host,
                    collection_id,
                };
                f.scan(&mut scan)?;
                pc += 1;
            }
            OpCode::Filter => {
                let f = frame.as_mut().ok_or_else(|| {
                    Error::QueryInvalid("run_vm: Filter before BindCollection".into())
                })?;
                let mut scan = HostScan {
                    host,
                    collection_id,
                };
                f.filter(&mut scan)?;
                pc += 1;
            }
            OpCode::Order => {
                let f = frame.as_mut().ok_or_else(|| {
                    Error::QueryInvalid("run_vm: Order before BindCollection".into())
                })?;
                f.order()?;
                pc += 1;
            }
            OpCode::Page => {
                let f = frame.as_mut().ok_or_else(|| {
                    Error::QueryInvalid("run_vm: Page before BindCollection".into())
                })?;
                f.page()?;
                pc += 1;
            }
            OpCode::ProjectPaths => {
                let f = frame.as_mut().ok_or_else(|| {
                    Error::QueryInvalid("run_vm: ProjectPaths before BindCollection".into())
                })?;
                let mut scan = HostScan {
                    host,
                    collection_id,
                };
                let p = f.project_paths(&mut scan)?;
                rows = p
                    .rows
                    .iter()
                    .map(|r| (r.key.clone(), r.value.clone()))
                    .collect();
                page = Some(p);
                // CoreFrame no longer needed; free borrow of pool/params.
                frame = None;
                pc += 1;
            }
            OpCode::Enrich => {
                if page.is_none() {
                    return Err(Error::QueryInvalid(
                        "run_vm: Enrich before ProjectPaths".into(),
                    ));
                }
                let VmImm::Enrich(e) = &instr.imm else {
                    return Err(Error::QueryInvalid(
                        "run_vm: Enrich immediate mismatch".into(),
                    ));
                };
                if within_stack.is_empty() {
                    let (foreign, mode) =
                        load_foreign_docs_for_root_enrich(host, e, &rows, force_enrich_scan)?;
                    enrich_loads.push(EnrichLoadEvidence {
                        using: e.using_name.clone(),
                        output: e.output.clone(),
                        mode,
                    });
                    if mode == EnrichAttachMode::Scan {
                        foreign_cache
                            .entry(e.using_id)
                            .or_insert_with(|| foreign.clone());
                    }
                    rows = attach_enrich_rows(&rows, &foreign, e, params)?;
                } else {
                    ensure_foreign_docs(host, e.using_id, &mut foreign_cache)?;
                    let foreign = foreign_cache.get(&e.using_id).ok_or_else(|| {
                        Error::QueryInvalid(format!(
                            "within attach missing foreign docs for id {} (`{}`)",
                            e.using_id, e.using_name
                        ))
                    })?;
                    rows = attach_enrich_rows(&rows, foreign, e, params)?;
                }
                pc += 1;
            }
            OpCode::Within => {
                if page.is_none() {
                    return Err(Error::QueryInvalid(
                        "run_vm: Within before ProjectPaths".into(),
                    ));
                }
                let VmImm::Within(w) = &instr.imm else {
                    return Err(Error::QueryInvalid(
                        "run_vm: Within immediate mismatch".into(),
                    ));
                };
                if !w.steps.is_empty() {
                    return Err(Error::QueryInvalid(
                        "run_vm: Within immediate must be a shell (empty steps); body is stream ops"
                            .into(),
                    ));
                }
                let parents = std::mem::take(&mut rows);
                rows = within_enter(&parents, &w.carrier)?;
                within_stack.push(WithinScope {
                    carrier: w.carrier.clone(),
                    parents,
                });
                pc += 1;
            }
            OpCode::WithinEnd => {
                let scope = within_stack.pop().ok_or_else(|| {
                    Error::QueryInvalid("run_vm: WithinEnd without matching Within".into())
                })?;
                rows = within_leave(&scope.parents, &scope.carrier, &rows)?;
                pc += 1;
            }
            OpCode::FilterAttach => {
                if page.is_none() {
                    return Err(Error::QueryInvalid(
                        "run_vm: FilterAttach before ProjectPaths".into(),
                    ));
                }
                let VmImm::FilterAttach(pred) = &instr.imm else {
                    return Err(Error::QueryInvalid(
                        "run_vm: FilterAttach immediate mismatch".into(),
                    ));
                };
                rows = filter_rows(&rows, pred, params)?;
                pc += 1;
            }
            OpCode::ProjectBrace => {
                if page.is_none() {
                    return Err(Error::QueryInvalid(
                        "run_vm: ProjectBrace before ProjectPaths".into(),
                    ));
                }
                let VmImm::ProjectBrace(fields) = &instr.imm else {
                    return Err(Error::QueryInvalid(
                        "run_vm: ProjectBrace immediate mismatch".into(),
                    ));
                };
                if !within_stack.is_empty() {
                    return Err(Error::QueryInvalid(
                        "run_vm: ProjectBrace inside Within is not supported".into(),
                    ));
                }
                rows = apply_project_rows(&rows, fields)?;
                pc += 1;
            }
            OpCode::Halt => {
                if !within_stack.is_empty() {
                    return Err(Error::QueryInvalid(
                        "run_vm: Halt with open Within scope".into(),
                    ));
                }
                break;
            }
        }
    }

    let page = page.ok_or_else(|| Error::QueryInvalid("run_vm: Halt without Core page".into()))?;
    Ok(VmOutcome {
        page,
        rows,
        enrich_loads,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan_v1::{CollectionBindings, PlanBuilder};
    use crate::predicate::Path;
    use residiuum_heap::CollectionId;

    fn uuidish(seed: u8) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[0] = seed;
        b[6] = (b[6] & 0x0f) | 0x40;
        b[8] = (b[8] & 0x3f) | 0x80;
        b
    }

    fn empty_core(id: CollectionId) -> RqlPlanV1 {
        let mut bindings = CollectionBindings::default();
        bindings.bind("items", id);
        PlanBuilder::from_source("items")
            .compile(&bindings)
            .expect("plan")
    }

    #[test]
    fn lower_core_shape() {
        let id = CollectionId::from_bytes(uuidish(1)).expect("id");
        let prog = lower_core(empty_core(id), None);
        assert_eq!(prog.profile, VM_PROFILE);
        let names: Vec<_> = prog.ops.iter().map(|o| o.op.name()).collect();
        assert_eq!(
            names,
            [
                "BindCollection",
                "IndexEq",
                "Scan",
                "Filter",
                "Order",
                "Page",
                "ProjectPaths",
                "Halt",
            ]
        );
        assert_eq!(prog.to_instructions().len(), 8);
    }

    #[test]
    fn lower_full_emits_enrich_and_within_end() {
        let id = CollectionId::from_bytes(uuidish(2)).expect("id");
        let fid = CollectionId::from_bytes(uuidish(3)).expect("id");
        let pipeline = vec![
            FullPipelineStepV1::Enrich(EnrichStepV1 {
                output: "f".into(),
                using_name: "foreign".into(),
                using_id: fid,
                left: Path::parse_dotted("a").unwrap(),
                right: Path::parse_dotted("b").unwrap(),
                candidate_where: None,
                expect: super::super::full_attach::EnrichCardinality::Optional,
            }),
            FullPipelineStepV1::Within(WithinStepV1 {
                carrier: Path::parse_dotted("items").unwrap(),
                element_alias: None,
                steps: Vec::new(),
            }),
        ];
        let prog = lower_full(empty_core(id), None, pipeline, None);
        let names: Vec<_> = prog.ops.iter().map(|o| o.op.name()).collect();
        assert!(names.contains(&"Enrich"));
        assert!(names.contains(&"Within"));
        assert!(names.contains(&"WithinEnd"));
        assert_eq!(*names.last().unwrap(), "Halt");
    }

    #[test]
    fn lower_full_flattens_nested_within_body() {
        let id = CollectionId::from_bytes(uuidish(4)).expect("id");
        let fid = CollectionId::from_bytes(uuidish(5)).expect("id");
        let nested_enrich = EnrichStepV1 {
            output: "sku".into(),
            using_name: "products".into(),
            using_id: fid,
            left: Path::parse_dotted("product_id").unwrap(),
            right: Path::parse_dotted("id").unwrap(),
            candidate_where: None,
            expect: super::super::full_attach::EnrichCardinality::Optional,
        };
        let pipeline = vec![FullPipelineStepV1::Within(WithinStepV1 {
            carrier: Path::parse_dotted("items").unwrap(),
            element_alias: Some("item".into()),
            steps: vec![FullPipelineStepV1::Enrich(nested_enrich)],
        })];
        let prog = lower_full(empty_core(id), None, pipeline, None);
        let names: Vec<_> = prog.ops.iter().map(|o| o.op.name()).collect();
        let attach: Vec<_> = names
            .iter()
            .skip_while(|n| **n != "Within")
            .copied()
            .collect();
        assert_eq!(
            attach,
            ["Within", "Enrich", "WithinEnd", "Halt"],
            "nested enrich must be stream ops, not Within imm body"
        );
        let within_imm = prog
            .ops
            .iter()
            .find(|i| i.op == OpCode::Within)
            .expect("Within");
        match &within_imm.imm {
            VmImm::Within(w) => assert!(
                w.steps.is_empty(),
                "Within imm must be shell after VM4 flatten"
            ),
            other => panic!("expected Within imm, got {other:?}"),
        }
    }
}

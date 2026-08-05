//! Query VM dispatch (RQL-VM1/VM2/VM3/VM4) — one instruction loop for Core + Full.
//!
//! Profile: **`residiuum-query-vm-v1`** (see [`super::vm`]).
//! Normative: [QUERY_VM_V1.md](../../../../../doc/todo/rql/QUERY_VM_V1.md)
//!
//! Product execute enters here after ISA decode + lower. Core pipeline opcodes
//! call [`super::core_phases::CoreFrame`] phase helpers (**RQL-VM2/VM3/VM3b**).
//! Scan establishes `PendingKeys`; Filter owns where (+ key-stream get/early-stop).
//! Full attach: nested Within enrich/filter expand onto the flat opcode stream
//! (**RQL-VM4**); `Within` imm is a shell (carrier + alias only).
//!
//! Decision 0 remains OPEN; **RQL-C1 must not be accepted.**

use crate::app_v1::{Parameters, QueryBudget, QueryPage, QueryRunOptions};
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

/// Open Within scope: parents saved; working set is carrier elements.
struct WithinScope {
    carrier: Path,
    parents: Vec<(String, JsonValue)>,
}

/// Typed immediate for one VM instruction (in-memory form; wire encoding later).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum VmImm {
    /// Bind / verify collection id.
    Collection(CollectionId),
    /// Core pipeline op: semantics come from [`VmProgram::core`] until VM2.
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

/// One typed VM instruction.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VmInstr {
    /// Opcode.
    pub op: OpCode,
    /// Immediate.
    pub imm: VmImm,
}

/// Lowered Query VM program (Core or Full).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VmProgram {
    /// Profile stamp.
    pub profile: &'static str,
    /// Instruction stream.
    pub ops: Vec<VmInstr>,
    /// Core plan (compiler intermediate; Core opcode body until VM2).
    pub core: RqlPlanV1,
    /// Optional execute budget from ISA.
    pub budget: Option<QueryBudget>,
    /// Original Full pipeline (diagnostics / page artefact; may be empty).
    pub pipeline: Vec<FullPipelineStepV1>,
    /// Original brace project (diagnostics).
    pub project: Option<Vec<ProjectItemV1>>,
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
        core,
        budget,
        pipeline: Vec::new(),
        project: None,
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
        core,
        budget,
        pipeline,
        project,
    }
}

/// Run a Core-only VM program (no Enrich / Within / …).
///
/// RQL-VM2/VM3: each Core opcode calls a [`CoreFrame`] phase helper.
/// IndexEq probes; Scan loads; Filter/Order/Page transform; ProjectPaths finishes.
pub(crate) fn run_vm_core<S: DocScan>(
    scan: &mut S,
    prog: &VmProgram,
    params: &BTreeMap<String, JsonValue>,
    options: &QueryRunOptions,
    heap_id: HeapId,
    collection_id: CollectionId,
) -> Result<QueryPage, Error> {
    if prog.profile != VM_PROFILE {
        return Err(Error::QueryInvalid(format!(
            "run_vm: profile mismatch: got {:?}, want {VM_PROFILE}",
            prog.profile
        )));
    }
    let mut pc = 0usize;
    let mut bound: Option<CollectionId> = None;
    let mut frame: Option<CoreFrame<'_>> = None;
    let mut page: Option<QueryPage> = None;

    while pc < prog.ops.len() {
        let instr = &prog.ops[pc];
        match instr.op {
            OpCode::BindCollection => {
                let VmImm::Collection(id) = &instr.imm else {
                    return Err(Error::QueryInvalid(
                        "run_vm: BindCollection immediate mismatch".into(),
                    ));
                };
                if *id != collection_id || *id != prog.core.from.collection_id {
                    return Err(Error::QueryInvalid(
                        "run_vm: BindCollection id mismatch".into(),
                    ));
                }
                bound = Some(*id);
                frame = Some(CoreFrame::begin(
                    &prog.core,
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
                f.index_eq(scan)?;
                pc += 1;
            }
            OpCode::Scan => {
                let f = frame.as_mut().ok_or_else(|| {
                    Error::QueryInvalid("run_vm: Scan before BindCollection".into())
                })?;
                f.scan(scan)?;
                pc += 1;
            }
            OpCode::Filter => {
                let f = frame.as_mut().ok_or_else(|| {
                    Error::QueryInvalid("run_vm: Filter before BindCollection".into())
                })?;
                f.filter(scan)?;
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
                page = Some(f.project_paths(scan)?);
                pc += 1;
            }
            OpCode::Halt => {
                break;
            }
            OpCode::Enrich
            | OpCode::Within
            | OpCode::WithinEnd
            | OpCode::FilterAttach
            | OpCode::ProjectBrace => {
                return Err(Error::QueryInvalid(format!(
                    "run_vm_core: unexpected Full opcode {}",
                    instr.op.name()
                )));
            }
        }
    }

    let _ = bound;
    page.ok_or_else(|| Error::QueryInvalid("run_vm_core: Halt without Core page".into()))
}

/// Run Full attach / brace-project opcodes after a Core page is already produced.
///
/// Expects `prog` from [`lower_full`]. Skips Bind + Core pipeline; dispatches
/// Enrich / Within / WithinEnd / FilterAttach / ProjectBrace until Halt.
/// Nested Within bodies are stream ops between Within and WithinEnd (**RQL-VM4**).
/// Foreign loads use collection-qualified [`HostCapabilities`] (RQL-P1b).
pub(crate) fn run_vm_attach<H: HostCapabilities>(
    host: &mut H,
    prog: &VmProgram,
    mut rows: Vec<(String, JsonValue)>,
    parameters: &Parameters,
    force_enrich_scan: bool,
) -> Result<(Vec<(String, JsonValue)>, Vec<EnrichLoadEvidence>), Error> {
    if prog.profile != VM_PROFILE {
        return Err(Error::QueryInvalid(format!(
            "run_vm: profile mismatch: got {:?}, want {VM_PROFILE}",
            prog.profile
        )));
    }
    let mut pc = 0usize;
    let mut enrich_loads: Vec<EnrichLoadEvidence> = Vec::new();
    let mut foreign_cache: BTreeMap<CollectionId, Vec<(String, JsonValue)>> = BTreeMap::new();
    let mut within_stack: Vec<WithinScope> = Vec::new();
    // Skip Core prefix (Bind + pipeline).
    while pc < prog.ops.len() {
        let op = prog.ops[pc].op;
        if matches!(
            op,
            OpCode::BindCollection
                | OpCode::IndexEq
                | OpCode::Scan
                | OpCode::Filter
                | OpCode::Order
                | OpCode::Page
                | OpCode::ProjectPaths
        ) {
            pc += 1;
            continue;
        }
        break;
    }

    while pc < prog.ops.len() {
        let instr = &prog.ops[pc];
        match instr.op {
            OpCode::Enrich => {
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
                    rows = attach_enrich_rows(&rows, &foreign, e, &parameters.values)?;
                } else {
                    // Nested enrich: scan-load (RQL-I1 residual); no root load evidence.
                    ensure_foreign_docs(host, e.using_id, &mut foreign_cache)?;
                    let foreign = foreign_cache.get(&e.using_id).ok_or_else(|| {
                        Error::QueryInvalid(format!(
                            "within attach missing foreign docs for id {} (`{}`)",
                            e.using_id, e.using_name
                        ))
                    })?;
                    rows = attach_enrich_rows(&rows, foreign, e, &parameters.values)?;
                }
                pc += 1;
            }
            OpCode::Within => {
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
                let VmImm::FilterAttach(pred) = &instr.imm else {
                    return Err(Error::QueryInvalid(
                        "run_vm: FilterAttach immediate mismatch".into(),
                    ));
                };
                rows = filter_rows(&rows, pred, &parameters.values)?;
                pc += 1;
            }
            OpCode::ProjectBrace => {
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
            OpCode::BindCollection
            | OpCode::IndexEq
            | OpCode::Scan
            | OpCode::Filter
            | OpCode::Order
            | OpCode::Page
            | OpCode::ProjectPaths => {
                return Err(Error::QueryInvalid(format!(
                    "run_vm_attach: unexpected Core opcode {} after prefix",
                    instr.op.name()
                )));
            }
        }
    }

    Ok((rows, enrich_loads))
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

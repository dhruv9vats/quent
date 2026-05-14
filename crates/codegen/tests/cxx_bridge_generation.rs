// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Tests for CXX bridge code generation.

use quent_codegen::{CxxOptions, emit_cxx};
use quent_model::Model;

mod fsm_struct_attrs_fixture {
    use quent_model::{Attributes, fsm, state};

    #[derive(Attributes, serde::Deserialize, serde::Serialize)]
    pub struct MemorySpaceId {
        pub tier: String,
        pub device_id: i32,
    }

    quent_model::entity! {
        Root: ResourceGroup<Root = true> {}
    }

    state! {
        Idle {
            attributes: {
                memory_space_id: MemorySpaceId,
            },
        }
    }

    fsm! {
        DataBatch {
            states: { idle: Idle },
            entry: idle,
            exit_from: { idle },
            transitions: {},
        }
    }
}

#[test]
fn generate_query_engine_cxx_bridge() {
    let builder = quent_query_engine_model::QueryEngineModel::build("QueryEngine");

    let options = CxxOptions {
        namespace: "quent::qe".into(),
        instrumentation_crate: "quent_query_engine_model".into(),
        ..Default::default()
    };
    let files = emit_cxx(&builder, &options);

    // uuid + context + custom_attributes + 6 entities + 1 FSM + lib.rs
    assert!(
        files.len() >= 10,
        "expected at least 10 files, got {}",
        files.len()
    );

    // All entity/FSM bridges must exist
    for name in [
        "uuid",
        "context",
        "engine",
        "worker",
        "query_group",
        "query",
        "plan",
        "operator",
        "port",
    ] {
        assert!(
            files.iter().any(|f| f.name == format!("{name}.rs")),
            "missing bridge file: {name}.rs"
        );
    }

    // Verify all generated Rust files are valid syntax
    for file in &files {
        if file.name.ends_with(".rs") {
            syn::parse_file(&file.content).unwrap_or_else(|e| panic!("{}: {}", file.name, e));
        }
    }

    // Verify nested structs are generated for complex types
    let plan_file = files.iter().find(|f| f.name == "plan.rs").unwrap();
    assert!(
        plan_file.content.contains("pub struct Parent"),
        "plan.rs should contain Parent shared struct (from PlanParent)"
    );
    assert!(
        plan_file.content.contains("pub struct Edges"),
        "plan.rs should contain Edges shared struct (from Vec<Edge>)"
    );

    let engine_file = files.iter().find(|f| f.name == "engine.rs").unwrap();
    assert!(
        engine_file.content.contains("pub struct Implementation"),
        "engine.rs should contain Implementation shared struct"
    );

    // Verify Option<Ref<T>> becomes UUID (nil = None)
    assert!(
        plan_file.content.contains("worker_id"),
        "plan.rs should have worker_id field"
    );

    // Verify Vec<Ref<T>> becomes Vec<UUID>
    let operator_file = files.iter().find(|f| f.name == "operator.rs").unwrap();
    assert!(
        operator_file.content.contains("parent_operator_ids"),
        "operator.rs should have parent_operator_ids"
    );

    // Verify CustomAttributes bridge is generated
    assert!(
        files.iter().any(|f| f.name == "custom_attributes.rs"),
        "custom_attributes.rs should be generated"
    );
}

#[test]
fn generate_simulator_cxx_bridge() {
    let builder = quent_simulator_instrumentation::SimulatorModel::build("Simulator");

    let options = CxxOptions {
        instrumentation_crate: "quent_simulator_instrumentation".into(),
        ..Default::default()
    };
    let files = emit_cxx(&builder, &options);

    let task_file = files.iter().find(|f| f.name == "task.rs").unwrap();
    assert!(task_file.content.contains("TaskHandle"));
    assert!(task_file.content.contains("#[cxx::bridge"));
    assert!(task_file.content.contains("Queueing"));
    assert!(
        !task_file.content.contains("<quent_stdlib::"),
        "qualified resource paths should be remapped through the instrumentation crate"
    );
    assert!(
        task_file
            .content
            .contains("<quent_simulator_instrumentation::memory::Memory as")
    );
    assert!(
        task_file
            .content
            .contains("<quent_simulator_instrumentation::processor::Processor as")
    );
    assert!(
        task_file
            .content
            .contains("<quent_simulator_instrumentation::channel::Channel as")
    );

    for file in &files {
        if file.name.ends_with(".rs") {
            syn::parse_file(&file.content).unwrap_or_else(|e| panic!("{}: {}", file.name, e));
        }
    }
}

#[test]
fn fsm_state_struct_attributes_are_converted() {
    type TestModel = Model<(
        fsm_struct_attrs_fixture::Root,
        fsm_struct_attrs_fixture::DataBatch,
    )>;
    let builder = TestModel::build("BridgeStructAttrs");

    let options = CxxOptions {
        instrumentation_crate: "test_instrumentation".into(),
        ..Default::default()
    };
    let files = emit_cxx(&builder, &options);

    let data_batch_file = files.iter().find(|f| f.name == "data_batch.rs").unwrap();
    assert!(
        data_batch_file
            .content
            .contains("let data = data.memory_space_id;"),
        "generated FSM bridge should convert the CXX shared struct before invoking the model callback"
    );
    assert!(
        data_batch_file
            .content
            .contains("test_instrumentation::fsm_struct_attrs_fixture::MemorySpaceId"),
        "generated FSM bridge should construct the model struct"
    );
    assert!(
        !data_batch_file.content.contains("data.memory_space_id,"),
        "ffi::MemorySpaceId should not be passed directly to the model callback"
    );
}

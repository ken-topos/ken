use ken_runtime::{
    run_frame_scope_harness, FrameScopeHarnessMutation, FrameScopeHarnessWitness,
};

fn assert_exact(witness: FrameScopeHarnessWitness) {
    assert!(witness.first_consume_succeeds);
    assert!(witness.same_successor_duplicate_rejected);
    assert!(witness.second_successor_first_consume_succeeds);
    assert!(witness.post_join_duplicate_rejected);
}

#[test]
fn branch_scope_keeps_exclusive_successors_distinct_and_joined() {
    assert_exact(run_frame_scope_harness(FrameScopeHarnessMutation::Exact));
}

#[test]
fn shared_ledger_mutation_collides_in_the_other_successor() {
    let witness = run_frame_scope_harness(FrameScopeHarnessMutation::SharedLedger);
    assert!(witness.first_consume_succeeds);
    assert!(witness.same_successor_duplicate_rejected);
    assert!(!witness.second_successor_first_consume_succeeds);
    assert!(witness.post_join_duplicate_rejected);
}

#[test]
fn dropped_union_mutation_loses_the_post_join_duplicate() {
    let witness = run_frame_scope_harness(FrameScopeHarnessMutation::DropUnion);
    assert!(witness.first_consume_succeeds);
    assert!(witness.same_successor_duplicate_rejected);
    assert!(witness.second_successor_first_consume_succeeds);
    assert!(!witness.post_join_duplicate_rejected);
}

mod support;
use redact_secret_restore::*;
use std::sync::{Arc, Barrier};
use support::*;

#[test]
fn concurrent_one_use_has_exactly_one_commit_without_timing() {
    let authority = Authority::new(1);
    let barrier = Arc::new(Barrier::new(2));
    let outcomes = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let mut authority = authority.clone();
                let barrier = Arc::clone(&barrier);
                scope.spawn(move || {
                    let fields = [RestoreField {
                        path: "a",
                        text: TOKEN,
                    }];
                    let plan = RestorePlan::build(&request(&fields), Limits::default()).unwrap();
                    let grant = authority.preflight(&plan).unwrap();
                    barrier.wait();
                    authority.consume(grant, &plan).is_ok()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(outcomes.iter().filter(|&&ok| ok).count(), 1);
    assert_eq!(authority.used(TOKEN), 1);
}

#[test]
fn mutable_eligibility_is_rechecked_at_commit() {
    for stage in 0..4 {
        let fields = [
            RestoreField {
                path: "a",
                text: TOKEN,
            },
            RestoreField {
                path: "b",
                text: OTHER,
            },
        ];
        let plan = RestorePlan::build(&request(&fields), Limits::default()).unwrap();
        let mut authority = Authority::new(1);
        let grant = authority.preflight(&plan).unwrap();
        {
            let mut state = authority.0.lock().unwrap();
            match stage {
                0 => state.entries.get_mut(TOKEN).unwrap().revoked = true,
                1 => state.clock = 10,
                2 => state.policy = false,
                _ => state.entries.get_mut(OTHER).unwrap().used = 1,
            }
        }
        assert_eq!(
            authority.consume(grant, &plan).unwrap_err().commit,
            CommitState::NotCommitted
        );
        assert_eq!(authority.used(TOKEN), 0);
    }
    let fields = [RestoreField {
        path: "a",
        text: TOKEN,
    }];
    let plan = RestorePlan::build(&request(&fields), Limits::default()).unwrap();
    let mut authority = Authority::new(2);
    let grant = authority.preflight(&plan).unwrap();
    assert!(authority.consume(grant, &plan).is_ok());
    authority
        .0
        .lock()
        .unwrap()
        .entries
        .get_mut(TOKEN)
        .unwrap()
        .revoked = true;
    assert!(authority.preflight(&plan).is_err());
    assert_eq!(authority.used(TOKEN), 1);
}

#[test]
fn grants_cannot_be_substituted_between_plans_or_authorities() {
    let fields = [RestoreField {
        path: "a",
        text: TOKEN,
    }];
    let first = RestorePlan::build(&request(&fields), Limits::default()).unwrap();
    let second = RestorePlan::build(&request(&fields), Limits::default()).unwrap();
    let mut authority = Authority::new(2);
    let grant = authority.preflight(&first).unwrap();
    assert!(authority.consume(grant, &second).is_err());
    let grant = authority.preflight(&first).unwrap();
    assert!(Authority::new(2).consume(grant, &first).is_err());
    assert_eq!(authority.used(TOKEN), 0);
}

#[test]
fn synchronous_abandonment_model_never_implies_rollback() {
    let fields = [RestoreField {
        path: "a",
        text: TOKEN,
    }];
    let plan = RestorePlan::build(&request(&fields), Limits::default()).unwrap();
    let mut authority = Authority::new(1);
    // Before preflight: abandon a plan. During/preflight completion: drop grant.
    assert_eq!(authority.used(TOKEN), 0);
    drop(authority.preflight(&plan).unwrap());
    assert_eq!(authority.used(TOKEN), 0);
    // After commit: abandon values; a retry cannot replay this one-use release.
    let grant = authority.preflight(&plan).unwrap();
    drop(authority.consume(grant, &plan).unwrap());
    assert_eq!(authority.used(TOKEN), 1);
    assert!(authority.preflight(&plan).is_err());
    // This models sync stage abandonment, not a public async cancellation API.
}

struct OutcomeAuthority {
    outcome: CommitState,
    preflight_fails: bool,
    wrong_count: bool,
}
impl RestoreAuthority for OutcomeAuthority {
    type Grant = ();
    fn preflight(&self, _: &RestorePlan<'_>) -> Result<(), RestoreError> {
        if self.preflight_fails {
            Err(RestoreError::new(
                ErrorCode::AuthorityUnavailable,
                self.outcome,
            ))
        } else {
            Ok(())
        }
    }
    fn consume(&mut self, _: (), _: &RestorePlan<'_>) -> Result<ResolvedValues, RestoreError> {
        if self.wrong_count {
            Ok(ResolvedValues::new(Vec::new()))
        } else {
            Err(RestoreError::new(
                ErrorCode::AuthorityUnavailable,
                self.outcome,
            ))
        }
    }
}
#[test]
fn outcome_taxonomy_survives_bulk_boundary_without_values() {
    let fields = [RestoreField {
        path: "a",
        text: TOKEN,
    }];
    for outcome in [
        CommitState::NotCommitted,
        CommitState::Committed,
        CommitState::Indeterminate,
    ] {
        let mut authority = OutcomeAuthority {
            outcome,
            preflight_fails: false,
            wrong_count: false,
        };
        let error = restore(&request(&fields), &mut authority, Limits::default()).unwrap_err();
        assert_eq!(error.commit, outcome);
        assert!(!format!("{error:?} {error}").contains(SECRET));
        authority.preflight_fails = true;
        assert_eq!(
            restore(&request(&fields), &mut authority, Limits::default())
                .unwrap_err()
                .commit,
            CommitState::NotCommitted
        );
    }
    let mut authority = OutcomeAuthority {
        outcome: CommitState::Committed,
        preflight_fails: false,
        wrong_count: true,
    };
    assert_eq!(
        restore(&request(&fields), &mut authority, Limits::default()).unwrap_err(),
        RestoreError::new(ErrorCode::AuthorityContract, CommitState::Committed)
    );
}

// Contract-level transaction model only; no database/backend qualification.
struct TransactionAuthority {
    committed: [usize; 2],
    fail_after_first_stage: bool,
}
impl RestoreAuthority for TransactionAuthority {
    type Grant = ();
    fn preflight(&self, _: &RestorePlan<'_>) -> Result<(), RestoreError> {
        Ok(())
    }
    fn consume(&mut self, _: (), plan: &RestorePlan<'_>) -> Result<ResolvedValues, RestoreError> {
        let mut provisional = self.committed;
        for (index, occurrence) in plan.occurrences().iter().enumerate() {
            let slot = match occurrence.token() {
                TOKEN => 0,
                OTHER => 1,
                _ => {
                    return Err(RestoreError::new(
                        ErrorCode::Denied,
                        CommitState::NotCommitted,
                    ))
                }
            };
            provisional[slot] += 1;
            if index == 0 && self.fail_after_first_stage {
                // Dropping the private provisional transaction rolls back every staged use.
                return Err(RestoreError::new(
                    ErrorCode::AuthorityUnavailable,
                    CommitState::NotCommitted,
                ));
            }
        }
        self.committed = provisional;
        Ok(ResolvedValues::new(
            plan.occurrences()
                .iter()
                .map(|_| SECRET.to_owned())
                .collect(),
        ))
    }
}

#[test]
fn provisional_multi_token_transaction_failure_rolls_back_every_use() {
    let fields = [
        RestoreField {
            path: "a",
            text: TOKEN,
        },
        RestoreField {
            path: "b",
            text: OTHER,
        },
    ];
    let req = request(&fields);
    let mut authority = TransactionAuthority {
        committed: [0, 0],
        fail_after_first_stage: true,
    };
    let error = restore(&req, &mut authority, Limits::default()).unwrap_err();
    assert_eq!(error.commit, CommitState::NotCommitted);
    assert_eq!(authority.committed, [0, 0]);
    authority.fail_after_first_stage = false;
    let output = restore(&req, &mut authority, Limits::default()).unwrap();
    assert_eq!(authority.committed, [1, 1]);
    assert!(
        output.fields() == [SECRET, SECRET],
        "transaction reconstruction mismatch"
    );
}

#[test]
fn abandonment_during_synchronous_preflight_consumes_no_budget() {
    let authority = Authority::new(1);
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    std::thread::scope(|scope| {
        let worker_authority = authority.clone();
        let worker_entered = Arc::clone(&entered);
        let worker_release = Arc::clone(&release);
        let worker = scope.spawn(move || {
            let fields = [RestoreField {
                path: "a",
                text: TOKEN,
            }];
            let plan = RestorePlan::build(&request(&fields), Limits::default()).unwrap();
            let authority = PausedPreflight {
                inner: worker_authority,
                entered: worker_entered,
                release: worker_release,
            };
            drop(authority.preflight(&plan).unwrap());
        });
        entered.wait();
        assert_eq!(authority.used(TOKEN), 0);
        release.wait();
        worker.join().unwrap();
    });
    assert_eq!(authority.used(TOKEN), 0);
    assert_eq!(authority.0.lock().unwrap().consumes, 0);
    // These stage hooks do not imply a public asynchronous cancellation guarantee.
}

struct PausedPreflight {
    inner: Authority,
    entered: Arc<Barrier>,
    release: Arc<Barrier>,
}
impl RestoreAuthority for PausedPreflight {
    type Grant = Grant;
    fn preflight(&self, plan: &RestorePlan<'_>) -> Result<Grant, RestoreError> {
        let grant = self.inner.preflight(plan)?;
        self.entered.wait();
        self.release.wait();
        Ok(grant)
    }
    fn consume(
        &mut self,
        grant: Grant,
        plan: &RestorePlan<'_>,
    ) -> Result<ResolvedValues, RestoreError> {
        self.inner.consume(grant, plan)
    }
}

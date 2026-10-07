use redact_secret_restore::*;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub const TOKEN: &str = "<rsv_aaaaaaaaaaaaaaaaaaaaaaaaaa>";
pub const OTHER: &str = "<rsv_bbbbbbbbbbbbbbbbbbbbbbbbbb>";
pub const SECRET: &str = "SYNTHETIC-PRIVATE-VALUE";

pub fn context() -> TrustedContext<'static> {
    TrustedContext {
        tenant: "tenant",
        principal: "principal",
        session: "session",
        sink: "sink",
        purpose: "purpose",
    }
}
pub fn request<'a>(fields: &'a [RestoreField<'a>]) -> RestoreRequest<'a> {
    RestoreRequest {
        context: context(),
        captures: &["capture"],
        fields,
    }
}

pub struct Entry {
    pub used: usize,
    pub budget: usize,
    pub expiry: u64,
    pub revoked: bool,
}
pub struct State {
    pub entries: BTreeMap<&'static str, Entry>,
    pub clock: u64,
    pub policy: bool,
    pub preflights: usize,
    pub consumes: usize,
}
#[derive(Clone)]
pub struct Authority(pub Arc<Mutex<State>>);
pub struct Grant {
    owner: Arc<Mutex<State>>,
    plan: usize,
    snapshot: Snapshot,
}
// Owned synthetic test-only binding; never a production plaintext/mapping store.
#[derive(PartialEq)]
struct Snapshot {
    context: [String; 5],
    captures: Vec<String>,
    fields: Vec<(String, String)>,
}
impl Snapshot {
    fn of(plan: &RestorePlan<'_>) -> Self {
        let c = plan.context();
        Self {
            context: [c.tenant, c.principal, c.session, c.sink, c.purpose].map(str::to_owned),
            captures: plan
                .captures()
                .iter()
                .map(|capture| (*capture).to_owned())
                .collect(),
            fields: plan
                .fields()
                .iter()
                .map(|field| (field.path.to_owned(), field.text.to_owned()))
                .collect(),
        }
    }
}
impl Authority {
    pub fn new(budget: usize) -> Self {
        Self(Arc::new(Mutex::new(State {
            entries: [TOKEN, OTHER]
                .into_iter()
                .map(|token| {
                    (
                        token,
                        Entry {
                            used: 0,
                            budget,
                            expiry: 10,
                            revoked: false,
                        },
                    )
                })
                .collect(),
            clock: 0,
            policy: true,
            preflights: 0,
            consumes: 0,
        })))
    }
    pub fn used(&self, token: &str) -> usize {
        self.0.lock().unwrap().entries[token].used
    }
}
fn denied() -> RestoreError {
    RestoreError::new(ErrorCode::Denied, CommitState::NotCommitted)
}
fn check(state: &State, plan: &RestorePlan<'_>) -> Result<BTreeMap<String, usize>, RestoreError> {
    let c = plan.context();
    if !state.policy
        || c.tenant != "tenant"
        || c.principal != "principal"
        || c.session != "session"
        || c.sink != "sink"
        || c.purpose != "purpose"
        || plan.captures() != ["capture"]
        || plan
            .fields()
            .iter()
            .any(|field| !["a", "b"].contains(&field.path))
    {
        return Err(denied());
    }
    let mut counts = BTreeMap::new();
    for occurrence in plan.occurrences() {
        *counts
            .entry(occurrence.token().to_owned())
            .or_insert(0usize) += 1;
    }
    for (token, count) in &counts {
        let entry = state.entries.get(token.as_str()).ok_or_else(denied)?;
        if entry.revoked
            || state.clock >= entry.expiry
            || *count > entry.budget.saturating_sub(entry.used)
        {
            return Err(denied());
        }
    }
    Ok(counts)
}
impl RestoreAuthority for Authority {
    type Grant = Grant;
    fn preflight(&self, plan: &RestorePlan<'_>) -> Result<Grant, RestoreError> {
        let mut state = self.0.lock().unwrap();
        state.preflights += 1;
        check(&state, plan)?;
        Ok(Grant {
            owner: Arc::clone(&self.0),
            plan: plan as *const _ as usize,
            snapshot: Snapshot::of(plan),
        })
    }
    fn consume(
        &mut self,
        grant: Grant,
        plan: &RestorePlan<'_>,
    ) -> Result<ResolvedValues, RestoreError> {
        if !Arc::ptr_eq(&grant.owner, &self.0)
            || grant.plan != plan as *const _ as usize
            || grant.snapshot != Snapshot::of(plan)
        {
            return Err(denied());
        }
        let mut state = self.0.lock().unwrap();
        state.consumes += 1;
        let counts = check(&state, plan)?;
        for (token, count) in counts {
            state.entries.get_mut(token.as_str()).unwrap().used += count;
        }
        Ok(ResolvedValues::new(
            plan.occurrences()
                .iter()
                .map(|_| SECRET.to_owned())
                .collect(),
        ))
    }
}

#[cfg(test)]
mod binding_tests {
    use super::*;

    #[test]
    fn stale_grant_cannot_authorize_a_different_request_at_reused_address() {
        let first_fields = [RestoreField {
            path: "a",
            text: TOKEN,
        }];
        let second_fields = [RestoreField {
            path: "b",
            text: OTHER,
        }];
        let first = RestorePlan::build(&request(&first_fields), Limits::default()).unwrap();
        let second = RestorePlan::build(&request(&second_fields), Limits::default()).unwrap();
        let mut authority = Authority::new(2);
        let mut grant = authority.preflight(&first).unwrap();
        // Deterministically simulate address reuse without relying on stack layout.
        grant.plan = &second as *const _ as usize;
        assert!(authority.consume(grant, &second).is_err());
        assert_eq!(authority.used(TOKEN), 0);
        assert_eq!(authority.used(OTHER), 0);
    }
}

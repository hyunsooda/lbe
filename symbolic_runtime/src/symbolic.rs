use instrument::symbolic::{
    ConstraintSerialized, PREDICATE_EQ, PREDICATE_NE, PREDICATE_SGE, PREDICATE_SGT, PREDICATE_SLE,
    PREDICATE_SLT, VAR_KIND,
};
use rand::prelude::IndexedRandom;
use std::collections::HashMap;
use std::sync::atomic::AtomicI64;
use std::sync::RwLock;
use z3::{
    ast::{Ast, Bool, Int},
    Config, Context, Params, SatResult, Solver as Z3Solver,
};

pub static NEXT_STATE: AtomicI64 = AtomicI64::new(0);
lazy_static::lazy_static! {
    pub static ref CONSTRAINTS: RwLock<Solver> = RwLock::new(Solver::new());
    pub static ref ADDRS: RwLock<HashMap< i32, i64>>= RwLock::new(HashMap::new());
}

pub struct Solver {
    pub constraints: HashMap<i64, Vec<ConstraintSerialized>>,
}

impl Solver {
    fn new() -> Self {
        Self {
            constraints: HashMap::new(),
        }
    }

    pub fn add_constraint(
        &mut self,
        id: i64,
        left_operand_kind: i8,
        left_operand_val: i64,
        right_operand_kind: i8,
        right_operand_val: i64,
        predicate: i8,
    ) {
        self.constraints
            .entry(id)
            .or_insert(vec![])
            .push(ConstraintSerialized::new(
                id,
                left_operand_kind,
                left_operand_val,
                right_operand_kind,
                right_operand_val,
                predicate,
            ));
    }

    pub fn solve(&self, id: i64) -> Option<Vec<(i64, i64)>> {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Z3Solver::new(&ctx);
        let mut stmts = vec![];
        let mut syms = HashMap::new();
        let mut var_names = HashMap::new();
        let mut var_cnt = 0;
        let constraints = self.constraints.get(&id).unwrap();
        for constraint in constraints {
            let (left_sym_val, right_sym_val) = (
                get_symbolic_val(
                    &ctx,
                    &mut var_names,
                    &mut var_cnt,
                    constraint.left_operand_kind,
                    constraint.left_operand_val,
                ),
                get_symbolic_val(
                    &ctx,
                    &mut var_names,
                    &mut var_cnt,
                    constraint.right_operand_kind,
                    constraint.right_operand_val,
                ),
            );
            let stmt;
            match constraint.predicate {
                PREDICATE_EQ => {
                    stmt = left_sym_val._eq(&right_sym_val);
                }
                PREDICATE_NE => {
                    stmt = left_sym_val._eq(&right_sym_val).not();
                }
                PREDICATE_SLT => {
                    stmt = left_sym_val.lt(&right_sym_val);
                }
                PREDICATE_SLE => {
                    stmt = left_sym_val.le(&right_sym_val);
                }
                PREDICATE_SGT => {
                    stmt = left_sym_val.gt(&right_sym_val);
                }
                PREDICATE_SGE => {
                    stmt = left_sym_val.ge(&right_sym_val);
                }
                _ => unreachable!("unexpected predicate: {}", constraint.predicate),
            };
            syms.insert(constraint.left_operand_val, left_sym_val);
            syms.insert(constraint.right_operand_val, right_sym_val);
            stmts.push(stmt.clone());
            solver.assert(&stmt);
        }
        let combined = Bool::and(&ctx, &stmts.iter().collect::<Vec<_>>());
        set_solver_timeout(&ctx, &solver, 5000);
        match solver.check() {
            SatResult::Sat => {
                let model = solver.get_model().unwrap();
                let mut solutions = vec![];
                for (val, sym) in &syms {
                    let solution = model.eval(sym).unwrap().as_i64().unwrap();
                    if var_names.get(val).is_some() {
                        solutions.push((*val, solution));
                    }
                }
                Some(solutions)
            }
            SatResult::Unsat => {
                println!("UNSAT: {:?}", combined);
                None
            }
            SatResult::Unknown => {
                println!("UNKNOWN: {:?}", combined);
                None
            }
        }
    }
}

fn get_symbolic_val<'ctx>(
    ctx: &'ctx Context,
    var_names: &mut HashMap<i64, String>,
    var_cnt: &mut u64,
    kind: i8,
    val: i64,
) -> Int<'ctx> {
    if kind == VAR_KIND {
        if var_names.get(&val).is_none() {
            let var_name = format!("var{}", var_cnt);
            *var_cnt += 1;
            var_names.insert(val, var_name);
        }
        let var_name = var_names.get(&val).unwrap().clone();
        Int::new_const(&ctx, var_name.clone())
    } else {
        Int::from_i64(&ctx, val)
    }
}

fn set_solver_timeout(ctx: &Context, solver: &Z3Solver, ms: u32) {
    let mut params = Params::new(&ctx);
    params.set_u32("timeout", ms);
    solver.set_params(&params);
}

pub fn select_id() -> Option<i64> {
    let constraints = CONSTRAINTS.read().unwrap();
    let ids: Vec<_> = constraints.constraints.keys().collect();
    if !ids.is_empty() {
        Some(**ids.choose(&mut rand::rng()).unwrap())
    } else {
        None
    }
}

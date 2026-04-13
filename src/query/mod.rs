pub mod condition;
mod aggregates;
mod builder;
mod conditions;
mod cte;
mod ergonomics;
mod executor_glue;
mod fts;
mod json;
mod sql_gen;
mod sql_write;
mod subquery;
mod window;
mod with_agg;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_advanced;
#[cfg(test)]
mod tests_phase10;

pub use builder::{Dialect, Join, QueryBuilder, SoftDeleteMode};
pub use condition::{Condition, JoinOp, OrderDir, SqlValue};
pub use cte::SubQueryBuilder;
pub use subquery::CountOp;

// ── shared WHERE-clause helpers ─────────────────────────────────────────────

pub(crate) fn build_where_from(
    conditions: &[(JoinOp, Condition)],
    params: &mut Vec<SqlValue>,
) -> String {
    build_where_from_dialect(Dialect::Postgres, conditions, params)
}

pub(crate) fn build_where_from_dialect(
    dialect: Dialect,
    conditions: &[(JoinOp, Condition)],
    params: &mut Vec<SqlValue>,
) -> String {
    if conditions.is_empty() {
        return String::new();
    }
    let mut out = " WHERE ".to_string();
    for (idx, (op, cond)) in conditions.iter().enumerate() {
        let (frag, ps) = match dialect {
            Dialect::Postgres => cond.to_param_sql(params.len() + 1),
            Dialect::Sqlite | Dialect::Mysql => cond.to_param_sql_sqlite(),
        };
        params.extend(ps);
        if idx > 0 {
            out.push(' ');
            out.push_str(&op.to_string());
            out.push(' ');
        }
        out.push_str(&frag);
    }
    out
}

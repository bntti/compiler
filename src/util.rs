use crate::parser::Ast;

#[derive(Debug, Clone, Copy)]
pub struct Location {
    #[allow(dead_code)] // Not expect because buggy?
    pub row: usize,
    #[allow(dead_code)] // Not expect because buggy?
    pub col: usize,
}

impl PartialEq for Location {
    fn eq(&self, _: &Self) -> bool {
        true // Not actually needed, makes coding tests easier.
    }
}

pub fn ast_loc(ast: &Ast) -> Location {
    #[expect(unused_variables)]
    match ast {
        Ast::BinaryOp {
            left,
            op,
            right,
            loc,
        } => *loc,
        Ast::Block { stats, loc } => *loc,
        Ast::BoolLiteral { val, loc } => *loc,
        Ast::Function { name, params, loc } => *loc,
        Ast::Identifier { name, loc } => *loc,
        Ast::If {
            cond,
            then,
            els,
            loc,
        } => *loc,
        Ast::IntLiteral { val, loc } => *loc,
        Ast::Minus { stat, loc } => *loc,
        Ast::Negate { stat, loc } => *loc,
        Ast::NoneLiteral { loc } => *loc,
        Ast::Root { stats, loc } => *loc,
        Ast::Var { name, value, loc } => *loc,
        Ast::While { cond, then, loc } => *loc,
    }
}

#[macro_export]
macro_rules! svec {
    ($($x:expr),+ $(,)?) => (
        vec![$($x.to_owned()),+]
    )
}

#[macro_export]
macro_rules! loc {
    () => {
        Location { row: 0, col: 0 }
    };
    ($row: expr, $col: expr) => {
        Location {
            row: $row,
            col: $col,
        }
    };
}

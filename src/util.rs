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
            typ,
            loc,
        } => *loc,
        Ast::Block { stats, typ, loc } => *loc,
        Ast::BoolLiteral { val, typ, loc } => *loc,
        Ast::Function {
            name,
            params,
            typ,
            loc,
        } => *loc,
        Ast::Identifier { name, typ, loc } => *loc,
        Ast::If {
            cond,
            then,
            els,
            typ,
            loc,
        } => *loc,
        Ast::IntLiteral { val, typ, loc } => *loc,
        Ast::Minus { stat, typ, loc } => *loc,
        Ast::Negate { stat, typ, loc } => *loc,
        Ast::NoneLiteral { typ, loc } => *loc,
        Ast::Root { stats, typ, loc } => *loc,
        Ast::Var {
            name,
            value,
            typ,
            loc,
        } => *loc,
        Ast::While {
            cond,
            then,
            typ,
            loc,
        } => *loc,
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

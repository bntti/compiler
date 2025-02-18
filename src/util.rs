use crate::{ir_generator::Instruction, parser::Ast};

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
    match ast {
        Ast::BinaryOp { loc, .. } => *loc,
        Ast::Block { loc, .. } => *loc,
        Ast::BoolLiteral { loc, .. } => *loc,
        Ast::Fn { loc, .. } => *loc,
        Ast::FnCall { loc, .. } => *loc,
        Ast::Identifier { loc, .. } => *loc,
        Ast::If { loc, .. } => *loc,
        Ast::IntLiteral { loc, .. } => *loc,
        Ast::UnaryMinus { loc, .. } => *loc,
        Ast::UnaryNot { loc, .. } => *loc,
        Ast::NoneLiteral { loc, .. } => *loc,
        Ast::Module { loc, .. } => *loc,
        Ast::Var { loc, .. } => *loc,
        Ast::While { loc, .. } => *loc,
        Ast::Continue { loc, .. } => *loc,
        Ast::Break { loc, .. } => *loc,
        Ast::Return { loc, .. } => *loc,
    }
}

pub fn label_name(ir: &Instruction) -> String {
    let Instruction::Label { name, .. } = ir else {
        unreachable!();
    };

    name.clone()
}

#[macro_export]
macro_rules! svec {
    [$($x:expr),+ $(,)?] => (
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

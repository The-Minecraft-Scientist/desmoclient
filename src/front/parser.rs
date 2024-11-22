pub(crate) use expr::{BrandedExprNode, Expr, ExprNode, Ident};
use logos::Lexer;
pub(crate) use parser_internals::{BrandedNodeId, NodeId};
use parser_internals::{BrandedParser, GenericSpanIter};

use crate::util::restartable_iter::RestartableIterExt;

use super::{error::ParseError, Interner, ParsedExpression, Parser, Token};

/// Defines the AST node type, ([`ExprNode`]) and parsed AST ([`Expr`])
pub(crate) mod expr;
/// Defines the [`Parser`] type, which consumes a stream of tokens and constructs a parsed AST
pub(crate) mod parser_internals;

/// Parses a full [`Expression`](super::Expression) (i.e. a full line of a Desmos graph).
/// Can be one of the following:
/// * `[ident] = [expr]`: variable definition ([`ParsedExpression::Var`])
/// * `[expr] [comparison] [expr]`: equation ([`ParsedExpression::Eq`])
/// * `[ident]([ident], [ident], ...) = [expr]`: function definition ([`ParsedExpression::Func`])
pub(crate) fn parse_expression(
    lex: Lexer<'_, Token>,
) -> (Interner, Result<ParsedExpression, ParseError>) {
    let spanned_iter = lex.spanned().restartable();
    loop {}

    let p = Parser::new_with(spanned_iter);
    todo!();
}

impl<'source, Lex: GenericSpanIter<'source>> BrandedParser<'source, '_, Lex> {
    /// Parses an [`Expr`] out of this, which is a self contained chunk of code that produces a value (akin to a Rust block)
    /// The actual semantics of [`Expr`]s are ambiguous without further context, for this reason the lowering
    /// code takes in a LoweringContext which is queried to resolve any ambiguities as the AST is lowered
    pub(crate) fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        todo!()
    }
}

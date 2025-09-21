use std::collections::HashMap;

use crate::{
    error::{CompilerError, ErrorKind},
    lexer::{token::Token, token_kind::TokenKind},
    parser::ast::{expr::Expr, stmt::Stmt},
    symbol::Symbol,
    toltype::TolType,
};

pub struct SemanticAnalyzer<'a> {
    ast: &'a Stmt,
    source_path: &'a str,
    symbol_table: Vec<HashMap<String, Symbol>>,
    has_error: bool,
    current_func_return_type: TolType,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(ast: &'a Stmt, source_path: &'a str) -> Self {
        Self {
            ast,
            source_path,
            symbol_table: vec![HashMap::new()],
            has_error: false,
            current_func_return_type: TolType::Unknown,
        }
    }

    pub fn has_error(&self) -> bool {
        self.has_error
    }

    pub fn analyze(&mut self) {
        let statements = match self.ast {
            Stmt::Program(stmts) => stmts,
            _ => panic!("ast did not start with a program node"),
        };

        for statement in statements {
            self.analyze_stmt(statement);
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Ang {
                ang_identifier,
                ang_type,
                rhs,
                line,
                column,
                ..
            } => {
                if let Err(e) = self.analyze_ang(ang_identifier, ang_type, rhs, line, column) {
                    self.has_error = true;
                    e.display(self.source_path);
                }
            }
            Stmt::Par {
                par_identifier,
                params,
                return_type,
                block,
                // line,
                // column,
                ..
            } => {
                if let Err(e) = self.analyze_par(par_identifier, params, return_type, block) {
                    self.has_error = true;
                    e.display(self.source_path);
                }
            }
            Stmt::Ibalik { rhs, line, column } => {
                if let Err(e) = self.analyze_ibalik(rhs, line, column) {
                    self.has_error = true;
                    e.display(self.source_path);
                }
            }
            _ => {}
        };
    }

    fn analyze_ang(
        &mut self,
        ang_identifier: &Token,
        ang_type: &TolType,
        rhs: &Expr,
        line: &usize,
        column: &usize,
    ) -> Result<(), CompilerError> {
        let rhs_type = self.analyze_expression(rhs)?;
        if !rhs_type.is_assignment_compatible(ang_type) {
            return Err(CompilerError::new(
                &format!(
                    "Ang tipong `{}` ay hindi pwede ilagay sa `{}`",
                    rhs_type, ang_type,
                ),
                ErrorKind::Error,
                *line,
                *column,
            ));
        }

        let var_symbol = Symbol::VarSymbol {
            name: ang_identifier.lexeme().to_string(),
            tol_type: (*ang_type).clone(),
        };

        if !self.declare_symbol(ang_identifier.lexeme(), var_symbol) {
            return Err(CompilerError::new(
                &format!(
                    "`{}` ay na-ideklara na sa kasalukuyang sakop",
                    ang_identifier.lexeme()
                ),
                ErrorKind::Error,
                ang_identifier.line(),
                ang_identifier.column(),
            ));
        }

        Ok(())
    }

    fn analyze_par(
        &mut self,
        par_identifier: &Token,
        params: &[(Token, TolType)],
        return_type: &TolType,
        block: &Expr,
        // line: &usize,
        // column: &usize,
    ) -> Result<(), CompilerError> {
        self.enter_scope();

        let param_types: Vec<TolType> = params.iter().map(|tup| tup.1.clone()).collect();
        let par_symbol = Symbol::ParSymbol {
            name: par_identifier.lexeme().to_string(),
            param_types,
            return_type: return_type.to_owned(),
        };

        if !self.declare_symbol(par_identifier.lexeme(), par_symbol) {
            return Err(CompilerError::new(
                &format!(
                    "`{}` ay na-ideklara na sa kasalukuyang sakop",
                    par_identifier.lexeme()
                ),
                ErrorKind::Error,
                par_identifier.line(),
                par_identifier.column(),
            ));
        }

        self.current_func_return_type = return_type.to_owned();
        self.analyze_expression(block)?;

        self.exit_scope();
        Ok(())
    }

    fn analyze_ibalik(
        &mut self,
        rhs: &Expr,
        line: &usize,
        column: &usize,
    ) -> Result<(), CompilerError> {
        let return_type = self.analyze_expression(rhs)?;

        if self.current_func_return_type == TolType::Unknown && return_type != TolType::Wala {
            return Err(CompilerError::new(
                "Hindi pwede magbalik sa labas ng paraan",
                ErrorKind::Error,
                *line,
                *column,
            ));
        }

        if !return_type.is_assignment_compatible(&self.current_func_return_type)
        {
            return Err(CompilerError::new(
                &format!(
                    "Hindi pwede mag return ng `{}` dahil ang kasalukuyang paraan ay umaasa ng `{}`",
                    return_type, self.current_func_return_type
                ),
                ErrorKind::Error,
                *line,
                *column,
            ));
        }

        Ok(())
    }

    fn analyze_expression(&mut self, expr: &Expr) -> Result<TolType, CompilerError> {
        match expr {
            Expr::IntLit(_) => Ok(TolType::UnsizedInt),
            Expr::FloatLit(_) => Ok(TolType::UnsizedFloat),
            Expr::Identifier(tok) => match self.lookup_symbol(tok.lexeme()) {
                Some(s) => Ok(s.get_type().to_owned()),
                None => Err(CompilerError::new(
                    &format!("`{}` ay hindi pa na-ideklara", tok.lexeme()),
                    ErrorKind::Error,
                    tok.line(),
                    tok.column(),
                )),
            },
            Expr::Binary { op, left, right } => match op.kind() {
                TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                    let left_type = self.analyze_expression(left)?;
                    let right_type = self.analyze_expression(right)?;

                    if !left_type.is_arithmetic_compatible(&right_type) {
                        Err(CompilerError::new(
                            &format!(
                                "Hindi pwede gawin ang `{}` na operasyon sa `{}` at `{}`",
                                op.lexeme(),
                                left_type,
                                right_type
                            ),
                            ErrorKind::Error,
                            op.line(),
                            op.column(),
                        ))
                    } else {
                        Ok(left_type)
                    }
                }
                _ => Err(CompilerError::new(
                    &format!("Hindi tamang operator `{}`", op.lexeme()),
                    ErrorKind::Error,
                    op.line(),
                    op.column(),
                )),
            },
            Expr::Block {
                statements,
                // line,
                // column,
                ..
            } => {
                self.enter_scope();
                for statement in statements {
                    self.analyze_stmt(statement);
                }
                self.exit_scope();
                Ok(TolType::Unknown)
            }
        }
    }

    fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        for scope in self.symbol_table.iter().rev() {
            if let Some(s) = scope.get(name) {
                return Some(s);
            }
        }

        None
    }

    /// Returns true if the key did not exist (means it is declared successfully), returns false otherwise.
    fn declare_symbol(&mut self, name: &str, symbol: Symbol) -> bool {
        let current_scope = self.symbol_table.last_mut().unwrap();

        if !current_scope.contains_key(name) {
            current_scope.insert(name.to_string(), symbol);
            true
        } else {
            false
        }
    }

    fn enter_scope(&mut self) {
        self.symbol_table.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        let _ = self.symbol_table.pop();
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    fn lex(source: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(source, "test");

        lexer.lex().clone()
    }

    fn parse(tokens: &Vec<Token>) -> Stmt {
        let mut parser = Parser::new(tokens, "test");

        parser.parse()
    }

    #[test]
    fn test_analyze_assignment() {
        let code = "ang x: i32 = 15; ang y: i32 = x;";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();
        assert!(!analyzer.has_error());
    }

    #[test]
    fn test_previously_declared_variable() {
        let code = "ang x: i32 = 15; ang y: i32 = 15;";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();
        assert!(!analyzer.has_error());
    }

    #[test]
    fn test_type_mismatch_assignment() {
        let code = "ang x: i32 = 15.5;";
        let code2 = "ang x: i32 = 12; ang y: i8 = x;";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();

        let tokens2 = lex(code2);
        let ast2 = parse(&tokens2);
        let mut analyzer2 = SemanticAnalyzer::new(&ast2, "None");
        analyzer2.analyze();

        assert!(analyzer.has_error());
        assert!(analyzer2.has_error());
    }

    #[test]
    fn test_undeclared_variable() {
        let code = "ang x: i32 = y;";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();
        assert!(analyzer.has_error());
    }

    #[test]
    fn test_variable_scoping() {
        let code = "par dummy() {\n    ang y: i32 = 10;\n}\nang x: i32 = y;";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();
        assert!(analyzer.has_error());
    }

    #[test]
    fn test_type_mismatch_binary_operation() {
        let code = "ang x: i32 = 10;\nang y: dobletang = 12.5;\nang z: i32 = x + y;";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();
        assert!(analyzer.has_error());
    }

    #[test]
    fn test_incompatible_return_type() {
        let code = "par test() -> i32 { ibalik 15.5; }";
        let code2 = "par test() -> dobletang { ibalik 15; }";
        let code3 = "par test() -> i32 { ibalik 15; }";

        let tokens = lex(code);
        let ast = parse(&tokens);
        let mut analyzer = SemanticAnalyzer::new(&ast, "None");
        analyzer.analyze();

        let tokens2 = lex(code2);
        let ast2 = parse(&tokens2);
        let mut analyzer2 = SemanticAnalyzer::new(&ast2, "None");
        analyzer2.analyze();

        let tokens3 = lex(code3);
        let ast3 = parse(&tokens3);
        let mut analyzer3 = SemanticAnalyzer::new(&ast3, "None");
        analyzer3.analyze();

        assert!(analyzer.has_error());
        assert!(analyzer2.has_error());
        assert!(!analyzer3.has_error());
    }
}

use std::collections::HashMap;

use crate::{
    codegen::block_context::BlockContext,
    lexer::token::Token,
    parser::ast::{expr::Expr, stmt::Stmt},
    toltype::TolType,
};

pub mod block_context;

pub struct CodeGenerator<'a> {
    ast: &'a Stmt,
    output: String,
    inferred_types: &'a HashMap<usize, TolType>,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(ast: &'a Stmt, inferred_types: &'a HashMap<usize, TolType>) -> Self {
        // Parents must exist first
        Self {
            ast,
            output: String::from(
                "#include<stdio.h>\n\
#include<stdlib.h>\n",
            ),
            inferred_types,
        }
    }

    pub fn generate(&mut self) -> &String {
        if let Stmt::Program(statements) = self.ast {
            self.output.push_str(&self.gen_statements(statements));
        }

        // Call the main function defined by the user in C's main function
        self.output.push_str("int main(){__TOL_main__();return 0;}");

        println!("{}", self.output);

        &self.output
    }

    fn gen_statements(&self, statements: &[Stmt]) -> String {
        let mut out = String::new();
        for stmt in statements {
            out.push_str(&self.gen_statement(stmt));
        }

        out
    }

    fn gen_statement(&self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Ang {
                mutable,
                ang_identifier,
                ang_type,
                rhs,
                id,
                ..
            } => {
                let modifier_c = if !mutable { "const " } else { "" };
                let ang_type = match ang_type {
                    TolType::Unknown => self.get_inferred_type(*id),
                    _ => ang_type,
                };
                let type_c = ang_type.as_c();
                let id_c = format!("{}{}", ang_identifier.lexeme(), ang_type.array_suffix());
                let rhs_c = self.gen_expression(rhs);

                format!("{modifier_c}{type_c} {id_c} = {rhs_c};")
            }
            Stmt::Par {
                par_identifier,
                params,
                return_type,
                block,
                ..
            } => {
                let type_c = return_type.as_c();
                let id_c = par_identifier.lexeme();
                let id_c = match id_c {
                    "una" => "__TOL_main__",
                    _ => id_c,
                };
                let params_c = self.gen_params(params, None);
                let block_c = self.gen_block(block, BlockContext::Function);

                format!("{type_c} {id_c}{params_c}{block_c}")
            }
            Stmt::Ibalik { rhs, .. } => {
                format!("return {};", self.gen_expression(rhs))
            }
            Stmt::ExprS { expr, .. } => {
                format!("{};", self.gen_expression(expr))
            }
            Stmt::Bagay {
                bagay_identifier,
                fields,
                ..
            } => {
                let bagay_id_c = bagay_identifier.lexeme();
                let mut fields_c = String::new();
                for field in fields {
                    fields_c.push_str(&format!("{} {};", field.1.as_c(), field.0.lexeme()));
                }
                format!("typedef struct {bagay_id_c}{{{fields_c}}}{bagay_id_c};")
            }
            Stmt::Itupad {
                itupad_for,
                itupad_block,
                ..
            } => {
                if let Stmt::ItupadBlock { methods, .. } = &**itupad_block {
                    let mut out = String::new();
                    for method in methods {
                        out.push_str(&self.gen_method(method, itupad_for));
                    }

                    out
                } else {
                    unreachable!("Itupadblock mismatch");
                }
            }
            Stmt::Kung { branches, .. } => {
                let mut if_c = String::new();
                for (i, branch) in branches.iter().enumerate() {
                    if i == 0 {
                        if_c.push_str(&format!(
                            "if ({}){}",
                            self.gen_expression(branch.condition.as_ref().unwrap()),
                            self.gen_block(&branch.block, BlockContext::Function)
                        ));
                    } else if let Some(expr) = &branch.condition {
                        if_c.push_str(&format!(
                            "else if ({}){}",
                            self.gen_expression(expr),
                            self.gen_block(&branch.block, BlockContext::Function)
                        ));
                    } else if branch.condition.is_none() {
                        if_c.push_str(&format!(
                            "else {}",
                            self.gen_block(&branch.block, BlockContext::Function)
                        ));
                    }
                }

                if_c
            }
            Stmt::Sa {
                iterator,
                bind,
                block,
                id,
                ..
            } => match iterator {
                Expr::RangeExclusive { start, end, .. } => {
                    let bind_type = self.get_inferred_type(*id).as_c();
                    let bind_id_c = bind.lexeme();
                    let start_c = self.gen_expression(start);
                    let end_c = self.gen_expression(end);
                    let block_c = self.gen_expression(block);

                    format!(
                        "for ({bind_type} {bind_id_c} = {start_c}; {bind_id_c} < {end_c}; {bind_id_c}++) {block_c}"
                    )
                }
                Expr::RangeInclusive { start, end, .. } => {
                    let bind_type = self.get_inferred_type(*id).as_c();
                    let bind_id_c = bind.lexeme();
                    let start_c = self.gen_expression(start);
                    let end_c = self.gen_expression(end);
                    let block_c = self.gen_expression(block);

                    format!(
                        "for ({bind_type} {bind_id_c} = {start_c}; {bind_id_c} <= {end_c}; {bind_id_c}++) {block_c}"
                    )
                }
                _ => {
                    panic!(
                        "Hindi muna pwede ang ibang expresyon bukod sa `..` sa `sa`, ito ay gagawin pa. :)"
                    );
                }
            },
            Stmt::Program(statements) => self.gen_statements(statements),
            _ => "".to_string(),
        }
    }

    fn gen_method(&self, method: &Stmt, itupad_for: &TolType) -> String {
        if let Stmt::Method {
            met_identifier,
            params,
            return_type,
            block,
            ..
        } = method
        {
            let type_c = return_type.as_c();
            let id_c = met_identifier.lexeme();
            let params_c = self.gen_params(params, Some(itupad_for));
            let block_c = self.gen_block(block, BlockContext::Function);

            format!("{type_c} {id_c}{params_c}{block_c}")
        } else {
            unreachable!("Stmt is not a method");
        }
    }

    fn gen_params(&self, params: &[(Token, TolType)], itupad_for: Option<&TolType>) -> String {
        let mut c_params = String::from("(");
        for (param_name, param_type) in params {
            if let TolType::AkoType = param_type {
                if let Some(t) = itupad_for {
                    c_params.push_str(&format!("{} {}", t.as_c(), param_name.lexeme()));
                }
            } else {
                c_params.push_str(&format!("{} {}", param_type.as_c(), param_name.lexeme()));
            }

            if param_name.lexeme() != params.last().unwrap().0.lexeme() {
                c_params.push(',');
            }
        }
        c_params += ")";

        c_params
    }

    fn gen_expression(&self, expr: &Expr) -> String {
        match expr {
            Expr::IntLit { token, .. }
            | Expr::FloatLit { token, .. }
            | Expr::Identifier { token, .. } => token.lexeme().to_string(),
            Expr::StringLit { token, .. } | Expr::ByteStringLit { token, .. } => {
                format!("\"{}\"", token.lexeme())
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                format!(
                    "({} {} {})",
                    self.gen_expression(left),
                    op.lexeme(),
                    self.gen_expression(right)
                )
            }
            Expr::Assign { left, right, .. } => {
                format!(
                    "{} = {}",
                    self.gen_expression(left),
                    self.gen_expression(right)
                )
            }
            Expr::Block { .. } => self.gen_block(expr, BlockContext::StandAlone),
            Expr::FnCall { callee, args, .. } => {
                let mut args_str_c = String::from("(");
                for arg in args {
                    args_str_c.push_str(&self.gen_expression(arg));
                    if arg != args.last().unwrap() {
                        args_str_c.push_str(", ");
                    }
                }
                args_str_c.push(')');

                format!("{}{}", callee.lexeme(), args_str_c)
            }
            Expr::MagicFnCall { fncall, .. } => {
                if let Expr::FnCall { callee, args, .. } = fncall.as_ref() {
                    match callee.lexeme() {
                        "print" => {
                            let str_arg = self.gen_expression(&args[0]);
                            format!("fputs({str_arg}, stdout)")
                        }
                        "println" => {
                            let str_arg = self.gen_expression(&args[0]);
                            format!("puts({str_arg})")
                        }
                        "alis" => {
                            let int_arg = self.gen_expression(&args[0]);
                            format!("exit({int_arg})")
                        }
                        _ => "".to_owned(),
                    }
                } else {
                    panic!("MagicFnCall did not contain a function call!")
                }
            }
            Expr::FieldAccess { left, member, .. } => {
                let left_expr_c = self.gen_expression(left);
                let right_member_c = member.lexeme();

                format!("({left_expr_c}.{right_member_c})")
            }
            Expr::MethodCall {
                left, callee, args, ..
            } => {
                let mut args_str_c = String::from("(");
                args_str_c.push_str(&self.gen_expression(left));
                if args.len() == 1 {
                    args_str_c.push(',');
                }
                for arg in args {
                    args_str_c.push_str(&self.gen_expression(arg));
                    if arg != args.last().unwrap() {
                        args_str_c.push(',');
                    }
                }
                args_str_c.push(')');

                format!("{}{}", callee.lexeme(), args_str_c)
            }
            Expr::StaticMethodCall { callee, args, .. } => {
                let mut args_str_c = String::from("(");
                for arg in args {
                    args_str_c.push_str(&self.gen_expression(arg));
                    if arg != args.last().unwrap() {
                        args_str_c.push_str(", ");
                    }
                }
                args_str_c.push(')');

                format!("{}{}", callee.lexeme(), args_str_c)
            }
            Expr::Struct { name, fields, .. } => {
                let struct_name_c = name.as_c();
                let mut struct_block_c = String::from("{");
                for (i, (field_name, field_expr)) in fields.iter().enumerate() {
                    struct_block_c.push_str(&format!(
                        ".{}={}",
                        field_name.lexeme(),
                        self.gen_expression(field_expr)
                    ));

                    let is_last = i == fields.len() - 1;
                    if !is_last {
                        struct_block_c.push(',');
                    }
                }
                struct_block_c.push('}');

                format!("(struct {}){}", struct_name_c, struct_block_c)
            }
            Expr::Array { elements, .. } => {
                let mut array_c = String::from("{");

                for (i, element) in elements.iter().enumerate() {
                    array_c.push_str(&self.gen_expression(element));
                    if i != elements.len() - 1 {
                        array_c.push_str(", ");
                    }
                }
                array_c.push('}');

                array_c
            }
            _ => String::from("Wala"),
        }
    }

    fn gen_block(&self, block: &Expr, context: BlockContext) -> String {
        if let Expr::Block {
            statements,
            block_value,
            ..
        } = block
        {
            let mut out = String::from("{");
            for statement in statements {
                out.push_str(&self.gen_statement(statement));
            }

            if let (BlockContext::Function, Some(val)) = (context, block_value) {
                out.push_str(&format!("return {};", self.gen_expression(val)));
            }
            out.push('}');

            out
        } else {
            unreachable!("block is not Expr::Block")
        }
    }

    fn get_inferred_type(&self, id: usize) -> &TolType {
        // println!("Getting id: {}", id);
        self.inferred_types.get(&id).unwrap()
    }
}

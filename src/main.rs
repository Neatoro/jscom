extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct CodeParser;

pub mod nodes;
use nodes::Node;

mod llvm;

fn main() {
    let code: &str = "function foo(message) { logger(message); } foo('Hello World');";
    let pairs: pest::iterators::Pairs<Rule> = CodeParser::parse(Rule::program, code).unwrap_or_else(|e| panic!("{}", e));

    llvm::build(parse_program(pairs));

    std::process::Command::new(build_clang_path())
        .args([
            format!("-L{}", get_lib_path()),
            "-llog".to_string(),
            "out.o".to_string()
        ])
        .output()
        .expect("Could not link executable");
}

pub fn build_clang_path() -> String {
    let exe_dir = std::env::current_exe().unwrap();
    let install_dir = exe_dir.parent().unwrap();
    install_dir.join("llvm").join("bin").join("clang").to_str().unwrap().to_string()
}

pub fn get_lib_path() -> String {
    let exe_dir = std::env::current_exe().unwrap();
    let install_dir = exe_dir.parent().unwrap();
    install_dir.join("lib").to_str().unwrap().to_string()
}

fn parse_program(pairs: pest::iterators::Pairs<Rule>) -> Node {
    let mut body = vec![];

    for pair in pairs {
        match pair.as_rule() {
            Rule::instruction => {
                body.push(parse_instruction(pair))
            },
            _ => {}
        }
    }

    return Node::Program {
        body: body
    };
}

fn parse_instruction(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::instruction => parse_instruction(pair.into_inner().next().unwrap()),
        Rule::func_decl => parse_func_decl(pair),
        Rule::assignment => parse_assignment(pair),
        Rule::func_call => parse_func_call(pair),
        _ => unreachable!()
    }
}

fn parse_func_decl(pair: pest::iterators::Pair<Rule>) -> Node {
    let inner_pairs: pest::iterators::Pairs<Rule> = pair.into_inner();
    match inner_pairs.peek().unwrap().as_rule() {
        Rule::id => parse_named_func_decl(inner_pairs),
        Rule::parameter_list => parse_unnamed_func_decl(inner_pairs),
        _ => unreachable!()
    }
}

fn parse_named_func_decl(mut pairs: pest::iterators::Pairs<Rule>) -> Node {
    return Node::NamedFunction {
        id: Box::new(parse_identifier(pairs.next().unwrap())),
        parameters: parse_parameters(pairs.next().unwrap()),
        body: parse_func_body(pairs.next().unwrap())
    };
}

fn parse_unnamed_func_decl(mut pairs: pest::iterators::Pairs<Rule>) -> Node {
    return Node::UnnamedFunction {
        parameters: parse_parameters(pairs.next().unwrap()),
        body: parse_func_body(pairs.next().unwrap())
    };
}

fn parse_parameters(pair: pest::iterators::Pair<Rule>) -> Vec<Node> {
    let inner = pair.into_inner();
    let mut parameters = vec![];

    for inner_pair in inner {
        match inner_pair.as_rule() {
            Rule::id => {
                parameters.push(
                    parse_identifier(inner_pair)
                )
            },
            _ => unreachable!()
        }
    }

    return parameters;
}

fn parse_func_body(pair: pest::iterators::Pair<Rule>) -> Vec<Node> {
    assert_eq!(pair.as_rule(), Rule::func_body);

    let mut body = vec![];

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::instruction => {
                body.push(parse_instruction(inner))
            },
            _ => {}
        }
    }

    return body;
}

fn parse_identifier(pair: pest::iterators::Pair<Rule>) -> Node {
    assert_eq!(pair.as_rule(), Rule::id);
    return Node::Identifier {
        name: pair.as_str().trim().to_owned()
    };
}

fn parse_assignment(pair: pest::iterators::Pair<Rule>) -> Node {
    let mut inner = pair.into_inner();
    return Node::Assignment {
        lhs: Box::new(parse_object_expr(inner.next().unwrap())),
        rhs: Box::new(parse_expression(inner.next().unwrap()))
    };
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Node {
    let mut inner = pair.into_inner();
    let next = inner.next().unwrap();
    match next.as_rule() {
        Rule::unary_expr => parse_unary_expr(next),
        Rule::binary_expr => parse_binary_expr(next),
        _ => unreachable!()
    }
}

fn parse_unary_expr(pair: pest::iterators::Pair<Rule>) -> Node {
    return Node::UnaryExpr {
        oparator: pair.as_str()[..1].to_owned(),
        rhs: Box::new(parse_term(pair.into_inner().next().unwrap()))
    };
}

fn parse_binary_expr(pair: pest::iterators::Pair<Rule>) -> Node {
    let mut inner = pair.into_inner();
    let lhs = parse_term(inner.next().unwrap());
    let oparator = if inner.peek().is_none() { "".to_owned() } else { parse_operator(inner.next().unwrap()) };
    let rhs = if inner.peek().is_none() { Node::Empty } else { parse_term(inner.next().unwrap()) };

    return Node::BinaryExpr {
        lhs: Box::new(lhs),
        operator: oparator,
        rhs: Box::new(rhs)
    };
}

fn parse_term(pair: pest::iterators::Pair<Rule>) -> Node {
    let mut inner = pair.into_inner();
    let next: pest::iterators::Pair<Rule> = inner.next().unwrap();
    match next.as_rule() {
        Rule::object_expr => parse_object_expr(next),
        Rule::int_literal => parse_int_literal(next),
        Rule::string_literal => parse_string_literal(next),
        Rule::expr => parse_expression(next),
        _ => unreachable!()
    }
}

fn parse_int_literal(pair: pest::iterators::Pair<Rule>) -> Node {
    assert_eq!(pair.as_rule(), Rule::int_literal);
    return Node::Int(pair.as_str().parse().unwrap());
}

fn parse_string_literal(pair: pest::iterators::Pair<Rule>) -> Node {
    assert_eq!(pair.as_rule(), Rule::string_literal);
    let value = pair.as_str();
    return Node::String(value[1..value.len() - 1].to_string());
}

fn parse_operator(pair: pest::iterators::Pair<Rule>) -> String {
    assert_eq!(pair.as_rule(), Rule::operator);
    return pair.as_str().trim().to_owned();
}

fn parse_func_call(pair: pest::iterators::Pair<Rule>) -> Node {
    assert_eq!(pair.as_rule(), Rule::func_call);
    let mut inner = pair.into_inner();
    return Node::FuncCall {
        id: Box::new(parse_object_expr(inner.next().unwrap())),
        arguments: parse_arguments(inner.next().unwrap())
    };
}

fn parse_object_expr(pair: pest::iterators::Pair<Rule>) -> Node {
    assert_eq!(pair.as_rule(), Rule::object_expr);
    let mut inner: pest::iterators::Pairs<Rule> = pair.into_inner();
    let object: Node = parse_identifier(inner.next().unwrap());
    if inner.peek().is_none() {
        return object;
    } else {
        return Node::ObjectExpression {
            object: Box::new(object),
            property: Box::new(parse_object_expr(inner.next().unwrap()))
        }
    }
}

fn parse_arguments(pair: pest::iterators::Pair<Rule>) -> Vec<Node> {
    assert_eq!(pair.as_rule(), Rule::argument_list);
    let inner: pest::iterators::Pairs<Rule> = pair.into_inner();
    let mut arguments = vec![];

    for argument_pair in inner {
        match argument_pair.as_rule() {
            Rule::int_literal => arguments.push(parse_int_literal(argument_pair)),
            Rule::object_expr => arguments.push(parse_object_expr(argument_pair)),
            Rule::string_literal => arguments.push(parse_string_literal(argument_pair)),
            _ => unreachable!()
        }
    }

    return arguments;
}

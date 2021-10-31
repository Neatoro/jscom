extern crate llvm_sys;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Int(i32),
    String(String),
    Program {
        body: Vec<Node>
    },
    NamedFunction {
        id: Box<Node>,
        parameters: Vec<Node>,
        body: Vec<Node>
    },
    UnnamedFunction {
        parameters: Vec<Node>,
        body: Vec<Node>
    },
    Identifier {
        name: String
    },
    Assignment {
        lhs: Box<Node>,
        rhs: Box<Node>
    },
    UnaryExpr {
        oparator: String,
        rhs: Box<Node>
    },
    BinaryExpr {
        lhs: Box<Node>,
        rhs: Box<Node>,
        operator: String
    },
    FuncCall {
        id: Box<Node>,
        arguments: Vec<Node>
    },
    ObjectExpression {
        object: Box<Node>,
        property: Box<Node>
    },
    Empty
}

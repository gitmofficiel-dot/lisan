#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Text(String),
    Bool(bool),
    Null,
    List(Vec<Expr>),
    Map(Vec<(String, Expr)>),
    Variable(String),
    Assign(String, Box<Expr>),
    AssignIndex(String, Box<Expr>, Box<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Unary(Op, Box<Expr>),
    Binary(Box<Expr>, Op, Box<Expr>),
    Call(String, Vec<Expr>),
    GetProperty(Box<Expr>, String),
    SetProperty(Box<Expr>, String, Box<Expr>),
    MethodCall(Box<Expr>, String, Vec<Expr>),
    Lambda(Vec<String>, Vec<Stmt>),
    Slice(Box<Expr>, Option<Box<Expr>>, Option<Box<Expr>>, Option<Box<Expr>>),
    ListComprehension {
        expr: Box<Expr>,
        var: String,
        iterable: Box<Expr>,
        condition: Option<Box<Expr>>,
    },
    AssignUnpack(Vec<String>, Box<Expr>),
    FString(Vec<Expr>),
    Spanned(Box<Expr>, usize, usize),
}

impl Expr {
    pub fn unwrap_spanned(&self) -> &Expr {
        let mut curr = self;
        while let Expr::Spanned(inner, _, _) = curr {
            curr = inner;
        }
        curr
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Add, Subtract, Multiply, Divide, Modulo, Power,
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    And, Or, Not, Negate,
    In, NotIn,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub defaults: Vec<Option<Expr>>,
    pub vararg: Option<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDef {
    pub name: String,
    pub parent: Option<String>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Import(String),
    ImportModule { module: String, alias: Option<String> },
    FromImport { module: String, items: Vec<String> },
    Let(String, Expr),
    LetUnpack(Vec<String>, Expr),
    Print(Expr),
    Expr(Expr),
    Block(Vec<Stmt>),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    ForEach(String, Expr, Vec<Stmt>),
    ForEachUnpack(Vec<String>, Expr, Vec<Stmt>),
    With { expr: Expr, alias: Option<String>, body: Vec<Stmt> },
    Function(Function),
    Class(ClassDef),
    Return(Option<Expr>),
    Break,
    Continue,
    TryCatch {
        try_body: Vec<Stmt>,
        catch_var: Option<String>,
        catch_body: Vec<Stmt>,
        finally_body: Option<Vec<Stmt>>,
    },
    Throw(Expr),
    Match {
        target: Expr,
        cases: Vec<(Expr, Vec<Stmt>)>,
        default_case: Option<Vec<Stmt>>,
    },
    Spanned(Box<Stmt>, usize, usize),
}

impl Stmt {
    pub fn unwrap_spanned(&self) -> &Stmt {
        let mut curr = self;
        while let Stmt::Spanned(inner, _, _) = curr {
            curr = inner;
        }
        curr
    }
}

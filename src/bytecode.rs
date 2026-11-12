use crate::runtime::Value;

/// أوامر بايت كود للآلة الافتراضية للغة لسان (Lisan Virtual Machine Bytecode)
#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    /// تحميل ثابت من جدول الثوابت إلى المكدس
    Constant(usize),
    /// تحميل قيمة خالية
    Null,
    /// تحميل قيمة منطقية (صحيح)
    True,
    /// تحميل قيمة منطقية (خطأ)
    False,

    /// العمليات الحسابية
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Negate,

    /// العمليات المنطقية والمقارنات
    Not,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    /// إدارة المتغيرات العامة
    DefineGlobal(String),
    GetGlobal(String),
    SetGlobal(String),

    /// القفز والتحكم بالتدفق
    Jump(usize),
    JumpIfFalse(usize),
    IterInit,
    IterNext(String, usize),
    IterPop,

    /// بناء المجموعات
    BuildList(usize),
    BuildMap(usize),
    GetIndex,
    SetIndex(String),

    /// استدعاء الدوال والدوال المدمجة
    CallBuiltin(String, usize),
    Print,
    Pop,
    Return,
}

/// كتلة بايت كود مجمعة تحتوي التعليمات وجدول الثوابت
#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub instructions: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn emit(&mut self, op: OpCode, line: usize) -> usize {
        self.instructions.push(op);
        self.lines.push(line);
        self.instructions.len() - 1
    }

    pub fn patch_jump(&mut self, jump_index: usize, target: usize) {
        match &mut self.instructions[jump_index] {
            OpCode::Jump(dest) | OpCode::JumpIfFalse(dest) | OpCode::IterNext(_, dest) => {
                *dest = target;
            }
            _ => panic!("محاولة تعديل قفزة غير صحيحة في البايت كود"),
        }
    }
}

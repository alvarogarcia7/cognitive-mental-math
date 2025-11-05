use rand::Rng;

#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Addition,
    Multiplication,
}

impl OperationType {
    pub fn as_str(&self) -> &str {
        match self {
            OperationType::Addition => "ADD",
            OperationType::Multiplication => "MULTIPLY",
        }
    }

    pub fn symbol(&self) -> &str {
        match self {
            OperationType::Addition => "+",
            OperationType::Multiplication => "×",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Operation {
    #[allow(dead_code)]
    pub id: Option<i64>,
    pub operation_type: OperationType,
    pub operand1: i32,
    pub operand2: i32,
    pub result: i32,
}

impl Operation {
    pub fn new(operation_type: OperationType, operand1: i32, operand2: i32) -> Self {
        let result = match operation_type {
            OperationType::Addition => operand1 + operand2,
            OperationType::Multiplication => operand1 * operand2,
        };

        Operation {
            id: None,
            operation_type,
            operand1,
            operand2,
            result,
        }
    }

    pub fn generate_random() -> Self {
        let mut rng = rand::thread_rng();

        // Generate 1-2 digit numbers (1-99)
        let operand1 = rng.gen_range(1..100);
        let operand2 = rng.gen_range(1..100);

        // Randomly choose between addition and multiplication
        let operation_type = if rng.gen_bool(0.5) {
            OperationType::Addition
        } else {
            OperationType::Multiplication
        };

        Operation::new(operation_type, operand1, operand2)
    }

    pub fn to_string(&self) -> String {
        format!(
            "{} {} {} = ?",
            self.operand1,
            self.operation_type.symbol(),
            self.operand2
        )
    }

    pub fn check_answer(&self, answer: i32) -> bool {
        self.result == answer
    }
}

pub fn generate_question_block(count: usize) -> Vec<Operation> {
    (0..count).map(|_| Operation::generate_random()).collect()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_new_addition() {
        let op = Operation::new(OperationType::Addition, 5, 3);
        assert_eq!(op.operand1, 5);
        assert_eq!(op.operand2, 3);
        assert_eq!(op.result, 8);
        assert_eq!(op.operation_type, OperationType::Addition);
    }

    #[test]
    fn test_operation_new_multiplication() {
        let op = Operation::new(OperationType::Multiplication, 7, 6);
        assert_eq!(op.operand1, 7);
        assert_eq!(op.operand2, 6);
        assert_eq!(op.result, 42);
        assert_eq!(op.operation_type, OperationType::Multiplication);
    }

    #[test]
    fn test_check_answer_correct() {
        let op = Operation::new(OperationType::Addition, 10, 15);
        assert!(op.check_answer(25));
    }

    #[test]
    fn test_check_answer_incorrect() {
        let op = Operation::new(OperationType::Multiplication, 8, 9);
        assert!(!op.check_answer(70));
    }

    #[test]
    fn test_to_string_addition() {
        let op = Operation::new(OperationType::Addition, 12, 34);
        assert_eq!(op.to_string(), "12 + 34 = ?");
    }

    #[test]
    fn test_to_string_multiplication() {
        let op = Operation::new(OperationType::Multiplication, 5, 7);
        assert_eq!(op.to_string(), "5 × 7 = ?");
    }

    #[test]
    fn test_operation_type_as_str() {
        assert_eq!(OperationType::Addition.as_str(), "ADD");
        assert_eq!(OperationType::Multiplication.as_str(), "MULTIPLY");
    }

    #[test]
    fn test_operation_type_symbol() {
        assert_eq!(OperationType::Addition.symbol(), "+");
        assert_eq!(OperationType::Multiplication.symbol(), "×");
    }

    #[test]
    fn test_generate_random_operands_in_range() {
        // Generate many random operations to test range
        for _ in 0..100 {
            let op = Operation::generate_random();
            assert!(op.operand1 >= 1 && op.operand1 < 100, "operand1 should be 1-99");
            assert!(op.operand2 >= 1 && op.operand2 < 100, "operand2 should be 1-99");
        }
    }

    #[test]
    fn test_generate_random_result_correct() {
        // Verify that generated operations have correct results
        for _ in 0..50 {
            let op = Operation::generate_random();
            let expected_result = match op.operation_type {
                OperationType::Addition => op.operand1 + op.operand2,
                OperationType::Multiplication => op.operand1 * op.operand2,
            };
            assert_eq!(op.result, expected_result);
        }
    }

    #[test]
    fn test_generate_question_block_count() {
        let block = generate_question_block(10);
        assert_eq!(block.len(), 10);

        let block = generate_question_block(5);
        assert_eq!(block.len(), 5);
    }

    #[test]
    fn test_generate_question_block_all_valid() {
        let block = generate_question_block(20);
        for op in block {
            assert!(op.operand1 >= 1 && op.operand1 < 100);
            assert!(op.operand2 >= 1 && op.operand2 < 100);

            let expected_result = match op.operation_type {
                OperationType::Addition => op.operand1 + op.operand2,
                OperationType::Multiplication => op.operand1 * op.operand2,
            };
            assert_eq!(op.result, expected_result);
        }
    }
}

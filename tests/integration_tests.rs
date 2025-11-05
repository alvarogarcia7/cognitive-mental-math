use memory_practice::database::Database;
use memory_practice::operations::{Operation, OperationType, generate_question_block};

#[test]
fn test_store_and_retrieve_operation() {
    let db = Database::new(":memory:").unwrap();
    let operation = Operation::new(OperationType::Addition, 15, 25);

    let op_id = db
        .insert_operation(
            operation.operation_type.as_str(),
            operation.operand1,
            operation.operand2,
            operation.result,
        )
        .unwrap();

    let stored = db.get_operation(op_id).unwrap().unwrap();

    assert_eq!(stored.operation_type, "ADD");
    assert_eq!(stored.operand1, 15);
    assert_eq!(stored.operand2, 25);
    assert_eq!(stored.result, 40);
}

#[test]
fn test_full_question_answer_workflow() {
    let db = Database::new(":memory:").unwrap();
    let operation = Operation::new(OperationType::Multiplication, 6, 7);

    // Store the operation
    let op_id = db
        .insert_operation(
            operation.operation_type.as_str(),
            operation.operand1,
            operation.operand2,
            operation.result,
        )
        .unwrap();

    // User answers correctly
    let user_answer = 42;
    let is_correct = operation.check_answer(user_answer);
    let time_spent = 2.3;

    db.insert_answer(op_id, user_answer, is_correct, time_spent)
        .unwrap();

    // Verify the answer was stored correctly
    let answer = db.get_answer(1).unwrap().unwrap();
    assert_eq!(answer.operation_id, op_id);
    assert_eq!(answer.user_answer, 42);
    assert!(answer.is_correct);
    assert_eq!(answer.time_spent_seconds, 2.3);
}

#[test]
fn test_question_block_storage() {
    let db = Database::new(":memory:").unwrap();
    let block = generate_question_block(10);

    // Store all operations from the block
    for operation in &block {
        db.insert_operation(
            operation.operation_type.as_str(),
            operation.operand1,
            operation.operand2,
            operation.result,
        )
        .unwrap();
    }

    assert_eq!(db.count_operations().unwrap(), 10);
}

#[test]
fn test_correct_and_incorrect_answers() {
    let db = Database::new(":memory:").unwrap();
    let operation = Operation::new(OperationType::Addition, 20, 30);

    let op_id = db
        .insert_operation(
            operation.operation_type.as_str(),
            operation.operand1,
            operation.operand2,
            operation.result,
        )
        .unwrap();

    // Test incorrect answer
    let wrong_answer = 60;
    let is_correct = operation.check_answer(wrong_answer);
    assert!(!is_correct);

    db.insert_answer(op_id, wrong_answer, is_correct, 3.0)
        .unwrap();

    let answer = db.get_answer(1).unwrap().unwrap();
    assert!(!answer.is_correct);
    assert_eq!(answer.user_answer, 60);

    // Test correct answer (retry)
    let correct_answer = 50;
    let is_correct = operation.check_answer(correct_answer);
    assert!(is_correct);

    db.insert_answer(op_id, correct_answer, is_correct, 1.5)
        .unwrap();

    let answer2 = db.get_answer(2).unwrap().unwrap();
    assert!(answer2.is_correct);
    assert_eq!(answer2.user_answer, 50);
}

#[test]
fn test_multiple_operations_with_answers() {
    let db = Database::new(":memory:").unwrap();
    let operations = vec![
        Operation::new(OperationType::Addition, 5, 5),
        Operation::new(OperationType::Multiplication, 3, 4),
        Operation::new(OperationType::Addition, 10, 15),
    ];

    let answers = vec![10, 12, 25];
    let times = vec![1.2, 2.5, 1.8];

    for (i, operation) in operations.iter().enumerate() {
        let op_id = db
            .insert_operation(
                operation.operation_type.as_str(),
                operation.operand1,
                operation.operand2,
                operation.result,
            )
            .unwrap();

        let is_correct = operation.check_answer(answers[i]);
        db.insert_answer(op_id, answers[i], is_correct, times[i])
            .unwrap();
    }

    assert_eq!(db.count_operations().unwrap(), 3);
    assert_eq!(db.count_answers().unwrap(), 3);

    // Verify all answers are correct
    for i in 1..=3 {
        let answer = db.get_answer(i as i64).unwrap().unwrap();
        assert!(answer.is_correct);
    }
}

#[test]
fn test_operation_types_in_database() {
    let db = Database::new(":memory:").unwrap();

    let add_op = Operation::new(OperationType::Addition, 1, 2);
    let mul_op = Operation::new(OperationType::Multiplication, 3, 4);

    let add_id = db
        .insert_operation(
            add_op.operation_type.as_str(),
            add_op.operand1,
            add_op.operand2,
            add_op.result,
        )
        .unwrap();

    let mul_id = db
        .insert_operation(
            mul_op.operation_type.as_str(),
            mul_op.operand1,
            mul_op.operand2,
            mul_op.result,
        )
        .unwrap();

    let stored_add = db.get_operation(add_id).unwrap().unwrap();
    let stored_mul = db.get_operation(mul_id).unwrap().unwrap();

    assert_eq!(stored_add.operation_type, "ADD");
    assert_eq!(stored_mul.operation_type, "MULTIPLY");
}

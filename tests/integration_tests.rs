use memory_practice::database::{AnswersRepository, Database, OperationsRepository};
use memory_practice::operations::{Operation, OperationType, generate_question_block};

#[test]
fn test_store_and_retrieve_operation() {
    let db = Database::new(":memory:").unwrap();
    let operation = Operation::new(OperationType::Addition, 15, 25);

    let operation_type = operation.operation_type.as_str();
    let operand1 = operation.operand1;
    let operand2 = operation.operand2;
    let result = operation.result;
    let repo = OperationsRepository::new(&db.conn);
    let op_id = repo
        .insert(operation_type, operand1, operand2, result, None)
        .unwrap();

    let repo1 = OperationsRepository::new(&db.conn);
    let stored = repo1.get(op_id).unwrap().unwrap();

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
    let operation_type = operation.operation_type.as_str();
    let operand1 = operation.operand1;
    let operand2 = operation.operand2;
    let result = operation.result;
    let repo = OperationsRepository::new(&db.conn);
    let op_id = repo
        .insert(operation_type, operand1, operand2, result, None)
        .unwrap();

    // User answers correctly
    let user_answer = 42;
    let is_correct = operation.check_answer(user_answer);
    let time_spent = 2.3;

    let repo1 = AnswersRepository::new(&db.conn);
    repo1
        .insert(op_id, user_answer, is_correct, time_spent, None)
        .unwrap();

    // Verify the answer was stored correctly
    let repo2 = AnswersRepository::new(&db.conn);
    let answer = repo2.get(1).unwrap().unwrap();
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
        let operation_type = operation.operation_type.as_str();
        let operand1 = operation.operand1;
        let operand2 = operation.operand2;
        let result = operation.result;
        let repo = OperationsRepository::new(&db.conn);
        repo.insert(operation_type, operand1, operand2, result, None)
            .unwrap();
    }

    let repo = OperationsRepository::new(&db.conn);
    assert_eq!(repo.count().unwrap(), 10);
}

#[test]
fn test_correct_and_incorrect_answers() {
    let db = Database::new(":memory:").unwrap();
    let operation = Operation::new(OperationType::Addition, 20, 30);

    let operation_type = operation.operation_type.as_str();
    let operand1 = operation.operand1;
    let operand2 = operation.operand2;
    let result = operation.result;
    let repo = OperationsRepository::new(&db.conn);
    let op_id = repo
        .insert(operation_type, operand1, operand2, result, None)
        .unwrap();

    // Test incorrect answer
    let wrong_answer = 60;
    let is_correct = operation.check_answer(wrong_answer);
    assert!(!is_correct);

    let repo1 = AnswersRepository::new(&db.conn);
    repo1
        .insert(op_id, wrong_answer, is_correct, 3.0, None)
        .unwrap();

    let repo2 = AnswersRepository::new(&db.conn);
    let answer = repo2.get(1).unwrap().unwrap();
    assert!(!answer.is_correct);
    assert_eq!(answer.user_answer, 60);

    // Test correct answer (retry)
    let correct_answer = 50;
    let is_correct = operation.check_answer(correct_answer);
    assert!(is_correct);

    let repo1 = AnswersRepository::new(&db.conn);
    repo1
        .insert(op_id, correct_answer, is_correct, 1.5, None)
        .unwrap();

    let repo2 = AnswersRepository::new(&db.conn);
    let answer2 = repo2.get(2).unwrap().unwrap();
    assert!(answer2.is_correct);
    assert_eq!(answer2.user_answer, 50);
}

#[test]
fn test_multiple_operations_with_answers() {
    let db = Database::new(":memory:").unwrap();
    let operations = [
        Operation::new(OperationType::Addition, 5, 5),
        Operation::new(OperationType::Multiplication, 3, 4),
        Operation::new(OperationType::Addition, 10, 15),
    ];

    let answers = [10, 12, 25];
    let times = [1.2, 2.5, 1.8];

    for (i, operation) in operations.iter().enumerate() {
        let operation_type = operation.operation_type.as_str();
        let operand1 = operation.operand1;
        let operand2 = operation.operand2;
        let result = operation.result;
        let repo = OperationsRepository::new(&db.conn);
        let op_id = repo
            .insert(operation_type, operand1, operand2, result, None)
            .unwrap();

        let is_correct = operation.check_answer(answers[i]);
        let user_answer = answers[i];
        let time_spent_seconds = times[i];
        let repo1 = AnswersRepository::new(&db.conn);
        repo1
            .insert(op_id, user_answer, is_correct, time_spent_seconds, None)
            .unwrap();
    }

    let repo = OperationsRepository::new(&db.conn);
    assert_eq!(repo.count().unwrap(), 3);
    let repo1 = AnswersRepository::new(&db.conn);
    assert_eq!(repo1.count().unwrap(), 3);

    // Verify all answers are correct
    for i in 1..=3 {
        let answer_id = i as i64;
        let repo1 = AnswersRepository::new(&db.conn);
        let answer = repo1.get(answer_id).unwrap().unwrap();
        assert!(answer.is_correct);
    }
}

#[test]
fn test_operation_types_in_database() {
    let db = Database::new(":memory:").unwrap();

    let add_op = Operation::new(OperationType::Addition, 1, 2);
    let mul_op = Operation::new(OperationType::Multiplication, 3, 4);

    let operation_type = add_op.operation_type.as_str();
    let operand1 = add_op.operand1;
    let operand2 = add_op.operand2;
    let result = add_op.result;
    let repo = OperationsRepository::new(&db.conn);
    let add_id = repo
        .insert(operation_type, operand1, operand2, result, None)
        .unwrap();

    let operation_type = mul_op.operation_type.as_str();
    let operand1 = mul_op.operand1;
    let operand2 = mul_op.operand2;
    let result = mul_op.result;
    let repo = OperationsRepository::new(&db.conn);
    let mul_id = repo
        .insert(operation_type, operand1, operand2, result, None)
        .unwrap();

    let repo1 = OperationsRepository::new(&db.conn);
    let stored_add = repo1.get(add_id).unwrap().unwrap();
    let repo1 = OperationsRepository::new(&db.conn);
    let stored_mul = repo1.get(mul_id).unwrap().unwrap();

    assert_eq!(stored_add.operation_type, "ADD");
    assert_eq!(stored_mul.operation_type, "MULTIPLY");
}

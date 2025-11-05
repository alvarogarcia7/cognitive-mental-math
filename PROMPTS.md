# PROMPTS

## 2025-11-05 14:57:14 AGB - Original Prompt

Create a rust program that has a GUI. The goal is to exercise the memory of the user by performing basic mathematical operations. Separate the responsibilities into modules.

The program should:
- Ask the user for multiplication and addition operations
- Use between one and two digits per operand
- Only one operand for now (note: interpreted as one operation type at a time)
- Send a block of 10 questions at once
- Store each operation into a SQLite database
- Store the correct (or wrong) answer from the user into another table
- Time the amount spent thinking


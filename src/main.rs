extern crate pest; // Parser implementation.
#[macro_use]
extern crate pest_derive; // Used for convenient macros.

use std::fs; // Used to read input script.
use pest::Parser; // Used to expose the important types.
use pest::iterators::Pair; // Used for the Pair type.

#[derive(Parser)]
#[grammar = "stack.pest"]
struct StackParser; // Automatically built parser with pest.

fn main() {
    // We try to get argv[1] if it exists.
    std::env::args()
        .skip(1) // We skip argv[0], the name of the program.
        .next() // We get an option for argv[1].
        .ok_or(
            // We raise an error if argv[1] was in fact not provided.
            "Not input file provided".to_string(),
        ).and_then(
            // If argv[1] was provided, we read the corresponding file to a
            // string.
            |input_path| fs::read_to_string(input_path).map_err(
                // There is a special kind of error for stdio, but we want a
                // Result<_, String> here.
                |io_error| format!("{}", io_error)
            )
        ).and_then(
            // We parse the source code.
            |source_code| StackParser::parse(Rule::stack, source_code.as_str()).map_err(
                // pest uses a custom type for the error, but we want a
                // Result<_, String> so we have to perform a cast here. Note
                // that pest seems to insert a faulty char at the end of the
                // line for some reason.
                |pest_error| format!("{}", pest_error)
            )
            .and_then(
                // If the code was parsed successfully, it returned the "Pairs"
                // associated with the topmost rules. Because peek returns an Option,
                // we have to cast the Option to a Result.
                |pairs| {
                    pairs
                        .peek()
                        .ok_or("Parsed code is empty".to_string())
                        .and_then(
                            // Because we have a single topmost rule, we only need to look at
                            // the first Pair, which should be the 'stack' rule. We then execute
                            // it, which returns a Result<u32, String>.
                            |pair| stack_execution(pair),
                        )
                },
            )
        )
        .err()
        .map(
            // If the execution was successfull, we do nothing. However if the
            // execution failed, we print an error message.
            |error_message| println!("{}", error_message),
        );
}

// Executes the stack rule and returns the number of the pair currently
// executed, used for error tracking.
fn stack_execution(node: Pair<Rule>) -> Result<u32, String> {
    // We create a stack to perform the operations onto.
    let mut stack = Vec::new();
    match node.as_rule() {
        Rule::stack => {
            // This is the topmost rule, it contains several expressions that
            // should be run in order. If any of those expressions fails, the
            // whole execution fails.
            node.into_inner().fold(
                // By default the execution is successfull with 0 instructions
                // run.
                Ok(0),
                // We only try the next expression if we haven't failed yet.
                |accumulator, next_pair| {
                    accumulator.and_then(|previous_index| {
                        expression_execution(&mut stack, next_pair, previous_index)
                    })
                },
            )
        }
        _ => {
            // If we were provided any other rule, we report an error.
            Err("Provided rule was not 'stack'".to_string())
        }
    }
}

// Execution of the stack expressions, returns index+1 on success.
fn expression_execution(
    stack: &mut Vec<String>,
    node: Pair<Rule>,
    index: u32,
) -> Result<u32, String> {
    // We perform different actions based on the rule we encountered.
    match node.as_rule() {
        Rule::push_expr => {
            // We are pushing a single string onto the stack. We have to get
            // that string from the AST. This expression never fails.
            // push_expr -> into_inner (Pairs) -> as_str (str) -> to_owned (String).
            stack.push(node.into_inner().as_str().to_owned());
            // This rule never fails.
            Ok(index + 1)
        }
        Rule::pop_expr => {
            // We are popping an expression from the stack. We get the next
            // element from the stack if possible and print it, otherwise we
            // fail.
            stack.pop().map_or(
                // If we could not pop an element from the stack, we return an
                // error.
                Err(format!(
                    "Cannot pop from empty stack at index {}: {}",
                    index,
                    node.as_str()
                )),
                // If we could pop an element, we print and return the updated
                // index.
                |element| {
                    println!("POP: {}", element);
                    Ok(index + 1)
                },
            )
        }
        Rule::EOI => {
            // We do noting when we receive the End Of Input, in fact I would
            // rather it were not there at all.
            Ok(index + 1)
        }
        // If we received any other unknown rule, we fail.
        _ => Err("Received unknown rule".to_string()),
    }
}

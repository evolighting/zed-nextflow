(process_definition
  "{" @start
  "}" @end) @indent

(workflow_definition
  "{" @start
  "}" @end) @indent

(block
  "{" @start
  "}" @end) @indent

(closure
  "{" @start
  "}" @end) @indent

(list
  "[" @start
  "]" @end) @indent

(map
  "[" @start
  "]" @end) @indent

(parenthesized_expression
  "(" @start
  ")" @end) @indent

(function_call
  "(" @start
  ")" @end) @indent

(process_invocation
  "(" @start
  ")" @end) @indent

(input_declaration
  "input:" @start) @indent

(output_declaration
  "output:" @start) @indent

(workflow_input
  "take:" @start) @indent

(workflow_main
  "main:" @start) @indent

(workflow_emit
  "emit:" @start) @indent

(if_statement) @start.if
(try_statement) @start.try

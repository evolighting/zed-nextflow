; General identifiers are refined by the more specific captures below.
(identifier) @variable
(parameter) @variable.parameter

[
  "process"
  "workflow"
  "include"
  "nextflow"
  "params"
  "def"
  "as"
  "from"
  "new"
  "tuple"
  "template"
  "if"
  "for"
  "else"
  "assert"
  "return"
  "try"
  "catch"
  "finally"
  "exit"
] @keyword

[
  "in"
  "instanceof"
] @keyword

[
  "env"
  "Channel"
] @type.builtin

[
  "script"
  "shell"
  "exec"
  "stub"
  "input:"
  "output:"
  "when:"
  "main:"
  "take:"
  "emit:"
] @label

(process_definition
  (identifier) @function)

(workflow_definition
  (identifier) @function)

(function_definition
  !return_type
  .
  (identifier) @function)

(function_definition
  return_type: (_) @type
  .
  (identifier) @function)

(function_definition
  (identifier) @variable.parameter)

(directive
  .
  (identifier) @keyword)

(function_call
  (identifier) @function)

(process_invocation
  (identifier) @function)

(command_expression
  .
  (identifier) @function)

(input_declaration
  (simple_statement
    (simple_expression
      (command_expression
        .
        (identifier) @type.builtin))))

(output_declaration
  (simple_statement
    (simple_expression
      (identifier) @type.builtin)))

(workflow_event_handler
  (identifier) @function)

(process_output
  "out" @variable.special)

(parameter
  (identifier) @property)

(feature_flag
  (identifier) @property)

(method_call
  "."
  .
  (identifier) @property)

(method_call
  (identifier) @function
  .
  "(")

(method_call
  (identifier) @function
  .
  (closure))

(channel_factory
  (identifier) @function)

(dotted_identifier
  "."
  .
  (identifier) @property)

[
  "="
  "+="
  "-="
  "*="
  "/="
  "%="
  "**="
  "<<="
  ">>="
  "&="
  "|="
  "^="
  "?="
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
  "&&"
  "||"
  "=~"
  "!~"
  "==~"
  "<=>"
  "?:"
  "!"
  "+"
  "-"
  "*"
  "/"
  "%"
  "**"
  ".."
  "..<"
  "<<"
  "&"
  "^"
  "~"
  "|"
  "->"
] @operator

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  ","
  ";"
  ":"
  "."
] @punctuation.delimiter

(string_literal) @string
(triple_quoted_string) @string
(string) @string
(slashy_string) @string.regex
(interpolated_string) @string
(interpolated_triple_quoted_string) @string
(interpolated_string "\"" @string)
(interpolated_triple_quoted_string "\"\"\"" @string)
(string_content) @string
(triple_string_content) @string
(interpolation) @embedded
(escape_sequence) @string.escape

(integer_literal) @number
(float_literal) @number
(number) @number
(boolean_literal) @boolean
(boolean) @boolean

(line_comment) @comment
(block_comment) @comment
(shebang) @comment

(script_content) @embedded

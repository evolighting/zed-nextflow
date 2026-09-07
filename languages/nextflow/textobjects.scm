(process_definition) @function.around

(process_definition
  "{"
  (_)* @function.inside
  "}")

(function_definition) @function.around

(function_definition
  (block
    "{"
    (_)* @function.inside
    "}"))

(workflow_definition) @class.around

(workflow_definition
  (workflow_body) @class.inside)

(line_comment)+ @comment.around
(block_comment) @comment.around

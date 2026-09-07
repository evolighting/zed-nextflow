(process_definition
  "process" @context
  (identifier) @name) @item

(workflow_definition
  "workflow" @context
  (identifier) @name) @item

(workflow_definition
  "workflow" @name
  .
  "{") @item

(workflow_event_handler
  "workflow" @context
  "." @context
  (identifier) @name) @item

(function_definition
  !return_type
  "def" @context
  .
  (identifier) @name) @item

(function_definition
  return_type: (_) @context
  .
  (identifier) @name) @item


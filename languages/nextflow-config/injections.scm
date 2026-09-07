; Highlight shell code embedded in process script, shell, and stub bodies.
(script_content
  (interpolated_triple_quoted_string
    (triple_string_content) @injection.content)
  (#set! injection.language "bash")
  (#set! injection.combined))

(script_content
  (interpolated_string
    (string_content) @injection.content)
  (#set! injection.language "bash")
  (#set! injection.combined))

(script_content
  (string_literal) @injection.content
  (#set! injection.language "bash"))

(script_content
  (triple_quoted_string) @injection.content
  (#set! injection.language "bash"))


; Declare integer variables x and y.
(declare-const x Int)
(declare-const y Int)

; Add constraints (assertions) to the solver.
; x must be greater than or equal to 0.
(assert (>= x 0))
; y must be greater than or equal to 0.
(assert (>= y 0))
; The sum of x and y must be 10.
(assert (= (+ x y) 10))
; x must be less than y.
(assert (< x y))

; Check if the current set of assertions is satisfiable.
(check-sat)

; If satisfiable, retrieve and print the model (variable assignments).
(get-model)

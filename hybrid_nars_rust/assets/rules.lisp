;; Copyright (C) 2022 by David Ireland
;; (Subset of rules.lisp provided by user)

(define-mediate-rules *nal1*
  ((:M --> :P) (:S --> :M) !- (((:S --> :P) (:t/deduction :d/strong)))
   :substitutions ((M C "$" "#"))
   :preconditions ((:!= S P)))

  ((:P --> :M) (:S --> :M) !- (((:S --> :P) (:t/abduction :d/weak)))
   :substitutions ((M C "$" "#"))
   :preconditions ((:!= S P)))

  ((:M --> :P) (:M --> :S) !- (((:S --> :P) (:t/induction :d/weak)))
   :substitutions ((M C "$" "#"))
   :preconditions ((:!= S P)))

  ((:P --> :M) (:M --> :S) !- (((:S --> :P) (:t/exemplification :d/weak)))
   :substitutions ((M C "$" "#"))
   :preconditions ((:!= S P))))

(define-mediate-rules *nal2*
  ((:S --> :P) (:P --> :S) !- (((:P <-> :S) (:t/intersection :d/strong))
                               ((:S <-> :P) (:t/intersection :d/strong)))
   :preconditions ((:!= S P)))

  ((:M --> :P) (:S <-> :M) !- (((:S --> :P) (:t/analogy :d/strong)))
   :preconditions ((:!= S P)))

  ((:P --> :M) (:S <-> :M) !- (((:P --> :S) (:t/analogy :d/strong)))
   :preconditions ((:!= S P)))

  ((:M <-> :P) (:S <-> :M) !- (((:P <-> :S) (:t/resemblance :d/strong))
                               ((:S <-> :P) (:t/resemblance :d/strong)))
   :preconditions ((:!= S P))))

(define-mediate-rules *nal5*
  ((:M ==> :P) (:S ==> :M) !- (((:S ==> :P) (:t/deduction)))
   :preconditions ((:!= S P)))

  ((:P ==> :M) (:S ==> :M) !- (((:S ==> :P) (:t/abduction)))
   :preconditions ((:!= S P)))

  ((:M ==> :P) (:M ==> :S) !- (((:S ==> :P) (:t/induction)))
   :preconditions ((:!= S P)))

  ((:S ==> :P) (:P ==> :S) !- (((:S <=> :P) (:t/intersection)))
   :preconditions ((:!= S P)))

  ((:M ==> :P) (:S <=> :M) !- (((:S ==> :P) (:t/analogy)))
   :preconditions ((:!= S P)))

  ((:P ==> :M) (:S <=> :M) !- (((:P ==> :S) (:t/analogy)))
   :preconditions ((:!= S P)))

  ((:M <=> :P) (:S <=> :M) !- (((:P <=> :S) (:t/resemblance))
                               ((:S <=> :P) (:t/resemblance)))
   :preconditions ((:!= S P))))

(define-immediate-rules *nal.immediate*
  ;; Negation
  ((-- :M) !- (((:M) (:t/negation :t/negation))))
  
  ;; Conversion (Inheritance): S --> P |- P --> S
  ((:S --> :P) !- (((:P --> :S) (:t/conversion))))
  
  ;; Conversion (Implication): S ==> P |- P ==> S
  ((:S ==> :P) !- (((:P ==> :S) (:t/conversion))))
  
  ;; Contraposition: S ==> P |- --P ==> --S
  ((:S ==> :P) !- ((((-- :P) ==> (-- :S)) (:t/contraposition)))))


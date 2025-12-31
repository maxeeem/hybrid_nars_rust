;; Copyright (C) 2022 by David Ireland
;; Patched for Hybrid NARS (Standard Immediate Rules + Declarative Patterns)

(in-package :telos)

(define-immediate-rules *nal.immediate*
  ;; NEGATION
  ((-- :M) !- ((:M (:t/negation :t/negation))))
  
  ;; CONVERSION (Inheritance)
  ((:S --> :P) !- (((:P --> :S) (:t/conversion))))
  
  ;; CONVERSION (Implication)
  ((:S ==> :P) !- (((:P ==> :S) (:t/conversion))))
  
  ;; CONTRAPOSITION
  ((:S ==> :P) !- ((((-- :P) ==> (-- :S)) (:t/contraposition))))
  
  ;; STRUCTURAL DEDUCTION (Sets)
  ((:S --> ([] . :C)) !- ((_ (:t/structural-deduction :d/structural-strong))))
  ((({} . :C) --> :S) !- ((_ (:t/structural-deduction :d/structural-strong))))
)

(define-mediate-rules *nal1*
  ((:M --> :P) (:S --> :M) !- (((:S --> :P) (:t/deduction :d/strong)))
   :preconditions ((:!= S P)))

  ((:P --> :M) (:S --> :M) !- (((:S --> :P) (:t/abduction :d/weak)))
   :preconditions ((:!= S P)))

  ((:M --> :P) (:M --> :S) !- (((:S --> :P) (:t/induction :d/weak)))
   :preconditions ((:!= S P)))

  ((:P --> :M) (:M --> :S) !- (((:S --> :P) (:t/exemplification :d/weak)))
   :preconditions ((:!= S P))))

(define-mediate-rules *nal2*
  ;; SIMILARITY FROM INHERITANCE
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

(define-mediate-rules *nal3-composition*
  ;; COMPOSITION
  ((:P --> :M) (:S --> :M) !- ((((& :S :P) --> :M) (:t/intersection)))
   :preconditions ((:!= S P)))

  ((:P --> :M) (:S --> :M) !- ((((+ :S :P) --> :M) (:t/union)))
   :preconditions ((:!= S P)))
    
  ((:M --> :P) (:M --> :S) !- (((:M --> (& :P :S)) (:t/intersection)))
   :preconditions ((:!= S P)))
  
  ((:M --> :P) (:M --> :S) !- (((:M --> (+ :P :S)) (:t/union)))
   :preconditions ((:!= S P)))

  ((:P --> :M) (:S --> :M) !- ((((~ :P :S) --> :M) (:t/difference)))
   :preconditions ((:!= S P)))
  
  ((:M --> :P) (:M --> :S) !- (((:M --> (- :P :S)) (:t/difference)))
   :preconditions ((:!= S P))))

(define-mediate-rules *nal3-decomposition*
  ;; DECOMPOSITION (Simplified Patterns for Declarative Loader)
  ((:S --> :M) ((+ :S :P) --> :M) !- (((:P --> :M) (:t/decompose-pnn))))
  ((:S --> :M) ((& :S :P) --> :M) !- (((:P --> :M) (:t/decompose-npp))))
  
  ((:M --> :S) (:M --> (& :S :P)) !- (((:M --> :P) (:t/decompose-pnn))))
  ((:M --> :S) (:M --> (+ :S :P)) !- (((:M --> :P) (:t/decompose-npp))))

  ((:S --> :M) ((~ :S :P) --> :M) !- (((:P --> :M) (:t/decompose-pnp))))
  ((:S --> :M) ((~ :P :S) --> :M) !- (((:P --> :M) (:t/decompose-nnn))))
  ((:M --> :S) (:M --> (- :S :P)) !- (((:M --> :P) (:t/decompose-pnp))))
  ((:M --> :S) (:M --> (- :P :S)) !- (((:M --> :P) (:t/decompose-nnn)))))

(define-mediate-rules *nal4*
  ;; PRODUCT & IMAGE (Standard Patterns)
  (((* :P :X) --> :R) ((* :S :X) --> :R) !- (((:S --> :P) (:t/abduction))))
  ((:R --> (* :P :X)) (:R --> (* :S :X)) !- (((:S --> :P) (:t/induction))))

  (((* :M :X) --> :R) (:S --> :M) !- ((((* :S :X) --> :R) (:t/deduction))))
  ((:R --> (* :M :X)) (:M --> :S) !- (((:R --> (* :S :X)) (:t/deduction))))
)

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

(define-mediate-rules *nal6*
  ;; INTENSIONAL VARIABLE
  ((:S --> :M) (:P --> :M) !- ((((:P --> $X) ==> (:S --> $X))  (:t/abduction))    
                               (((:S --> $X) ==> (:P --> $X))  (:t/induction))    
                               (((:P --> $X) <=> (:S --> $X))  (:t/comparison)))
   :preconditions ((:!= S P)))

  ((:S --> :M) (:P --> :M) !- (((&& (:S --> #Y) (:P --> #Y)) (:t/intersection)))
   :preconditions ((:!= S P)))
  
  ;; EXTENSIONAL VARIABLE
  ((:M --> :S) (:M --> :P) !- (((($X --> :S) ==> ($X --> :P))  (:t/induction)) 
                               ((($X --> :P) ==> ($X --> :S))  (:t/abduction)) 
                               ((($X --> :S) <=> ($X --> :P))  (:t/comparison)))
   :preconditions ((:!= S P)))

  ((:M --> :S) (:M --> :P) !- (((&& (#Y --> :S) (#Y --> :P)) (:t/intersection)))
   :preconditions ((:!= S P)))
  
  ;; CONDITIONAL SYLLOGISM
  (:M (:S ==> :P) !- ((:P (:t/deduction :d/induction)))
   :substitutions ((M S "$" "#")))
  
  (:M (:S ==> :P) !- ((:S (:t/abduction :d/deduction)))
   :substitutions ((M P "$" "#")))

  (:M (:S <=> :P) !- ((:P (:t/analogy :d/deduction)))
   :substitutions ((M S "$" "#")))

  (:M (:S <=> :P) !- ((:S (:t/analogy :d/deduction)))
   :substitutions ((M P "$" "#"))))

(define-mediate-rules *nal.merging*
  (:M :S !- ((:M (:t/revision :t/revision)))
   :substitutions ((M S "$" "#"))))

(define-mediate-rules *temporal-induction*
  (:S :P !- (((:S ==> :P ) (:t/combine))) 
   :preconditions ((:!= S P))))

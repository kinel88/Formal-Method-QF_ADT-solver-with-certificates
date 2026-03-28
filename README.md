# QF_ADT Parser and E-Graph Encoding

This repository implements the QF_ADT-related fragment of SMT-LIB 2.6, including parsing, type checking, lowering to egg, incremental state management, and validation of core QF_ADT reasoning principles.

## Current Features

### 1. Parsing Layer

The parser currently supports the SMT-LIB 2.6 commands relevant to QF_ADT:

- `set-logic`
- `declare-datatypes`
- `declare-datatype`
- `declare-sort`
- `declare-const`
- `declare-fun`
- `assert`
- `check-sat`
- `push`
- `pop`
- `get-model`
- `get-value`
- `exit`

Supported language constructs include:

- **Single-constructor and multi-constructor ADTs**
  - `Unit`
  - `Wrapper(Int)`
  - `Color(Red | Green | Blue)`
  - `List(Nil | Cons)`
  - `Tree(Leaf | Node)`

- **Parametric datatypes**
  - `(par (T) ...)` declarations
  - disambiguation such as `(as nil (List Int))`

- **Boolean and term-level constructs**
  - `=`
  - `not`
  - `and`
  - `or`
  - `=>`
  - `ite`

- **ADT-specific syntax**
  - testers such as `is-Cons`
  - selectors such as `head` and `tail`
  - `match` expressions

### 2. Type Checking Layer

The implementation performs sort checking before lowering terms into the e-graph.

It rejects ill-typed inputs such as:

- **Sort mismatch**
  - `(= x:List n:Int)`

- **Constructor argument mismatch**
  - `Cons(Nil, Nil)` when the first argument is expected to be `Int`

- **Incompatible parametric instantiations**
  - `(= xs:(List Int) ys:(List Bool))`

This ensures that only well-typed QF_ADT terms are passed to later stages.

### 3. egg Encoding Layer

The egg encoding currently includes:

- **Automatically generated rewrite rules**
  - 17 Boolean simplification rules
  - including reflexivity:
    - `(= ?x ?x) => true`
  - for each datatype, automatically generated:
    - selector rules
    - positive tester rules
    - negative tester rules

- **Custom `AdtAnalysis`**
  - replaces egg’s default `()` analysis
  - actively detects constructor conflicts during e-class merges
  - actively supports injectivity-based reasoning

- **Match desugaring**
  - for example:
    - `match x { Nil => 0, Cons(h, t) => h }`
    - is lowered to:
    - `ite(is-Nil(x), 0, head(x))`

### 4. Decision Layer — Five Core QF_ADT Axiom Schemas

The current implementation validates the following five core QF_ADT reasoning schemas:

- **Selector–Constructor**
  - selectors return the corresponding constructor field
  - example:
    - `head(Cons(42, Nil)) = 42`
  - nested selector chains are also validated, e.g.:
    - `head(tail(Cons(1, Cons(2, Nil)))) = 2`

- **Constructor Disjointness**
  - distinct constructors are never equal
  - validated on both:
    - enumeration-style datatypes, e.g. `Red != Green`
    - recursive datatypes, e.g. `Nil != Cons(1, Nil)`

- **Constructor Injectivity**
  - if two constructor terms are equal, then their corresponding arguments must be equal
  - example:
    - `Cons(a, xs) = Cons(b, ys) => a = b ∧ xs = ys`
  - both the first and second fields are validated

- **Tester Semantics**
  - testers evaluate to the correct Boolean result
  - examples:
    - `is-Cons(Cons(...)) = true`
    - `is-Nil(Cons(...)) = false`
  - both positive and negative cases, including propagation through variables, are validated

- **Match Semantics**
  - `match` expressions select the correct branch according to the constructor
  - both `Nil` and `Cons` branches are validated

### 5. State Management

The implementation supports incremental SMT-style interaction:

- **`push` / `pop`**
  - nested assertion stacks can be saved and restored
  - contradiction state is correctly cleared after `pop`

- **Incremental `check-sat`**
  - repeated `check-sat` calls in the same context reflect the accumulated assertions correctly

- **`get-model` / `get-value`**
  - when the context is satisfiable, simplified representatives can be extracted

### 6. Composite Reasoning

The following combined reasoning scenarios have been validated:

- On `Tree = Leaf(Int) | Node(Tree, Tree)`:
  - `val(left(Node(Leaf(1), Leaf(2)))) = 1`
  - combines nested selectors with a multi-constructor ADT

- `ite(is-Nil(x), 0, head(x))` evaluates to `5` when:
  - `x = Cons(5, Nil)`
  - combines tester reasoning, selector reasoning, and Boolean simplification

- The negation of:
  - `x = Cons(1, Nil) => head(x) = 1`
  - is `unsat`
  - combines selector rules, reflexivity, and Boolean reasoning

## Scope

This project targets the QF_ADT-relevant fragment of SMT-LIB 2.6 rather than the entire standard.

## Status

The current implementation covers:

- parsing of QF_ADT-relevant SMT-LIB commands and terms
- type checking for ADT and parametric sorts
- lowering into egg
- automatically generated ADT and Boolean rewrite rules
- custom ADT-aware e-graph analysis
- incremental assertion-stack management
- basic model and value extraction for satisfiable contexts

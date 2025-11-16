# faer-macros/src Memory

## Overview
The `faer-macros` crate provides procedural macros for the faer library. It contains transformation utilities that convert mathematical expressions and method calls into appropriate function calls.

## File Structure
- **lib.rs**: Main and only source file containing all procedural macros and AST transformations

---

## lib.rs - Core Components

### Dependencies
- `quote`: For generating Rust code
- `syn`: For parsing and manipulating Rust syntax trees
- Uses `syn::visit_mut::VisitMut` for AST mutation

### Main Structures

#### 1. MigrationCtx
**Location**: faer-macros/src/lib.rs:11

A visitor context for migrating old `faer_*` method calls to new function-based syntax.

**Fields**:
- `HashMap<&'static str, &'static str>`: Maps old method names to new function names

**Purpose**: Transforms deprecated method-call syntax (e.g., `x.faer_add(y)`) into modern syntax.

**Implementation Details**:
- Implements `VisitMut` trait to walk and mutate the AST
- Handles macro bodies by parsing them as comma-separated expressions
- Transforms specific method calls:
  - `faer_add` → binary `+` operator (lines 32-44)
  - `faer_sub` → binary `-` operator (lines 45-57)
  - `faer_mul` → binary `*` operator (lines 58-70)
  - `faer_div` → binary `/` operator (lines 71-83)
  - `faer_neg` → unary `-` operator (lines 84-95)
  - Generic `faer_*` methods → function calls based on mapping (lines 97-109)

**Wrapping**: All transformed expressions are wrapped in `ExprParen` for proper precedence.

#### 2. MathCtx
**Location**: faer-macros/src/lib.rs:115

A visitor context for converting standard Rust operators into explicit math function calls.

**Purpose**: Transforms operator-based syntax into function-call syntax that can work with generic numeric types.

**Implementation Details**:
- Implements `VisitMut` trait
- Transforms unary operators:
  - `-expr` → `neg(&expr)` (lines 149-168)
- Transforms binary operators:
  - `a + b` → `add(&a, &b)` (line 173)
  - `a - b` → `sub(&a, &b)` (lines 174-176)
  - `a * b` → `mul(&a, &b)` (line 177)
  - `a / b` → `div(&a, &b)` (line 178)
- Recognizes and adds references to known math functions (lines 201-230):
  - `sqrt`, `from_real`, `copy`, `max`, `min`, `conj`
  - `absmax`, `abs2`, `abs1`, `abs`
  - `add`, `sub`, `div`, `mul`, `mul_real`, `mul_pow2`, `hypot`
  - `neg`, `recip`, `real`, `imag`
  - `is_nan`, `is_finite`, `is_zero`
  - `lt_zero`, `gt_zero`, `le_zero`, `ge_zero`

**Reference Wrapping**: Arguments to recognized functions are wrapped in `ExprReference` (immutable borrows).

### Helper Functions

#### ident_expr
**Location**: faer-macros/src/lib.rs:116-128

Creates an expression node from an identifier.

**Signature**: `fn ident_expr(ident: &syn::Ident) -> Expr`

**Returns**: `Expr::Path` containing a simple path with the given identifier

#### math_expr
**Location**: faer-macros/src/lib.rs:235-243

Creates a function call expression from a method name and arguments.

**Signature**: `fn math_expr<'a>(method: &Ident, args: impl Iterator<Item = &'a Expr>) -> Expr`

**Returns**: `Expr::Call` with the given method as function and cloned arguments

### Procedural Macros

#### #[math]
**Location**: faer-macros/src/lib.rs:244-256

**Type**: Attribute procedural macro

**Signature**: `#[proc_macro_attribute]`

**Purpose**: Converts operators in a function to explicit math function calls

**Behavior**:
1. Parses the item as `syn::ItemFn`
2. If parsing fails, returns original token stream unchanged
3. Applies `MathCtx` visitor to transform the function body
4. Returns the transformed function

**Use case**: Apply to functions that need generic numeric operations to work with custom numeric types

**Example transformation**:
```rust
#[math]
fn compute(a: T, b: T) -> T {
    a + b * (-c)
}
// Becomes:
fn compute(a: T, b: T) -> T {
    add(&a, &mul(&b, &neg(&c)))
}
```

#### #[migrate]
**Location**: faer-macros/src/lib.rs:257-290

**Type**: Attribute procedural macro

**Signature**: `#[proc_macro_attribute]`

**Purpose**: Migrates old faer method-call syntax to new function-call syntax

**Behavior**:
1. Parses the item as `syn::ItemFn`
2. If parsing fails, returns original token stream unchanged
3. Applies `MigrationCtx` with method mapping:
   - `faer_add` → `add`
   - `faer_sub` → `sub`
   - `faer_mul` → `mul`
   - `faer_div` → `div`
   - `faer_neg` → `neg`
   - `faer_inv` → `recip`
   - `faer_abs` → `abs`
   - `faer_abs2` → `abs2`
   - `faer_sqrt` → `sqrt`
   - `faer_conj` → `conj`
   - `faer_real` → `real`
   - `faer_scale_real` → `mul_real`
   - `faer_scale_power_of_two` → `mul_pow2`
4. Then applies `MathCtx` to further transform operators
5. Returns the transformed function

**Use case**: Apply to legacy code using old faer API to automatically migrate to new API

**Example transformation**:
```rust
#[migrate]
fn old_code(x: T, y: T) -> T {
    x.faer_add(y.faer_neg())
}
// Becomes (after migration + math transformation):
fn old_code(x: T, y: T) -> T {
    add(&(x + (-(y))))  // Then further to: add(&x, &neg(&y))
}
```

## Key Design Patterns

### AST Visitor Pattern
Both `MigrationCtx` and `MathCtx` use the visitor pattern via `syn::visit_mut::VisitMut` to traverse and modify syntax trees.

### Two-Phase Transformation
The `#[migrate]` macro applies two transformations sequentially:
1. Migration phase: old methods → operators
2. Math phase: operators → function calls

This allows for clean separation of concerns and reusability of the `MathCtx`.

### Parenthesization
Transformed expressions are wrapped in parentheses to preserve correct operator precedence during transformation.

### Reference Injection
The `MathCtx` automatically adds borrows (`&`) to arguments, enabling the math functions to work with borrowed values without explicit syntax from the user.

## Notes
- Error handling is minimal: if parsing fails, original tokens are returned unchanged
- Both macros only work on function items (`syn::ItemFn`)
- The macros also process macro invocations within the function body
- All transformations preserve the original attributes and spans where possible

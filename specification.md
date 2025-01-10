# The Numerus Language
Numerus is a toy programming language to learn about building compilers, the syntax is very simple
- each line of the program is a statement
- each statement is either an assignment or an expression
### Expressions
- any line that is an expression gets printed to stdout
- expressions are mathematical equations made up of calls, variables, number literals, operators, and parenthesis
    - for example `2 + 3.0` or `f(4) / (17 - x)`
    - function calls can have multiple parameters `func(arg1, arg2)`
    - identifiers for variables and functions must start with a letter, but can follow with numbers and underscores as well
    - number literals can optionally have a decimal component
    - `+`, `-`, `*`, `/`, `^`, `%` are the allowed operators, `^` is for exponentiation not xor
    - parenthesis are used for order of operations
### assignments
- assignments are used to assign values to variables and functions and do not get printed out
- assignments are in the form `identifier = expression`
    - function signatures are in the form of `identifier(arg1, arg2)`
    - the arguments can be used in the right side of the function declaration
    - recursive functions are not available

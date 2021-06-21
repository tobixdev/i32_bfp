# i32_bfp
This project is for the course "dynamic compilation" at the Technical University of Vienna. 

The main goal of this project is to create a working code repository for a JIT-compiler. The actual compiler and other aspects of the project are kept simple in order to limit the complexity of the project. When a new function is declared the AST is stored in memory. Once a function is called a stub procedure is executed which calls the compiler and replaces the stub with the actual code. Afterwards, the newly compiled code is executed.

# Usage

1) Check out this repository
2) Optional: run `cargo test` to run the (few) tests
3) Run the program via cargo: `cargo run --release`. (Release mode is recommended for better performance)
4) Use the tool via the command line :)


# Supported Operations

- Arithmetic (+, -. *, /, %)
- Relators (>=, <=, =, <>, >, <)
- Define functions (f(x) := x + 1)
- Function calls
- `.code <function_name>` shows the hex representation of the compiled code
- `.list` list all defined functions
- `.delete <function_name>` deletes a function
- `.mode (proof | fast | benchmark)` switches between execution modes (how many numbers are tested)
- `.executor (compiled | interpreted)` switches executor
- `.test <expression>` tests if the expression is evaluated equivalently for both execution modes on the interval `[-1000,1000]` (good for testing)
- `.benchmark` runs 3 queries against both executors and prints the time
- `quit` quits the application

# Limitations

- Only works on `x86-64` machines.
- Only supports function call with at most one parameter
- many other handy things...

# Some things you could improve

Probably you want to do a university project by just improving on this code. Because, the language / compiler is relatively simple you should be able to extend / improve the functionality in no time. I'm happy to help with any open questions. See the list below for improvement ideas:

- Implement more operators (e.g. `&&`, `||`, ....)
- Better compiler
  - So many improvements possible...
- Remove "graveyard" hack
  - Currently we keep the stub procedure in memory in order to avoid access violations when returning from the newly compiled function. A better approach would just delete the activation record of the stub procedure (at least in my theory).
- Support functions with arbitrary number of parameters
- ...

Please don't use this for anything serious, but you're more than welcome to experiment with the code :).

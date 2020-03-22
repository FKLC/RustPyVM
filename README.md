# PyVM
PyVM is a Python Virtual Machine implemented in Rust. For learning purposes.

## What can it run?
Not much. Only 39 instruction types are implemented and there are 130 different types of instructions in Python 3.8. Also, not all types are implemented neither built-in functions (but hey, we have the `print` function).

So basically these are implemented:
 - Variables
 - `int`, `bool`, `float`, `str` and `None` types.
 - `add`, `subtract`, `multiply`, `true_divide` and `floor_divide` operations.
 - `<`, `<=`,  `==`, `!=`, `>` and `>=` comparison operations
 - Only `if/elif/else` and `while` because `for` requires more thing to be implemented
 - Supports only functions with positional arguments
 - Global and local scope but not `global` keyword
 - Deleting variables (only from local)

## Then, what is the purpose?
The purpose is learning about both Python's Virtual Machine and Rust. Hence the code is ugly and slow but this is my first program in Rust, so this is expected.

## Usage
 1. Edit `bytecode_gen/source.py`
 1. Run any of the batch files according to the situation:
     - `compile_to_json.bat`: Creates a file called `bytecode.json` that contains instructions and all that stuff.
     - `run.bat`: Runs the virtual machine with the instructions from `bytecode.json`
     - `compile_to_json_and_run.bat`: Creates `bytecode.json` and runs the virtual machine

import dis, sys, types, json


def to_camel_case(str):
    return "".join(map(lambda x: x[0].upper() + x[1:].lower(), str.split("_")))


def parse_code(code, parsed_code={"instructions": [], "constants": []}):
    parsed_code["co_names"] = code.co_names
    parsed_code["co_varnames"] = code.co_varnames

    bytecode = dis.Bytecode(code)
    for instruction in bytecode:
        parsed_code["instructions"].append(
            {to_camel_case(instruction.opname): instruction.arg}
        )

    for constant in code.co_consts:
        if isinstance(constant, types.CodeType):
            code = {"instructions": [], "constants": []}
            parse_code(constant, code)
            parsed_code["constants"].append({"Frame": code})
        else:
            parsed_code["constants"].append({to_camel_case(type(constant).__name__): constant})

    return parsed_code


if __name__ == "__main__":
    with open(sys.argv[1]) as source_file:
        source = source_file.read()
    code = compile(source, sys.argv[1], "exec")
    print(json.dumps(parse_code(code)))

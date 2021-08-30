use crate::functions::FunctionList;
use crate::prelude::Primitive;
use crate::types::{Field, Type, Variant};
use inflector::Inflector;
use std::collections::BTreeSet;
use std::fs;

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    serializable_types: BTreeSet<Type>,
    mut deserializable_types: BTreeSet<Type>,
    path: &str,
) {
    let mut all_types = serializable_types;
    all_types.append(&mut deserializable_types);

    generate_type_bindings(&all_types, path);

    let import_decls = format_function_declarations(&import_functions, FunctionType::Import);
    let export_decls = format_function_declarations(&export_functions, FunctionType::Export);

    let import_wrappers = format_import_wrappers(&import_functions);
    let export_wrappers = format_export_wrappers(&export_functions);

    let contents = format!(
        "import {{ encode, decode }} from \"@msgpack/msgpack\";

import * from \"./types\";

type FatPtr = bigint;

export type Imports = {{
{}
}};

export type Exports = {{
{}
}};

/**
 * Represents an unrecoverable error in the FP runtime.
 *
 * After this, your only recourse is to create a new runtime, probably with a different WASM plugin.
 */
export class FPRuntimeError extends Error {{
    constructor(message) {{
        super(message);
    }}
}}

/**
 * Creates a runtime for executing the given plugin.
 *
 * @param plugin The raw WASM plugin.
 * @param importFunctions The host functions that may be imported by the plugin.
 * @returns The functions that may be exported by the plugin.
 */
export async function createRuntime(
    plugin: ArrayBuffer,
    importFunctions: Imports
): Promise<Exports> {{
    const promises = new Map<FatPtr, (result: unknown) => void>();

    function assignAsyncValue<T>(fatPtr: FatPtr, result: T) {{
        const [ptr, len] = fromFatPtr(fatPtr);
        const buffer = new Uint32Array(memory.buffer, ptr, len / 4);
        const [resultPtr, resultLen] = fromFatPtr(serializeObject(result));
        buffer[1] = resultPtr;
        buffer[2] = resultLen;
        buffer[0] = 1; // Set status to ready.
    }}

    function createAsyncValue(): FatPtr {{
        const len = 12; // std::mem::size_of::<AsyncValue>()
        const fatPtr = malloc(len);
        const [ptr] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        buffer.fill(0);
        return fatPtr;
    }}

    function parseObject<T>(fatPtr: FatPtr): T {{
        const [ptr, len] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        const object = decode<T>(buffer) as T;
        free(fatPtr);
        return object;
    }}

    function promiseFromPtr<T>(ptr: FatPtr): Promise<T> {{
        return new Promise<T>((resolve) => {{
            promises.set(ptr, resolve as (result: unknown) => void);
        }});
    }}

    function resolvePromise(ptr: FatPtr) {{
        const resolve = promises.get(ptr);
        if (resolve) {{
            const [asyncPtr, asyncLen] = fromFatPtr(ptr);
            const buffer = new Uint32Array(memory.buffer, asyncPtr, asyncLen / 4);
            switch (buffer[0]) {{
                case 0:
                    throw new FPRuntimeError(\"Tried to resolve promise that is not ready\");
                case 1:
                    resolve(parseObject(toFatPtr(buffer[1]!, buffer[2]!)));
                    break;
                default:
                    throw new FPRuntimeError(\"Unexpected status: \" + buffer[0]);
            }}
        }} else {{
            throw new FPRuntimeError(\"Tried to resolve unknown promise\");
        }}
    }}

    function serializeObject<T>(object: T): FatPtr {{
        const serialized = encode(object);
        const fatPtr = malloc(serialized.length);
        const [ptr, len] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        buffer.set(serialized);
        return fatPtr;
    }}

    const {{ instance }} = await WebAssembly.instantiate(plugin, {{
        fp: {{
            __fp_host_resolve_async_value: resolvePromise,
{}
        }},
    }});

    const getExport = <T>(name: string): T => {{
        const exp = instance.exports[name];
        if (!exp) {{
            throw new FPRuntimeError(`Plugin did not export expected symbol: \"${{name}}\"`);
        }}
        return exp as unknown as T;
    }};

    const memory = getExport<WebAssembly.Memory>(\"memory\");
    const malloc = getExport<(len: number) => FatPtr>(\"__fp_malloc\");
    const free = getExport<(ptr: FatPtr) => void>(\"__fp_free\");
    const resolveFuture = getExport<(ptr: FatPtr) => void>(\"__fp_guest_resolve_async_value\");

    return {{
{}
    }};
}}

function fromFatPtr(fatPtr: FatPtr): [ptr: number, len: number] {{
    return [
        Number.parseInt((fatPtr >> 32n).toString()),
        Number.parseInt((fatPtr & 0xffff_ffffn).toString()),
    ];
}}

function toFatPtr(ptr: number, len: number): FatPtr {{
    return (BigInt(ptr) << 32n) | BigInt(len);
}}
",
        join_lines(&import_decls, |line| format!("   {};", line)),
        join_lines(&export_decls, |line| format!("   {};", line)),
        join_lines(&import_wrappers, |line| format!("            {},", line)),
        join_lines(&export_wrappers, |line| format!("        {},", line)),
    );
    write_bindings_file(format!("{}/index.ts", path), &contents);
}

enum FunctionType {
    Import,
    Export,
}

fn format_function_declarations(
    functions: &FunctionList,
    function_type: FunctionType,
) -> Vec<String> {
    // Plugins can always omit exports, while runtimes are always expected to provide all imports:
    let optional_marker = match function_type {
        FunctionType::Import => "",
        FunctionType::Export => "?",
    };

    functions
        .iter()
        .map(|function| {
            let args = function
                .args
                .iter()
                .map(|arg| {
                    format!(
                        "{}{}: {}",
                        arg.name.to_camel_case(),
                        optional_marker,
                        format_type(&arg.ty)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = match &function.return_type {
                None => "".to_owned(),
                Some(ty) => {
                    if function.is_async {
                        format!(" => Promise<{}>", format_type(ty))
                    } else {
                        format!(" => {}", format_type(ty))
                    }
                }
            };
            format!(
                "    {}: ({}){},",
                function.name.to_camel_case(),
                args,
                return_type
            )
        })
        .collect()
}

fn format_import_wrappers(import_functions: &FunctionList) -> Vec<String> {
    import_functions
        .into_iter()
        .map(|function| {
            let name = &function.name;
            let args_with_ptr_types = function
                .args
                .iter()
                .map(|arg| match &arg.ty {
                    Type::Primitive(primitive) => format!(
                        "{}: {}",
                        arg.name.to_camel_case(),
                        format_primitive(*primitive)
                    ),
                    _ => format!("{}_ptr: FatPtr", arg.name),
                })
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = match &function.return_type {
                None => "".to_owned(),
                Some(Type::Primitive(primitive)) => format!(": {}", format_primitive(*primitive)),
                Some(_) => ": FatPtr".to_owned(),
            };
            let import_args = function
                .args
                .iter()
                .filter_map(|arg| match &arg.ty {
                    Type::Primitive(_) => None,
                    ty => Some(format!(
                        "const {} = parseObject<{}>({}_ptr);",
                        arg.name.to_camel_case(),
                        format_type(ty),
                        arg.name
                    )),
                })
                .collect::<Vec<_>>();
            let args = function
                .args
                .iter()
                .map(|arg| arg.name.to_camel_case())
                .collect::<Vec<_>>()
                .join(", ");
            if function.is_async {
                let assign_async_value = match &function.return_type {
                    None => "",
                    Some(Type::Primitive(_)) => {
                        "\n            assignAsyncValue(_async_result_ptr, result);"
                    }
                    Some(_) => {
                        "\n            assignAsyncValue(_async_result_ptr, serializeObject(result));"
                    },
                };

                format!(
                    "{}: ({}){} {{
{}
    const _async_result_ptr = createAsyncValue();
    importFunctions.{}({})
        .then((result) => {{{}
            resolveFuture(_async_result_ptr);
        }})
        .catch((error) => {{
            console.error(
                'Unrecoverable exception trying to call async plugin function \"${}\"',
                error
            );
        }});
    return _async_result_ptr;
}}",
                    name.to_camel_case(),
                    args_with_ptr_types,
                    return_type,
                    join_lines(&import_args, |line| format!("    {}", line)),
                    name,
                    args,
                    assign_async_value,
                    name
                )
            } else {
                let fn_call = match &function.return_type {
                    None => format!("importFunctions.{}({});", name, args),
                    Some(Type::Primitive(_)) => {
                        format!("return importFunctions.{}({});", name, args)
                    }
                    Some(_) => format!(
                        "return serializeObject(importFunctions.{}({}));",
                        name, args
                    ),
                };

                format!(
                    "{}: ({}){} {{
{}
    {}
}}",
                    name.to_camel_case(),
                    args_with_ptr_types,
                    return_type,
                    join_lines(&import_args, |line| format!("    {}", line)),
                    fn_call
                )
            }
        })
        .collect()
}

fn format_export_wrappers(import_functions: &FunctionList) -> Vec<String> {
    import_functions
        .into_iter()
        .map(|function| {
            let name = &function.name;
            let args_with_ptr_types = function
                .args
                .iter()
                .map(|arg| match &arg.ty {
                    Type::Primitive(primitive) => format!(
                        "{}: {}",
                        arg.name.to_camel_case(),
                        format_primitive(*primitive)
                    ),
                    _ => format!("{}_ptr: FatPtr", arg.name),
                })
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = match &function.return_type {
                None => "".to_owned(),
                Some(Type::Primitive(primitive)) => format!(": {}", format_primitive(*primitive)),
                Some(_) => ": FatPtr".to_owned(),
            };
            let import_args = function
                .args
                .iter()
                .filter_map(|arg| match &arg.ty {
                    Type::Primitive(_) => None,
                    ty => Some(format!(
                        "const {} = parseObject<{}>({}_ptr);",
                        arg.name.to_camel_case(),
                        format_type(ty),
                        arg.name
                    )),
                })
                .collect::<Vec<_>>();
            let args = function
                .args
                .iter()
                .map(|arg| arg.name.to_camel_case())
                .collect::<Vec<_>>()
                .join(", ");
            let fn_call = match &function.return_type {
                None => format!("importFunctions.{}({});", name, args),
                Some(Type::Primitive(_)) => {
                    format!("return importFunctions.{}({});", name, args)
                }
                Some(_) => format!(
                    "return serializeObject(importFunctions.{}({}));",
                    name, args
                ),
            };
            format!(
                "{}: (() => {{
        const fn = instance.exports.{};
        if (!fn) return;

{}    ({}){} {{
    {}
        {}
    }}
}})()",
                name.to_camel_case(),
                name,
                args_with_ptr_types,
                return_type,
                join_lines(&import_args, |line| format!("    {}", line)),
                fn_call
            )
        })
        .collect()
}

fn generate_type_bindings(types: &BTreeSet<Type>, path: &str) {
    let type_defs = types
        .iter()
        .filter_map(|ty| match ty {
            Type::Enum(name, variants) => Some(create_enum_definition(name, variants)),
            Type::Struct(name, fields) => Some(create_struct_definition(name, fields)),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    write_bindings_file(format!("{}/types.ts", path), format!("{}\n", type_defs))
}

fn create_enum_definition(name: &str, variants: &[Variant]) -> String {
    "TODO".to_owned() // TODO
}

fn create_struct_definition(name: &str, fields: &[Field]) -> String {
    let fields = fields
        .iter()
        .map(|field| {
            format!(
                "    {}: {};",
                field.name.to_camel_case(),
                format_type(&field.ty)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("export type {} = {{\n{}\n}};", name, fields)
}

/// Formats a type so it's valid TypeScript.
fn format_type(ty: &Type) -> String {
    match ty {
        Type::Enum(name, _) => name.clone(),
        Type::List(_, ty) => format!("Array<{}>", format_type(ty)),
        Type::Map(_, k, v) => format!("Record<{}, {}>", format_type(k), format_type(v)),
        Type::Option(ty) => format!("{} | undefined", format_type(ty)),
        Type::Primitive(primitive) => format_primitive(*primitive),
        Type::Struct(name, _) => name.clone(),
    }
}

fn format_primitive(primitive: Primitive) -> String {
    let string = match primitive {
        Primitive::Bool => "boolean",
        Primitive::F32 => "number",
        Primitive::F64 => "number",
        Primitive::I8 => "number",
        Primitive::I16 => "number",
        Primitive::I32 => "number",
        Primitive::I64 => "bigint",
        Primitive::I128 => "bigint",
        Primitive::String => "string",
        Primitive::U8 => "number",
        Primitive::U16 => "number",
        Primitive::U32 => "number",
        Primitive::U64 => "bigint",
        Primitive::U128 => "bigint",
    };
    string.to_owned()
}

fn join_lines<F>(lines: &[String], formatter: F) -> String
where
    F: FnMut(&String) -> String,
{
    lines.iter().map(formatter).collect::<Vec<_>>().join("\n")
}

fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}

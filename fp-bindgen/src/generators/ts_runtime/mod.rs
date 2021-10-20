use crate::functions::FunctionList;
use crate::prelude::Primitive;
use crate::serializable::Serializable;
use crate::types::{
    format_name_with_generics, EnumOptions, Field, GenericArgument, StructOptions, Type, Variant,
};
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

    let mut type_names = all_types
        .into_iter()
        .filter_map(|ty| match ty {
            Type::Alias(name, _) => Some(name),
            Type::Enum(name, _, _, _, _) => Some(name),
            Type::Struct(name, _, _, _, _) => Some(name),
            _ => None,
        })
        .collect::<Vec<_>>();
    type_names.dedup();

    let import_wrappers = format_import_wrappers(&import_functions);
    let export_wrappers = format_export_wrappers(&export_functions);

    let contents = format!(
        "// ============================================= //
// WebAssembly runtime for TypeScript            //
//                                               //
// This file is generated. PLEASE DO NOT MODIFY. //
// ============================================= //

import {{ encode, decode }} from \"@msgpack/msgpack\";

import type {{
{}}} from \"./types\";

type FatPtr = bigint;

export type Imports = {{
{}}};

export type Exports = {{
{}}};

/**
 * Represents an unrecoverable error in the FP runtime.
 *
 * After this, your only recourse is to create a new runtime, probably with a different WASM plugin.
 */
export class FPRuntimeError extends Error {{
    constructor(message: string) {{
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

    function resolvePromise(asyncValuePtr: FatPtr, resultPtr: FatPtr) {{
        const resolve = promises.get(asyncValuePtr);
        if (!resolve) {{
            throw new FPRuntimeError(\"Tried to resolve unknown promise\");
        }}

        resolve(resultPtr ? parseObject(resultPtr) : undefined);
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
{}        }},
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
    const resolveFuture = getExport<(asyncValuePtr: FatPtr, resultPtr: FatPtr) => void>(\"__fp_guest_resolve_async_value\");

    return {{
{}    }};
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
        join_lines(&type_names, |line| format!("    {},", line)),
        join_lines(&import_decls, |line| format!("    {};", line)),
        join_lines(&export_decls, |line| format!("    {};", line)),
        join_lines(&import_wrappers, |line| format!("            {}", line)),
        join_lines(&export_wrappers, |line| format!("        {}", line)),
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
                .map(|arg| format!("{}: {}", arg.name.to_camel_case(), format_type(&arg.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = if function.is_async {
                format!(" => Promise<{}>", format_type(&function.return_type))
            } else {
                format!(" => {}", format_type(&function.return_type))
            };
            format!(
                "{}{}: ({}){}",
                function.name.to_camel_case(),
                optional_marker,
                args,
                return_type
            )
        })
        .collect()
}

fn format_import_wrappers(import_functions: &FunctionList) -> Vec<String> {
    import_functions
        .into_iter()
        .flat_map(|function| {
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
                Type::Unit => "".to_owned(),
                Type::Primitive(primitive) => format!(": {}", format_primitive(*primitive)),
                _ => ": FatPtr".to_owned(),
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
                let async_result = match &function.return_type {
                    Type::Unit => "0",
                    _ => "serializeObject(result)",
                };

                format!(
                    "__fp_gen_{}: ({}){} => {{
{}    const _async_result_ptr = createAsyncValue();
    importFunctions.{}({})
        .then((result) => {{
            resolveFuture(_async_result_ptr, {});
        }})
        .catch((error) => {{
            console.error(
                'Unrecoverable exception trying to call async host function \"{}\"',
                error
            );
        }});
    return _async_result_ptr;
}},",
                    name,
                    args_with_ptr_types,
                    return_type,
                    import_args
                        .iter()
                        .map(|line| format!("    {}\n", line))
                        .collect::<Vec<_>>()
                        .join(""),
                    name.to_camel_case(),
                    args,
                    async_result,
                    name
                )
                .split('\n')
                .map(|line| line.to_owned())
                .collect::<Vec<_>>()
            } else {
                let fn_call = match &function.return_type {
                    Type::Unit => format!("importFunctions.{}({});", name.to_camel_case(), args),
                    Type::Primitive(_) => {
                        format!("return importFunctions.{}({});", name.to_camel_case(), args)
                    }
                    _ => format!(
                        "return serializeObject(importFunctions.{}({}));",
                        name.to_camel_case(),
                        args
                    ),
                };

                format!(
                    "__fp_gen_{}: ({}){} => {{\n{}    {}\n}},",
                    name,
                    args_with_ptr_types,
                    return_type,
                    import_args
                        .iter()
                        .map(|line| format!("    {}\n", line))
                        .collect::<Vec<_>>()
                        .join(""),
                    fn_call
                )
                .split('\n')
                .map(|line| line.to_owned())
                .collect::<Vec<_>>()
            }
        })
        .collect()
}

fn format_export_wrappers(import_functions: &FunctionList) -> Vec<String> {
    import_functions
        .into_iter()
        .flat_map(|function| {
            let name = &function.name;
            let args = function
                .args
                .iter()
                .map(|arg| format!("{}: {}", arg.name.to_camel_case(), format_type(&arg.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let export_args = function
                .args
                .iter()
                .filter_map(|arg| match &arg.ty {
                    Type::Primitive(_) => None,
                    _ => Some(format!(
                        "const {}_ptr = serializeObject({});",
                        arg.name,
                        arg.name.to_camel_case()
                    )),
                })
                .collect::<Vec<_>>();

            // Trivial functions can simply be returned as is:
            if export_args.is_empty() && !function.is_async {
                return vec![format!(
                    "{}: instance.exports.__fp_gen_{} as any,",
                    name.to_camel_case(),
                    name
                )];
            }

            let call_args = function
                .args
                .iter()
                .map(|arg| match &arg.ty {
                    Type::Primitive(_) => arg.name.to_camel_case(),
                    _ => format!("{}_ptr", arg.name),
                })
                .collect::<Vec<_>>()
                .join(", ");
            let fn_call = if function.is_async {
                format!(
                    "return promiseFromPtr<{}>(export_fn({}));",
                    format_type(&function.return_type),
                    call_args
                )
            } else {
                match &function.return_type {
                    Type::Unit => format!("export_fn({});", call_args),
                    Type::Primitive(_) => {
                        format!("return export_fn({});", call_args)
                    }
                    ty => format!(
                        "return parseObject<{}>(export_fn({}));",
                        format_type(ty),
                        call_args
                    ),
                }
            };
            let return_fn = if export_args.is_empty() {
                format!("return ({}) => {}", args, fn_call.replace("return ", ""))
            } else {
                format!(
                    "return ({}) => {{\n{}        {}\n    }};",
                    args,
                    join_lines(&export_args, |line| format!("        {}", line)),
                    fn_call
                )
            };
            format!(
                "{}: (() => {{
    const export_fn = instance.exports.__fp_gen_{} as any;
    if (!export_fn) return;

    {}
}})(),",
                name.to_camel_case(),
                name,
                return_fn
            )
            .split('\n')
            .map(str::to_owned)
            .collect::<Vec<_>>()
        })
        .collect()
}

fn generate_type_bindings(types: &BTreeSet<Type>, path: &str) {
    let types = types
        .iter()
        .filter_map(|ty| match ty {
            Type::Enum(name, generic_args, _, _, _) => {
                if name == "Result" {
                    // Just make sure we get an unspecialized version of the type...
                    Some(Result::<u8, u8>::ty())
                } else if generic_args.iter().all(|arg| arg.ty.is_none()) {
                    Some(ty.clone())
                } else {
                    None // We don't generate definitions for specialized types.
                }
            }
            Type::Struct(_, generic_args, _, _, _) => {
                if generic_args.iter().all(|arg| arg.ty.is_none()) {
                    Some(ty.clone())
                } else {
                    None // We don't generate definitions for specialized types.
                }
            }
            other => Some(other.clone()),
        })
        .collect::<BTreeSet<_>>();

    let type_defs = types
        .iter()
        .filter_map(|ty| match ty {
            Type::Alias(name, ty) => Some(format!(
                "export type {} = {};",
                name,
                format_type(ty.as_ref())
            )),
            Type::Enum(name, generic_args, doc_lines, variants, opts) => Some(
                create_enum_definition(name, generic_args, doc_lines, variants, opts.clone()),
            ),
            Type::Struct(name, generic_args, doc_lines, fields, opts) => Some(
                create_struct_definition(name, generic_args, doc_lines, fields, opts.clone()),
            ),
            _ => None,
        })
        .collect::<Vec<_>>();

    write_bindings_file(
        format!("{}/types.ts", path),
        format!(
            "// ============================================= //
// Types for WebAssembly runtime                 //
//                                               //
// This file is generated. PLEASE DO NOT MODIFY. //
// ============================================= //

{}\n",
            type_defs.join("\n\n")
        ),
    )
}

fn create_enum_definition(
    name: &str,
    generic_args: &[GenericArgument],
    doc_lines: &[String],
    variants: &[Variant],
    opts: EnumOptions,
) -> String {
    let variants = variants
        .iter()
        .map(|variant| {
            let variant_name = opts.variant_casing.format_string(&variant.name);
            let variant_decl = match &variant.ty {
                Type::Unit => {
                    if let Some(tag) = &opts.tag_prop_name {
                        format!("| {{ {}: \"{}\" }}", tag, variant_name)
                    } else {
                        format!("| \"{}\"", variant_name)
                    }
                }
                Type::Struct(_, _, _, fields, _) => {
                    if opts.untagged {
                        format!("| {{ {} }}", format_struct_fields(fields).join(" "))
                    } else {
                        let field_lines = format_struct_fields(fields);
                        let formatted_fields = if field_lines.len() > fields.len() {
                            format!(
                                "\n{}",
                                join_lines(&field_lines, |line| format!("    {}", line))
                            )
                        } else {
                            format!(" {} ", field_lines.join("").trim_end_matches(';'))
                        };

                        match (&opts.tag_prop_name, &opts.content_prop_name) {
                            (Some(tag), Some(content)) => {
                                format!(
                                    "| {{ {}: \"{}\"; {}: {{{}}} }}",
                                    tag, variant_name, content, formatted_fields
                                )
                            }
                            (Some(tag), None) => {
                                let space = if formatted_fields.contains('\n') {
                                    "\n    "
                                } else {
                                    " "
                                };
                                format!(
                                    "| {{{}{}: \"{}\";{}}}",
                                    space, tag, variant_name, formatted_fields
                                )
                            }
                            (None, _) => {
                                format!("| {{ {}: {{{}}} }}", variant_name, formatted_fields)
                            }
                        }
                    }
                }
                Type::Tuple(items) if items.len() == 1 => {
                    if opts.untagged {
                        format!("| {}", format_type(items.first().unwrap()))
                    } else {
                        match (&opts.tag_prop_name, &opts.content_prop_name) {
                            (Some(tag), Some(content)) => {
                                format!(
                                    "| {{ {}: \"{}\"; {}: {} }}",
                                    tag,
                                    variant_name,
                                    content,
                                    format_type(items.first().unwrap())
                                )
                            }
                            (Some(tag), None) => {
                                format!(
                                    "| {{ {}: \"{}\" }} & {}",
                                    tag,
                                    variant_name,
                                    format_type(items.first().unwrap())
                                )
                            }
                            (None, _) => {
                                format!(
                                    "| {{ {}: {} }}",
                                    variant_name,
                                    format_type(items.first().unwrap())
                                )
                            }
                        }
                    }
                }
                other => panic!("Unsupported type for enum variant: {:?}", other),
            };

            let lines = if variant.doc_lines.is_empty() {
                variant_decl
                    .split('\n')
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            } else {
                let mut lines = format_docs(&variant.doc_lines);
                lines.append(
                    &mut variant_decl
                        .split('\n')
                        .map(str::to_owned)
                        .collect::<Vec<_>>(),
                );
                lines
            };

            join_lines(&lines, |line| format!("    {}", line))
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        "{}export type {} =\n{};",
        join_lines(&format_docs(doc_lines), String::to_owned),
        format_name_with_generics(name, generic_args),
        variants.trim_end()
    )
}

fn create_struct_definition(
    name: &str,
    generic_args: &[GenericArgument],
    doc_lines: &[String],
    fields: &[Field],
    _: StructOptions,
) -> String {
    format!(
        "{}export type {} = {{\n{}}};",
        join_lines(&format_docs(doc_lines), String::to_owned),
        format_name_with_generics(name, generic_args),
        join_lines(&format_struct_fields(fields), |line| format!(
            "    {}",
            line
        ))
        .trim_start_matches('\n')
    )
}

fn format_docs(doc_lines: &[String]) -> Vec<String> {
    if doc_lines.is_empty() {
        Vec::new()
    } else {
        let mut lines = vec!["/**".to_owned()];
        lines.append(
            &mut doc_lines
                .iter()
                .map(|doc_line| format!(" *{}", doc_line))
                .collect(),
        );
        lines.push(" */".to_owned());
        lines
    }
}

fn format_name_with_types(name: &str, generic_args: &[GenericArgument]) -> String {
    if generic_args.is_empty() {
        name.to_owned()
    } else {
        format!(
            "{}<{}>",
            name,
            generic_args
                .iter()
                .map(|arg| match &arg.ty {
                    Some(ty) => format_type(ty),
                    None => arg.name.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn format_struct_fields(fields: &[Field]) -> Vec<String> {
    let format_opts = FormatOptions {
        optimize_binary_types: true,
    };

    fields
        .iter()
        .flat_map(|field| {
            let field_decl = match &field.ty {
                Type::Container(name, ty) => {
                    let optional = if name == "Option" { "?" } else { "" };
                    format!(
                        "{}{}: {};",
                        field.name.to_camel_case(),
                        optional,
                        format_type_with_options(ty, format_opts)
                    )
                }
                ty => format!(
                    "{}: {};",
                    field.name.to_camel_case(),
                    format_type_with_options(ty, format_opts)
                ),
            };
            if field.doc_lines.is_empty() {
                vec![field_decl]
            } else {
                let mut lines = vec!["".to_owned()];
                lines.append(&mut format_docs(&field.doc_lines));
                lines.push(field_decl);
                lines
            }
        })
        .collect()
}

/// Formats a type so it's valid TypeScript.
fn format_type(ty: &Type) -> String {
    format_type_with_options(ty, FormatOptions::default())
}

#[derive(Clone, Copy)]
struct FormatOptions {
    optimize_binary_types: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        FormatOptions {
            // We can only optimize in limited contexts, so default is `false`.
            optimize_binary_types: false,
        }
    }
}

fn format_type_with_options(ty: &Type, opts: FormatOptions) -> String {
    match ty {
        Type::Alias(name, _) => name.clone(),
        Type::Container(name, ty) => {
            if name == "Option" {
                format!("{} | null", format_type_with_options(ty, opts))
            } else {
                format_type_with_options(ty, opts)
            }
        }
        Type::Custom(custom) => custom.ts_ty.clone(),
        Type::Enum(name, generic_args, _, _, _) => format_name_with_types(name, generic_args),
        Type::GenericArgument(arg) => arg.name.clone(),
        Type::List(name, ty) => {
            if opts.optimize_binary_types
                && name == "Vec"
                && ty.as_ref() == &Type::Primitive(Primitive::U8)
            {
                "ArrayBuffer".to_owned()
            } else {
                format!("Array<{}>", format_type_with_options(ty, opts))
            }
        }
        Type::Map(_, k, v) => format!(
            "Record<{}, {}>",
            format_type_with_options(k, opts),
            format_type_with_options(v, opts)
        ),
        Type::Primitive(primitive) => format_primitive(*primitive),
        Type::String => "string".to_owned(),
        Type::Struct(name, generic_args, _, _, _) => format_name_with_types(name, generic_args),
        Type::Tuple(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|item| item.name())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Unit => "void".to_owned(),
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
        Primitive::U8 => "number",
        Primitive::U16 => "number",
        Primitive::U32 => "number",
        Primitive::U64 => "bigint",
    };
    string.to_owned()
}

fn join_lines<F>(lines: &[String], formatter: F) -> String
where
    F: Fn(&String) -> String,
{
    let lines = lines
        .iter()
        .map(|line| {
            if line.is_empty() {
                line.clone()
            } else {
                formatter(line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if lines.is_empty() {
        lines
    } else {
        format!("{}\n", lines)
    }
}

fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}

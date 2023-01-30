use crate::{
    casing::Casing,
    functions::{Function, FunctionList},
    prelude::Primitive,
    types::{CustomType, Enum, EnumOptions, Field, Struct, Type, TypeIdent, TypeMap, Variant},
    TsExtendedRuntimeConfig,
};
use inflector::Inflector;
use std::fs;

pub(crate) fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    types: TypeMap,
    config: TsExtendedRuntimeConfig,
    path: &str,
) {
    generate_type_bindings(&types, path);

    let import_decls =
        format_function_declarations(&import_functions, &types, FunctionType::Import);
    let export_decls =
        format_function_declarations(&export_functions, &types, FunctionType::Export);
    let raw_export_decls = if config.generate_raw_export_wrappers {
        format_raw_function_declarations(&export_functions, FunctionType::Export)
    } else {
        Vec::new()
    };

    let has_async_import_functions = import_functions.iter().any(|function| function.is_async);
    let has_async_export_functions = export_functions.iter().any(|function| function.is_async);

    let mut import_wrappers = format_import_wrappers(&import_functions, &types);
    if has_async_export_functions {
        import_wrappers.push("__fp_host_resolve_async_value: resolvePromise,".to_owned());
    }

    let export_wrappers = format_export_wrappers(&export_functions, &types);
    let raw_export_wrappers = if config.generate_raw_export_wrappers {
        format_raw_export_wrappers(&export_functions)
    } else {
        Vec::new()
    };

    let contents = format!(
        "// ============================================= //
// WebAssembly runtime for TypeScript            //
//                                               //
// This file is generated. PLEASE DO NOT MODIFY. //
// ============================================= //
// deno-lint-ignore-file no-explicit-any no-unused-vars

import {{ encode, decode }} from \"{}\";

import type * as types from \"./types{}\";

type FatPtr = bigint;

export type Imports = {{
{}}};

export type Exports = {{
{}{}}};

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
    const promises = new Map<FatPtr, ((result: FatPtr) => void) | FatPtr>();

    function createAsyncValue(): FatPtr {{
        const len = 12; // std::mem::size_of::<AsyncValue>()
        const fatPtr = malloc(len);
        const [ptr] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        buffer.fill(0);
        return fatPtr;
    }}

    function interpretSign(num: number, cap: number) {{
        if (num < cap) {{
            return num;
        }} else {{
            return num - (cap << 1);
        }}
    }}

    function interpretBigSign(num: bigint, cap: bigint) {{
        if (num < cap) {{
            return num;
        }} else {{
            return num - (cap << 1n);
        }}
    }}

    function parseObject<T>(fatPtr: FatPtr): T {{
        const [ptr, len] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        // Without creating a copy of the memory, we risk corruption of any
        // embedded `Uint8Array` objects returned from `decode()` after `free()`
        // has been called :(
        const copy = new Uint8Array(len);
        copy.set(buffer);
        free(fatPtr);
        const object = decode(copy) as unknown as T;
        return object;
    }}

    function promiseFromPtr(ptr: FatPtr): Promise<FatPtr> {{
        const resultPtr = promises.get(ptr);
        if (resultPtr) {{
            if (typeof resultPtr === \"function\") {{
                throw new FPRuntimeError(\"Already created promise for this value\");
            }}

            promises.delete(ptr);
            return Promise.resolve(resultPtr);
        }} else {{
            return new Promise((resolve) => {{
                promises.set(ptr, resolve as (result: FatPtr) => void);
            }});
        }}
    }}

    function resolvePromise(asyncValuePtr: FatPtr, resultPtr: FatPtr) {{
        const resolve = promises.get(asyncValuePtr);
        if (resolve) {{
            if (typeof resolve !== \"function\") {{
                throw new FPRuntimeError(\"Tried to resolve invalid promise\");
            }}

            promises.delete(asyncValuePtr);
            resolve(resultPtr);
        }} else {{
            promises.set(asyncValuePtr, resultPtr);
        }}
    }}

    function serializeObject<T>(object: T): FatPtr {{
        return exportToMemory(encode(object));
    }}

    function exportToMemory(serialized: Uint8Array): FatPtr {{
        const fatPtr = malloc(serialized.length);
        const [ptr, len] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        buffer.set(serialized);
        return fatPtr;
    }}

    function importFromMemory(fatPtr: FatPtr): Uint8Array {{
        const [ptr, len] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        const copy = new Uint8Array(len);
        copy.set(buffer);
        free(fatPtr);
        return copy;
    }}

    const {{ instance }} = await WebAssembly.instantiate(plugin, {{
        fp: {{
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
{}
    return {{
{}{}    }};
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
        config.msgpack_module,
        // HACK: Import paths in TypeScript are a bit of a mess. Usually, you
        // shouldn't need an extension, but with some configurations you do.
        // For now, we just try to detect Deno users by looking at the
        // `msgpack_module` and accomodate them here:
        if config.msgpack_module.ends_with(".ts") {
            ".ts"
        } else {
            ""
        },
        join_lines(&import_decls, |line| format!("    {line};")),
        join_lines(&export_decls, |line| format!("    {line};")),
        join_lines(&raw_export_decls, |line| format!("    {line};")),
        join_lines(&import_wrappers, |line| format!("            {line}")),
        if has_async_import_functions {
            "    const resolveFuture = getExport<(asyncValuePtr: FatPtr, resultPtr: FatPtr) => void>(\"__fp_guest_resolve_async_value\");\n"
        } else {
            ""
        },
        join_lines(&export_wrappers, |line| format!("        {line}")),
        join_lines(&raw_export_wrappers, |line| format!("        {line}")),
    );
    write_bindings_file(format!("{path}/index.ts"), contents);
}

enum FunctionType {
    Import,
    Export,
}

fn format_function_declarations(
    functions: &FunctionList,
    types: &TypeMap,
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
                        "{}: {}",
                        arg.name.to_camel_case(),
                        format_plain_primitive_or_ident(&arg.ty, types)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = if function.is_async {
                format!(
                    " => Promise<{}>",
                    match &function.return_type {
                        Some(ty) => format_ident(ty, types, "types."),
                        None => "void".to_owned(),
                    }
                )
            } else {
                format!(
                    " => {}",
                    match &function.return_type {
                        Some(ty) => format_plain_primitive_or_ident(ty, types),
                        None => "void".to_owned(),
                    }
                )
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

fn format_raw_function_declarations(
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
        .filter(|function| !is_primitive_function(function))
        .map(|function| {
            let args = function
                .args
                .iter()
                .map(|arg| format!("{}: {}", arg.name.to_camel_case(), format_raw_type(&arg.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = if function.is_async {
                format!(
                    " => Promise<{}>",
                    function
                        .return_type
                        .as_ref()
                        .map(format_raw_type)
                        .unwrap_or("void")
                )
            } else {
                format!(
                    " => {}",
                    function
                        .return_type
                        .as_ref()
                        .map(format_raw_type)
                        .unwrap_or("void")
                )
            };
            format!(
                "{}Raw{}: ({}){}",
                function.name.to_camel_case(),
                optional_marker,
                args,
                return_type
            )
        })
        .collect()
}

fn format_import_wrappers(import_functions: &FunctionList, types: &TypeMap) -> Vec<String> {
    import_functions
        .into_iter()
        .flat_map(|function| {
            let name = &function.name;
            let args_with_ptr_types = function
                .args
                .iter()
                .map(|arg| {
                    if let Some(primitive) = arg.ty.as_primitive() {
                        format!(
                            "{}: {}",
                            arg.name.to_camel_case(),
                            format_plain_primitive(primitive)
                        )
                    } else {
                        format!("{}: FatPtr", get_pointer_name(&arg.name))
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = match &function.return_type.as_ref().map(|ty| ty.as_primitive()) {
                None => "".to_owned(),
                Some(Some(primitive)) => format!(": {}", format_plain_primitive(*primitive)),
                Some(_) => ": FatPtr".to_owned(),
            };
            let import_args = function
                .args
                .iter()
                .filter_map(|arg| {
                    if arg.ty.is_primitive() {
                        None
                    } else {
                        Some(format!(
                            "const {} = parseObject<{}>({});",
                            arg.name.to_camel_case(),
                            format_ident(&arg.ty, types, "types."),
                            get_pointer_name(&arg.name)
                        ))
                    }
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
                    Some(_) => "serializeObject(result)",
                    None => "0",
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
                        .map(|line| format!("    {line}\n"))
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
                    None => format!("importFunctions.{}({});", name.to_camel_case(), args),
                    Some(ty) if ty.is_primitive() => {
                        format!(
                            "return {};",
                            import_primitive(
                                ty,
                                &format!("importFunctions.{}({})", name.to_camel_case(), args)
                            )
                        )
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
                        .map(|line| format!("    {line}\n"))
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

fn format_export_wrappers(export_functions: &FunctionList, types: &TypeMap) -> Vec<String> {
    export_functions
        .into_iter()
        .flat_map(|function| {
            let name = &function.name;

            // Trivial functions can simply be returned as is:
            if is_primitive_function(function) {
                return vec![format!(
                    "{}: instance.exports.__fp_gen_{} as any,",
                    name.to_camel_case(),
                    name
                )];
            }

            let args = function
                .args
                .iter()
                .map(|arg| {
                    format!(
                        "{}: {}",
                        arg.name.to_camel_case(),
                        format_plain_primitive_or_ident(&arg.ty, types)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let export_args = function
                .args
                .iter()
                .filter(|arg| !arg.ty.is_primitive())
                .map(|arg| {
                    let wrapped_arg = if arg.ty.is_array() {
                        // Arrays need to be converted from a typed array to a regular array,
                        // otherwise msgpack decoding on the Rust side will fail (byte arrays
                        // cannot be deserialized to Rust arrays by rmp-serde, currently).
                        // Importing from Rust --> TS works fine though, so we don't need the
                        // conversion there.
                        format!("Array.from({})", arg.name.to_camel_case())
                    } else {
                        arg.name.to_camel_case()
                    };

                    format!(
                        "const {} = serializeObject({});",
                        get_pointer_name(&arg.name),
                        wrapped_arg
                    )
                })
                .collect::<Vec<_>>();

            let call_args = function
                .args
                .iter()
                .map(|arg| {
                    if arg.ty.is_primitive() {
                        arg.name.to_camel_case()
                    } else {
                        get_pointer_name(&arg.name)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let fn_call = if function.is_async {
                format!(
                    "return promiseFromPtr(export_fn({})).then((ptr) => parseObject<{}>(ptr));",
                    call_args,
                    function
                        .return_type
                        .as_ref()
                        .map(|ty| format_ident(ty, types, "types."))
                        .unwrap_or_else(|| "void".to_owned()),
                )
            } else {
                match &function.return_type {
                    None => format!("export_fn({call_args});"),
                    Some(ty) if ty.is_primitive() => format!(
                        "return {};",
                        import_primitive(ty, &format!("export_fn({call_args})"))
                    ),
                    Some(ty) => format!(
                        "return parseObject<{}>(export_fn({}));",
                        format_ident(ty, types, "types."),
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
                    join_lines(&export_args, |line| format!("        {line}")),
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

fn format_raw_export_wrappers(export_functions: &FunctionList) -> Vec<String> {
    export_functions
        .into_iter()
        .filter(|function| !is_primitive_function(function))
        .flat_map(|function| {
            let name = &function.name;
            let args = function
                .args
                .iter()
                .map(|arg| format!("{}: {}", arg.name.to_camel_case(), format_raw_type(&arg.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let export_args = function
                .args
                .iter()
                .filter(|arg| !arg.ty.is_primitive())
                .map(|arg| {
                    format!(
                        "const {} = exportToMemory({});",
                        get_pointer_name(&arg.name),
                        arg.name.to_camel_case()
                    )
                })
                .collect::<Vec<_>>();

            let call_args = function
                .args
                .iter()
                .map(|arg| {
                    if arg.ty.is_primitive() {
                        arg.name.to_camel_case()
                    } else {
                        get_pointer_name(&arg.name)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let fn_call = if function.is_async {
                format!("return promiseFromPtr(export_fn({call_args})).then(importFromMemory);")
            } else {
                match &function.return_type {
                    None => format!("export_fn({call_args});"),
                    Some(ty) => format!(
                        "return {};",
                        if ty.is_primitive() {
                            import_primitive(ty, &format!("export_fn({call_args})"))
                        } else {
                            format!("importFromMemory(export_fn({call_args}))")
                        }
                    ),
                }
            };
            let return_fn = if export_args.is_empty() {
                format!("return ({}) => {}", args, fn_call.replace("return ", ""))
            } else {
                format!(
                    "return ({}) => {{\n{}        {}\n    }};",
                    args,
                    join_lines(&export_args, |line| format!("        {line}")),
                    fn_call
                )
            };
            format!(
                "{}Raw: (() => {{
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

fn generate_type_bindings(types: &TypeMap, path: &str) {
    let type_defs = types
        .values()
        .filter_map(|ty| match ty {
            Type::Alias(name, ty) => Some(format!(
                "export type {} = {};",
                name,
                // Now we're in a real pickle: We don't know the context in
                // which this alias will be used. It could be either a plain
                // primitive or a MessagePack-encoded one, so we account for
                // both cases:
                match ty.name.as_str() {
                    "i64" | "u64" => "number | bigint".to_owned(),
                    _ => format_ident(ty, types, ""),
                }
            )),
            Type::Custom(CustomType {
                ts_ty,
                ts_declaration: Some(ts_declaration),
                ..
            }) => Some(format!("export type {ts_ty} = {ts_declaration};")),
            Type::Enum(ty) => Some(create_enum_definition(ty, types)),
            Type::Struct(ty) => Some(create_struct_definition(ty, types)),
            _ => None,
        })
        .collect::<Vec<_>>();

    write_bindings_file(
        format!("{path}/types.ts"),
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

fn is_primitive_function(function: &Function) -> bool {
    function
        .args
        .iter()
        .all(|arg| arg.ty.is_primitive() && !needs_primitive_cast(&arg.ty))
        && !function.is_async
        && function
            .return_type
            .as_ref()
            .map(TypeIdent::is_primitive)
            .unwrap_or(true)
}

fn create_enum_definition(ty: &Enum, types: &TypeMap) -> String {
    let variants = ty
        .variants
        .iter()
        .map(|variant| {
            let variant_name = get_variant_name(variant, &ty.options);
            let variant_decl = match &variant.ty {
                Type::Unit => {
                    if let Some(tag) = &ty.options.tag_prop_name {
                        format!("| {{ {tag}: \"{variant_name}\" }}")
                    } else {
                        format!("| \"{variant_name}\"")
                    }
                }
                Type::Struct(struct_variant) => {
                    if ty.options.untagged {
                        format!(
                            "| {{ {} }}",
                            format_struct_fields(
                                &struct_variant.fields,
                                types,
                                variant.attrs.field_casing
                            )
                            .join(" ")
                        )
                    } else {
                        let field_lines = format_struct_fields(
                            &struct_variant.fields,
                            types,
                            variant.attrs.field_casing,
                        );
                        let formatted_fields = if field_lines.len() > struct_variant.fields.len() {
                            format!(
                                "\n{}",
                                join_lines(&field_lines, |line| format!("    {line}"))
                            )
                        } else {
                            format!(" {} ", field_lines.join(" ").trim_end_matches(';'))
                        };

                        match (&ty.options.tag_prop_name, &ty.options.content_prop_name) {
                            (Some(tag), Some(content)) => {
                                format!(
                                    "| {{ {tag}: \"{variant_name}\"; {content}: {{{formatted_fields}}} }}"
                                )
                            }
                            (Some(tag), None) => {
                                let space = if formatted_fields.contains('\n') {
                                    "\n    "
                                } else {
                                    " "
                                };
                                format!(
                                    "| {{{space}{tag}: \"{variant_name}\";{formatted_fields}}}"
                                )
                            }
                            (None, _) => {
                                format!("| {{ {variant_name}: {{{formatted_fields}}} }}")
                            }
                        }
                    }
                }
                Type::Tuple(items) if items.len() == 1 => {
                    let item = items.first().unwrap();
                    if ty.options.untagged {
                        format!("| {}", format_ident(item, types, ""))
                    } else {
                        match (&ty.options.tag_prop_name, &ty.options.content_prop_name) {
                            (Some(tag), Some(content)) => {
                                format!(
                                    "| {{ {}: \"{}\"; {}: {} }}",
                                    tag,
                                    variant_name,
                                    content,
                                    format_ident(item, types, "")
                                )
                            }
                            (Some(tag), None) => {
                                format!(
                                    "| {{ {}: \"{}\" }} & {}",
                                    tag,
                                    variant_name,
                                    format_ident(item, types, "")
                                )
                            }
                            (None, _) => {
                                format!(
                                    "| {{ {}: {} }}",
                                    variant_name,
                                    format_ident(item, types, "")
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

            join_lines(&lines, |line| format!("    {line}"))
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        "{}export type {} =\n{};",
        join_lines(&format_docs(&ty.doc_lines), String::to_owned),
        ty.ident.format(false),
        variants.trim_end()
    )
}

fn create_struct_definition(ty: &Struct, types: &TypeMap) -> String {
    let is_newtype = ty.fields.len() == 1 && ty.fields.iter().any(|field| field.name.is_none());
    if is_newtype {
        format!(
            "{}export type {} = {};",
            join_lines(&format_docs(&ty.doc_lines), String::to_owned),
            ty.ident,
            ty.fields
                .first()
                .map(|field| format_ident(&field.ty, types, ""))
                .unwrap()
        )
    } else {
        let (flattened_fields, fields): (Vec<_>, Vec<_>) =
            ty.fields.iter().partition(|field| field.attrs.flatten);

        format!(
            "{}export type {} = {{\n{}}}{};",
            join_lines(&format_docs(&ty.doc_lines), String::to_owned),
            ty.ident.format(false),
            join_lines(
                &format_struct_fields(
                    &fields.into_iter().cloned().collect::<Vec<_>>(),
                    types,
                    ty.options.field_casing
                ),
                |line| format!("    {line}")
            )
            .trim_start_matches('\n'),
            flattened_fields
                .iter()
                .map(|field| format!(" & {}", field.ty))
                .collect::<Vec<_>>()
                .join("")
        )
    }
}

fn format_docs(doc_lines: &[String]) -> Vec<String> {
    if doc_lines.is_empty() {
        Vec::new()
    } else {
        let mut lines = vec!["/**".to_owned()];
        lines.append(
            &mut doc_lines
                .iter()
                .map(|doc_line| format!(" *{doc_line}"))
                .collect(),
        );
        lines.push(" */".to_owned());
        lines
    }
}

fn format_struct_fields(fields: &[Field], types: &TypeMap, casing: Casing) -> Vec<String> {
    fields
        .iter()
        .flat_map(|field| {
            let has_skip_serializing_attribute = field.attrs.skip_serializing_if.is_some();
            let field_decl = match types.get(&field.ty) {
                Some(Type::Container(name, _)) => {
                    let is_option_type = name == "Option";
                    let (arg, _) = field
                        .ty
                        .generic_args
                        .first()
                        .expect("Identifier was expected to contain a generic argument");
                    format!(
                        "{}{}: {}{};",
                        get_field_name(field, casing),
                        if is_option_type && has_skip_serializing_attribute {
                            "?"
                        } else {
                            ""
                        },
                        format_ident(arg, types, ""),
                        if is_option_type && !has_skip_serializing_attribute {
                            " | null"
                        } else {
                            ""
                        },
                    )
                }
                _ => format!(
                    "{}{}: {};",
                    get_field_name(field, casing),
                    if has_skip_serializing_attribute {
                        "?"
                    } else {
                        ""
                    },
                    format_ident(&field.ty, types, ""),
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

fn format_raw_type(ty: &TypeIdent) -> &str {
    if let Some(primitive) = ty.as_primitive() {
        format_plain_primitive(primitive)
    } else {
        "Uint8Array"
    }
}

/// Formats a type so it's valid TypeScript.
fn format_ident(ident: &TypeIdent, types: &TypeMap, scope: &str) -> String {
    match types.get(ident) {
        Some(ty) => format_type_with_ident(ty, ident, types, scope),
        None => ident.to_string(), // Must be a generic.
    }
}

/// Formats a type so it's valid TypeScript.
fn format_type_with_ident(ty: &Type, ident: &TypeIdent, types: &TypeMap, scope: &str) -> String {
    match ty {
        Type::Alias(name, _) => format!("{scope}{name}"),
        Type::Array(primitive, _) => primitive.js_array_name().unwrap_or_else(|| {
            panic!(
                "Could not determine JS array type for primitive: {:?}",
                primitive
            )
        }),
        Type::Container(name, _) => {
            let (arg, _) = ident
                .generic_args
                .first()
                .expect("Identifier was expected to contain a generic argument");

            if name == "Option" {
                format!("{} | null", format_ident(arg, types, scope))
            } else {
                format_ident(arg, types, scope)
            }
        }
        Type::Custom(custom) => custom.ts_ty.clone(),
        Type::Enum(_) | Type::Struct(_) => {
            let args: Vec<_> = ident
                .generic_args
                .iter()
                .map(|(arg, _)| format_ident(arg, types, scope))
                .collect();
            if args.is_empty() {
                format!("{}{}", scope, ident.name)
            } else {
                format!("{}{}<{}>", scope, ident.name, args.join(", "))
            }
        }
        Type::List(_, _) => {
            let (arg, _) = ident
                .generic_args
                .first()
                .expect("Identifier was expected to contain a generic argument");
            format!("Array<{}>", format_ident(arg, types, scope))
        }
        Type::Map(_, _, _) => {
            let (arg1, _) = ident
                .generic_args
                .first()
                .expect("Identifier was expected to contain a generic argument");
            let (arg2, _) = ident
                .generic_args
                .get(1)
                .expect("Identifier was expected to contain two arguments");
            format!(
                "Record<{}, {}>",
                format_ident(arg1, types, scope),
                format_ident(arg2, types, scope)
            )
        }
        Type::Primitive(primitive) => format_encoded_primitive(*primitive).to_owned(),
        Type::String => "string".to_owned(),
        Type::Tuple(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|item| format_ident(item, types, scope))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Unit => "void".to_owned(),
    }
}

fn format_plain_primitive(primitive: Primitive) -> &'static str {
    match primitive {
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
    }
}

fn format_plain_primitive_or_ident(ident: &TypeIdent, types: &TypeMap) -> String {
    if let Some(primitive) = ident.as_primitive() {
        format_plain_primitive(primitive).to_owned()
    } else {
        format_ident(ident, types, "types.")
    }
}

// When encoded as part of a MessagePack type, 64-bit numbers are decoded into
// regular numbers rather than BigInt. This effectively limits them to a maximum
// value of `2^53 - 1`.
// See: https://github.com/msgpack/msgpack-javascript/issues/115
fn format_encoded_primitive(primitive: Primitive) -> &'static str {
    match primitive {
        Primitive::I64 => "number",
        Primitive::U64 => "number",
        primitive => format_plain_primitive(primitive),
    }
}

fn get_field_name(field: &Field, casing: Casing) -> String {
    if let Some(rename) = field.attrs.rename.as_ref() {
        rename.to_owned()
    } else {
        casing.format_string(get_variable_name(field.name.as_deref().unwrap_or_default()))
    }
}

fn get_variant_name(variant: &Variant, opts: &EnumOptions) -> String {
    if let Some(rename) = variant.attrs.rename.as_ref() {
        rename.to_owned()
    } else {
        opts.variant_casing
            .format_string(get_variable_name(&variant.name))
    }
}

fn get_variable_name(name: &str) -> &str {
    if let Some(stripped) = name.strip_prefix("r#") {
        stripped
    } else {
        name
    }
}

fn get_pointer_name(name: &str) -> String {
    format!("{}_ptr", get_variable_name(name))
}

fn import_primitive(ty: &TypeIdent, value: &str) -> String {
    match ty.name.as_str() {
        "bool" => format!("!!{value}"),
        "i8" => format!("interpretSign({value}, 128)"),
        "i16" => format!("interpretSign({value}, 32768)"),
        "i32" => format!("interpretSign({value}, 2147483648)"),
        "i64" => format!("interpretBigSign({value}, 9223372036854775808n)"),
        _ => value.to_owned(),
    }
}

fn needs_primitive_cast(ty: &TypeIdent) -> bool {
    matches!(ty.name.as_str(), "bool" | "i8" | "i16" | "i32" | "i64")
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
        format!("{lines}\n")
    }
}

fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(file_path, &contents).expect("Could not write bindings file");
}

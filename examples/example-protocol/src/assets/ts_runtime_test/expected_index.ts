// ============================================= //
// WebAssembly runtime for TypeScript            //
//                                               //
// This file is generated. PLEASE DO NOT MODIFY. //
// ============================================= //
// deno-lint-ignore-file no-explicit-any no-unused-vars

import { encode, decode } from "https://unpkg.com/@msgpack/msgpack@2.7.2/mod.ts";

import type {
    Body,
    DocExampleEnum,
    DocExampleStruct,
    ExplicitedlyImportedType,
    FlattenedStruct,
    FloatingPoint,
    FpAdjacentlyTagged,
    FpFlatten,
    FpInternallyTagged,
    FpPropertyRenaming,
    FpUntagged,
    FpVariantRenaming,
    GroupImportedType1,
    GroupImportedType2,
    HttpResult,
    Int64,
    Point,
    ReduxAction,
    Request,
    RequestError,
    Response,
    Result,
    SerdeAdjacentlyTagged,
    SerdeFlatten,
    SerdeInternallyTagged,
    SerdePropertyRenaming,
    SerdeUntagged,
    SerdeVariantRenaming,
    StateUpdate,
    StructWithGenerics,
} from "./types.ts";

type FatPtr = bigint;

export type Imports = {
    importFpAdjacentlyTagged: (arg: FpAdjacentlyTagged) => FpAdjacentlyTagged;
    importFpEnum: (arg: FpVariantRenaming) => FpVariantRenaming;
    importFpFlatten: (arg: FpFlatten) => FpFlatten;
    importFpInternallyTagged: (arg: FpInternallyTagged) => FpInternallyTagged;
    importFpStruct: (arg: FpPropertyRenaming) => FpPropertyRenaming;
    importFpUntagged: (arg: FpUntagged) => FpUntagged;
    importGenerics: (arg: StructWithGenerics<number>) => StructWithGenerics<number>;
    importMultiplePrimitives: (arg1: number, arg2: string) => bigint;
    importPrimitiveBool: (arg: boolean) => boolean;
    importPrimitiveF32: (arg: number) => number;
    importPrimitiveF64: (arg: number) => number;
    importPrimitiveI16: (arg: number) => number;
    importPrimitiveI32: (arg: number) => number;
    importPrimitiveI64: (arg: bigint) => bigint;
    importPrimitiveI8: (arg: number) => number;
    importPrimitiveU16: (arg: number) => number;
    importPrimitiveU32: (arg: number) => number;
    importPrimitiveU64: (arg: bigint) => bigint;
    importPrimitiveU8: (arg: number) => number;
    importSerdeAdjacentlyTagged: (arg: SerdeAdjacentlyTagged) => SerdeAdjacentlyTagged;
    importSerdeEnum: (arg: SerdeVariantRenaming) => SerdeVariantRenaming;
    importSerdeFlatten: (arg: SerdeFlatten) => SerdeFlatten;
    importSerdeInternallyTagged: (arg: SerdeInternallyTagged) => SerdeInternallyTagged;
    importSerdeStruct: (arg: SerdePropertyRenaming) => SerdePropertyRenaming;
    importSerdeUntagged: (arg: SerdeUntagged) => SerdeUntagged;
    importString: (arg: string) => string;
    importTimestamp: (arg: string) => string;
    importVoidFunction: () => void;
    log: (message: string) => void;
    makeHttpRequest: (request: Request) => Promise<HttpResult>;
};

export type Exports = {
    exportAsyncStruct?: (arg1: FpPropertyRenaming, arg2: bigint) => Promise<FpPropertyRenaming>;
    exportFpAdjacentlyTagged?: (arg: FpAdjacentlyTagged) => FpAdjacentlyTagged;
    exportFpEnum?: (arg: FpVariantRenaming) => FpVariantRenaming;
    exportFpFlatten?: (arg: FpFlatten) => FpFlatten;
    exportFpInternallyTagged?: (arg: FpInternallyTagged) => FpInternallyTagged;
    exportFpStruct?: (arg: FpPropertyRenaming) => FpPropertyRenaming;
    exportFpUntagged?: (arg: FpUntagged) => FpUntagged;
    exportGenerics?: (arg: StructWithGenerics<number>) => StructWithGenerics<number>;
    exportMultiplePrimitives?: (arg1: number, arg2: string) => bigint;
    exportPrimitiveBool?: (arg: boolean) => boolean;
    exportPrimitiveF32?: (arg: number) => number;
    exportPrimitiveF64?: (arg: number) => number;
    exportPrimitiveI16?: (arg: number) => number;
    exportPrimitiveI32?: (arg: number) => number;
    exportPrimitiveI64?: (arg: bigint) => bigint;
    exportPrimitiveI8?: (arg: number) => number;
    exportPrimitiveU16?: (arg: number) => number;
    exportPrimitiveU32?: (arg: number) => number;
    exportPrimitiveU64?: (arg: bigint) => bigint;
    exportPrimitiveU8?: (arg: number) => number;
    exportSerdeAdjacentlyTagged?: (arg: SerdeAdjacentlyTagged) => SerdeAdjacentlyTagged;
    exportSerdeEnum?: (arg: SerdeVariantRenaming) => SerdeVariantRenaming;
    exportSerdeFlatten?: (arg: SerdeFlatten) => SerdeFlatten;
    exportSerdeInternallyTagged?: (arg: SerdeInternallyTagged) => SerdeInternallyTagged;
    exportSerdeStruct?: (arg: SerdePropertyRenaming) => SerdePropertyRenaming;
    exportSerdeUntagged?: (arg: SerdeUntagged) => SerdeUntagged;
    exportString?: (arg: string) => string;
    exportTimestamp?: (arg: string) => string;
    exportVoidFunction?: () => void;
    fetchData?: (rType: string) => Promise<Result<string, string>>;
    init?: () => void;
    reducerBridge?: (action: ReduxAction) => StateUpdate;
    exportAsyncStructRaw?: (arg1: Uint8Array, arg2: bigint) => Promise<Uint8Array>;
    exportFpAdjacentlyTaggedRaw?: (arg: Uint8Array) => Uint8Array;
    exportFpEnumRaw?: (arg: Uint8Array) => Uint8Array;
    exportFpFlattenRaw?: (arg: Uint8Array) => Uint8Array;
    exportFpInternallyTaggedRaw?: (arg: Uint8Array) => Uint8Array;
    exportFpStructRaw?: (arg: Uint8Array) => Uint8Array;
    exportFpUntaggedRaw?: (arg: Uint8Array) => Uint8Array;
    exportGenericsRaw?: (arg: Uint8Array) => Uint8Array;
    exportMultiplePrimitivesRaw?: (arg1: number, arg2: Uint8Array) => bigint;
    exportPrimitiveBoolRaw?: (arg: boolean) => boolean;
    exportPrimitiveI16Raw?: (arg: number) => number;
    exportPrimitiveI32Raw?: (arg: number) => number;
    exportPrimitiveI64Raw?: (arg: bigint) => bigint;
    exportPrimitiveI8Raw?: (arg: number) => number;
    exportSerdeAdjacentlyTaggedRaw?: (arg: Uint8Array) => Uint8Array;
    exportSerdeEnumRaw?: (arg: Uint8Array) => Uint8Array;
    exportSerdeFlattenRaw?: (arg: Uint8Array) => Uint8Array;
    exportSerdeInternallyTaggedRaw?: (arg: Uint8Array) => Uint8Array;
    exportSerdeStructRaw?: (arg: Uint8Array) => Uint8Array;
    exportSerdeUntaggedRaw?: (arg: Uint8Array) => Uint8Array;
    exportStringRaw?: (arg: Uint8Array) => Uint8Array;
    exportTimestampRaw?: (arg: Uint8Array) => Uint8Array;
    fetchDataRaw?: (rType: Uint8Array) => Promise<Uint8Array>;
    reducerBridgeRaw?: (action: Uint8Array) => Uint8Array;
};

/**
 * Represents an unrecoverable error in the FP runtime.
 *
 * After this, your only recourse is to create a new runtime, probably with a different WASM plugin.
 */
export class FPRuntimeError extends Error {
    constructor(message: string) {
        super(message);
    }
}

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
): Promise<Exports> {
    const promises = new Map<FatPtr, (result: FatPtr) => void>();

    function createAsyncValue(): FatPtr {
        const len = 12; // std::mem::size_of::<AsyncValue>()
        const fatPtr = malloc(len);
        const [ptr] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        buffer.fill(0);
        return fatPtr;
    }

    function interpretSign(num: number, cap: number) {
        if (num < cap) {
            return num;
        } else {
            return num - (cap << 1);
        }
    }

    function interpretBigSign(num: bigint, cap: bigint) {
        if (num < cap) {
            return num;
        } else {
            return num - (cap << 1n);
        }
    }

    function parseObject<T>(fatPtr: FatPtr): T {
        const [ptr, len] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        const object = decode(buffer) as unknown as T;
        free(fatPtr);
        return object;
    }

    function promiseFromPtr(ptr: FatPtr): Promise<FatPtr> {
        return new Promise((resolve) => {
            promises.set(ptr, resolve as (result: FatPtr) => void);
        });
    }

    function resolvePromise(asyncValuePtr: FatPtr, resultPtr: FatPtr) {
        const resolve = promises.get(asyncValuePtr);
        if (!resolve) {
            throw new FPRuntimeError("Tried to resolve unknown promise");
        }

        resolve(resultPtr);
    }

    function serializeObject<T>(object: T): FatPtr {
        return exportToMemory(encode(object));
    }

    function exportToMemory(serialized: Uint8Array): FatPtr {
        const fatPtr = malloc(serialized.length);
        const [ptr, len] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        buffer.set(serialized);
        return fatPtr;
    }

    function importFromMemory(fatPtr: FatPtr): Uint8Array {
        const [ptr, len] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        const copy = new Uint8Array(len);
        copy.set(buffer);
        free(fatPtr);
        return copy;
    }

    const { instance } = await WebAssembly.instantiate(plugin, {
        fp: {
            __fp_gen_import_fp_adjacently_tagged: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<FpAdjacentlyTagged>(arg_ptr);
                return serializeObject(importFunctions.importFpAdjacentlyTagged(arg));
            },
            __fp_gen_import_fp_enum: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<FpVariantRenaming>(arg_ptr);
                return serializeObject(importFunctions.importFpEnum(arg));
            },
            __fp_gen_import_fp_flatten: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<FpFlatten>(arg_ptr);
                return serializeObject(importFunctions.importFpFlatten(arg));
            },
            __fp_gen_import_fp_internally_tagged: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<FpInternallyTagged>(arg_ptr);
                return serializeObject(importFunctions.importFpInternallyTagged(arg));
            },
            __fp_gen_import_fp_struct: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<FpPropertyRenaming>(arg_ptr);
                return serializeObject(importFunctions.importFpStruct(arg));
            },
            __fp_gen_import_fp_untagged: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<FpUntagged>(arg_ptr);
                return serializeObject(importFunctions.importFpUntagged(arg));
            },
            __fp_gen_import_generics: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<StructWithGenerics<number>>(arg_ptr);
                return serializeObject(importFunctions.importGenerics(arg));
            },
            __fp_gen_import_multiple_primitives: (arg1: number, arg2_ptr: FatPtr): bigint => {
                const arg2 = parseObject<string>(arg2_ptr);
                return interpretBigSign(importFunctions.importMultiplePrimitives(arg1, arg2), 9223372036854775808n);
            },
            __fp_gen_import_primitive_bool: (arg: boolean): boolean => {
                return !!importFunctions.importPrimitiveBool(arg);
            },
            __fp_gen_import_primitive_f32: (arg: number): number => {
                return importFunctions.importPrimitiveF32(arg);
            },
            __fp_gen_import_primitive_f64: (arg: number): number => {
                return importFunctions.importPrimitiveF64(arg);
            },
            __fp_gen_import_primitive_i16: (arg: number): number => {
                return interpretSign(importFunctions.importPrimitiveI16(arg), 32768);
            },
            __fp_gen_import_primitive_i32: (arg: number): number => {
                return interpretSign(importFunctions.importPrimitiveI32(arg), 2147483648);
            },
            __fp_gen_import_primitive_i64: (arg: bigint): bigint => {
                return interpretBigSign(importFunctions.importPrimitiveI64(arg), 9223372036854775808n);
            },
            __fp_gen_import_primitive_i8: (arg: number): number => {
                return interpretSign(importFunctions.importPrimitiveI8(arg), 128);
            },
            __fp_gen_import_primitive_u16: (arg: number): number => {
                return importFunctions.importPrimitiveU16(arg);
            },
            __fp_gen_import_primitive_u32: (arg: number): number => {
                return importFunctions.importPrimitiveU32(arg);
            },
            __fp_gen_import_primitive_u64: (arg: bigint): bigint => {
                return importFunctions.importPrimitiveU64(arg);
            },
            __fp_gen_import_primitive_u8: (arg: number): number => {
                return importFunctions.importPrimitiveU8(arg);
            },
            __fp_gen_import_serde_adjacently_tagged: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<SerdeAdjacentlyTagged>(arg_ptr);
                return serializeObject(importFunctions.importSerdeAdjacentlyTagged(arg));
            },
            __fp_gen_import_serde_enum: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<SerdeVariantRenaming>(arg_ptr);
                return serializeObject(importFunctions.importSerdeEnum(arg));
            },
            __fp_gen_import_serde_flatten: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<SerdeFlatten>(arg_ptr);
                return serializeObject(importFunctions.importSerdeFlatten(arg));
            },
            __fp_gen_import_serde_internally_tagged: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<SerdeInternallyTagged>(arg_ptr);
                return serializeObject(importFunctions.importSerdeInternallyTagged(arg));
            },
            __fp_gen_import_serde_struct: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<SerdePropertyRenaming>(arg_ptr);
                return serializeObject(importFunctions.importSerdeStruct(arg));
            },
            __fp_gen_import_serde_untagged: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<SerdeUntagged>(arg_ptr);
                return serializeObject(importFunctions.importSerdeUntagged(arg));
            },
            __fp_gen_import_string: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<string>(arg_ptr);
                return serializeObject(importFunctions.importString(arg));
            },
            __fp_gen_import_timestamp: (arg_ptr: FatPtr): FatPtr => {
                const arg = parseObject<string>(arg_ptr);
                return serializeObject(importFunctions.importTimestamp(arg));
            },
            __fp_gen_import_void_function: () => {
                importFunctions.importVoidFunction();
            },
            __fp_gen_log: (message_ptr: FatPtr) => {
                const message = parseObject<string>(message_ptr);
                importFunctions.log(message);
            },
            __fp_gen_make_http_request: (request_ptr: FatPtr): FatPtr => {
                const request = parseObject<Request>(request_ptr);
                const _async_result_ptr = createAsyncValue();
                importFunctions.makeHttpRequest(request)
                    .then((result) => {
                        resolveFuture(_async_result_ptr, serializeObject(result));
                    })
                    .catch((error) => {
                        console.error(
                            'Unrecoverable exception trying to call async host function "make_http_request"',
                            error
                        );
                    });
                return _async_result_ptr;
            },
            __fp_host_resolve_async_value: resolvePromise,
        },
    });

    const getExport = <T>(name: string): T => {
        const exp = instance.exports[name];
        if (!exp) {
            throw new FPRuntimeError(`Plugin did not export expected symbol: "${name}"`);
        }
        return exp as unknown as T;
    };

    const memory = getExport<WebAssembly.Memory>("memory");
    const malloc = getExport<(len: number) => FatPtr>("__fp_malloc");
    const free = getExport<(ptr: FatPtr) => void>("__fp_free");
    const resolveFuture = getExport<(asyncValuePtr: FatPtr, resultPtr: FatPtr) => void>("__fp_guest_resolve_async_value");

    return {
        exportAsyncStruct: (() => {
            const export_fn = instance.exports.__fp_gen_export_async_struct as any;
            if (!export_fn) return;

            return (arg1: FpPropertyRenaming, arg2: bigint) => {
                const arg1_ptr = serializeObject(arg1);
                return promiseFromPtr(export_fn(arg1_ptr, arg2)).then((ptr) => parseObject<FpPropertyRenaming>(ptr));
            };
        })(),
        exportFpAdjacentlyTagged: (() => {
            const export_fn = instance.exports.__fp_gen_export_fp_adjacently_tagged as any;
            if (!export_fn) return;

            return (arg: FpAdjacentlyTagged) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<FpAdjacentlyTagged>(export_fn(arg_ptr));
            };
        })(),
        exportFpEnum: (() => {
            const export_fn = instance.exports.__fp_gen_export_fp_enum as any;
            if (!export_fn) return;

            return (arg: FpVariantRenaming) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<FpVariantRenaming>(export_fn(arg_ptr));
            };
        })(),
        exportFpFlatten: (() => {
            const export_fn = instance.exports.__fp_gen_export_fp_flatten as any;
            if (!export_fn) return;

            return (arg: FpFlatten) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<FpFlatten>(export_fn(arg_ptr));
            };
        })(),
        exportFpInternallyTagged: (() => {
            const export_fn = instance.exports.__fp_gen_export_fp_internally_tagged as any;
            if (!export_fn) return;

            return (arg: FpInternallyTagged) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<FpInternallyTagged>(export_fn(arg_ptr));
            };
        })(),
        exportFpStruct: (() => {
            const export_fn = instance.exports.__fp_gen_export_fp_struct as any;
            if (!export_fn) return;

            return (arg: FpPropertyRenaming) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<FpPropertyRenaming>(export_fn(arg_ptr));
            };
        })(),
        exportFpUntagged: (() => {
            const export_fn = instance.exports.__fp_gen_export_fp_untagged as any;
            if (!export_fn) return;

            return (arg: FpUntagged) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<FpUntagged>(export_fn(arg_ptr));
            };
        })(),
        exportGenerics: (() => {
            const export_fn = instance.exports.__fp_gen_export_generics as any;
            if (!export_fn) return;

            return (arg: StructWithGenerics<number>) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<StructWithGenerics<number>>(export_fn(arg_ptr));
            };
        })(),
        exportMultiplePrimitives: (() => {
            const export_fn = instance.exports.__fp_gen_export_multiple_primitives as any;
            if (!export_fn) return;

            return (arg1: number, arg2: string) => {
                const arg2_ptr = serializeObject(arg2);
                return interpretBigSign(export_fn(arg1, arg2_ptr), 9223372036854775808n);
            };
        })(),
        exportPrimitiveBool: (() => {
            const export_fn = instance.exports.__fp_gen_export_primitive_bool as any;
            if (!export_fn) return;

            return (arg: boolean) => !!export_fn(arg);
        })(),
        exportPrimitiveF32: instance.exports.__fp_gen_export_primitive_f32 as any,
        exportPrimitiveF64: instance.exports.__fp_gen_export_primitive_f64 as any,
        exportPrimitiveI16: (() => {
            const export_fn = instance.exports.__fp_gen_export_primitive_i16 as any;
            if (!export_fn) return;

            return (arg: number) => interpretSign(export_fn(arg), 32768);
        })(),
        exportPrimitiveI32: (() => {
            const export_fn = instance.exports.__fp_gen_export_primitive_i32 as any;
            if (!export_fn) return;

            return (arg: number) => interpretSign(export_fn(arg), 2147483648);
        })(),
        exportPrimitiveI64: (() => {
            const export_fn = instance.exports.__fp_gen_export_primitive_i64 as any;
            if (!export_fn) return;

            return (arg: bigint) => interpretBigSign(export_fn(arg), 9223372036854775808n);
        })(),
        exportPrimitiveI8: (() => {
            const export_fn = instance.exports.__fp_gen_export_primitive_i8 as any;
            if (!export_fn) return;

            return (arg: number) => interpretSign(export_fn(arg), 128);
        })(),
        exportPrimitiveU16: instance.exports.__fp_gen_export_primitive_u16 as any,
        exportPrimitiveU32: instance.exports.__fp_gen_export_primitive_u32 as any,
        exportPrimitiveU64: instance.exports.__fp_gen_export_primitive_u64 as any,
        exportPrimitiveU8: instance.exports.__fp_gen_export_primitive_u8 as any,
        exportSerdeAdjacentlyTagged: (() => {
            const export_fn = instance.exports.__fp_gen_export_serde_adjacently_tagged as any;
            if (!export_fn) return;

            return (arg: SerdeAdjacentlyTagged) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<SerdeAdjacentlyTagged>(export_fn(arg_ptr));
            };
        })(),
        exportSerdeEnum: (() => {
            const export_fn = instance.exports.__fp_gen_export_serde_enum as any;
            if (!export_fn) return;

            return (arg: SerdeVariantRenaming) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<SerdeVariantRenaming>(export_fn(arg_ptr));
            };
        })(),
        exportSerdeFlatten: (() => {
            const export_fn = instance.exports.__fp_gen_export_serde_flatten as any;
            if (!export_fn) return;

            return (arg: SerdeFlatten) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<SerdeFlatten>(export_fn(arg_ptr));
            };
        })(),
        exportSerdeInternallyTagged: (() => {
            const export_fn = instance.exports.__fp_gen_export_serde_internally_tagged as any;
            if (!export_fn) return;

            return (arg: SerdeInternallyTagged) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<SerdeInternallyTagged>(export_fn(arg_ptr));
            };
        })(),
        exportSerdeStruct: (() => {
            const export_fn = instance.exports.__fp_gen_export_serde_struct as any;
            if (!export_fn) return;

            return (arg: SerdePropertyRenaming) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<SerdePropertyRenaming>(export_fn(arg_ptr));
            };
        })(),
        exportSerdeUntagged: (() => {
            const export_fn = instance.exports.__fp_gen_export_serde_untagged as any;
            if (!export_fn) return;

            return (arg: SerdeUntagged) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<SerdeUntagged>(export_fn(arg_ptr));
            };
        })(),
        exportString: (() => {
            const export_fn = instance.exports.__fp_gen_export_string as any;
            if (!export_fn) return;

            return (arg: string) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<string>(export_fn(arg_ptr));
            };
        })(),
        exportTimestamp: (() => {
            const export_fn = instance.exports.__fp_gen_export_timestamp as any;
            if (!export_fn) return;

            return (arg: string) => {
                const arg_ptr = serializeObject(arg);
                return parseObject<string>(export_fn(arg_ptr));
            };
        })(),
        exportVoidFunction: instance.exports.__fp_gen_export_void_function as any,
        fetchData: (() => {
            const export_fn = instance.exports.__fp_gen_fetch_data as any;
            if (!export_fn) return;

            return (rType: string) => {
                const type_ptr = serializeObject(rType);
                return promiseFromPtr(export_fn(type_ptr)).then((ptr) => parseObject<Result<string, string>>(ptr));
            };
        })(),
        init: instance.exports.__fp_gen_init as any,
        reducerBridge: (() => {
            const export_fn = instance.exports.__fp_gen_reducer_bridge as any;
            if (!export_fn) return;

            return (action: ReduxAction) => {
                const action_ptr = serializeObject(action);
                return parseObject<StateUpdate>(export_fn(action_ptr));
            };
        })(),
        exportAsyncStructRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_async_struct as any;
            if (!export_fn) return;

            return (arg1: Uint8Array, arg2: bigint) => {
                const arg1_ptr = exportToMemory(arg1);
                return promiseFromPtr(export_fn(arg1_ptr, arg2)).then(importFromMemory);
            };
        })(),
        exportFpAdjacentlyTaggedRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_fp_adjacently_tagged as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportFpEnumRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_fp_enum as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportFpFlattenRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_fp_flatten as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportFpInternallyTaggedRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_fp_internally_tagged as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportFpStructRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_fp_struct as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportFpUntaggedRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_fp_untagged as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportGenericsRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_generics as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportMultiplePrimitivesRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_multiple_primitives as any;
            if (!export_fn) return;

            return (arg1: number, arg2: Uint8Array) => {
                const arg2_ptr = exportToMemory(arg2);
                return interpretBigSign(export_fn(arg1, arg2_ptr), 9223372036854775808n);
            };
        })(),
        exportPrimitiveBoolRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_primitive_bool as any;
            if (!export_fn) return;

            return (arg: boolean) => !!export_fn(arg);
        })(),
        exportPrimitiveI16Raw: (() => {
            const export_fn = instance.exports.__fp_gen_export_primitive_i16 as any;
            if (!export_fn) return;

            return (arg: number) => interpretSign(export_fn(arg), 32768);
        })(),
        exportPrimitiveI32Raw: (() => {
            const export_fn = instance.exports.__fp_gen_export_primitive_i32 as any;
            if (!export_fn) return;

            return (arg: number) => interpretSign(export_fn(arg), 2147483648);
        })(),
        exportPrimitiveI64Raw: (() => {
            const export_fn = instance.exports.__fp_gen_export_primitive_i64 as any;
            if (!export_fn) return;

            return (arg: bigint) => interpretBigSign(export_fn(arg), 9223372036854775808n);
        })(),
        exportPrimitiveI8Raw: (() => {
            const export_fn = instance.exports.__fp_gen_export_primitive_i8 as any;
            if (!export_fn) return;

            return (arg: number) => interpretSign(export_fn(arg), 128);
        })(),
        exportSerdeAdjacentlyTaggedRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_serde_adjacently_tagged as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportSerdeEnumRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_serde_enum as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportSerdeFlattenRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_serde_flatten as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportSerdeInternallyTaggedRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_serde_internally_tagged as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportSerdeStructRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_serde_struct as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportSerdeUntaggedRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_serde_untagged as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportStringRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_string as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        exportTimestampRaw: (() => {
            const export_fn = instance.exports.__fp_gen_export_timestamp as any;
            if (!export_fn) return;

            return (arg: Uint8Array) => {
                const arg_ptr = exportToMemory(arg);
                return importFromMemory(export_fn(arg_ptr));
            };
        })(),
        fetchDataRaw: (() => {
            const export_fn = instance.exports.__fp_gen_fetch_data as any;
            if (!export_fn) return;

            return (rType: Uint8Array) => {
                const type_ptr = exportToMemory(rType);
                return promiseFromPtr(export_fn(type_ptr)).then(importFromMemory);
            };
        })(),
        reducerBridgeRaw: (() => {
            const export_fn = instance.exports.__fp_gen_reducer_bridge as any;
            if (!export_fn) return;

            return (action: Uint8Array) => {
                const action_ptr = exportToMemory(action);
                return importFromMemory(export_fn(action_ptr));
            };
        })(),
    };
}

function fromFatPtr(fatPtr: FatPtr): [ptr: number, len: number] {
    return [
        Number.parseInt((fatPtr >> 32n).toString()),
        Number.parseInt((fatPtr & 0xffff_ffffn).toString()),
    ];
}

function toFatPtr(ptr: number, len: number): FatPtr {
    return (BigInt(ptr) << 32n) | BigInt(len);
}

import { encode, decode } from "@msgpack/msgpack";

import type {
    Body,
    ComplexAlias,
    ComplexGuestToHost,
    ComplexHostToGuest,
    RequestError,
    RequestMethod,
    RequestOptions,
    Response,
    Result,
    Simple,
} from "./types";

type FatPtr = bigint;

export type Imports = {
    countWords: (string: string) => Result<number, string>;
    log: (message: string) => void;
    makeRequest: (opts: RequestOptions) => Promise<Result<Response, RequestError>>;
    myAsyncImportedFunction: () => Promise<ComplexHostToGuest>;
    myComplexImportedFunction: (a: ComplexAlias) => ComplexHostToGuest;
    myPlainImportedFunction: (a: number, b: number) => number;
};

export type Exports = {
    fetchData?: (url: string) => Promise<string>;
    myAsyncExportedFunction?: () => Promise<ComplexGuestToHost>;
    myComplexExportedFunction?: (a: ComplexHostToGuest) => ComplexAlias;
    myPlainExportedFunction?: (a: number, b: number) => number;
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
    const promises = new Map<FatPtr, (result: unknown) => void>();

    function createAsyncValue(): FatPtr {
        const len = 12; // std::mem::size_of::<AsyncValue>()
        const fatPtr = malloc(len);
        const [ptr] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        buffer.fill(0);
        return fatPtr;
    }

    function parseObject<T>(fatPtr: FatPtr): T {
        const [ptr, len] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        const object = decode<T>(buffer) as T;
        free(fatPtr);
        return object;
    }

    function promiseFromPtr<T>(ptr: FatPtr): Promise<T> {
        return new Promise<T>((resolve) => {
            promises.set(ptr, resolve as (result: unknown) => void);
        });
    }

    function resolvePromise(ptr: FatPtr) {
        const resolve = promises.get(ptr);
        if (resolve) {
            const [asyncPtr, asyncLen] = fromFatPtr(ptr);
            const buffer = new Uint32Array(memory.buffer, asyncPtr, asyncLen / 4);
            switch (buffer[0]) {
                case 0:
                    throw new FPRuntimeError("Tried to resolve promise that is not ready");
                case 1:
                    resolve(parseObject(toFatPtr(buffer[1]!, buffer[2]!)));
                    break;
                default:
                    throw new FPRuntimeError("Unexpected status: " + buffer[0]);
            }
        } else {
            throw new FPRuntimeError("Tried to resolve unknown promise");
        }
    }

    function serializeObject<T>(object: T): FatPtr {
        const serialized = encode(object);
        const fatPtr = malloc(serialized.length);
        const [ptr, len] = fromFatPtr(fatPtr);
        const buffer = new Uint8Array(memory.buffer, ptr, len);
        buffer.set(serialized);
        return fatPtr;
    }

    const { instance } = await WebAssembly.instantiate(plugin, {
        fp: {
            __fp_host_resolve_async_value: resolvePromise,
            __fp_gen_count_words: (string_ptr: FatPtr): FatPtr => {
                const string = parseObject<string>(string_ptr);
                return serializeObject(importFunctions.countWords(string));
            },
            __fp_gen_log: (message_ptr: FatPtr) => {
                const message = parseObject<string>(message_ptr);
                importFunctions.log(message);
            },
            __fp_gen_make_request: (opts_ptr: FatPtr): FatPtr => {
                const opts = parseObject<RequestOptions>(opts_ptr);
                const _async_result_ptr = createAsyncValue();
                importFunctions.makeRequest(opts)
                    .then((result) => {
                        resolveFuture(_async_result_ptr, serializeObject(result));
                    })
                    .catch((error) => {
                        console.error(
                            'Unrecoverable exception trying to call async host function "make_request"',
                            error
                        );
                    });
                return _async_result_ptr;
            },
            __fp_gen_my_async_imported_function: (): FatPtr => {
                const _async_result_ptr = createAsyncValue();
                importFunctions.myAsyncImportedFunction()
                    .then((result) => {
                        resolveFuture(_async_result_ptr, serializeObject(result));
                    })
                    .catch((error) => {
                        console.error(
                            'Unrecoverable exception trying to call async host function "my_async_imported_function"',
                            error
                        );
                    });
                return _async_result_ptr;
            },
            __fp_gen_my_complex_imported_function: (a_ptr: FatPtr): FatPtr => {
                const a = parseObject<ComplexAlias>(a_ptr);
                return serializeObject(importFunctions.myComplexImportedFunction(a));
            },
            __fp_gen_my_plain_imported_function: (a: number, b: number): number => {
                return importFunctions.myPlainImportedFunction(a, b);
            },
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
        fetchData: (() => {
            const export_fn = instance.exports.__fp_gen_fetch_data as any;
            if (!export_fn) return;
        
            return (url: string) => {
                const url_ptr = serializeObject(url);
                return promiseFromPtr<string>(export_fn(url_ptr));
            };
        })(),
        myAsyncExportedFunction: (() => {
            const export_fn = instance.exports.__fp_gen_my_async_exported_function as any;
            if (!export_fn) return;
        
            return () => promiseFromPtr<ComplexGuestToHost>(export_fn());
        })(),
        myComplexExportedFunction: (() => {
            const export_fn = instance.exports.__fp_gen_my_complex_exported_function as any;
            if (!export_fn) return;
        
            return (a: ComplexHostToGuest) => {
                const a_ptr = serializeObject(a);
                return parseObject<ComplexAlias>(export_fn(a_ptr));
            };
        })(),
        myPlainExportedFunction: instance.exports.__fp_gen_my_plain_exported_function as any,
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

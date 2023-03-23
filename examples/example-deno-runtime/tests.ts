import {
  assert,
  assertEquals,
  assertStrictEquals,
} from "https://deno.land/std@0.135.0/testing/asserts.ts";
import type {
  Exports,
  Imports,
} from "../example-protocol/bindings/ts-runtime/index.ts";
import type {
  ExplicitBoundPoint,
  FpAdjacentlyTagged,
  FpFlatten,
  FpInternallyTagged,
  FpPropertyRenaming,
  FpUntagged,
  FpVariantRenaming,
  HttpResult,
  Request,
  SerdeAdjacentlyTagged,
  SerdeFlatten,
  SerdeInternallyTagged,
  SerdePropertyRenaming,
  SerdeUntagged,
  SerdeVariantRenaming,
  StructWithGenerics,
  StructWithOptions,
} from "../example-protocol/bindings/ts-runtime/types.ts";
import { Result } from "../example-protocol/bindings/ts-runtime/types.ts";
import { loadPlugin } from "./loader.ts";

let voidFunctionCalled = false;
let globalState = 0;

const imports: Imports = {
  importExplicitBoundPoint: (arg: ExplicitBoundPoint<number>) => {
    assertEquals(arg.value, 123);
  },

  importFpAdjacentlyTagged: (arg: FpAdjacentlyTagged): FpAdjacentlyTagged => {
    assertEquals(arg, { type: "Bar", payload: "Hello, plugin!" });
    return { type: "Baz", payload: { a: -8, b: 64 } };
  },

  importFpEnum: (arg: FpVariantRenaming): FpVariantRenaming => {
    assertEquals(arg, "foo_bar");
    return {
      QUX_BAZ: {
        FOO_BAR: "foo_bar",
        qux_baz: 64.0,
      },
    };
  },

  importFpFlatten: (arg: FpFlatten): FpFlatten => {
    assertEquals(arg, { foo: "Hello, 🇳🇱!", bar: -64n });
    return { foo: "Hello, 🇩🇪!", bar: -64 };
  },

  importFpInternallyTagged: (arg: FpInternallyTagged): FpInternallyTagged => {
    assertEquals(arg, { type: "Foo" });
    return { type: "Baz", a: -8, b: 64 };
  },

  importFpStruct: (arg: FpPropertyRenaming): FpPropertyRenaming => {
    assertEquals(arg, { fooBar: "foo_bar", QUX_BAZ: 64.0, rawStruct: -32 });
    return { fooBar: "fooBar", QUX_BAZ: -64.0, rawStruct: 32 };
  },

  importFpUntagged: (arg: FpUntagged): FpUntagged => {
    assertEquals(arg, "Hello, plugin!");
    return { a: -8, b: 64 };
  },

  importGenerics: (
    arg: StructWithGenerics<number>
  ): StructWithGenerics<number> => {
    assertEquals(arg, {
      list: [0, 64],
      points: [{ value: 64 }],
      recursive: [{ value: { value: 64 } }],
      complex_nested: {
        one: [{ value: 1.0 }],
        two: [{ value: 2.0 }],
      },
      optional_timestamp: "1970-01-01T00:00:00Z",
    });
    return {
      list: [0, 64],
      points: [{ value: 64 }],
      recursive: [{ value: { value: 64 } }],
      complex_nested: {
        een: [{ value: 1.0 }],
        twee: [{ value: 2.0 }],
      },
      optional_timestamp: "1970-01-01T00:00:00Z",
    };
  },

  importGetBytes: (): Result<Uint8Array, string> => {
    return { Ok: new TextEncoder().encode("hello") };
  },

  importGetSerdeBytes: (): Result<ArrayBuffer, string> => {
    return { Ok: new TextEncoder().encode("hello") };
  },

  importMultiplePrimitives: (arg1: number, arg2: string): bigint => {
    assertEquals(arg1, -8);
    assertEquals(arg2, "Hello, 🇳🇱!");
    return -64n;
  },

  importPrimitiveBoolNegate: (arg: boolean): boolean => {
    return !arg;
  },

  importPrimitiveF32AddOne: (arg: number): number => {
    return arg + 1.0;
  },

  importPrimitiveF64AddOne: (arg: number): number => {
    return arg + 1.0;
  },

  importPrimitiveF32AddOneWasmer2: (arg: Float32Array): number => {
    return arg[0] + 1.0;
  },

  importPrimitiveF64AddOneWasmer2: (arg: Float64Array): number => {
    return arg[0] + 1.0;
  },

  importPrimitiveI16AddOne: (arg: number): number => {
    return arg + 1;
  },

  importPrimitiveI32AddOne: (arg: number): number => {
    return arg + 1;
  },

  importPrimitiveI64AddOne: (arg: bigint): bigint => {
    return arg + 1n;
  },

  importPrimitiveI8AddOne: (arg: number): number => {
    return arg + 1;
  },

  importPrimitiveU16AddOne: (arg: number): number => {
    return arg + 1;
  },

  importPrimitiveU32AddOne: (arg: number): number => {
    return arg + 1;
  },

  importPrimitiveU64AddOne: (arg: bigint): bigint => {
    return arg + 1n;
  },

  importPrimitiveU8AddOne: (arg: number): number => {
    return arg + 1;
  },

  importArrayU8: (arg: Uint8Array): Uint8Array => {
    assertEquals(arg, [1, 2, 3]);
    return new Uint8Array([1, 2, 3]);
  },

  importArrayU16: (arg: Uint16Array): Uint16Array => {
    assertEquals(arg, [1, 2, 3]);
    return new Uint16Array([1, 2, 3]);
  },

  importArrayU32: (arg: Uint32Array): Uint32Array => {
    assertEquals(arg, [1, 2, 3]);
    return new Uint32Array([1, 2, 3]);
  },

  importArrayI8: (arg: Int8Array): Int8Array => {
    assertEquals(arg, [1, 2, 3]);
    return new Int8Array([1, 2, 3]);
  },

  importArrayI16: (arg: Int16Array): Int16Array => {
    assertEquals(arg, [1, 2, 3]);
    return new Int16Array([1, 2, 3]);
  },

  importArrayI32: (arg: Int32Array): Int32Array => {
    assertEquals(arg, [1, 2, 3]);
    return new Int32Array([1, 2, 3]);
  },

  importArrayF32: (arg: Float32Array): Float32Array => {
    assertEquals(arg, [1, 2, 3]);
    return new Float32Array([1, 2, 3]);
  },

  importArrayF64: (arg: Float64Array): Float64Array => {
    assertEquals(arg, [1, 2, 3]);
    return new Float64Array([1, 2, 3]);
  },

  importSerdeAdjacentlyTagged: (
    arg: SerdeAdjacentlyTagged
  ): SerdeAdjacentlyTagged => {
    assertEquals(arg, { type: "Bar", payload: "Hello, plugin!" });
    return { type: "Baz", payload: { a: -8, b: 64 } };
  },

  importSerdeEnum: (arg: SerdeVariantRenaming): SerdeVariantRenaming => {
    assertEquals(arg, "foo_bar");
    return {
      QUX_BAZ: {
        FooBar: "foo_bar",
        qux_baz: 64.0,
      },
    };
  },

  importSerdeFlatten: (arg: SerdeFlatten): SerdeFlatten => {
    assertEquals(arg, { foo: "Hello, 🇳🇱!", bar: -64n });
    return { foo: "Hello, 🇩🇪!", bar: -64 };
  },

  importSerdeInternallyTagged: (
    arg: SerdeInternallyTagged
  ): SerdeInternallyTagged => {
    assertEquals(arg, { type: "Foo" });
    return { type: "Baz", a: -8, b: 64 };
  },

  importSerdeStruct: (arg: SerdePropertyRenaming): SerdePropertyRenaming => {
    assertEquals(arg, { fooBar: "foo_bar", QUX_BAZ: 64.0, rawStruct: -32 });
    return { fooBar: "fooBar", QUX_BAZ: -64.0, rawStruct: 32 };
  },

  importSerdeUntagged: (arg: SerdeUntagged): SerdeUntagged => {
    assertEquals(arg, "Hello, plugin!");
    return { a: -8, b: 64 };
  },

  importString: (arg: string): string => {
    assertEquals(arg, "Hello, world!");
    return "Hello, plugin!";
  },

  importTimestamp: (arg: string): string => {
    assertEquals(arg, "2022-04-12T19:10:00Z");
    return "2022-04-13T12:37:00Z";
  },

  importVoidFunction: (): void => {
    voidFunctionCalled = true;
  },

  importVoidFunctionEmptyResult: (): Result<void, number> => {
    return {
      Ok: undefined,
    };
  },

  importVoidFunctionEmptyReturn: (): void => {},

  importPrimitiveBoolNegateAsync: (arg: boolean): Promise<boolean> => {
    return Promise.resolve(!arg);
  },

  importPrimitiveF32AddOneAsync: (arg: number): Promise<number> => {
    return Promise.resolve(arg + 1);
  },

  importPrimitiveF64AddOneAsync: (arg: number): Promise<number> => {
    return Promise.resolve(arg + 1);
  },

  importPrimitiveI8AddOneAsync: (arg: number): Promise<number> => {
    return Promise.resolve(arg + 1);
  },

  importPrimitiveI16AddOneAsync: (arg: number): Promise<number> => {
    return Promise.resolve(arg + 1);
  },

  importPrimitiveI32AddOneAsync: (arg: number): Promise<number> => {
    return Promise.resolve(arg + 1);
  },

  // Message pack doesn't support bigint yet, so 64-bit numbers are
  // represented as `number` in complex structs, including promises.
  // See https://github.com/msgpack/msgpack-javascript/pull/211
  importPrimitiveI64AddOneAsync: (arg: bigint): Promise<number> => {
    return Promise.resolve(Number(arg) + 1);
  },

  importPrimitiveU8AddOneAsync: (arg: number): Promise<number> => {
    return Promise.resolve(arg + 1);
  },

  importPrimitiveU16AddOneAsync: (arg: number): Promise<number> => {
    return Promise.resolve(arg + 1);
  },

  importPrimitiveU32AddOneAsync: (arg: number): Promise<number> => {
    return Promise.resolve(arg + 1);
  },

  importPrimitiveU64AddOneAsync: (arg: bigint): Promise<number> => {
    return Promise.resolve(Number(arg) + 1);
  },

  importIncrementGlobalState: (): Promise<void> => {
    globalState = globalState + 1;
    return Promise.resolve();
  },

  importResetGlobalState: (): Promise<void> => {
    globalState = 0;
    return Promise.resolve();
  },

  log: (message: string): void => {
    console.log("Plugin log: " + message);
  },

  makeHttpRequest: (request: Request): Promise<HttpResult> => {
    const encoder = new TextEncoder();

    assertEquals(request, {
      url: "https://fiberplane.dev/",
      method: "POST",
      headers: {
        "content-type": encoder.encode("application/json"),
      },
      body: encoder.encode(JSON.stringify({ country: "🇳🇱", type: "sign-up" })),
    });
    return Promise.resolve({
      Ok: {
        body: encoder.encode(JSON.stringify({ status: "confirmed" })),
        headers: {
          "content-type": encoder.encode("application/json"),
        },
        status_code: 200,
      },
    });
  },

  importStructWithOptions: (arg: StructWithOptions): StructWithOptions => {
    assertStrictEquals(arg.filledString, "Hello!");
    assertStrictEquals(arg.emptyString, undefined);
    assertStrictEquals(arg.filledOptionString, "Hello!");
    assertStrictEquals(arg.emptyOptionString, undefined);
    assertStrictEquals(arg.neverSkippedFilledOptionString, "Hello!");
    assertStrictEquals(arg.neverSkippedEmptyOptionString, null);
    return arg;
  },
};

let examplePlugin: Exports | null = null;
async function loadExamplePlugin() {
  if (!examplePlugin) {
    examplePlugin = await loadPlugin(
      "../example-plugin/target/wasm32-unknown-unknown/debug/example_plugin.wasm",
      imports
    );

    const { init } = examplePlugin;
    assert(init);
    init();
  }

  return examplePlugin;
}

Deno.test("primitives", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(plugin.exportPrimitiveBoolNegate?.(true), false);
  assertEquals(plugin.exportPrimitiveBoolNegate?.(false), true);

  assertEquals(plugin.exportPrimitiveU8AddThree?.(8), 8 + 3);
  assertEquals(plugin.exportPrimitiveU16AddThree?.(16), 16 + 3);
  assertEquals(plugin.exportPrimitiveU32AddThree?.(32), 32 + 3);
  assertEquals(plugin.exportPrimitiveU64AddThree?.(64n), 64n + 3n);
  assertEquals(plugin.exportPrimitiveI8AddThree?.(-8), -8 + 3);
  assertEquals(plugin.exportPrimitiveI16AddThree?.(-16), -16 + 3);
  assertEquals(plugin.exportPrimitiveI32AddThree?.(-32), -32 + 3);
  assertEquals(plugin.exportPrimitiveI64AddThree?.(-64n), -64n + 3n);

  assertEquals(plugin.exportMultiplePrimitives?.(-8, "Hello, 🇳🇱!"), -64n);

  // Precise float comparison is fine as long as the denominator is a power of two
  assertEquals(plugin.exportPrimitiveF32AddThree?.(3.5), 3.5 + 3.0);
  assertEquals(plugin.exportPrimitiveF64AddThree?.(2.5), 2.5 + 3.0);

  // We need to define the workaround methods for wasmer2, so we might as well test them
  assertEquals(plugin.exportPrimitiveF32AddThreeWasmer2?.(13.5), 13.5 + 3.0);
  assertEquals(plugin.exportPrimitiveF64AddThreeWasmer2?.(12.5), 12.5 + 3.0);
});

Deno.test("arrays", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(plugin.exportArrayU8?.(new Uint8Array([1, 2, 3])), [1, 2, 3]);
  assertEquals(plugin.exportArrayU16?.(new Uint16Array([1, 2, 3])), [1, 2, 3]);
  assertEquals(plugin.exportArrayU32?.(new Uint32Array([1, 2, 3])), [1, 2, 3]);
  assertEquals(plugin.exportArrayI8?.(new Int8Array([1, 2, 3])), [1, 2, 3]);
  assertEquals(plugin.exportArrayI16?.(new Int16Array([1, 2, 3])), [1, 2, 3]);
  assertEquals(plugin.exportArrayI32?.(new Int32Array([1, 2, 3])), [1, 2, 3]);
  assertEquals(plugin.exportArrayF32?.(new Float32Array([1, 2, 3])), [1, 2, 3]);
  assertEquals(plugin.exportArrayF64?.(new Float64Array([1, 2, 3])), [1, 2, 3]);
});

Deno.test("string", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(plugin.exportString?.("Hello, plugin!"), "Hello, world!");
});

Deno.test("timestamp", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(
    plugin.exportTimestamp?.("2022-04-12T19:10:00Z"),
    "2022-04-13T12:37:00Z"
  );
});

Deno.test("flattened structs", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(
    plugin.exportFpStruct?.({
      fooBar: "foo_bar",
      QUX_BAZ: 64.0,
      rawStruct: -32,
    }),
    {
      fooBar: "fooBar",
      QUX_BAZ: -64.0,
      rawStruct: 32,
    }
  );

  assertEquals(plugin.exportFpEnum?.("foo_bar"), {
    QUX_BAZ: {
      FOO_BAR: "foo_bar",
      qux_baz: 64.0,
    },
  });

  assertEquals(
    plugin.exportSerdeStruct?.({
      fooBar: "foo_bar",
      QUX_BAZ: 64.0,
      rawStruct: -32,
    }),
    {
      fooBar: "fooBar",
      QUX_BAZ: -64.0,
      rawStruct: 32,
    }
  );

  assertEquals(plugin.exportSerdeEnum?.("foo_bar"), {
    QUX_BAZ: {
      FooBar: "foo_bar",
      qux_baz: 64.0,
    },
  });
});

Deno.test("generics", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(
    plugin.exportGenerics?.({
      list: [0, 64],
      points: [{ value: 64 }],
      recursive: [{ value: { value: 64 } }],
      complex_nested: {
        one: [{ value: 1.0 }],
        two: [{ value: 2.0 }],
      },
      optional_timestamp: "1970-01-01T00:00:00Z",
    }),
    {
      list: [0, 64],
      points: [{ value: 64 }],
      recursive: [{ value: { value: 64 } }],
      complex_nested: {
        een: [{ value: 1.0 }],
        twee: [{ value: 2.0 }],
      },
      optional_timestamp: "1970-01-01T00:00:00Z",
    }
  );
});

Deno.test("property renaming", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(plugin.exportFpFlatten?.({ foo: "Hello, 🇳🇱!", bar: -64 }), {
    foo: "Hello, 🇩🇪!",
    bar: -64,
  });

  assertEquals(plugin.exportSerdeFlatten?.({ foo: "Hello, 🇳🇱!", bar: -64 }), {
    foo: "Hello, 🇩🇪!",
    bar: -64,
  });
});

Deno.test("tagged enums", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(
    plugin.exportFpAdjacentlyTagged?.({
      type: "Bar",
      payload: "Hello, plugin!",
    }),
    {
      type: "Baz",
      payload: { a: -8, b: 64 },
    }
  );

  assertEquals(plugin.exportFpInternallyTagged?.({ type: "Foo" }), {
    type: "Baz",
    a: -8,
    b: 64,
  });

  assertEquals(plugin.exportFpUntagged?.("Hello, plugin!"), { a: -8, b: 64 });

  assertEquals(
    plugin.exportSerdeAdjacentlyTagged?.({
      type: "Bar",
      payload: "Hello, plugin!",
    }),
    {
      type: "Baz",
      payload: { a: -8, b: 64 },
    }
  );

  assertEquals(plugin.exportSerdeInternallyTagged?.({ type: "Foo" }), {
    type: "Baz",
    a: -8,
    b: 64,
  });

  assertEquals(plugin.exportSerdeUntagged?.("Hello, plugin!"), {
    a: -8,
    b: 64,
  });
});

Deno.test("async primitives", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(await plugin.exportPrimitiveBoolNegateAsync?.(true), false);
  assertEquals(await plugin.exportPrimitiveBoolNegateAsync?.(false), true);

  // Precise float comparison is fine as long as the denominator is a power of two
  assertEquals(await plugin.exportPrimitiveF32AddThreeAsync?.(3.5), 3.5 + 3.0);
  assertEquals(await plugin.exportPrimitiveF64AddThreeAsync?.(2.5), 2.5 + 3.0);

  assertEquals(await plugin.exportPrimitiveU8AddThreeAsync?.(8), 8 + 3);
  assertEquals(await plugin.exportPrimitiveU16AddThreeAsync?.(16), 16 + 3);
  assertEquals(await plugin.exportPrimitiveU32AddThreeAsync?.(32), 32 + 3);
  assertEquals(await plugin.exportPrimitiveU64AddThreeAsync?.(64n), 64 + 3);
  assertEquals(await plugin.exportPrimitiveI8AddThreeAsync?.(-8), -8 + 3);
  assertEquals(await plugin.exportPrimitiveI16AddThreeAsync?.(-16), -16 + 3);
  assertEquals(await plugin.exportPrimitiveI32AddThreeAsync?.(-32), -32 + 3);
  assertEquals(await plugin.exportPrimitiveI64AddThreeAsync?.(-64n), -64 + 3);

  await plugin.exportResetGlobalState?.();
  await plugin.exportIncrementGlobalState?.();
  assertEquals(globalState, 1);

  await plugin.exportResetGlobalState?.();
  await plugin.exportIncrementGlobalState?.();
  await plugin.exportIncrementGlobalState?.();
  assertEquals(globalState, 2);
});

Deno.test("async struct", async () => {
  const { exportAsyncStruct } = await loadExamplePlugin();
  assert(exportAsyncStruct);

  assertEquals(
    await exportAsyncStruct(
      {
        fooBar: "foo_bar",
        QUX_BAZ: 64.0,
        rawStruct: -32,
      },
      64n
    ),
    {
      fooBar: "fooBar",
      QUX_BAZ: -64.0,
      rawStruct: 32,
    }
  );
});

Deno.test("fetch async data", async () => {
  const { fetchData } = await loadExamplePlugin();
  assert(fetchData);

  const data = await fetchData("sign-up");
  assertEquals(data, {
    Ok: JSON.stringify({ status: "confirmed" }),
  });
});

Deno.test("bytes", async () => {
  const { exportGetBytes, exportGetSerdeBytes } = await loadExamplePlugin();
  assert(exportGetBytes);
  assert(exportGetSerdeBytes);

  const encoder = new TextEncoder();
  assertEquals(unwrap(exportGetBytes()), encoder.encode("hello, world"));
  assertEquals(unwrap(exportGetSerdeBytes()), encoder.encode("hello, world"));
});

Deno.test("options", async () => {
  const plugin = await loadExamplePlugin();

  const value = {
    filledString: "Hello!",
    filledOptionString: "Hello!",
    emptyString: "",
    emptyOptionString: undefined,
    neverSkippedFilledOptionString: "Hello!",
    neverSkippedEmptyOptionString: null,
  };
  assertEquals(plugin.exportStructWithOptions?.(value), {
    filledString: "Hello!",
    filledOptionString: "Hello!",
    neverSkippedFilledOptionString: "Hello!",
    neverSkippedEmptyOptionString: null,
  });
});

function isOk<T, E>(result: Result<T, E>): result is { Ok: T } {
  return "Ok" in result;
}

function unwrap<T, E>(result: Result<T, E>): T {
  if (!isOk(result)) {
    throw result.Err;
  }

  return result.Ok;
}

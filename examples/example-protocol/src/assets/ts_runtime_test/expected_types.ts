// ============================================= //
// Types for WebAssembly runtime                 //
//                                               //
// This file is generated. PLEASE DO NOT MODIFY. //
// ============================================= //

export type Body = ArrayBuffer;

/**
 * # This is an enum with doc comments.
 */
export type DocExampleEnum =
    /**
     * Multi-line doc comment with complex characters
     * & " , \ ! '
     */
    | { Variant1: string }
    /**
     * Raw identifiers are supported too.
     */
    | { Variant2: {

        /**
         * Variant property.
         */
        inner: number;
    } };

/**
 * # This is a struct with doc comments.
 */
export type DocExampleStruct = {
    /**
     * Multi-line doc comment with complex characters
     * & " , \ ! '
     */
    multi_line: string;

    /**
     * Raw identifiers are supported too.
     */
    type: string;
};

/**
 * This struct is also not referenced by any function or data structure, but
 * it will show up because there is an explicit `use` statement for it in the
 * `fp_import!` macro.
 */
export type ExplicitedlyImportedType = {
    you_will_see_this: boolean;
};

export type FlattenedStruct = {
    foo: string;
    bar: bigint;
};

export type FloatingPoint = Point<number>;

export type FpAdjacentlyTagged =
    | { type: "Foo" }
    | { type: "Bar"; payload: string }
    | { type: "Baz"; payload: { a: number;b: bigint } };

export type FpFlatten = {
} & FlattenedStruct;

export type FpInternallyTagged =
    | { type: "Foo" }
    | { type: "Baz"; a: number;b: bigint };

export type FpPropertyRenaming = {
    fooBar: string;
    QUX_BAZ: number;
    rawStruct: number;
};

export type FpUntagged =
    | string
    | { a: number; b: bigint; };

export type FpVariantRenaming =
    | "foo_bar"
    | { QUX_BAZ: {

        /**
         * Will be renamed to "FOO_BAR" because of the `rename_all` on the
         * variant.
         */
        FOO_BAR: string;
        qux_baz: number;
    } };

export type GroupImportedType1 = {
    you_will_see_this: boolean;
};

export type GroupImportedType2 = {
    you_will_see_this: boolean;
};

export type HttpResult = Result<Response, RequestError>;

export type Method = 
    | "GET"
    | "POST"
    | "PUT"
    | "DELETE"
    | "HEAD"
    | "OPTIONS"
    | "CONNECT"
    | "PATCH"
    | "TRACE";

/**
 * A point of an arbitrary type.
 */
export type Point<T> = {
    value: T;
};

/**
 * Example for representing Redux actions.
 */
export type ReduxAction =
    | { type: "clear_title" }
    | { type: "update_title"; payload: { title: string } };

/**
 * Represents an HTTP request to be sent.
 */
export type Request = {
    /**
     * The URI to submit the request to.
     */
    url: string;

    /**
     * HTTP method to use for the request.
     */
    method: Method;

    /**
     * HTTP headers to submit with the request.
     *
     * Note: We currently do not support the `Headers` type from the `http`
     *       crate. See: https://github.com/fiberplane/fp-bindgen/issues/102
     */
    headers: Record<string, string>;

    /**
     * The body to submit with the request.
     */
    body?: Body;
};

/**
 * Represents an error that occurred while attempting to submit the request.
 */
export type RequestError =
    /**
     * Used when we know we don't have an active network connection.
     */
    | { type: "offline" }
    | { type: "no_route" }
    | { type: "connection_refused" }
    | { type: "timeout" }
    | {
        type: "server_error";

        /**
         * HTTP status code.
         */
        status_code: number;

        /**
         * Response body.
         */
        response: Body;
    }
    /**
     * Misc.
     */
    | { type: "other/misc"; reason: string };

/**
 * Represents an HTTP response we received.
 *
 * Please note we currently do not support streaming responses.
 */
export type Response = {
    /**
     * The response body. May be empty.
     */
    body: Body;

    /**
     * HTTP headers that were part of the response.
     *
     * Note: We currently do not support the `Headers` type from the `http`
     *       crate. See: https://github.com/fiberplane/fp-bindgen/issues/102
     */
    headers: Record<string, string>;

    /**
     * HTTP status code.
     */
    status_code: number;
};

/**
 * A result that can be either successful (`Ok)` or represent an error (`Err`).
 */
export type Result<T, E> =
    /**
     * Represents a succesful result.
     */
    | { Ok: T }
    /**
     * Represents an error.
     */
    | { Err: E };

export type SerdeAdjacentlyTagged =
    | { type: "Foo" }
    | { type: "Bar"; payload: string }
    | { type: "Baz"; payload: { a: number;b: bigint } };

export type SerdeFlatten = {
} & FlattenedStruct;

export type SerdeInternallyTagged =
    | { type: "Foo" }
    | { type: "Baz"; a: number;b: bigint };

export type SerdePropertyRenaming = {
    fooBar: string;
    QUX_BAZ: number;
    rawStruct: number;
};

export type SerdeUntagged =
    | string
    | { a: number; b: bigint; };

export type SerdeVariantRenaming =
    | "foo_bar"
    | { QUX_BAZ: {

        /**
         * Will be renamed to "FooBar" because of the `rename_all` on the
         * variant.
         */
        FooBar: string;
        qux_baz: number;
    } };

/**
 * A state update to communicate to the Redux host.
 *
 * Fields are wrapped in `Option`. If any field is `None` it means it hasn't
 * changed.
 */
export type StateUpdate = {
    title?: string;
    revision?: number;
};

export type StructWithGenerics<T> = {
    list: Array<T>;
    points: Array<Point<T>>;
    recursive: Array<Point<Point<T>>>;
    complex_nested?: Record<string, Array<FloatingPoint>>;
    optional_timestamp?: string;
};

// ============================================= //
// Types for WebAssembly runtime                 //
//                                               //
// This file is generated. PLEASE DO NOT MODIFY. //
// ============================================= //

export type Body = ArrayBuffer;

export type ComplexAlias = ComplexGuestToHost;

export type ComplexGuestToHost = {
    simple: Simple;
    map: Record<string, Simple>;
    timestamp: string;
};

/**
 * Multi-line doc comment with complex characters
 * & " , \ ! '
 */
export type ComplexHostToGuest = {
    simple: Simple;
    list: Array<number>;
    points: Array<Point<number>>;
    recursive: Array<Point<Point<number>>>;
    complexNested?: Record<string, Array<Point<number>>>;
    timestamp: string;
    optional_timestamp?: string;

    /**
     * Raw identifiers are supported too.
     */
    type: string;
    id: string;
};

export type ExplicitedlyImportedType = {
    youWillSeeThis: boolean;
};

export type GroupImportedType1 = {
    youWillSeeThis: boolean;
};

export type GroupImportedType2 = {
    youWillSeeThis: boolean;
};

export type Point<T> = {
    value: T;
};

/**
 * Represents an error with the request.
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
        statusCode: number;

        /**
         * Response body.
         */
        response: Body;
    }
    /**
     * Misc.
     */
    | { type: "other/misc"; reason: string };

export type RequestMethod =
    | "DELETE"
    | "GET"
    | "OPTIONS"
    | "POST"
    | "PUT";

export type RequestOptions = {
    url: string;
    method: RequestMethod;
    headers: Record<string, string>;
    body?: ArrayBuffer;
};

/**
 * A response to a request.
 */
export type Response = {
    /**
     * Response headers, by name.
     */
    headers: Record<string, string>;

    /**
     * Response body.
     */
    body: Body;
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

export type Simple = {
    foo: number;
    bar: string;
};

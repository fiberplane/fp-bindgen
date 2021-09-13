export type ComplexGuestToHost = {
    simple: Simple;
    map: Record<string, Simple>;
};

export type ComplexHostToGuest = {
    simple: Simple;
    list: Array<number>;
};

export type RequestError =
    | { type: "offline" }
    | { type: "no_route" }
    | { type: "connection_refused" }
    | { type: "timeout" }
    | { type: "server_error"; statusCode: number; response: ArrayBuffer }
    | { type: "other"; reason: string };

export type RequestMethod =
    | "DELETE"
    | "GET"
    | "OPTIONS"
    | "POST"
    | "UPDATE";

export type RequestOptions = {
    url: string;
    method: RequestMethod;
    headers: Record<string, string>;
    body?: ArrayBuffer;
};

export type Response = {
    headers: Record<string, string>;
    body: ArrayBuffer;
};

export type Result<T, E> =
    | { Ok: T }
    | { Err: E };

export type Simple = {
    foo: number;
    bar: string;
};

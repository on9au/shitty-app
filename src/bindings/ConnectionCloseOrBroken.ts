// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ConnectionInfo } from "./ConnectionInfo";

/**
 * Struct representing an unexpected connection closure.
 */
export type ConnectionCloseOrBroken = { 
/**
 * The connection info.
 */
connection_info: ConnectionInfo, 
/**
 * The error message.
 */
message: string | null, };

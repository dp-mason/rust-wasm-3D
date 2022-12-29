/* tslint:disable */
/* eslint-disable */
/**
* @returns {number}
*/
export function get_output_buffer_pointer(): number;
/**
* @param {number} seconds
* @returns {number}
*/
export function cube_anim(seconds: number): number;
/**
* @param {number} seconds
* @returns {number}
*/
export function ico_anim(seconds: number): number;
/**
* @param {number} time_since_start_sc
* @returns {number}
*/
export function line_test_animation(time_since_start_sc: number): number;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly get_output_buffer_pointer: () => number;
  readonly cube_anim: (a: number) => number;
  readonly ico_anim: (a: number) => number;
  readonly line_test_animation: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;

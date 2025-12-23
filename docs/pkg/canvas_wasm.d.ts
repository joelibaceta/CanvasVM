/* tslint:disable */
/* eslint-disable */

export class Canvas {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Lee la salida como string
   * ink_string(): string
   */
  ink_string(): string;
  /**
   * Escribe un carácter en la entrada (desde su código)
   * input_char(charCode: number): void
   */
  input_char(char_code: number): void;
  /**
   * Obtiene el tamaño del stack
   * stack_size(): number
   */
  stack_size(): number;
  /**
   * Compila la grilla actual a bytecode y retorna las instrucciones
   * compile_to_bytecode(): BytecodeInstruction[]
   */
  compile_to_bytecode(): any;
  /**
   * Lee la salida como array de números
   * ink(): Int32Array
   */
  ink(): Int32Array;
  /**
   * Crea una nueva instancia de Canvas
   */
  constructor();
  /**
   * Ejecuta múltiples pasos (hasta maxSteps o hasta que se detenga)
   * play(maxSteps: number): number - retorna pasos ejecutados
   */
  play(max_steps: number): number;
  /**
   * Escribe un número en la entrada
   * input(value: number): void
   */
  input(value: number): void;
  /**
   * Carga una grilla desde datos RGBA (4 bytes por píxel)
   * paint(rgbaData: Uint8Array, width: number, height: number)
   */
  paint(rgba_data: Uint8Array, width: number, height: number): void;
  /**
   * Resetea la VM a su estado inicial (recarga la imagen)
   * reset(): void
   */
  reset(): void;
  /**
   * Ejecuta un solo paso de la VM
   * stroke(): void
   */
  stroke(): void;
  /**
   * Obtiene el estado actual de la VM
   * snapshot(): VmSnapshot
   */
  snapshot(): any;
  /**
   * Obtiene el número de pasos ejecutados
   * get_steps(): number
   */
  get_steps(): number;
  /**
   * Verifica si la VM está detenida
   * is_halted(): boolean
   */
  is_halted(): boolean;
}

/**
 * Función de inicialización para configurar panic hook
 */
export function init(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_canvas_free: (a: number, b: number) => void;
  readonly canvas_compile_to_bytecode: (a: number) => [number, number, number];
  readonly canvas_get_steps: (a: number) => [number, number, number];
  readonly canvas_ink: (a: number) => [number, number, number, number];
  readonly canvas_ink_string: (a: number) => [number, number, number, number];
  readonly canvas_input: (a: number, b: number) => [number, number];
  readonly canvas_input_char: (a: number, b: number) => [number, number];
  readonly canvas_is_halted: (a: number) => [number, number, number];
  readonly canvas_new: () => number;
  readonly canvas_paint: (a: number, b: number, c: number, d: number, e: number) => [number, number];
  readonly canvas_play: (a: number, b: number) => [number, number, number];
  readonly canvas_reset: (a: number) => [number, number];
  readonly canvas_snapshot: (a: number) => [number, number, number];
  readonly canvas_stack_size: (a: number) => [number, number, number];
  readonly canvas_stroke: (a: number) => [number, number];
  readonly init: () => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

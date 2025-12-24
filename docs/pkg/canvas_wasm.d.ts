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
   * Check if the next instruction requires input (InNumber or InChar)
   * needs_input(): string | null  (returns "number", "char", or null)
   */
  needs_input(): any;
  /**
   * Get the current watchdog limit
   * get_max_steps(): number (0 = disabled)
   */
  get_max_steps(): number;
  /**
   * Gets the current compilation mode
   * is_debug_mode(): boolean
   */
  is_debug_mode(): boolean;
  /**
   * Set the watchdog limit (maximum steps before timeout)
   * set_max_steps(maxSteps: number): void
   */
  set_max_steps(max_steps: number): void;
  /**
   * Sets the compilation mode
   * set_debug_mode(debug: boolean): void
   */
  set_debug_mode(debug: boolean): void;
  /**
   * Get the next instruction opcode (for detecting InNumber/InChar)
   * Returns null if no next instruction, otherwise returns opcode like "InNumber", "InChar", "Push", etc.
   * get_next_opcode(): string | null
   */
  get_next_opcode(): any;
  /**
   * Detecta automáticamente el tamaño del codel de una imagen
   * detect_codel_size(rgbaData: Uint8Array, width: number, height: number): number
   */
  static detect_codel_size(rgba_data: Uint8Array, width: number, height: number): number;
  /**
   * Compila la grilla actual a bytecode y retorna las instrucciones
   * compile_to_bytecode(): BytecodeInstruction[]
   */
  compile_to_bytecode(): any;
  /**
   * Obtiene la metadata del programa compilado
   * get_program_metadata(): JsProgramMetadata
   */
  get_program_metadata(): any;
  /**
   * Carga una grilla con un tamaño de codel específico
   * Si codel_size es 0, se auto-detecta
   * paint_with_codel_size(rgbaData: Uint8Array, width: number, height: number, codelSize: number)
   */
  paint_with_codel_size(rgba_data: Uint8Array, width: number, height: number, codel_size: number): void;
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
   * Usa auto-detección de codel size
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
   * Check if input is available in the buffer
   * has_input(): boolean
   */
  has_input(): boolean;
  /**
   * Verifica si la VM está detenida
   * is_halted(): boolean
   */
  is_halted(): boolean;
}

export class PietDebugger {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get current instruction pointer
   * current_ip(): number
   */
  current_ip(): number;
  /**
   * Provide input character
   * input_char(charCode: number): void
   */
  input_char(char_code: number): void;
  /**
   * Get all breakpoints
   * breakpoints(): number[]
   */
  breakpoints(): Uint32Array;
  /**
   * Clear all inputs
   * clear_input(): void
   */
  clear_input(): void;
  /**
   * Rewind input buffer to start (re-read inputs from beginning)
   * rewind_input(): void
   */
  rewind_input(): void;
  /**
   * Get the current watchdog limit
   * get_max_steps(): number | null
   */
  get_max_steps(): any;
  /**
   * Get output as string
   * output_string(): string
   */
  output_string(): string;
  /**
   * Provide input value and resume execution (for interactive mode)
   * provide_input(value: number): void
   */
  provide_input(value: number): void;
  /**
   * Set the watchdog limit (maximum steps before timeout)
   * set_max_steps(maxSteps: number): void
   */
  set_max_steps(max_steps: number): void;
  /**
   * Add a breakpoint at instruction index
   * add_breakpoint(index: number): void
   */
  add_breakpoint(index: number): void;
  /**
   * Enable the watchdog with the default limit
   * enable_watchdog(): void
   */
  enable_watchdog(): void;
  /**
   * Load text as character inputs (each character becomes an input for in_char operations)
   * load_input_text(text: string): void
   */
  load_input_text(text: string): void;
  /**
   * Get remaining input count
   * remaining_input(): number
   */
  remaining_input(): number;
  /**
   * Disable the watchdog (allow infinite execution - use with caution!)
   * disable_watchdog(): void
   */
  disable_watchdog(): void;
  /**
   * Check if at a breakpoint
   * is_at_breakpoint(): boolean
   */
  is_at_breakpoint(): boolean;
  /**
   * Clear all breakpoints
   * clear_breakpoints(): void
   */
  clear_breakpoints(): void;
  /**
   * Get what type of input is being waited for
   * get_input_request(): string | null  ("number" or "char")
   */
  get_input_request(): any;
  /**
   * Get total instruction count
   * instruction_count(): number
   */
  instruction_count(): number;
  /**
   * Remove a breakpoint
   * remove_breakpoint(index: number): void
   */
  remove_breakpoint(index: number): void;
  /**
   * Load numbers from string (whitespace-separated, for in_number operations)
   * load_input_numbers(text: string): void
   */
  load_input_numbers(text: string): void;
  /**
   * Provide character input and resume execution (for interactive mode)
   * provide_input_char(charCode: number): void
   */
  provide_input_char(char_code: number): void;
  /**
   * Check if watchdog is enabled
   * is_watchdog_enabled(): boolean
   */
  is_watchdog_enabled(): boolean;
  /**
   * Check if waiting for input
   * is_waiting_for_input(): boolean
   */
  is_waiting_for_input(): boolean;
  /**
   * Continue execution until next breakpoint
   * continue_to_breakpoint(): number | null
   */
  continue_to_breakpoint(): any;
  /**
   * Load a vector of numbers as inputs
   * load_input_numbers_array(numbers: Int32Array): void
   */
  load_input_numbers_array(numbers: Int32Array): void;
  /**
   * Create a new debugger instance
   */
  constructor();
  /**
   * Run until halt, breakpoint, or watchdog timeout
   * run(): JsExecutionTrace
   */
  run(): any;
  /**
   * Load an image for debugging
   * load(rgbaData: Uint8Array, width: number, height: number, codelSize?: number)
   */
  load(rgba_data: Uint8Array, width: number, height: number, codel_size: number): void;
  /**
   * Execute a single step
   * step(): JsExecutionStep | null
   */
  step(): any;
  /**
   * Provide input number
   * input(value: number): void
   */
  input(value: number): void;
  /**
   * Reset debugger to initial state
   * reset(): void
   */
  reset(): void;
  /**
   * Get current debugger state
   * state(): JsDebuggerState
   */
  state(): any;
  /**
   * Get execution trace
   * trace(): JsExecutionTrace
   */
  trace(): any;
  /**
   * Get all bytecode instructions (for display)
   * bytecode(): BytecodeInstruction[]
   */
  bytecode(): any;
  /**
   * Get program metadata
   * metadata(): JsProgramMetadata
   */
  metadata(): any;
  /**
   * Check if there are inputs available
   * has_input(): boolean
   */
  has_input(): boolean;
  /**
   * Check if halted
   * is_halted(): boolean
   */
  is_halted(): boolean;
  /**
   * Run for a maximum number of steps
   * run_steps(maxSteps: number): number
   */
  run_steps(max_steps: number): number;
}

/**
 * Función de inicialización para configurar panic hook
 */
export function init(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_canvas_free: (a: number, b: number) => void;
  readonly __wbg_pietdebugger_free: (a: number, b: number) => void;
  readonly canvas_compile_to_bytecode: (a: number) => [number, number, number];
  readonly canvas_detect_codel_size: (a: number, b: number, c: number, d: number) => number;
  readonly canvas_get_max_steps: (a: number) => number;
  readonly canvas_get_next_opcode: (a: number) => [number, number, number];
  readonly canvas_get_program_metadata: (a: number) => [number, number, number];
  readonly canvas_get_steps: (a: number) => [number, number, number];
  readonly canvas_has_input: (a: number) => [number, number, number];
  readonly canvas_ink: (a: number) => [number, number, number, number];
  readonly canvas_ink_string: (a: number) => [number, number, number, number];
  readonly canvas_input: (a: number, b: number) => [number, number];
  readonly canvas_input_char: (a: number, b: number) => [number, number];
  readonly canvas_is_debug_mode: (a: number) => number;
  readonly canvas_is_halted: (a: number) => [number, number, number];
  readonly canvas_needs_input: (a: number) => [number, number, number];
  readonly canvas_new: () => number;
  readonly canvas_paint: (a: number, b: number, c: number, d: number, e: number) => [number, number];
  readonly canvas_paint_with_codel_size: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
  readonly canvas_play: (a: number, b: number) => [number, number, number];
  readonly canvas_reset: (a: number) => [number, number];
  readonly canvas_set_debug_mode: (a: number, b: number) => void;
  readonly canvas_set_max_steps: (a: number, b: number) => void;
  readonly canvas_snapshot: (a: number) => [number, number, number];
  readonly canvas_stack_size: (a: number) => [number, number, number];
  readonly canvas_stroke: (a: number) => [number, number];
  readonly pietdebugger_add_breakpoint: (a: number, b: number) => [number, number];
  readonly pietdebugger_breakpoints: (a: number) => [number, number, number, number];
  readonly pietdebugger_bytecode: (a: number) => [number, number, number];
  readonly pietdebugger_clear_breakpoints: (a: number) => [number, number];
  readonly pietdebugger_clear_input: (a: number) => [number, number];
  readonly pietdebugger_continue_to_breakpoint: (a: number) => [number, number, number];
  readonly pietdebugger_current_ip: (a: number) => [number, number, number];
  readonly pietdebugger_disable_watchdog: (a: number) => void;
  readonly pietdebugger_enable_watchdog: (a: number) => void;
  readonly pietdebugger_get_input_request: (a: number) => [number, number, number];
  readonly pietdebugger_get_max_steps: (a: number) => any;
  readonly pietdebugger_has_input: (a: number) => [number, number, number];
  readonly pietdebugger_input: (a: number, b: number) => [number, number];
  readonly pietdebugger_input_char: (a: number, b: number) => [number, number];
  readonly pietdebugger_instruction_count: (a: number) => [number, number, number];
  readonly pietdebugger_is_at_breakpoint: (a: number) => [number, number, number];
  readonly pietdebugger_is_halted: (a: number) => [number, number, number];
  readonly pietdebugger_is_waiting_for_input: (a: number) => [number, number, number];
  readonly pietdebugger_is_watchdog_enabled: (a: number) => number;
  readonly pietdebugger_load: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
  readonly pietdebugger_load_input_numbers: (a: number, b: number, c: number) => [number, number];
  readonly pietdebugger_load_input_numbers_array: (a: number, b: number, c: number) => [number, number];
  readonly pietdebugger_load_input_text: (a: number, b: number, c: number) => [number, number];
  readonly pietdebugger_metadata: (a: number) => [number, number, number];
  readonly pietdebugger_new: () => number;
  readonly pietdebugger_output_string: (a: number) => [number, number, number, number];
  readonly pietdebugger_provide_input: (a: number, b: number) => [number, number];
  readonly pietdebugger_provide_input_char: (a: number, b: number) => [number, number];
  readonly pietdebugger_remaining_input: (a: number) => [number, number, number];
  readonly pietdebugger_remove_breakpoint: (a: number, b: number) => [number, number];
  readonly pietdebugger_reset: (a: number) => [number, number];
  readonly pietdebugger_rewind_input: (a: number) => [number, number];
  readonly pietdebugger_run: (a: number) => [number, number, number];
  readonly pietdebugger_run_steps: (a: number, b: number) => [number, number, number];
  readonly pietdebugger_set_max_steps: (a: number, b: number) => void;
  readonly pietdebugger_state: (a: number) => [number, number, number];
  readonly pietdebugger_step: (a: number) => [number, number, number];
  readonly pietdebugger_trace: (a: number) => [number, number, number];
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

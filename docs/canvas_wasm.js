let wasm;

function getArrayI32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedInt32ArrayMemory0 = null;
function getInt32ArrayMemory0() {
    if (cachedInt32ArrayMemory0 === null || cachedInt32ArrayMemory0.byteLength === 0) {
        cachedInt32ArrayMemory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passArray32ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 4, 4) >>> 0;
    getUint32ArrayMemory0().set(arg, ptr / 4);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    }
}

let WASM_VECTOR_LEN = 0;

const CanvasFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_canvas_free(ptr >>> 0, 1));

const PietDebuggerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_pietdebugger_free(ptr >>> 0, 1));

export class Canvas {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CanvasFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_canvas_free(ptr, 0);
    }
    /**
     * Lee la salida como string
     * ink_string(): string
     * @returns {string}
     */
    ink_string() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.canvas_ink_string(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Escribe un carácter en la entrada (desde su código)
     * input_char(charCode: number): void
     * @param {number} char_code
     */
    input_char(char_code) {
        const ret = wasm.canvas_input_char(this.__wbg_ptr, char_code);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Obtiene el tamaño del stack
     * stack_size(): number
     * @returns {number}
     */
    stack_size() {
        const ret = wasm.canvas_stack_size(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Check if the next instruction requires input (InNumber or InChar)
     * needs_input(): string | null  (returns "number", "char", or null)
     * @returns {any}
     */
    needs_input() {
        const ret = wasm.canvas_needs_input(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Gets the current compilation mode
     * is_debug_mode(): boolean
     * @returns {boolean}
     */
    is_debug_mode() {
        const ret = wasm.canvas_is_debug_mode(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Sets the compilation mode
     * set_debug_mode(debug: boolean): void
     * @param {boolean} debug
     */
    set_debug_mode(debug) {
        wasm.canvas_set_debug_mode(this.__wbg_ptr, debug);
    }
    /**
     * Get the next instruction opcode (for detecting InNumber/InChar)
     * Returns null if no next instruction, otherwise returns opcode like "InNumber", "InChar", "Push", etc.
     * get_next_opcode(): string | null
     * @returns {any}
     */
    get_next_opcode() {
        const ret = wasm.canvas_get_next_opcode(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Detecta automáticamente el tamaño del codel de una imagen
     * detect_codel_size(rgbaData: Uint8Array, width: number, height: number): number
     * @param {Uint8Array} rgba_data
     * @param {number} width
     * @param {number} height
     * @returns {number}
     */
    static detect_codel_size(rgba_data, width, height) {
        const ptr0 = passArray8ToWasm0(rgba_data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.canvas_detect_codel_size(ptr0, len0, width, height);
        return ret >>> 0;
    }
    /**
     * Compila la grilla actual a bytecode y retorna las instrucciones
     * compile_to_bytecode(): BytecodeInstruction[]
     * @returns {any}
     */
    compile_to_bytecode() {
        const ret = wasm.canvas_compile_to_bytecode(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Obtiene la metadata del programa compilado
     * get_program_metadata(): JsProgramMetadata
     * @returns {any}
     */
    get_program_metadata() {
        const ret = wasm.canvas_get_program_metadata(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Carga una grilla con un tamaño de codel específico
     * Si codel_size es 0, se auto-detecta
     * paint_with_codel_size(rgbaData: Uint8Array, width: number, height: number, codelSize: number)
     * @param {Uint8Array} rgba_data
     * @param {number} width
     * @param {number} height
     * @param {number} codel_size
     */
    paint_with_codel_size(rgba_data, width, height, codel_size) {
        const ptr0 = passArray8ToWasm0(rgba_data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.canvas_paint_with_codel_size(this.__wbg_ptr, ptr0, len0, width, height, codel_size);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Lee la salida como array de números
     * ink(): Int32Array
     * @returns {Int32Array}
     */
    ink() {
        const ret = wasm.canvas_ink(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayI32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Crea una nueva instancia de Canvas
     */
    constructor() {
        const ret = wasm.canvas_new();
        this.__wbg_ptr = ret >>> 0;
        CanvasFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Ejecuta múltiples pasos (hasta maxSteps o hasta que se detenga)
     * play(maxSteps: number): number - retorna pasos ejecutados
     * @param {number} max_steps
     * @returns {number}
     */
    play(max_steps) {
        const ret = wasm.canvas_play(this.__wbg_ptr, max_steps);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Escribe un número en la entrada
     * input(value: number): void
     * @param {number} value
     */
    input(value) {
        const ret = wasm.canvas_input(this.__wbg_ptr, value);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Carga una grilla desde datos RGBA (4 bytes por píxel)
     * Usa auto-detección de codel size
     * paint(rgbaData: Uint8Array, width: number, height: number)
     * @param {Uint8Array} rgba_data
     * @param {number} width
     * @param {number} height
     */
    paint(rgba_data, width, height) {
        const ptr0 = passArray8ToWasm0(rgba_data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.canvas_paint(this.__wbg_ptr, ptr0, len0, width, height);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Resetea la VM a su estado inicial (recarga la imagen)
     * reset(): void
     */
    reset() {
        const ret = wasm.canvas_reset(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Ejecuta un solo paso de la VM
     * stroke(): void
     */
    stroke() {
        const ret = wasm.canvas_stroke(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Obtiene el estado actual de la VM
     * snapshot(): VmSnapshot
     * @returns {any}
     */
    snapshot() {
        const ret = wasm.canvas_snapshot(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Obtiene el número de pasos ejecutados
     * get_steps(): number
     * @returns {number}
     */
    get_steps() {
        const ret = wasm.canvas_get_steps(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Check if input is available in the buffer
     * has_input(): boolean
     * @returns {boolean}
     */
    has_input() {
        const ret = wasm.canvas_has_input(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Verifica si la VM está detenida
     * is_halted(): boolean
     * @returns {boolean}
     */
    is_halted() {
        const ret = wasm.canvas_is_halted(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
}
if (Symbol.dispose) Canvas.prototype[Symbol.dispose] = Canvas.prototype.free;

/**
 * PietDebugger - Debugger WASM para ejecución paso a paso
 *
 * Provee una experiencia de debugging completa:
 * - Step-by-step execution
 * - Breakpoints
 * - State inspection (stack, position, DP/CC)
 * - Execution trace
 */
export class PietDebugger {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PietDebuggerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_pietdebugger_free(ptr, 0);
    }
    /**
     * Get current instruction pointer
     * current_ip(): number
     * @returns {number}
     */
    current_ip() {
        const ret = wasm.pietdebugger_current_ip(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Provide input character
     * input_char(charCode: number): void
     * @param {number} char_code
     */
    input_char(char_code) {
        const ret = wasm.pietdebugger_input_char(this.__wbg_ptr, char_code);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Get all breakpoints
     * breakpoints(): number[]
     * @returns {Uint32Array}
     */
    breakpoints() {
        const ret = wasm.pietdebugger_breakpoints(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Clear all inputs
     * clear_input(): void
     */
    clear_input() {
        const ret = wasm.pietdebugger_clear_input(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Rewind input buffer to start (re-read inputs from beginning)
     * rewind_input(): void
     */
    rewind_input() {
        const ret = wasm.pietdebugger_rewind_input(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Get output as string
     * output_string(): string
     * @returns {string}
     */
    output_string() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.pietdebugger_output_string(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Provide input value and resume execution (for interactive mode)
     * provide_input(value: number): void
     * @param {number} value
     */
    provide_input(value) {
        const ret = wasm.pietdebugger_provide_input(this.__wbg_ptr, value);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Add a breakpoint at instruction index
     * add_breakpoint(index: number): void
     * @param {number} index
     */
    add_breakpoint(index) {
        const ret = wasm.pietdebugger_add_breakpoint(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Load text as character inputs (each character becomes an input for in_char operations)
     * load_input_text(text: string): void
     * @param {string} text
     */
    load_input_text(text) {
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.pietdebugger_load_input_text(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Get remaining input count
     * remaining_input(): number
     * @returns {number}
     */
    remaining_input() {
        const ret = wasm.pietdebugger_remaining_input(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Check if at a breakpoint
     * is_at_breakpoint(): boolean
     * @returns {boolean}
     */
    is_at_breakpoint() {
        const ret = wasm.pietdebugger_is_at_breakpoint(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Clear all breakpoints
     * clear_breakpoints(): void
     */
    clear_breakpoints() {
        const ret = wasm.pietdebugger_clear_breakpoints(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Get what type of input is being waited for
     * get_input_request(): string | null  ("number" or "char")
     * @returns {any}
     */
    get_input_request() {
        const ret = wasm.pietdebugger_get_input_request(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get total instruction count
     * instruction_count(): number
     * @returns {number}
     */
    instruction_count() {
        const ret = wasm.pietdebugger_instruction_count(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Remove a breakpoint
     * remove_breakpoint(index: number): void
     * @param {number} index
     */
    remove_breakpoint(index) {
        const ret = wasm.pietdebugger_remove_breakpoint(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Load numbers from string (whitespace-separated, for in_number operations)
     * load_input_numbers(text: string): void
     * @param {string} text
     */
    load_input_numbers(text) {
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.pietdebugger_load_input_numbers(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Provide character input and resume execution (for interactive mode)
     * provide_input_char(charCode: number): void
     * @param {number} char_code
     */
    provide_input_char(char_code) {
        const ret = wasm.pietdebugger_provide_input_char(this.__wbg_ptr, char_code);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Check if waiting for input
     * is_waiting_for_input(): boolean
     * @returns {boolean}
     */
    is_waiting_for_input() {
        const ret = wasm.pietdebugger_is_waiting_for_input(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Continue execution until next breakpoint
     * continue_to_breakpoint(): number | null
     * @returns {any}
     */
    continue_to_breakpoint() {
        const ret = wasm.pietdebugger_continue_to_breakpoint(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Load a vector of numbers as inputs
     * load_input_numbers_array(numbers: Int32Array): void
     * @param {Int32Array} numbers
     */
    load_input_numbers_array(numbers) {
        const ptr0 = passArray32ToWasm0(numbers, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.pietdebugger_load_input_numbers_array(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Create a new debugger instance
     */
    constructor() {
        const ret = wasm.pietdebugger_new();
        this.__wbg_ptr = ret >>> 0;
        PietDebuggerFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Run until halt or breakpoint
     * run(): JsExecutionTrace
     * @returns {any}
     */
    run() {
        const ret = wasm.pietdebugger_run(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Load an image for debugging
     * load(rgbaData: Uint8Array, width: number, height: number, codelSize?: number)
     * @param {Uint8Array} rgba_data
     * @param {number} width
     * @param {number} height
     * @param {number} codel_size
     */
    load(rgba_data, width, height, codel_size) {
        const ptr0 = passArray8ToWasm0(rgba_data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.pietdebugger_load(this.__wbg_ptr, ptr0, len0, width, height, codel_size);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Execute a single step
     * step(): JsExecutionStep | null
     * @returns {any}
     */
    step() {
        const ret = wasm.pietdebugger_step(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Provide input number
     * input(value: number): void
     * @param {number} value
     */
    input(value) {
        const ret = wasm.pietdebugger_input(this.__wbg_ptr, value);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Reset debugger to initial state
     * reset(): void
     */
    reset() {
        const ret = wasm.pietdebugger_reset(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Get current debugger state
     * state(): JsDebuggerState
     * @returns {any}
     */
    state() {
        const ret = wasm.pietdebugger_state(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get execution trace
     * trace(): JsExecutionTrace
     * @returns {any}
     */
    trace() {
        const ret = wasm.pietdebugger_trace(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get all bytecode instructions (for display)
     * bytecode(): BytecodeInstruction[]
     * @returns {any}
     */
    bytecode() {
        const ret = wasm.pietdebugger_bytecode(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get program metadata
     * metadata(): JsProgramMetadata
     * @returns {any}
     */
    metadata() {
        const ret = wasm.pietdebugger_metadata(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Check if there are inputs available
     * has_input(): boolean
     * @returns {boolean}
     */
    has_input() {
        const ret = wasm.pietdebugger_has_input(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Check if halted
     * is_halted(): boolean
     * @returns {boolean}
     */
    is_halted() {
        const ret = wasm.pietdebugger_is_halted(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Run for a maximum number of steps
     * run_steps(maxSteps: number): number
     * @param {number} max_steps
     * @returns {number}
     */
    run_steps(max_steps) {
        const ret = wasm.pietdebugger_run_steps(this.__wbg_ptr, max_steps);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
}
if (Symbol.dispose) PietDebugger.prototype[Symbol.dispose] = PietDebugger.prototype.free;

/**
 * Función de inicialización para configurar panic hook
 */
export function init() {
    wasm.init();
}

const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_Error_52673b7de5a0ca89 = function(arg0, arg1) {
        const ret = Error(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_String_8f0eb39a4a4c2f66 = function(arg0, arg1) {
        const ret = String(arg1);
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg___wbindgen_throw_dd24417ed36fc46e = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_log_9061abf01added57 = function(arg0, arg1) {
        console.log(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_new_1ba21ce319a06297 = function() {
        const ret = new Object();
        return ret;
    };
    imports.wbg.__wbg_new_25f239778d6112b9 = function() {
        const ret = new Array();
        return ret;
    };
    imports.wbg.__wbg_set_3f1d0b984ed272ed = function(arg0, arg1, arg2) {
        arg0[arg1] = arg2;
    };
    imports.wbg.__wbg_set_7df433eea03a5c14 = function(arg0, arg1, arg2) {
        arg0[arg1 >>> 0] = arg2;
    };
    imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
        // Cast intrinsic for `Ref(String) -> Externref`.
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_4625c577ab2ec9ee = function(arg0) {
        // Cast intrinsic for `U64 -> Externref`.
        const ret = BigInt.asUintN(64, arg0);
        return ret;
    };
    imports.wbg.__wbindgen_cast_d6cd19b81560fd6e = function(arg0) {
        // Cast intrinsic for `F64 -> Externref`.
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_externrefs;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
    };

    return imports;
}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedInt32ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('canvas_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;

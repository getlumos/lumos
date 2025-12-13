
let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

function decodeText(ptr, len) {
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let WASM_VECTOR_LEN = 0;

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

function getObject(idx) { return heap[idx]; }

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}
/**
 * Generate Rust and TypeScript code from a LUMOS schema
 *
 * # Arguments
 *
 * * `source` - The .lumos schema source code
 *
 * # Returns
 *
 * A `GeneratedCode` struct containing both Rust and TypeScript outputs,
 * or a JavaScript Error if parsing/generation fails
 *
 * # Example (JavaScript)
 *
 * ```js
 * import { generateCode } from 'lumos-wasm';
 *
 * const schema = `
 * #[solana]
 * #[account]
 * struct PlayerAccount {
 *     wallet: PublicKey,
 *     level: u16,
 * }
 * `;
 *
 * try {
 *     const result = generateCode(schema);
 *     console.log('Rust:', result.rust);
 *     console.log('TypeScript:', result.typescript);
 * } catch (error) {
 *     console.error('Generation failed:', error.message);
 * }
 * ```
 * @param {string} source
 * @returns {GeneratedCode}
 */
exports.generateCode = function(source) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.generateCode(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return GeneratedCode.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
};

/**
 * Validate a LUMOS schema without generating code
 *
 * Useful for providing real-time feedback in the editor without
 * the overhead of full code generation.
 *
 * # Arguments
 *
 * * `source` - The .lumos schema source code
 *
 * # Returns
 *
 * `Ok(())` if the schema is valid, or a JavaScript Error with the validation message
 * @param {string} source
 */
exports.validateSchema = function(source) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.validateSchema(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
};

const GeneratedCodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_generatedcode_free(ptr >>> 0, 1));
/**
 * Result of code generation containing both Rust and TypeScript outputs
 */
class GeneratedCode {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GeneratedCode.prototype);
        obj.__wbg_ptr = ptr;
        GeneratedCodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GeneratedCodeFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_generatedcode_free(ptr, 0);
    }
    /**
     * Generated Rust code
     * @returns {string}
     */
    get rust() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_generatedcode_rust(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Generated Rust code
     * @param {string} arg0
     */
    set rust(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_generatedcode_rust(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Generated TypeScript code
     * @returns {string}
     */
    get typescript() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_generatedcode_typescript(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Generated TypeScript code
     * @param {string} arg0
     */
    set typescript(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_generatedcode_typescript(this.__wbg_ptr, ptr0, len0);
    }
}
if (Symbol.dispose) GeneratedCode.prototype[Symbol.dispose] = GeneratedCode.prototype.free;

exports.GeneratedCode = GeneratedCode;

exports.__wbg___wbindgen_throw_b855445ff6a94295 = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

exports.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
};

const wasmPath = `${__dirname}/lumos_core_bg.wasm`;
const wasmBytes = require('fs').readFileSync(wasmPath);
const wasmModule = new WebAssembly.Module(wasmBytes);
const wasm = exports.__wasm = new WebAssembly.Instance(wasmModule, imports).exports;


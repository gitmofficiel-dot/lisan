/**
 * Lisan WebAssembly Runtime Driver
 * يوفر جسراً سريعاً بين جافاسكريبت وWebAssembly لتشغيل لغة لسان في المتصفح
 */

class LisanRuntime {
    constructor() {
        this.wasm = null;
        this.memory = null;
    }

    async init(wasmUrl = './lisan.wasm') {
        const response = await fetch(wasmUrl);
        const bytes = await response.arrayBuffer();
        const { instance } = await WebAssembly.instantiate(bytes, {
            env: {
                // دوال افتراضية إذا طلبتها الذاكرة
            }
        });
        this.wasm = instance.exports;
        this.memory = this.wasm.memory;
        return this;
    }

    eval(sourceCode) {
        if (!this.wasm) {
            throw new Error('لم يتم تهيئة محرك WebAssembly بعد');
        }

        const encoder = new TextEncoder();
        const encoded = encoder.encode(sourceCode + '\0');
        const len = encoded.length;

        // تخصيص ذاكرة في WebAssembly للنص
        const ptr = this.wasm.lisan_alloc(len);
        const memoryView = new Uint8Array(this.memory.buffer);
        memoryView.set(encoded, ptr);

        // تنفيذ الكود
        const resPtr = this.wasm.lisan_eval_wasm(ptr);

        // قراءة النتيجة من الذاكرة
        const outView = new Uint8Array(this.memory.buffer);
        let end = resPtr;
        while (outView[end] !== 0) {
            end++;
        }
        const decoder = new TextDecoder('utf-8');
        const output = decoder.decode(outView.subarray(resPtr, end));

        // تحرير الذاكرة
        this.wasm.lisan_dealloc(ptr, len);
        this.wasm.lisan_free_string(resPtr);

        return output;
    }

    arduino(sourceCode) {
        if (!this.wasm || !this.wasm.lisan_arduino_wasm) {
            throw new Error('مترجم الأردوينو في WebAssembly غير مهيأ');
        }
        const encoder = new TextEncoder();
        const encoded = encoder.encode(sourceCode + '\0');
        const len = encoded.length;

        const ptr = this.wasm.lisan_alloc(len);
        const memoryView = new Uint8Array(this.memory.buffer);
        memoryView.set(encoded, ptr);

        const resPtr = this.wasm.lisan_arduino_wasm(ptr);
        const outView = new Uint8Array(this.memory.buffer);
        let end = resPtr;
        while (outView[end] !== 0) {
            end++;
        }
        const decoder = new TextDecoder('utf-8');
        const output = decoder.decode(outView.subarray(resPtr, end));

        this.wasm.lisan_dealloc(ptr, len);
        this.wasm.lisan_free_string(resPtr);
        return output;
    }
}

window.LisanRuntime = LisanRuntime;

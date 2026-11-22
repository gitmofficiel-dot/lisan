/**
 * Lisan WebAssembly & Desktop Studio Runtime Driver (v2.0)
 * يوفر جسراً موحداً فائق السرعة بين المتصفح ومحرك WebAssembly المدمج
 * وخادم استوديو لسان المحلي لسطح المكتب (lisan --studio).
 */

class LisanRuntime {
    constructor() {
        this.wasm = null;
        this.memory = null;
        this.wasmReady = false;
        this.nativeApiAvailable = false;
        this.useNativeBackend = true;
        this.nativeInfo = null;
    }

    /**
     * تهيئة المحرك: فحص توفر خادم استوديو سطح المكتب المحلي (/api/info)
     * وتحميل وحدة WebAssembly للعمل دون اتصال بالإنترنت 100%.
     */
    async init(wasmUrl = './lisan.wasm') {
        await this._probeNativeStudio();

        try {
            const response = await fetch(wasmUrl);
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}`);
            }
            const bytes = await response.arrayBuffer();
            const { instance } = await WebAssembly.instantiate(bytes, {
                env: {
                    // دوال بيئية احتياطية إذا طلبتها الذاكرة الخطية
                }
            });
            this.wasm = instance.exports;
            this.memory = this.wasm.memory;
            this.wasmReady = true;
        } catch (wasmErr) {
            this.wasmReady = false;
            if (!this.nativeApiAvailable) {
                throw wasmErr;
            }
        }

        return this;
    }

    /**
     * فحص تلقائي سريع لخادم استوديو لسان المحلي (lisan --studio)
     */
    async _probeNativeStudio(timeoutMs = 1200) {
        if (typeof window !== 'undefined' && window.location && window.location.protocol === 'file:') {
            this.nativeApiAvailable = false;
            return false;
        }

        const controller = typeof AbortController !== 'undefined' ? new AbortController() : null;
        const timer = controller ? setTimeout(() => controller.abort(), timeoutMs) : null;

        try {
            const res = await fetch('/api/info', {
                method: 'GET',
                headers: { 'Accept': 'application/json' },
                signal: controller ? controller.signal : undefined
            });
            if (res.ok) {
                const contentType = res.headers.get('content-type') || '';
                if (contentType.includes('application/json')) {
                    this.nativeInfo = await res.json();
                    this.nativeApiAvailable = true;
                    return true;
                }
            }
        } catch (_) {
            // وضع التصفح الثابت أو العمل دون خادم محلي (Static / Offline WASM Mode)
        } finally {
            if (timer) clearTimeout(timer);
        }

        this.nativeApiAvailable = false;
        return false;
    }

    /**
     * دالة مساعدة موحدة لاستدعاء دوال النصوص المصدرة من WebAssembly بأمان تام في الذاكرة
     */
    _callWasmStringFn(fnName, sourceCode) {
        if (!this.wasm) {
            throw new Error('لم يتم تهيئة محرك WebAssembly بعد');
        }
        if (typeof this.wasm[fnName] !== 'function') {
            throw new Error(`الدالة '${fnName}' غير متوفرة في نسخة WebAssembly الحالية`);
        }

        const encoder = new TextEncoder();
        const encoded = encoder.encode(String(sourceCode) + '\0');
        const len = encoded.length;

        const ptr = this.wasm.lisan_alloc(len);
        const inView = new Uint8Array(this.memory.buffer);
        inView.set(encoded, ptr);

        let resPtr = 0;
        try {
            resPtr = this.wasm[fnName](ptr);
            const outView = new Uint8Array(this.memory.buffer);
            let end = resPtr;
            while (outView[end] !== 0) {
                end++;
            }
            const decoder = new TextDecoder('utf-8');
            return decoder.decode(outView.subarray(resPtr, end));
        } finally {
            this.wasm.lisan_dealloc(ptr, len);
            if (resPtr) {
                this.wasm.lisan_free_string(resPtr);
            }
        }
    }

    /**
     * تفسير وتشغيل شيفرة لسان مباشرة عبر المفسر في WebAssembly
     */
    eval(sourceCode) {
        return this._callWasmStringFn('lisan_eval_wasm', sourceCode);
    }

    /**
     * تشغيل شيفرة لسان عبر آلة البايت كود الافتراضية (Bytecode VM)
     */
    evalVm(sourceCode) {
        if (!this.wasm || typeof this.wasm.lisan_vm_eval_wasm !== 'function') {
            return this.eval(sourceCode);
        }
        return this._callWasmStringFn('lisan_vm_eval_wasm', sourceCode);
    }

    /**
     * ترجمة شيفرة لسان إلى كود أردوينو C++ (.ino)
     */
    arduino(sourceCode) {
        if (!this.wasm || typeof this.wasm.lisan_arduino_wasm !== 'function') {
            throw new Error('مترجم الأردوينو في WebAssembly غير مهيأ');
        }
        return this._callWasmStringFn('lisan_arduino_wasm', sourceCode);
    }

    /**
     * ترجمة شيفرة لسان إلى كود متحكمات ESP32 / ESP8266 C++ (.ino)
     */
    esp32(sourceCode) {
        if (this.wasm && typeof this.wasm.lisan_esp32_wasm === 'function') {
            return this._callWasmStringFn('lisan_esp32_wasm', sourceCode);
        }
        return this.arduino(sourceCode);
    }

    /**
     * ترجمة شيفرة لسان إلى كود متحكمات STM32 C++ (.ino)
     */
    stm32(sourceCode) {
        if (this.wasm && typeof this.wasm.lisan_stm32_wasm === 'function') {
            return this._callWasmStringFn('lisan_stm32_wasm', sourceCode);
        }
        return this.arduino(sourceCode);
    }

    /**
     * ترجمة شيفرة لسان إلى كود راسبيري باي Python 3 (.py)
     */
    raspberry(sourceCode) {
        if (this.wasm && typeof this.wasm.lisan_raspberry_wasm === 'function') {
            return this._callWasmStringFn('lisan_raspberry_wasm', sourceCode);
        }
        return this._fallbackRaspberryTranspile(sourceCode);
    }

    /**
     * ترجمة شيفرة لسان إلى كود MicroPython لراسبيري باي بيكو (RP2040 / Pico W)
     */
    pico(sourceCode) {
        if (this.wasm && typeof this.wasm.lisan_pico_wasm === 'function') {
            return this._callWasmStringFn('lisan_pico_wasm', sourceCode);
        }
        return this.raspberry(sourceCode);
    }

    /**
     * تحويل شيفرة لسان إلى كود بايثون (Python 3)
     */
    python(sourceCode) {
        return this._callWasmStringFn('lisan_python_wasm', sourceCode);
    }

    /**
     * تحويل شيفرة لسان إلى كود جافاسكربت (ES2022)
     */
    js(sourceCode) {
        return this._callWasmStringFn('lisan_js_wasm', sourceCode);
    }

    /**
     * فحص شيفرة لسان (قواعد + عربية فقط + أنواع اختيارية) وإرجاع قائمة تشخيصات JSON
     */
    checkJson(sourceCode) {
        const raw = this._callWasmStringFn('lisan_check_json_wasm', sourceCode);
        try {
            return JSON.parse(raw);
        } catch (_) {
            return [];
        }
    }

    /**
     * تنسيق شيفرة لسان العربية وضبط المحاذاة والمسافات
     */
    fmt(sourceCode) {
        if (!this.wasm || typeof this.wasm.lisan_fmt_wasm !== 'function') {
            throw new Error('منسق الكود في WebAssembly غير مهيأ');
        }
        return this._callWasmStringFn('lisan_fmt_wasm', sourceCode);
    }

    /**
     * تشغيل ذكي غير متزامن: يستخدم خادم استوديو لسان الأصلي (/api/run) عند توفره
     * لدعم الملفات وقواعد البيانات ونظام التشغيل، ويعود تلقائياً إلى WebAssembly دون إنترنت.
     */
    async runAsync(sourceCode, mode = 'interpret') {
        if (this.nativeApiAvailable && this.useNativeBackend) {
            try {
                const res = await fetch('/api/run', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        code: sourceCode,
                        source: sourceCode,
                        mode: mode
                    })
                });
                if (res.ok || res.status === 400 || res.status === 422) {
                    return await this._parseApiResponse(res);
                }
            } catch (_) {
                // التراجع التلقائي السلس إلى محرك WebAssembly
            }
        }

        if (mode === 'vm') {
            return this.evalVm(sourceCode);
        }
        return this.eval(sourceCode);
    }

    /**
     * ترجمة ذكية غير متزامنة عبر (/api/transpile) أو محرك WebAssembly المحلي
     */
    async transpileAsync(sourceCode, target = 'arduino') {
        if (this.nativeApiAvailable && this.useNativeBackend) {
            try {
                const res = await fetch('/api/transpile', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        code: sourceCode,
                        source: sourceCode,
                        target: target
                    })
                });
                if (res.ok || res.status === 400 || res.status === 422) {
                    return await this._parseApiResponse(res);
                }
            } catch (_) {
                // التراجع التلقائي السلس إلى محرك WebAssembly
            }
        }

        switch (target) {
            case 'esp32':
                return this.esp32(sourceCode);
            case 'stm32':
                return this.stm32(sourceCode);
            case 'raspberry':
            case 'rpi':
                return this.raspberry(sourceCode);
            case 'pico':
            case 'rp2040':
                return this.pico(sourceCode);
            case 'python':
            case 'py':
                return this.python(sourceCode);
            case 'js':
            case 'javascript':
                return this.js(sourceCode);
            case 'fmt':
                return this.fmt(sourceCode);
            case 'arduino':
            default:
                return this.arduino(sourceCode);
        }
    }

    async _parseApiResponse(res) {
        const contentType = res.headers.get('content-type') || '';
        if (contentType.includes('application/json')) {
            const data = await res.json();
            if (data.ok === false || (data.error && !data.output && !data.code)) {
                const errText = String(data.error || 'خطأ غير معروف');
                return errText.startsWith('خطأ') ? errText : `خطأ: ${errText}`;
            }
            if (typeof data.output === 'string') return data.output;
            if (typeof data.code === 'string') return data.code;
            if (typeof data.result === 'string') return data.result;
            if (Array.isArray(data.lines)) return data.lines.join('\n');
            return '';
        }
        return await res.text();
    }

    /**
     * مترجم احتياطي خفيف إلى Python 3 لراسبيري باي في حال كانت نسخة lisan.wasm قديمة
     */
    _fallbackRaspberryTranspile(sourceCode) {
        const lines = [
            '#!/usr/bin/env python3',
            '# -*- coding: utf-8 -*-',
            '# ==================================================================',
            '# تم توليد كود راسبيري باي (Raspberry Pi 5 / Python 3) بواسطة لغة لسان',
            '# المكتبات المستهدفة: gpiozero / picamera2 / sense_hat / smbus2',
            '# ==================================================================',
            'import time',
            'from gpiozero import LED, Button, PWMOutputDevice, AngularServo, DistanceSensor'
        ];
        if (sourceCode.includes('كاميرا')) {
            lines.push('from picamera2 import Picamera2');
        }
        if (sourceCode.includes('سنس_هات') || sourceCode.includes('sense')) {
            lines.push('from sense_hat import SenseHat');
        }
        lines.push('', '# تنفيذ أوامر لسان المحوّلة:');
        for (const raw of String(sourceCode).split('\n')) {
            const t = raw.trim();
            if (!t || t.startsWith('#') || t.startsWith('//')) {
                lines.push('# ' + t.replace(/^(\/\/|#)\s*/, ''));
            } else if (t.startsWith('استورد')) {
                continue;
            } else if (t.startsWith('اطبع(') && t.endsWith(')')) {
                lines.push('print(' + t.slice(5, -1).replace(/،/g, ', ') + ')');
            } else {
                lines.push('# [لسان] ' + t);
            }
        }
        return lines.join('\n');
    }
}

window.LisanRuntime = LisanRuntime;

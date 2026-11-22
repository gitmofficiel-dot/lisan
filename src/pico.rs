//! مترجم ومحاكي لوحات راسبيري باي بيكو (Raspberry Pi Pico / Pico W / RP2040) بلغة لسان إلى MicroPython.
use crate::ast::{ClassDef, Expr, Function, Op, Stmt};
use std::path::{Path, PathBuf};

/// المكتبة القياسية المدمجة للوحات Raspberry Pi Pico / Pico W (RP2040 / MicroPython)
pub const STDLIB_PICO: &str = r##"
# مكتبة راسبيري باي بيكو القياسية في لغة لسان (RP2040 / Pico W / MicroPython)

دع بيكو_عالي = 1
دع بيكو_منخفض = 0
دع مخرج_بيكو = "مخرج"
دع مدخل_بيكو = "مدخل"
دع سحب_علوي_بيكو = "علوي"
دع سحب_سفلي_بيكو = "سفلي"
دع بدون_سحب_بيكو = "بدون"

صنف لوحة_بيكو {
    دالة بناء(طراز = "Raspberry Pi Pico W") {
        هذا.طراز = طراز
        هذا.تردد_ميجا = 125
        هذا.حرارة = 31.5
        هذا.ذاكرة_حرة = 192
        إذا (طراز.تحتوي("Pico 2")) {
            هذا.تردد_ميجا = 150
            هذا.ذاكرة_حرة = 420
        }
        هذا.وضع_المصباح = خطأ
    }

    دالة اسم_الطراز() {
        أرجع هذا.طراز
    }

    دالة تردد_المعالج_ميجا() {
        أرجع هذا.تردد_ميجا
    }

    دالة حرارة_الشريحة() {
        أرجع هذا.حرارة
    }

    دالة ضبط_حرارة_محاكاة(حرارة_جديدة) {
        هذا.حرارة = حرارة_جديدة
        أرجع هذا.حرارة
    }

    دالة ذاكرة_حرة_كيلوبايت() {
        أرجع هذا.ذاكرة_حرة
    }

    دالة مصباح_اللوحة(تشغيل) {
        دع تفعيل = خطأ
        إذا (تشغيل == صحيح || تشغيل == 1 || تشغيل == "تشغيل" || تشغيل == "ON") {
            تفعيل = صحيح
        }
        هذا.وضع_المصباح = تفعيل
        أرجع هذا.وضع_المصباح
    }
}

صنف منفذ_بيكو {
    دالة بناء(رقم_المنفذ، اتجاه = "مخرج"، سحب = "بدون") {
        هذا.رقم_المنفذ = رقم_المنفذ
        هذا.اتجاه = اتجاه
        هذا.سحب = سحب
        هذا.القيمة = 0
        إذا (اتجاه == "مدخل" && سحب == "علوي") {
            هذا.القيمة = 1
        }
    }

    دالة اكتب(قيمة) {
        دع قيمة_رقمية = 0
        إذا (قيمة == 1 || قيمة == صحيح || قيمة == "عالي" || قيمة == "HIGH") {
            قيمة_رقمية = 1
        }
        هذا.القيمة = قيمة_رقمية
        أرجع هذا.القيمة
    }

    دالة شغّل() {
        أرجع هذا.اكتب(1)
    }

    دالة شغل() {
        أرجع هذا.شغّل()
    }

    دالة أوقف() {
        أرجع هذا.اكتب(0)
    }

    دالة اوقف() {
        أرجع هذا.أوقف()
    }

    دالة اقلب() {
        إذا (هذا.القيمة == 0) {
            هذا.القيمة = 1
        } وإلا {
            هذا.القيمة = 0
        }
        أرجع هذا.القيمة
    }

    دالة اقرأ() {
        أرجع هذا.القيمة
    }

    دالة اقرا() {
        أرجع هذا.اقرأ()
    }

    دالة ضبط_محاكاة(قيمة) {
        أرجع هذا.اكتب(قيمة)
    }
}

صنف نبضة_بيكو {
    دالة بناء(رقم_المنفذ، تردد = 1000) {
        هذا.رقم_المنفذ = رقم_المنفذ
        هذا.تردد = تردد
        هذا.دورة_16bit = 0
        هذا.نسبة = 0
        هذا.نشط = صحيح
    }

    دالة اضبط_تردد(تردد) {
        إذا (تردد > 0) {
            هذا.تردد = تردد
        }
        أرجع هذا.تردد
    }

    دالة اضبط_دورة_16bit(قيمة) {
        دع ق = قيمة
        إذا (ق < 0) {
            ق = 0
        }
        إذا (ق > 65535) {
            ق = 65535
        }
        هذا.دورة_16bit = قرّب(ق)
        هذا.نسبة = قرّب((هذا.دورة_16bit * 100) / 65535)
        هذا.نشط = صحيح
        أرجع هذا.دورة_16bit
    }

    دالة اضبط_نسبة(نسبة) {
        دع ن = نسبة
        إذا (ن < 0) {
            ن = 0
        }
        إذا (ن > 100) {
            ن = 100
        }
        هذا.نسبة = ن
        هذا.دورة_16bit = قرّب((ن / 100) * 65535)
        هذا.نشط = صحيح
        أرجع هذا.نسبة
    }

    دالة أوقف() {
        هذا.دورة_16bit = 0
        هذا.نسبة = 0
        هذا.نشط = خطأ
        أرجع صحيح
    }

    دالة اوقف() {
        أرجع هذا.أوقف()
    }
}

صنف محول_تماثلي_بيكو {
    دالة بناء(رقم_المنفذ = 26) {
        هذا.رقم_المنفذ = رقم_المنفذ
        هذا.قيمة_خام = 32768
    }

    دالة اقرأ_16bit() {
        أرجع هذا.قيمة_خام
    }

    دالة اقرا_16bit() {
        أرجع هذا.اقرأ_16bit()
    }

    دالة اقرأ_جهد() {
        دع جهد = (هذا.قيمة_خام / 65535) * 3.3
        أرجع قرّب(جهد * 1000) / 1000
    }

    دالة اقرا_جهد() {
        أرجع هذا.اقرأ_جهد()
    }

    دالة ضبط_محاكاة(قيمة_16bit) {
        دع ق = قيمة_16bit
        إذا (ق < 0) {
            ق = 0
        }
        إذا (ق > 65535) {
            ق = 65535
        }
        هذا.قيمة_خام = قرّب(ق)
        أرجع هذا.قيمة_خام
    }
}

صنف ناقل_i2c_بيكو {
    دالة بناء(رقم = 0، sda = 0، scl = 1، تردد = 400000) {
        هذا.رقم = رقم
        هذا.sda = sda
        هذا.scl = scl
        هذا.تردد = تردد
        هذا.سجلات = {}
        هذا.أجهزة = [60، 118]
    }

    دالة فحص_الأجهزة() {
        أرجع هذا.أجهزة
    }

    دالة فحص_الاجهزة() {
        أرجع هذا.فحص_الأجهزة()
    }

    دالة اكتب_بايت(عنوان، سجل، قيمة) {
        دع مفتاح = نص(عنوان) + ":" + نص(سجل)
        دع خريطة = هذا.سجلات
        خريطة[مفتاح] = قيمة
        إذا (!(عنوان في هذا.أجهزة)) {
            هذا.أجهزة.أضف(عنوان)
        }
        أرجع صحيح
    }

    دالة اقرأ_بايت(عنوان، سجل) {
        دع مفتاح = نص(عنوان) + ":" + نص(سجل)
        إذا (مفتاح في هذا.سجلات) {
            أرجع هذا.سجلات[مفتاح]
        }
        أرجع 0
    }

    دالة اقرا_بايت(عنوان، سجل) {
        أرجع هذا.اقرأ_بايت(عنوان، سجل)
    }
}

صنف ناقل_spi_بيكو {
    دالة بناء(رقم = 0، سرعة = 1000000) {
        هذا.رقم = رقم
        هذا.سرعة = سرعة
        هذا.مهيأ = صحيح
    }

    دالة نقل(قائمة_بايتات) {
        دع رد = []
        لكل بايت في قائمة_بايتات {
            رد.أضف(بايت)
        }
        أرجع رد
    }
}

صنف منفذ_uart_بيكو {
    دالة بناء(رقم = 0، سرعة = 115200، tx = 0، rx = 1) {
        هذا.رقم = رقم
        هذا.سرعة = سرعة
        هذا.tx = tx
        هذا.rx = rx
        هذا.آخر_مرسل = ""
        هذا.مخزن_استقبال = "OK"
    }

    دالة اكتب(رسالة) {
        هذا.آخر_مرسل = نص(رسالة)
        أرجع صحيح
    }

    دالة اقرأ_سطر() {
        أرجع هذا.مخزن_استقبال
    }

    دالة اقرا_سطر() {
        أرجع هذا.اقرأ_سطر()
    }

    دالة ضبط_محاكاة(رسالة) {
        هذا.مخزن_استقبال = نص(رسالة)
        أرجع هذا.مخزن_استقبال
    }
}

صنف واي_فاي_بيكو_w {
    دالة بناء() {
        هذا.متصل = خطأ
        هذا.الشبكة = ""
        هذا.عنوان = "0.0.0.0"
    }

    دالة اتصل(شبكة، كلمة_سر) {
        هذا.الشبكة = نص(شبكة)
        هذا.متصل = صحيح
        هذا.عنوان = "192.168.4.120"
        أرجع صحيح
    }

    دالة هل_متصل() {
        أرجع هذا.متصل
    }

    دالة عنوان_ip() {
        أرجع هذا.عنوان
    }

    دالة قطع_الاتصال() {
        هذا.متصل = خطأ
        هذا.عنوان = "0.0.0.0"
        أرجع صحيح
    }
}

صنف شريط_نيوبكسل_بيكو {
    دالة بناء(رقم_المنفذ، عدد = 8) {
        هذا.رقم_المنفذ = رقم_المنفذ
        هذا.العدد = عدد
        هذا.مصابيح = []
        هذا.مرات_العرض = 0
        لكل مؤشر في مدى(عدد) {
            هذا.مصابيح.أضف([0، 0، 0])
        }
    }

    دالة اضبط_مصباح(فهرس، أحمر، أخضر، أزرق) {
        إذا (فهرس >= 0 && فهرس < هذا.العدد) {
            دع قائمة_مصابيح = هذا.مصابيح
            قائمة_مصابيح[فهرس] = [أحمر، أخضر، أزرق]
            أرجع صحيح
        }
        أرجع خطأ
    }

    دالة املأ(أحمر، أخضر، أزرق) {
        دع قائمة_مصابيح = هذا.مصابيح
        لكل مؤشر في مدى(هذا.العدد) {
            قائمة_مصابيح[مؤشر] = [أحمر، أخضر، أزرق]
        }
        أرجع صحيح
    }

    دالة املا(أحمر، أخضر، أزرق) {
        أرجع هذا.املأ(أحمر، أخضر، أزرق)
    }

    دالة اعرض() {
        هذا.مرات_العرض = هذا.مرات_العرض + 1
        أرجع صحيح
    }
}

صنف مؤقت_بيكو {
    دالة بناء(فترة_ملي = 1000) {
        هذا.فترة_ملي = فترة_ملي
        هذا.يعمل = خطأ
        هذا.نبضات = 0
    }

    دالة ابدأ() {
        هذا.يعمل = صحيح
        أرجع صحيح
    }

    دالة ابدا() {
        أرجع هذا.ابدأ()
    }

    دالة أوقف() {
        هذا.يعمل = خطأ
        أرجع صحيح
    }

    دالة اوقف() {
        أرجع هذا.أوقف()
    }

    دالة عدد_النبضات() {
        أرجع هذا.نبضات
    }

    دالة نبضة_محاكاة() {
        إذا (هذا.يعمل) {
            هذا.نبضات = هذا.نبضات + 1
        }
        أرجع هذا.نبضات
    }
}

صنف وحدة_pio_بيكو {
    دالة بناء(آلة_رقم = 0، تردد = 2000000، منفذ_أساسي = 2) {
        هذا.آلة_رقم = آلة_رقم
        هذا.تردد = تردد
        هذا.منفذ_أساسي = منفذ_أساسي
        هذا.مفعلة = خطأ
        هذا.طابور_الإرسال = []
    }

    دالة فعّل() {
        هذا.مفعلة = صحيح
        أرجع صحيح
    }

    دالة فعل() {
        أرجع هذا.فعّل()
    }

    دالة عطّل() {
        هذا.مفعلة = خطأ
        أرجع صحيح
    }

    دالة عطل() {
        أرجع هذا.عطّل()
    }

    دالة أرسل_كلمة(قيمة_32bit) {
        هذا.طابور_الإرسال.أضف(قيمة_32bit)
        أرجع صحيح
    }

    دالة ارسل_كلمة(قيمة_32bit) {
        أرجع هذا.أرسل_كلمة(قيمة_32bit)
    }
}

صنف مستشعر_حرارة_داخلي_بيكو {
    دالة بناء() {
        هذا.مئوية = 27.5
    }

    دالة اقرأ_مئوية() {
        أرجع هذا.مئوية
    }

    دالة اقرا_مئوية() {
        أرجع هذا.اقرأ_مئوية()
    }

    دالة اقرأ_فهرنهايت() {
        دع فهرنهايت = (هذا.مئوية * 9 / 5) + 32
        أرجع قرّب(فهرنهايت * 100) / 100
    }

    دالة اقرا_فهرنهايت() {
        أرجع هذا.اقرأ_فهرنهايت()
    }

    دالة ضبط_محاكاة(مئوية) {
        هذا.مئوية = مئوية
        أرجع هذا.مئوية
    }
}
"##;

/// وقت التشغيل المدمج في سكربت MicroPython المولّد للوحات Raspberry Pi Pico / Pico W (RP2040)
/// يعمل على العتاد الحقيقي (MicroPython) ويتراجع تلقائيًا لمحاكاة سطح المكتب على Python 3
const MICROPYTHON_PICO_RUNTIME: &str = r##"# مولّد تلقائيًا بواسطة مترجم لغة لسان للوحات راسبيري باي بيكو (RP2040 / Pico W - MicroPython)
# يعمل مباشرة كـ main.py على لوحة Pico W كما يعمل في وضع المحاكاة على Python 3!

import math
import time

try:
    import gc as _gc
except ImportError:
    _gc = None

try:
    import machine
    _IS_MICROPYTHON = True
except ImportError:
    _IS_MICROPYTHON = False

    class _SimPin:
        IN = 0
        OUT = 1
        PULL_UP = 1
        PULL_DOWN = 2

        def __init__(self, pin_id, mode=OUT, pull=None):
            self.pin_id = pin_id
            self.mode = mode
            self.pull = pull
            self._val = 1 if (mode == self.IN and pull == self.PULL_UP) else 0

        def value(self, val=None):
            if val is not None:
                self._val = 1 if val else 0
            return self._val

        def on(self):
            self._val = 1

        def off(self):
            self._val = 0

        def toggle(self):
            self._val = 0 if self._val else 1

    class _SimPWM:
        def __init__(self, pin):
            self.pin = pin
            self._freq = 1000
            self._duty = 0

        def freq(self, val=None):
            if val is not None:
                self._freq = int(val)
            return self._freq

        def duty_u16(self, val=None):
            if val is not None:
                self._duty = max(0, min(65535, int(val)))
            return self._duty

        def deinit(self):
            self._duty = 0

    class _SimADC:
        def __init__(self, pin_or_ch):
            self.channel = pin_or_ch
            self._raw = 32768

        def read_u16(self):
            return self._raw

    class _SimI2C:
        def __init__(self, bus_id, sda=None, scl=None, freq=400000):
            self.bus_id = bus_id
            self.sda = sda
            self.scl = scl
            self.freq = freq
            self._regs = {}
            self._devices = [60, 118]

        def scan(self):
            return list(self._devices)

        def writeto_mem(self, addr, reg, data):
            val = data[0] if len(data) > 0 else 0
            self._regs[(addr, reg)] = val
            if addr not in self._devices:
                self._devices.append(addr)

        def readfrom_mem(self, addr, reg, nbytes):
            val = self._regs.get((addr, reg), 0)
            return bytes([val & 0xFF] * max(1, int(nbytes)))

    class _SimSPI:
        def __init__(self, bus_id, baudrate=1000000):
            self.bus_id = bus_id
            self.baudrate = baudrate

        def write_readinto(self, write_buf, read_buf):
            for idx in range(min(len(write_buf), len(read_buf))):
                read_buf[idx] = write_buf[idx]

    class _SimUART:
        def __init__(self, uart_id, baudrate=115200, tx=None, rx=None):
            self.uart_id = uart_id
            self.baudrate = baudrate
            self.tx = tx
            self.rx = rx
            self._rx_buf = b"OK\n"

        def write(self, data):
            return len(data)

        def readline(self):
            return self._rx_buf

    class _SimTimer:
        PERIODIC = 1

        def __init__(self):
            self._running = False

        def init(self, period=1000, mode=PERIODIC, callback=None):
            self._running = True
            self._callback = callback

        def deinit(self):
            self._running = False

    class _SimMachineModule:
        Pin = _SimPin
        PWM = _SimPWM
        ADC = _SimADC
        I2C = _SimI2C
        SPI = _SimSPI
        UART = _SimUART
        Timer = _SimTimer

        @staticmethod
        def freq():
            return 125000000

    machine = _SimMachineModule()

try:
    import network
    _HAS_NETWORK = True
except ImportError:
    _HAS_NETWORK = False

    class _SimWLAN:
        def __init__(self, mode):
            self.mode = mode
            self._active = False
            self._connected = False
            self._ip = "0.0.0.0"

        def active(self, state=None):
            if state is not None:
                self._active = bool(state)
            return self._active

        def connect(self, ssid, password):
            self._active = True
            self._connected = True
            self._ip = "192.168.4.120"

        def isconnected(self):
            return self._connected

        def ifconfig(self):
            return (self._ip, "255.255.255.0", "192.168.4.1", "8.8.8.8")

        def disconnect(self):
            self._connected = False
            self._ip = "0.0.0.0"

    class _SimNetworkModule:
        STA_IF = 0
        WLAN = _SimWLAN

    network = _SimNetworkModule()

try:
    import neopixel
    _HAS_NEOPIXEL = True
except ImportError:
    _HAS_NEOPIXEL = False

    class _SimNeoPixel:
        def __init__(self, pin, count):
            self.pin = pin
            self._pixels = [(0, 0, 0)] * int(count)

        def __setitem__(self, index, color):
            self._pixels[index] = color

        def __getitem__(self, index):
            return self._pixels[index]

        def fill(self, color):
            for idx in range(len(self._pixels)):
                self._pixels[idx] = color

        def write(self):
            pass

    class _SimNeoPixelModule:
        NeoPixel = _SimNeoPixel

    neopixel = _SimNeoPixelModule()

try:
    import rp2
    _HAS_RP2 = True
except ImportError:
    _HAS_RP2 = False

    class _SimStateMachine:
        def __init__(self, sm_id, prog=None, freq=2000000, set_base=None):
            self.sm_id = sm_id
            self.freq = freq
            self.set_base = set_base
            self._active = False
            self._fifo = []

        def active(self, val=None):
            if val is not None:
                self._active = bool(val)
            return self._active

        def put(self, word):
            self._fifo.append(int(word) & 0xFFFFFFFF)

    class _SimRP2Module:
        StateMachine = _SimStateMachine

        @staticmethod
        def asm_pio(**kwargs):
            def _decorator(fn):
                return fn
            return _decorator

    rp2 = _SimRP2Module()

بيكو_عالي = 1
بيكو_منخفض = 0
مخرج_بيكو = "مخرج"
مدخل_بيكو = "مدخل"
سحب_علوي_بيكو = "علوي"
سحب_سفلي_بيكو = "سفلي"
بدون_سحب_بيكو = "بدون"


def _lsn_add(a, b):
    if isinstance(a, str) or isinstance(b, str):
        return str(a) + str(b)
    return a + b


def _lsn_sleep_ms(ms):
    if hasattr(time, "sleep_ms"):
        time.sleep_ms(max(0, int(ms)))
    else:
        time.sleep(max(0.0, float(ms)) / 1000.0)


def _lsn_sleep_us(us):
    if hasattr(time, "sleep_us"):
        time.sleep_us(max(0, int(us)))
    else:
        time.sleep(max(0.0, float(us)) / 1000000.0)


def _lsn_ticks_ms():
    if hasattr(time, "ticks_ms"):
        return time.ticks_ms()
    return int(time.monotonic() * 1000)


class لوحة_بيكو:
    def __init__(self, طراز="Raspberry Pi Pico W"):
        self.طراز = طراز
        self.تردد_ميجا = int(machine.freq() // 1000000) if _IS_MICROPYTHON else (150 if "Pico 2" in str(طراز) else 125)
        self.حرارة = 31.5
        self.ذاكرة_حرة = 420 if "Pico 2" in str(طراز) else 192
        self.وضع_المصباح = False
        self._led = None
        if _IS_MICROPYTHON:
            try:
                self._led = machine.Pin("LED", machine.Pin.OUT)
            except Exception:
                self._led = machine.Pin(25, machine.Pin.OUT)

    def اسم_الطراز(self):
        return self.طراز

    def تردد_المعالج_ميجا(self):
        if _IS_MICROPYTHON:
            return int(machine.freq() // 1000000)
        return self.تردد_ميجا

    def حرارة_الشريحة(self):
        if _IS_MICROPYTHON:
            try:
                raw = machine.ADC(4).read_u16()
                voltage = raw * (3.3 / 65535.0)
                return round(27.0 - (voltage - 0.706) / 0.001721, 2)
            except Exception:
                pass
        return self.حرارة

    def ضبط_حرارة_محاكاة(self, حرارة_جديدة):
        self.حرارة = float(حرارة_جديدة)
        return self.حرارة

    def ذاكرة_حرة_كيلوبايت(self):
        if _IS_MICROPYTHON and _gc is not None and hasattr(_gc, "mem_free"):
            return round(_gc.mem_free() / 1024.0, 1)
        return self.ذاكرة_حرة

    def مصباح_اللوحة(self, تشغيل):
        تفعيل = تشغيل in (True, 1, "تشغيل", "ON")
        self.وضع_المصباح = تفعيل
        if self._led is not None:
            self._led.value(1 if تفعيل else 0)
        return self.وضع_المصباح


class منفذ_بيكو:
    def __init__(self, رقم_المنفذ, اتجاه="مخرج", سحب="بدون"):
        self.رقم_المنفذ = رقم_المنفذ
        self.اتجاه = اتجاه
        self.سحب = سحب
        self.القيمة = 1 if (اتجاه == "مدخل" and سحب == "علوي") else 0
        mode = machine.Pin.OUT if اتجاه in ("مخرج", "OUTPUT") else machine.Pin.IN
        pull = None
        if سحب == "علوي":
            pull = machine.Pin.PULL_UP
        elif سحب == "سفلي":
            pull = machine.Pin.PULL_DOWN
        self._pin = machine.Pin(رقم_المنفذ, mode, pull)

    def اكتب(self, قيمة):
        قيمة_رقمية = 1 if قيمة in (1, True, "عالي", "HIGH") else 0
        self.القيمة = قيمة_رقمية
        self._pin.value(قيمة_رقمية)
        return self.القيمة

    def شغّل(self):
        return self.اكتب(1)

    شغل = شغّل

    def أوقف(self):
        return self.اكتب(0)

    اوقف = أوقف

    def اقلب(self):
        self.القيمة = 0 if self.القيمة else 1
        self._pin.value(self.القيمة)
        return self.القيمة

    def اقرأ(self):
        if _IS_MICROPYTHON:
            self.القيمة = int(self._pin.value())
        return self.القيمة

    اقرا = اقرأ

    def ضبط_محاكاة(self, قيمة):
        return self.اكتب(قيمة)


class نبضة_بيكو:
    def __init__(self, رقم_المنفذ, تردد=1000):
        self.رقم_المنفذ = int(رقم_المنفذ)
        self.تردد = int(تردد)
        self.دورة_16bit = 0
        self.نسبة = 0
        self.نشط = True
        self._pwm = machine.PWM(machine.Pin(self.رقم_المنفذ))
        self._pwm.freq(self.تردد)

    def اضبط_تردد(self, تردد):
        if تردد > 0:
            self.تردد = int(تردد)
            self._pwm.freq(self.تردد)
        return self.تردد

    def اضبط_دورة_16bit(self, قيمة):
        ق = max(0, min(65535, int(round(قيمة))))
        self.دورة_16bit = ق
        self.نسبة = round((ق * 100.0) / 65535.0)
        self.نشط = True
        self._pwm.duty_u16(ق)
        return self.دورة_16bit

    def اضبط_نسبة(self, نسبة):
        ن = max(0.0, min(100.0, float(نسبة)))
        self.نسبة = ن
        self.دورة_16bit = int(round((ن / 100.0) * 65535.0))
        self.نشط = True
        self._pwm.duty_u16(self.دورة_16bit)
        return self.نسبة

    def أوقف(self):
        self.دورة_16bit = 0
        self.نسبة = 0
        self.نشط = False
        self._pwm.duty_u16(0)
        self._pwm.deinit()
        return True

    اوقف = أوقف


class محول_تماثلي_بيكو:
    def __init__(self, رقم_المنفذ=26):
        self.رقم_المنفذ = int(رقم_المنفذ)
        self.قيمة_خام = 32768
        self._adc = machine.ADC(self.رقم_المنفذ)

    def اقرأ_16bit(self):
        if _IS_MICROPYTHON:
            self.قيمة_خام = int(self._adc.read_u16())
        return self.قيمة_خام

    اقرا_16bit = اقرأ_16bit

    def اقرأ_جهد(self):
        raw = self.اقرأ_16bit()
        return round((raw / 65535.0) * 3.3, 3)

    اقرا_جهد = اقرأ_جهد

    def ضبط_محاكاة(self, قيمة_16bit):
        self.قيمة_خام = max(0, min(65535, int(round(قيمة_16bit))))
        return self.قيمة_خام


class ناقل_i2c_بيكو:
    def __init__(self, رقم=0, sda=0, scl=1, تردد=400000):
        self.رقم = int(رقم)
        self.sda = int(sda)
        self.scl = int(scl)
        self.تردد = int(تردد)
        self.سجلات = {}
        self.أجهزة = [60, 118]
        self._i2c = machine.I2C(
            self.رقم,
            sda=machine.Pin(self.sda),
            scl=machine.Pin(self.scl),
            freq=self.تردد,
        )

    def فحص_الأجهزة(self):
        if _IS_MICROPYTHON:
            return self._i2c.scan()
        return list(self.أجهزة)

    فحص_الاجهزة = فحص_الأجهزة

    def اكتب_بايت(self, عنوان, سجل, قيمة):
        addr = int(عنوان)
        reg = int(سجل)
        val = int(قيمة) & 0xFF
        self.سجلات[(addr, reg)] = val
        if addr not in self.أجهزة:
            self.أجهزة.append(addr)
        if _IS_MICROPYTHON:
            self._i2c.writeto_mem(addr, reg, bytes([val]))
        return True

    def اقرأ_بايت(self, عنوان, سجل):
        addr = int(عنوان)
        reg = int(سجل)
        if _IS_MICROPYTHON:
            data = self._i2c.readfrom_mem(addr, reg, 1)
            return data[0] if len(data) > 0 else 0
        return self.سجلات.get((addr, reg), 0)

    اقرا_بايت = اقرأ_بايت


class ناقل_spi_بيكو:
    def __init__(self, رقم=0, سرعة=1000000):
        self.رقم = int(رقم)
        self.سرعة = int(سرعة)
        self.مهيأ = True
        self._spi = machine.SPI(self.رقم, baudrate=self.سرعة)

    def نقل(self, قائمة_بايتات):
        tx = bytearray([int(b) & 0xFF for b in قائمة_بايتات])
        rx = bytearray(len(tx))
        self._spi.write_readinto(tx, rx)
        return list(rx)


class منفذ_uart_بيكو:
    def __init__(self, رقم=0, سرعة=115200, tx=0, rx=1):
        self.رقم = int(رقم)
        self.سرعة = int(سرعة)
        self.tx = int(tx)
        self.rx = int(rx)
        self.آخر_مرسل = ""
        self.مخزن_استقبال = "OK"
        self._uart = machine.UART(
            self.رقم,
            baudrate=self.سرعة,
            tx=machine.Pin(self.tx),
            rx=machine.Pin(self.rx),
        )

    def اكتب(self, رسالة):
        self.آخر_مرسل = str(رسالة)
        self._uart.write(self.آخر_مرسل.encode("utf-8"))
        return True

    def اقرأ_سطر(self):
        if _IS_MICROPYTHON:
            line = self._uart.readline()
            if line:
                return line.decode("utf-8").strip()
            return ""
        return self.مخزن_استقبال

    اقرا_سطر = اقرأ_سطر

    def ضبط_محاكاة(self, رسالة):
        self.مخزن_استقبال = str(رسالة)
        return self.مخزن_استقبال


class واي_فاي_بيكو_w:
    def __init__(self):
        self.متصل = False
        self.الشبكة = ""
        self.عنوان = "0.0.0.0"
        self._wlan = network.WLAN(network.STA_IF)

    def اتصل(self, شبكة, كلمة_سر):
        self.الشبكة = str(شبكة)
        self._wlan.active(True)
        self._wlan.connect(str(شبكة), str(كلمة_سر))
        if _IS_MICROPYTHON:
            for _ in range(20):
                if self._wlan.isconnected():
                    break
                _lsn_sleep_ms(250)
        self.متصل = bool(self._wlan.isconnected())
        if self.متصل:
            self.عنوان = self._wlan.ifconfig()[0]
        return self.متصل

    def هل_متصل(self):
        if _IS_MICROPYTHON:
            self.متصل = bool(self._wlan.isconnected())
        return self.متصل

    def عنوان_ip(self):
        if self.هل_متصل():
            self.عنوان = self._wlan.ifconfig()[0]
        return self.عنوان

    def قطع_الاتصال(self):
        self._wlan.disconnect()
        self._wlan.active(False)
        self.متصل = False
        self.عنوان = "0.0.0.0"
        return True


class شريط_نيوبكسل_بيكو:
    def __init__(self, رقم_المنفذ, عدد=8):
        self.رقم_المنفذ = int(رقم_المنفذ)
        self.العدد = int(عدد)
        self.مصابيح = [[0, 0, 0] for _ in range(self.العدد)]
        self.مرات_العرض = 0
        self._strip = neopixel.NeoPixel(machine.Pin(self.رقم_المنفذ), self.العدد)

    def اضبط_مصباح(self, فهرس, أحمر, أخضر, أزرق):
        idx = int(فهرس)
        if 0 <= idx < self.العدد:
            r, g, b = int(أحمر) & 0xFF, int(أخضر) & 0xFF, int(أزرق) & 0xFF
            self.مصابيح[idx] = [r, g, b]
            self._strip[idx] = (r, g, b)
            return True
        return False

    def املأ(self, أحمر, أخضر, أزرق):
        r, g, b = int(أحمر) & 0xFF, int(أخضر) & 0xFF, int(أزرق) & 0xFF
        for idx in range(self.العدد):
            self.مصابيح[idx] = [r, g, b]
            self._strip[idx] = (r, g, b)
        return True

    املا = املأ

    def اعرض(self):
        self._strip.write()
        self.مرات_العرض += 1
        return True


class مؤقت_بيكو:
    def __init__(self, فترة_ملي=1000):
        self.فترة_ملي = int(فترة_ملي)
        self.يعمل = False
        self.نبضات = 0
        self._timer = machine.Timer()

    def _on_tick(self, _t):
        self.نبضات += 1

    def ابدأ(self):
        self.يعمل = True
        if _IS_MICROPYTHON:
            self._timer.init(period=self.فترة_ملي, mode=machine.Timer.PERIODIC, callback=self._on_tick)
        return True

    ابدا = ابدأ

    def أوقف(self):
        self.يعمل = False
        self._timer.deinit()
        return True

    اوقف = أوقف

    def عدد_النبضات(self):
        return self.نبضات

    def نبضة_محاكاة(self):
        if self.يعمل:
            self.نبضات += 1
        return self.نبضات


class وحدة_pio_بيكو:
    def __init__(self, آلة_رقم=0, تردد=2000000, منفذ_أساسي=2):
        self.آلة_رقم = int(آلة_رقم)
        self.تردد = int(تردد)
        self.منفذ_أساسي = int(منفذ_أساسي)
        self.مفعلة = False
        self.طابور_الإرسال = []
        self._sm = None
        if not _IS_MICROPYTHON:
            self._sm = rp2.StateMachine(
                self.آلة_رقم,
                None,
                freq=self.تردد,
                set_base=machine.Pin(self.منفذ_أساسي),
            )

    def فعّل(self):
        self.مفعلة = True
        if self._sm is not None:
            self._sm.active(True)
        return True

    فعل = فعّل

    def عطّل(self):
        self.مفعلة = False
        if self._sm is not None:
            self._sm.active(False)
        return True

    عطل = عطّل

    def أرسل_كلمة(self, قيمة_32bit):
        word = int(قيمة_32bit) & 0xFFFFFFFF
        self.طابور_الإرسال.append(word)
        if self._sm is not None:
            self._sm.put(word)
        return True

    ارسل_كلمة = أرسل_كلمة


class مستشعر_حرارة_داخلي_بيكو:
    def __init__(self):
        self.مئوية = 27.5
        self._adc = machine.ADC(4)

    def اقرأ_مئوية(self):
        if _IS_MICROPYTHON:
            raw = self._adc.read_u16()
            voltage = raw * (3.3 / 65535.0)
            self.مئوية = round(27.0 - (voltage - 0.706) / 0.001721, 2)
        return self.مئوية

    اقرا_مئوية = اقرأ_مئوية

    def اقرأ_فهرنهايت(self):
        c = self.اقرأ_مئوية()
        return round((c * 9.0 / 5.0) + 32.0, 2)

    اقرا_فهرنهايت = اقرأ_فهرنهايت

    def ضبط_محاكاة(self, مئوية):
        self.مئوية = float(مئوية)
        return self.مئوية
"##;

/// تحويل شيفرة لسان إلى سكربت MicroPython (`main.py`) مخصص للوحات Raspberry Pi Pico / Pico W (RP2040)
/// مع طبقة محاكاة مدمجة تسمح بتشغيله وفحصه أيضًا على Python 3
pub fn transpile_pico_micropython(source: &str) -> Result<String, String> {
    let tokens = crate::lexer::tokenize(source)?;
    let statements = crate::parser::parse(tokens)?;
    emit_pico_program(&statements)
}

/// بناء وحفظ ملف MicroPython (`<stem>_pico.py`) للوحات Raspberry Pi Pico من ملف لسان (`.lsn`)
pub fn build_pico_file(input: &Path, output: Option<&Path>) -> Result<PathBuf, String> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| format!("تعذرت قراءة ملف لسان '{}': {}", input.display(), e))?;
    let py_code = transpile_pico_micropython(&source)?;

    let out_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("main");
            input.with_file_name(format!("{}_pico.py", stem))
        }
    };

    std::fs::write(&out_path, py_code)
        .map_err(|e| format!("تعذر حفظ ملف بيكو MicroPython '{}': {}", out_path.display(), e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&out_path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&out_path, perms);
        }
    }

    Ok(out_path)
}

fn emit_pico_program(statements: &[Stmt]) -> Result<String, String> {
    let mut out = String::new();
    out.push_str("#!/usr/bin/env python3\n");
    out.push_str("# -*- coding: utf-8 -*-\n");
    out.push_str(MICROPYTHON_PICO_RUNTIME);
    out.push_str("\n# --- بداية شيفرة المستخدم المترجمة من لغة لسان إلى MicroPython (RP2040) ---\n\n");

    for stmt in statements {
        let rendered = emit_py_stmt(stmt, 0, false);
        if !rendered.is_empty() {
            out.push_str(&rendered);
        }
    }

    Ok(out)
}

fn is_pico_builtin_module(name: &str) -> bool {
    let trimmed = name.trim().strip_suffix(".lsn").unwrap_or(name.trim());
    let norm = crate::diagnostics::normalize_arabic(trimmed);
    matches!(
        norm.as_str(),
        "بيكو"
            | "بيكو_w"
            | "بيكو_دبليو"
            | "راسبيري_بيكو"
            | "pico"
            | "pico_w"
            | "picow"
            | "rp2040"
            | "micropython"
            | "مايكروبايثون"
            | "متحكمات"
            | "embedded"
    )
}

fn emit_py_body(stmts: &[Stmt], indent: usize, in_init: bool) -> String {
    let pad = "    ".repeat(indent);
    if stmts.is_empty() {
        return format!("{}pass\n", pad);
    }
    let mut s = String::new();
    for inner in stmts {
        s.push_str(&emit_py_stmt(inner, indent, in_init));
    }
    if s.trim().is_empty() {
        format!("{}pass\n", pad)
    } else {
        s
    }
}

fn emit_py_stmt(stmt: &Stmt, indent: usize, in_init: bool) -> String {
    let pad = "    ".repeat(indent);
    match stmt {
        Stmt::Spanned(inner, _, _) => emit_py_stmt(inner, indent, in_init),
        Stmt::Import(module) => {
            if is_pico_builtin_module(module) {
                String::new()
            } else {
                format!("{}# استورد {}\n", pad, module)
            }
        }
        Stmt::ImportModule { module, alias } => {
            if is_pico_builtin_module(module) {
                String::new()
            } else if let Some(al) = alias {
                format!("{}import {} as {}\n", pad, py_ident(module), py_ident(al))
            } else {
                format!("{}import {}\n", pad, py_ident(module))
            }
        }
        Stmt::FromImport { module, items } => {
            if is_pico_builtin_module(module) {
                String::new()
            } else {
                let py_items: Vec<String> = items.iter().map(|i| py_ident(i)).collect();
                format!(
                    "{}from {} import {}\n",
                    pad,
                    py_ident(module),
                    py_items.join(", ")
                )
            }
        }
        Stmt::Let(name, expr) => {
            format!("{}{} = {}\n", pad, py_ident(name), emit_py_expr(expr))
        }
        Stmt::LetUnpack(names, expr) => {
            let lhs: Vec<String> = names.iter().map(|n| py_ident(n)).collect();
            format!("{}{} = {}\n", pad, lhs.join(", "), emit_py_expr(expr))
        }
        Stmt::Print(expr) => {
            format!("{}print({})\n", pad, emit_py_expr(expr))
        }
        Stmt::Expr(expr) => match expr.unwrap_spanned() {
            Expr::Assign(name, val) => {
                format!("{}{} = {}\n", pad, py_ident(name), emit_py_expr(val))
            }
            Expr::SetProperty(target, prop, val) => {
                format!(
                    "{}{}.{} = {}\n",
                    pad,
                    emit_py_expr(target),
                    py_ident(prop),
                    emit_py_expr(val)
                )
            }
            Expr::AssignIndex(name, idx, val) => {
                format!(
                    "{}{}[{}] = {}\n",
                    pad,
                    py_ident(name),
                    emit_py_expr(idx),
                    emit_py_expr(val)
                )
            }
            Expr::AssignUnpack(names, val) => {
                let lhs: Vec<String> = names.iter().map(|n| py_ident(n)).collect();
                format!("{}{} = {}\n", pad, lhs.join(", "), emit_py_expr(val))
            }
            other => format!("{}{}\n", pad, emit_py_expr(other)),
        },
        Stmt::Block(stmts) => emit_py_body(stmts, indent, in_init),
        Stmt::If(cond, yes, no) => emit_py_if(cond, yes, no.as_deref(), indent, in_init, "if"),
        Stmt::While(cond, body) => {
            let mut s = format!("{}while {}:\n", pad, emit_py_expr(cond));
            s.push_str(&emit_py_body(body, indent + 1, in_init));
            s
        }
        Stmt::ForEach(var, iter, body) => {
            let mut s = format!(
                "{}for {} in {}:\n",
                pad,
                py_ident(var),
                emit_py_expr(iter)
            );
            s.push_str(&emit_py_body(body, indent + 1, in_init));
            s
        }
        Stmt::ForEachUnpack(vars, iter, body) => {
            let py_vars: Vec<String> = vars.iter().map(|v| py_ident(v)).collect();
            let mut s = format!(
                "{}for {} in {}:\n",
                pad,
                py_vars.join(", "),
                emit_py_expr(iter)
            );
            s.push_str(&emit_py_body(body, indent + 1, in_init));
            s
        }
        Stmt::With { expr, alias, body } => {
            let mut s = if let Some(al) = alias {
                format!(
                    "{}with {} as {}:\n",
                    pad,
                    emit_py_expr(expr),
                    py_ident(al)
                )
            } else {
                format!("{}with {}:\n", pad, emit_py_expr(expr))
            };
            s.push_str(&emit_py_body(body, indent + 1, in_init));
            s
        }
        Stmt::Function(func) => emit_py_function(func, indent, false),
        Stmt::Class(cls) => emit_py_class(cls, indent),
        Stmt::Return(Some(expr)) => {
            if in_init {
                format!("{}return\n", pad)
            } else {
                format!("{}return {}\n", pad, emit_py_expr(expr))
            }
        }
        Stmt::Return(None) => format!("{}return\n", pad),
        Stmt::Break => format!("{}break\n", pad),
        Stmt::Continue => format!("{}continue\n", pad),
        Stmt::TryCatch {
            try_body,
            catch_var,
            catch_body,
            finally_body,
        } => {
            let mut s = format!("{}try:\n", pad);
            s.push_str(&emit_py_body(try_body, indent + 1, in_init));
            if let Some(var) = catch_var {
                s.push_str(&format!("{}except Exception as {}:\n", pad, py_ident(var)));
            } else {
                s.push_str(&format!("{}except Exception:\n", pad));
            }
            s.push_str(&emit_py_body(catch_body, indent + 1, in_init));
            if let Some(fin) = finally_body {
                s.push_str(&format!("{}finally:\n", pad));
                s.push_str(&emit_py_body(fin, indent + 1, in_init));
            }
            s
        }
        Stmt::Throw(expr) => {
            format!("{}raise RuntimeError({})\n", pad, emit_py_expr(expr))
        }
        Stmt::Match {
            target,
            cases,
            default_case,
        } => {
            let mut s = format!("{}_lsn_match = {}\n", pad, emit_py_expr(target));
            if cases.is_empty() {
                if let Some(def_stmts) = default_case {
                    s.push_str(&emit_py_body(def_stmts, indent, in_init));
                }
                return s;
            }
            for (idx, (case_expr, case_body)) in cases.iter().enumerate() {
                let kw = if idx == 0 { "if" } else { "elif" };
                s.push_str(&format!(
                    "{}{} _lsn_match == {}:\n",
                    pad,
                    kw,
                    emit_py_expr(case_expr)
                ));
                s.push_str(&emit_py_body(case_body, indent + 1, in_init));
            }
            if let Some(def_stmts) = default_case {
                s.push_str(&format!("{}else:\n", pad));
                s.push_str(&emit_py_body(def_stmts, indent + 1, in_init));
            }
            s
        }
    }
}

fn emit_py_if(
    cond: &Expr,
    yes: &[Stmt],
    no: Option<&[Stmt]>,
    indent: usize,
    in_init: bool,
    kw: &str,
) -> String {
    let pad = "    ".repeat(indent);
    let mut s = format!("{}{} {}:\n", pad, kw, emit_py_expr(cond));
    s.push_str(&emit_py_body(yes, indent + 1, in_init));
    if let Some(no_body) = no {
        if no_body.len() == 1 {
            if let Stmt::If(elif_cond, elif_yes, elif_no) = no_body[0].unwrap_spanned() {
                s.push_str(&emit_py_if(
                    elif_cond,
                    elif_yes,
                    elif_no.as_deref(),
                    indent,
                    in_init,
                    "elif",
                ));
                return s;
            }
        }
        s.push_str(&format!("{}else:\n", pad));
        s.push_str(&emit_py_body(no_body, indent + 1, in_init));
    }
    s
}

fn emit_py_function(func: &Function, indent: usize, is_method: bool) -> String {
    let pad = "    ".repeat(indent);
    let is_init = is_method && (func.name == "بناء" || func.name == "تهيئة");
    let fn_name = if is_init {
        "__init__".to_string()
    } else {
        py_ident(&func.name)
    };

    let mut params = Vec::new();
    if is_method {
        params.push("self".to_string());
    }
    for (idx, param) in func.params.iter().enumerate() {
        let p_ident = py_ident(param);
        if let Some(Some(def_expr)) = func.defaults.get(idx) {
            params.push(format!("{}={}", p_ident, emit_py_expr(def_expr)));
        } else {
            params.push(p_ident);
        }
    }
    if let Some(vararg) = &func.vararg {
        params.push(format!("*{}", py_ident(vararg)));
    }

    let mut s = format!("{}def {}({}):\n", pad, fn_name, params.join(", "));
    s.push_str(&emit_py_body(&func.body, indent + 1, is_init));
    s.push('\n');
    s
}

fn emit_py_class(cls: &ClassDef, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let cname = py_ident(&cls.name);
    let mut s = if let Some(parent) = &cls.parent {
        format!("{}class {}({}):\n", pad, cname, py_ident(parent))
    } else {
        format!("{}class {}:\n", pad, cname)
    };

    if cls.methods.is_empty() {
        s.push_str(&format!("{}    pass\n\n", pad));
    } else {
        for m in &cls.methods {
            s.push_str(&emit_py_function(m, indent + 1, true));
        }
    }
    s
}

fn is_known_str_expr(expr: &Expr) -> bool {
    match expr.unwrap_spanned() {
        Expr::Text(_) | Expr::FString(_) => true,
        Expr::Call(name, _) if name == "نص" => true,
        Expr::Binary(l, Op::Add, r) => is_known_str_expr(l) || is_known_str_expr(r),
        _ => false,
    }
}

fn emit_py_expr(expr: &Expr) -> String {
    match expr {
        Expr::Spanned(inner, _, _) => emit_py_expr(inner),
        Expr::Number(n) => {
            if n.fract() == 0.0 && n.abs() < (i64::MAX as f64) {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        Expr::Text(t) => py_string_literal(t),
        Expr::Bool(true) => "True".into(),
        Expr::Bool(false) => "False".into(),
        Expr::Null => "None".into(),
        Expr::List(items) => {
            let vals: Vec<String> = items.iter().map(emit_py_expr).collect();
            format!("[{}]", vals.join(", "))
        }
        Expr::Map(pairs) => {
            let entries: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{}: {}", py_string_literal(k), emit_py_expr(v)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
        Expr::Variable(name) => match name.as_str() {
            "هذا" | "ذاتي" => "self".into(),
            "ط" | "قيمة_ط" => "math.pi".into(),
            "هـ" | "قيمة_هـ" => "math.e".into(),
            other => py_ident(other),
        },
        Expr::Assign(name, val) => {
            format!("({} := {})", py_ident(name), emit_py_expr(val))
        }
        Expr::AssignIndex(name, idx, val) => {
            format!(
                "{}.__setitem__({}, {})",
                py_ident(name),
                emit_py_expr(idx),
                emit_py_expr(val)
            )
        }
        Expr::Index(target, idx) => {
            format!("{}[{}]", emit_py_expr(target), emit_py_expr(idx))
        }
        Expr::Unary(op, inner) => {
            let v = emit_py_expr(inner);
            match op {
                Op::Negate => format!("(-{})", v),
                Op::Not => format!("(not {})", v),
                _ => v,
            }
        }
        Expr::Binary(left, op, right) => {
            let l = emit_py_expr(left);
            let r = emit_py_expr(right);
            if matches!(op, Op::Add) {
                let l_str = is_known_str_expr(left);
                let r_str = is_known_str_expr(right);
                if l_str && r_str {
                    return format!("({} + {})", l, r);
                } else if l_str {
                    return format!("({} + str({}))", l, r);
                } else if r_str {
                    return format!("(str({}) + {})", l, r);
                } else {
                    return format!("_lsn_add({}, {})", l, r);
                }
            }
            let op_str = match op {
                Op::Add => "+",
                Op::Subtract => "-",
                Op::Multiply => "*",
                Op::Divide => "/",
                Op::Modulo => "%",
                Op::Power => "**",
                Op::Equal => "==",
                Op::NotEqual => "!=",
                Op::Less => "<",
                Op::LessEqual => "<=",
                Op::Greater => ">",
                Op::GreaterEqual => ">=",
                Op::And => "and",
                Op::Or => "or",
                Op::In => "in",
                Op::NotIn => "not in",
                Op::Not | Op::Negate => "==",
            };
            format!("({} {} {})", l, op_str, r)
        }
        Expr::Call(name, args) => emit_py_call(name, args),
        Expr::GetProperty(target, prop) => {
            format!("{}.{}", emit_py_expr(target), py_ident(prop))
        }
        Expr::SetProperty(target, prop, val) => {
            format!(
                "setattr({}, {}, {})",
                emit_py_expr(target),
                py_string_literal(prop),
                emit_py_expr(val)
            )
        }
        Expr::MethodCall(target, method, args) => emit_py_method_call(target, method, args),
        Expr::Lambda(params, body) => {
            let py_params: Vec<String> = params.iter().map(|p| py_ident(p)).collect();
            if body.len() == 1 {
                if let Stmt::Return(Some(ret_expr)) = body[0].unwrap_spanned() {
                    return format!("(lambda {}: {})", py_params.join(", "), emit_py_expr(ret_expr));
                }
            }
            format!("(lambda {}: None)", py_params.join(", "))
        }
        Expr::Slice(target, start, end, step) => {
            let t = emit_py_expr(target);
            let s = start.as_ref().map(|e| emit_py_int_slice(e)).unwrap_or_default();
            let e = end.as_ref().map(|e| emit_py_int_slice(e)).unwrap_or_default();
            if let Some(st) = step {
                format!("{}[{}:{}:{}]", t, s, e, emit_py_int_slice(st))
            } else {
                format!("{}[{}:{}]", t, s, e)
            }
        }
        Expr::ListComprehension {
            expr,
            var,
            iterable,
            condition,
        } => {
            let e = emit_py_expr(expr);
            let v = py_ident(var);
            let it = emit_py_expr(iterable);
            if let Some(cond) = condition {
                format!("[{} for {} in {} if {}]", e, v, it, emit_py_expr(cond))
            } else {
                format!("[{} for {} in {}]", e, v, it)
            }
        }
        Expr::AssignUnpack(names, val) => {
            let lhs: Vec<String> = names.iter().map(|n| py_ident(n)).collect();
            format!("({} := {})", lhs.join(", "), emit_py_expr(val))
        }
        Expr::FString(segments) => emit_py_fstring(segments),
    }
}

fn emit_py_int_slice(expr: &Expr) -> String {
    match expr.unwrap_spanned() {
        Expr::Number(n) if n.fract() == 0.0 => format!("{}", *n as i64),
        other => format!("int({})", emit_py_expr(other)),
    }
}

fn emit_py_fstring(segments: &[Expr]) -> String {
    if segments.is_empty() {
        return "\"\"".into();
    }
    let parts: Vec<String> = segments
        .iter()
        .map(|seg| match seg.unwrap_spanned() {
            Expr::Text(t) => py_string_literal(t),
            other => format!("str({})", emit_py_expr(other)),
        })
        .collect();
    if parts.len() == 1 {
        parts[0].clone()
    } else {
        format!("({})", parts.join(" + "))
    }
}

fn emit_py_call(name: &str, args: &[Expr]) -> String {
    let py_args: Vec<String> = args.iter().map(emit_py_expr).collect();
    match name {
        "نص" if args.len() == 1 => format!("str({})", py_args[0]),
        "عدد" if args.len() == 1 => format!("float({})", py_args[0]),
        "عدد_صحيح" if args.len() == 1 => format!("int({})", py_args[0]),
        "طول" if args.len() == 1 => format!("len({})", py_args[0]),
        "نوع" if args.len() == 1 => format!("type({}).__name__", py_args[0]),
        "مدى" | "نطاق" if !args.is_empty() && args.len() <= 3 => {
            let int_args: Vec<String> = args.iter().map(emit_py_int_slice).collect();
            format!("range({})", int_args.join(", "))
        }
        "أضف" | "اضف" | "ألحق" | "الحق" if args.len() == 2 => {
            format!("{}.append({})", py_args[0], py_args[1])
        }
        "مطلق" if args.len() == 1 => format!("abs({})", py_args[0]),
        "جذر" if args.len() == 1 => format!("math.sqrt({})", py_args[0]),
        "جا" if args.len() == 1 => format!("math.sin({})", py_args[0]),
        "جتا" if args.len() == 1 => format!("math.cos({})", py_args[0]),
        "ظا" if args.len() == 1 => format!("math.tan({})", py_args[0]),
        "ط" | "قيمة_ط" if args.is_empty() => "math.pi".into(),
        "قرّب" | "قرب" if args.len() == 1 => format!("round({})", py_args[0]),
        "قرّب" | "قرب" if args.len() == 2 => format!("round({}, int({}))", py_args[0], py_args[1]),
        "أرضية" | "ارضية" if args.len() == 1 => format!("math.floor({})", py_args[0]),
        "سقف" if args.len() == 1 => format!("math.ceil({})", py_args[0]),
        "قوة" if args.len() == 2 => format!("pow({}, {})", py_args[0], py_args[1]),
        "أدنى" | "ادنى" if !args.is_empty() => format!("min({})", py_args.join(", ")),
        "أعلى" | "اعلى" if !args.is_empty() => format!("max({})", py_args.join(", ")),
        "مجموع" if args.len() == 1 => format!("sum({})", py_args[0]),
        "متوسط" if args.len() == 1 => format!("(sum({}) / len({}))", py_args[0], py_args[0]),
        "فرز" if args.len() == 1 => format!("sorted({})", py_args[0]),
        "عكس" | "اعكس" if args.len() == 1 => format!("list(reversed({}))", py_args[0]),
        "انتظر_ملي" | "تأخير" | "تاخير" if args.len() == 1 => {
            format!("_lsn_sleep_ms({})", py_args[0])
        }
        "انتظر_ميكرو" if args.len() == 1 => {
            format!("_lsn_sleep_us({})", py_args[0])
        }
        "انتظر_ثواني" if args.len() == 1 => {
            format!("_lsn_sleep_ms(({}) * 1000)", py_args[0])
        }
        "وقت_التشغيل" if args.is_empty() => "_lsn_ticks_ms()".into(),
        "طابع_زمني" | "الآن" | "الان" if args.is_empty() => "time.time()".into(),
        "خريطة_قيمة" if args.len() == 5 => format!(
            "((({} - {}) * ({} - {}) / ({} - {})) + {})",
            py_args[0], py_args[1], py_args[4], py_args[3], py_args[2], py_args[1], py_args[3]
        ),
        "قيد_قيمة" if args.len() == 3 => {
            format!("max({}, min({}, {}))", py_args[1], py_args[2], py_args[0])
        }
        other => format!("{}({})", py_ident(other), py_args.join(", ")),
    }
}

fn emit_py_method_call(target: &Expr, method: &str, args: &[Expr]) -> String {
    let t = emit_py_expr(target);
    let py_args: Vec<String> = args.iter().map(emit_py_expr).collect();
    match method {
        "أضف" | "اضف" | "ألحق" | "الحق" if args.len() == 1 => {
            format!("{}.append({})", t, py_args[0])
        }
        "طول" if args.is_empty() => format!("len({})", t),
        "مسح" if args.is_empty() => format!("{}.clear()", t),
        "نسخ" if args.is_empty() => format!("{}.copy()", t),
        "مجموع" if args.is_empty() => format!("sum({})", t),
        "تحتوي" | "يحتوي" if args.len() == 1 => format!("({} in {})", py_args[0], t),
        "تقسيم" if args.len() == 1 => format!("{}.split({})", t, py_args[0]),
        "أحرف_كبيرة" | "احرف_كبيرة" if args.is_empty() => format!("{}.upper()", t),
        "أحرف_صغيرة" | "احرف_صغيرة" if args.is_empty() => format!("{}.lower()", t),
        "استبدل" | "أعد" if args.len() == 2 => {
            format!("{}.replace({}, {})", t, py_args[0], py_args[1])
        }
        other => format!("{}.{}({})", t, py_ident(other), py_args.join(", ")),
    }
}

fn py_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn py_ident(name: &str) -> String {
    if is_python_keyword(name) {
        return format!("{}_", name);
    }
    let mut out = String::with_capacity(name.len());
    for (idx, c) in name.chars().enumerate() {
        let valid = if idx == 0 {
            c == '_' || c.is_alphabetic()
        } else {
            c == '_'
                || c.is_alphanumeric()
                || ('\u{064B}'..='\u{065F}').contains(&c)
                || c == '\u{0670}'
                || c == '\u{0640}'
        };
        if valid {
            out.push(c);
        } else {
            if idx == 0 {
                out.push('_');
            }
            out.push_str(&format!("_u{:x}_", c as u32));
        }
    }
    if out.is_empty() {
        "_lsn_id".into()
    } else {
        out
    }
}

fn is_python_keyword(name: &str) -> bool {
    matches!(
        name,
        "False"
            | "None"
            | "True"
            | "and"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "del"
            | "elif"
            | "else"
            | "except"
            | "finally"
            | "for"
            | "from"
            | "global"
            | "if"
            | "import"
            | "in"
            | "is"
            | "lambda"
            | "nonlocal"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_pico_parses_and_simulates_all_classes() {
        let user_code = r##"
دع لوحة = جديد لوحة_بيكو("Raspberry Pi Pico W")
تحقق_يساوي(لوحة.اسم_الطراز()، "Raspberry Pi Pico W")
تحقق_يساوي(لوحة.تردد_المعالج_ميجا()، 125)
تحقق(لوحة.حرارة_الشريحة() > 0)
تحقق(لوحة.ذاكرة_حرة_كيلوبايت() > 100)
تحقق_يساوي(لوحة.مصباح_اللوحة(صحيح)، صحيح)
تحقق_يساوي(لوحة.مصباح_اللوحة(خطأ)، خطأ)

دع منفذ = جديد منفذ_بيكو(15، "مخرج")
منفذ.شغّل()
تحقق_يساوي(منفذ.اقرأ()، 1)
منفذ.اقلب()
تحقق_يساوي(منفذ.اقرأ()، 0)
منفذ.اكتب(1)
تحقق_يساوي(منفذ.اقرأ()، 1)
منفذ.أوقف()
تحقق_يساوي(منفذ.اقرأ()، 0)

دع زر = جديد منفذ_بيكو(14، "مدخل"، "علوي")
تحقق_يساوي(زر.اقرأ()، 1)
زر.ضبط_محاكاة(0)
تحقق_يساوي(زر.اقرأ()، 0)

دع نبضة = جديد نبضة_بيكو(16، 1000)
تحقق_يساوي(نبضة.اضبط_تردد(5000)، 5000)
تحقق_يساوي(نبضة.اضبط_دورة_16bit(32768)، 32768)
تحقق_يساوي(نبضة.اضبط_نسبة(50)، 50)
تحقق(نبضة.أوقف())

دع محول = جديد محول_تماثلي_بيكو(26)
محول.ضبط_محاكاة(65535)
تحقق_يساوي(محول.اقرأ_16bit()، 65535)
تحقق_يساوي(محول.اقرأ_جهد()، 3.3)

دع ناقل_i2c = جديد ناقل_i2c_بيكو(0، 0، 1، 400000)
تحقق(ناقل_i2c.فحص_الأجهزة().طول() >= 2)
تحقق(ناقل_i2c.اكتب_بايت(60، 1، 200))
تحقق_يساوي(ناقل_i2c.اقرأ_بايت(60، 1)، 200)

دع ناقل_spi = جديد ناقل_spi_بيكو(0، 1000000)
دع رد_spi = ناقل_spi.نقل([10، 20، 30])
تحقق_يساوي(رد_spi.طول()، 3)
تحقق_يساوي(رد_spi[1]، 20)

دع تسلسلي = جديد منفذ_uart_بيكو(0، 115200، 0، 1)
تحقق(تسلسلي.اكتب("PING"))
تسلسلي.ضبط_محاكاة("PONG")
تحقق_يساوي(تسلسلي.اقرأ_سطر()، "PONG")

دع شبكة = جديد واي_فاي_بيكو_w()
تحقق_يساوي(شبكة.هل_متصل()، خطأ)
تحقق(شبكة.اتصل("Lisan_Lab"، "secret123"))
تحقق_يساوي(شبكة.هل_متصل()، صحيح)
تحقق_يساوي(شبكة.عنوان_ip()، "192.168.4.120")
تحقق(شبكة.قطع_الاتصال())
تحقق_يساوي(شبكة.هل_متصل()، خطأ)

دع شريط = جديد شريط_نيوبكسل_بيكو(28، 8)
تحقق(شريط.اضبط_مصباح(0، 255، 128، 0))
تحقق(شريط.املأ(0، 255، 64))
تحقق(شريط.اعرض())

دع مؤقت = جديد مؤقت_بيكو(500)
تحقق(مؤقت.ابدأ())
تحقق_يساوي(مؤقت.نبضة_محاكاة()، 1)
تحقق_يساوي(مؤقت.نبضة_محاكاة()، 2)
تحقق_يساوي(مؤقت.عدد_النبضات()، 2)
تحقق(مؤقت.أوقف())

دع وحدة_pio = جديد وحدة_pio_بيكو(0، 2000000، 2)
تحقق(وحدة_pio.فعّل())
تحقق(وحدة_pio.أرسل_كلمة(16711935))
تحقق(وحدة_pio.عطّل())

دع حرارة_داخلية = جديد مستشعر_حرارة_داخلي_بيكو()
حرارة_داخلية.ضبط_محاكاة(25.0)
تحقق_يساوي(حرارة_داخلية.اقرأ_مئوية()، 25.0)
تحقق_يساوي(حرارة_داخلية.اقرأ_فهرنهايت()، 77.0)
"##;
        let combined = format!("{}\n{}", STDLIB_PICO, user_code);
        let result = crate::Interpreter::default().run_source(&combined);
        assert!(
            result.is_ok(),
            "فشل تنفيذ مكتبة بيكو القياسية: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_transpile_pico_micropython_output() {
        let code = r##"
استورد "بيكو"
دع لوحة = جديد لوحة_بيكو("Raspberry Pi Pico W")
اطبع("اللوحة: " + لوحة.اسم_الطراز())
لوحة.مصباح_اللوحة(صحيح)
دع مصباح = جديد منفذ_بيكو(15، "مخرج")
لكل دورة في مدى(3) {
    إذا (دورة % 2 == 0) {
        مصباح.شغّل()
    } وإلا {
        مصباح.أوقف()
    }
}
دع شبكة = جديد واي_فاي_بيكو_w()
شبكة.اتصل("Lisan_Pico_W"، "pass1234")
دع شريط = جديد شريط_نيوبكسل_بيكو(28، 4)
شريط.املأ(0، 255، 0)
شريط.اعرض()
دع وحدة = جديد وحدة_pio_بيكو(0، 2000000، 2)
وحدة.فعّل()
وحدة.أرسل_كلمة(255)
"##;
        let py = transpile_pico_micropython(code)
            .expect("يجب أن تنجح ترجمة بيكو إلى MicroPython");
        assert!(py.starts_with("#!/usr/bin/env python3\n# -*- coding: utf-8 -*-\n"));
        assert!(py.contains("import machine"));
        assert!(py.contains("import network"));
        assert!(py.contains("import neopixel"));
        assert!(py.contains("import rp2"));
        assert!(py.contains("except ImportError:"));
        assert!(py.contains("لوحة = لوحة_بيكو(\"Raspberry Pi Pico W\")"));
        assert!(py.contains("for دورة in range(3):"));
    }

    #[test]
    fn test_build_pico_file_creates_py() {
        let tmp_dir = std::env::temp_dir().join(format!("lisan_pico_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp_dir);
        let in_file = tmp_dir.join("blink.lsn");
        std::fs::write(
            &in_file,
            "استورد \"بيكو\"\nدع م = جديد منفذ_بيكو(15)\nم.شغّل()\nم.أوقف()\n",
        )
        .unwrap();

        let out_file = build_pico_file(&in_file, None).expect("يجب إنشاء ملف MicroPython بيكو");
        assert_eq!(
            out_file.file_name().unwrap().to_str().unwrap(),
            "blink_pico.py"
        );
        assert!(out_file.exists());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&out_file).unwrap().permissions().mode();
            assert_eq!(mode & 0o111, 0o111);
        }

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}

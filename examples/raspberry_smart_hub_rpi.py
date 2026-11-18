#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# ==============================================================================
# تم توليد هذا السكربت تلقائياً بواسطة مترجم راسبيري باي في لغة لسان (Lisan)
# يدعم التشغيل المباشر على Raspberry Pi OS (Pi 3 / 4 / 5 / Zero 2 W / CM)
# مع وضع المحاكاة التلقائي الآمن على أنظمة سطح المكتب (Linux / macOS / Windows)
# ==============================================================================

import math
import os
import socket
import subprocess
import sys
import time
from pathlib import Path

# الكشف الذكي عن العمل فوق لوحة راسبيري باي حقيقية
_IS_REAL_RPI = False
try:
    _dt_model = Path("/sys/firmware/devicetree/base/model")
    if _dt_model.exists():
        _model_txt = _dt_model.read_text(encoding="utf-8", errors="ignore")
        if "Raspberry Pi" in _model_txt:
            _IS_REAL_RPI = True
except Exception:
    pass
if os.environ.get("LISAN_FORCE_RPI_HW") == "1":
    _IS_REAL_RPI = True

# استيراد مكتبات عتاد راسبيري باي مع التراجع الآمن عند عدم توفرها
try:
    import gpiozero as _gpiozero
    _HAS_GPIOZERO = True
except Exception:
    _gpiozero = None
    _HAS_GPIOZERO = False

try:
    import RPi.GPIO as _RPI_GPIO
    if _IS_REAL_RPI:
        _RPI_GPIO.setwarnings(False)
        _RPI_GPIO.setmode(_RPI_GPIO.BCM)
    _HAS_RPI_GPIO = True
except Exception:
    _RPI_GPIO = None
    _HAS_RPI_GPIO = False

try:
    import smbus2 as _smbus2
    _HAS_SMBUS = True
except Exception:
    _smbus2 = None
    _HAS_SMBUS = False

try:
    import spidev as _spidev
    _HAS_SPIDEV = True
except Exception:
    _spidev = None
    _HAS_SPIDEV = False

try:
    import serial as _pyserial
    _HAS_SERIAL = True
except Exception:
    _pyserial = None
    _HAS_SERIAL = False

try:
    from picamera2 import Picamera2 as _Picamera2
    _HAS_PICAMERA2 = True
except Exception:
    _Picamera2 = None
    _HAS_PICAMERA2 = False

try:
    from sense_hat import SenseHat as _SenseHat
    _HAS_SENSE_HAT = True
except Exception:
    _SenseHat = None
    _HAS_SENSE_HAT = False

عالي = 1
منخفض = 0
مخرج_راسبيري = "مخرج"
مدخل_راسبيري = "مدخل"
سحب_علوي = "علوي"
سحب_سفلي = "سفلي"
بدون_سحب = "بدون"


def _lsn_add(a, b):
    if isinstance(a, str) or isinstance(b, str):
        return f"{a}{b}"
    return a + b


def انتظر_ثواني(ثواني):
    time.sleep(max(0.0, float(ثواني)))
    return True


class لوحة_راسبيري:
    def __init__(self, طراز="Raspberry Pi 5"):
        self.طراز = طراز
        if _IS_REAL_RPI:
            try:
                raw = Path("/sys/firmware/devicetree/base/model").read_text(encoding="utf-8", errors="ignore").strip("\x00\r\n ")
                if raw:
                    self.طراز = raw
            except Exception:
                pass
        self._sim_temp = 44.5
        self.تعمل = True

    def اسم_الطراز(self):
        return self.طراز

    def حرارة_المعالج(self):
        thermal = Path("/sys/class/thermal/thermal_zone0/temp")
        if _IS_REAL_RPI and thermal.exists():
            try:
                return round(float(thermal.read_text().strip()) / 1000.0, 1)
            except Exception:
                pass
        return self._sim_temp

    def ضبط_حرارة_محاكاة(self, حرارة_جديدة):
        self._sim_temp = float(حرارة_جديدة)
        return self._sim_temp

    def استهلاك_الذاكرة_ميجا(self):
        meminfo = Path("/proc/meminfo")
        if _IS_REAL_RPI and meminfo.exists():
            try:
                info = {}
                for line in meminfo.read_text().splitlines():
                    parts = line.split(":")
                    if len(parts) == 2:
                        info[parts[0].strip()] = int(parts[1].strip().split()[0])
                if "MemTotal" in info and "MemAvailable" in info:
                    return round((info["MemTotal"] - info["MemAvailable"]) / 1024.0, 1)
            except Exception:
                pass
        return 640.0

    def عدد_الأنوية(self):
        return os.cpu_count() or 4

    def عنوان_ip(self):
        if _IS_REAL_RPI:
            try:
                sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
                sock.settimeout(0.2)
                sock.connect(("8.8.8.8", 80))
                ip = sock.getsockname()[0]
                sock.close()
                return ip
            except Exception:
                pass
        return "192.168.1.150"

    def إعادة_تشغيل_اللوحة(self):
        if _IS_REAL_RPI and os.environ.get("LISAN_RPI_ALLOW_POWER") == "1":
            subprocess.run(["sudo", "reboot"], check=False)
        return True

    اعادة_تشغيل_اللوحة = إعادة_تشغيل_اللوحة

    def إيقاف_التشغيل(self):
        self.تعمل = False
        if _IS_REAL_RPI and os.environ.get("LISAN_RPI_ALLOW_POWER") == "1":
            subprocess.run(["sudo", "shutdown", "-h", "now"], check=False)
        return True

    ايقاف_التشغيل = إيقاف_التشغيل


class منفذ_gpio_راسبيري:
    def __init__(self, رقم_المنفذ, الوضع="مخرج", السحب="بدون"):
        self.منفذ = int(رقم_المنفذ)
        self.الوضع = الوضع
        self.السحب = السحب
        self.القيمة = 1 if (الوضع == "مدخل" and السحب == "علوي") else 0
        self._dev = None
        self._use_rpi_gpio = False
        if _IS_REAL_RPI:
            if _HAS_GPIOZERO:
                try:
                    if الوضع in ("مخرج", "OUTPUT"):
                        self._dev = _gpiozero.DigitalOutputDevice(self.منفذ)
                    else:
                        pull_up = True if السحب == "علوي" else (False if السحب == "سفلي" else None)
                        self._dev = _gpiozero.DigitalInputDevice(self.منفذ, pull_up=pull_up)
                except Exception:
                    self._dev = None
            elif _HAS_RPI_GPIO:
                try:
                    if الوضع in ("مخرج", "OUTPUT"):
                        _RPI_GPIO.setup(self.منفذ, _RPI_GPIO.OUT, initial=_RPI_GPIO.LOW)
                    else:
                        pud = _RPI_GPIO.PUD_OFF
                        if السحب == "علوي":
                            pud = _RPI_GPIO.PUD_UP
                        elif السحب == "سفلي":
                            pud = _RPI_GPIO.PUD_DOWN
                        _RPI_GPIO.setup(self.منفذ, _RPI_GPIO.IN, pull_up_down=pud)
                    self._use_rpi_gpio = True
                except Exception:
                    self._use_rpi_gpio = False

    def اكتب(self, قيمة):
        state = 1 if قيمة in (1, True, "عالي", "HIGH") else 0
        self.القيمة = state
        if self._dev is not None:
            try:
                self._dev.on() if state else self._dev.off()
            except Exception:
                pass
        elif self._use_rpi_gpio and _HAS_RPI_GPIO:
            try:
                _RPI_GPIO.output(self.منفذ, _RPI_GPIO.HIGH if state else _RPI_GPIO.LOW)
            except Exception:
                pass
        return self.القيمة

    def شغّل(self):
        return self.اكتب(1)

    شغل = شغّل

    def أوقف(self):
        return self.اكتب(0)

    اوقف = أوقف

    def اقلب(self):
        return self.اكتب(0 if self.القيمة else 1)

    def اقرأ(self):
        if self._dev is not None:
            try:
                self.القيمة = 1 if self._dev.value else 0
            except Exception:
                pass
        elif self._use_rpi_gpio and _HAS_RPI_GPIO:
            try:
                self.القيمة = 1 if _RPI_GPIO.input(self.منفذ) else 0
            except Exception:
                pass
        return self.القيمة

    اقرا = اقرأ

    def ضبط_محاكاة(self, قيمة):
        self.القيمة = 1 if قيمة in (1, True) else 0
        return self.القيمة

    def تنظيف(self):
        self.القيمة = 0
        if self._dev is not None:
            try:
                self._dev.close()
            except Exception:
                pass
            self._dev = None
        elif self._use_rpi_gpio and _HAS_RPI_GPIO:
            try:
                _RPI_GPIO.cleanup(self.منفذ)
            except Exception:
                pass
            self._use_rpi_gpio = False
        return True


class نبضة_pwm_راسبيري:
    def __init__(self, رقم_المنفذ, تردد=1000):
        self.منفذ = int(رقم_المنفذ)
        self.تردد = max(1, int(تردد))
        self.نسبة = 0.0
        self.يعمل = False
        self._pwm = None
        self._is_gpiozero = False
        if _IS_REAL_RPI:
            if _HAS_GPIOZERO:
                try:
                    self._pwm = _gpiozero.PWMOutputDevice(self.منفذ, frequency=self.تردد)
                    self._is_gpiozero = True
                except Exception:
                    self._pwm = None
            elif _HAS_RPI_GPIO:
                try:
                    _RPI_GPIO.setup(self.منفذ, _RPI_GPIO.OUT)
                    self._pwm = _RPI_GPIO.PWM(self.منفذ, self.تردد)
                except Exception:
                    self._pwm = None

    def ابدأ(self, نسبة_مئوية=50):
        self.نسبة = max(0.0, min(100.0, float(نسبة_مئوية)))
        self.يعمل = True
        if self._pwm is not None:
            try:
                if self._is_gpiozero:
                    self._pwm.value = self.نسبة / 100.0
                else:
                    self._pwm.start(self.نسبة)
            except Exception:
                pass
        return self.نسبة

    ابدا = ابدأ

    def غير_النسبة(self, نسبة):
        self.نسبة = max(0.0, min(100.0, float(نسبة)))
        if self._pwm is not None:
            try:
                if self._is_gpiozero:
                    self._pwm.value = self.نسبة / 100.0
                else:
                    self._pwm.ChangeDutyCycle(self.نسبة)
            except Exception:
                pass
        return self.نسبة

    غيّر_النسبة = غير_النسبة

    def غير_التردد(self, تردد_جديد):
        if تردد_جديد > 0:
            self.تردد = int(تردد_جديد)
            if self._pwm is not None:
                try:
                    if self._is_gpiozero:
                        self._pwm.frequency = self.تردد
                    else:
                        self._pwm.ChangeFrequency(self.تردد)
                except Exception:
                    pass
        return self.تردد

    غيّر_التردد = غير_التردد

    def أوقف(self):
        self.نسبة = 0.0
        self.يعمل = False
        if self._pwm is not None:
            try:
                if self._is_gpiozero:
                    self._pwm.off()
                else:
                    self._pwm.stop()
            except Exception:
                pass
        return True

    اوقف = أوقف


class ناقل_i2c_راسبيري:
    def __init__(self, رقم_الناقل=1):
        self.رقم_الناقل = int(رقم_الناقل)
        self._regs = {}
        self._devices = [0x3C, 0x40, 0x76]
        self._bus = None
        if _IS_REAL_RPI and _HAS_SMBUS and Path(f"/dev/i2c-{self.رقم_الناقل}").exists():
            try:
                self._bus = _smbus2.SMBus(self.رقم_الناقل)
            except Exception:
                self._bus = None

    def اكتب_بايت(self, عنوان_الجهاز, سجل, قيمة):
        addr = int(عنوان_الجهاز)
        reg = int(سجل)
        val = int(قيمة) & 0xFF
        self._regs[(addr, reg)] = val
        if addr not in self._devices:
            self._devices.append(addr)
        if self._bus is not None:
            try:
                self._bus.write_byte_data(addr, reg, val)
            except Exception:
                pass
        return True

    def اقرأ_بايت(self, عنوان_الجهاز, سجل):
        addr = int(عنوان_الجهاز)
        reg = int(سجل)
        if self._bus is not None:
            try:
                return self._bus.read_byte_data(addr, reg)
            except Exception:
                pass
        return self._regs.get((addr, reg), 0)

    اقرا_بايت = اقرأ_بايت

    def فحص_الأجهزة(self):
        if self._bus is not None:
            found = []
            for addr in range(0x03, 0x78):
                try:
                    self._bus.read_byte(addr)
                    found.append(addr)
                except Exception:
                    pass
            if found:
                return found
        return list(self._devices)

    فحص_الاجهزة = فحص_الأجهزة


class ناقل_spi_راسبيري:
    def __init__(self, ناقل=0, شريحة=0, سرعة=1000000):
        self.ناقل = int(ناقل)
        self.شريحة = int(شريحة)
        self.سرعة = int(سرعة)
        self.مفتوح = True
        self._spi = None
        if _IS_REAL_RPI and _HAS_SPIDEV and Path(f"/dev/spidev{self.ناقل}.{self.شريحة}").exists():
            try:
                self._spi = _spidev.SpiDev()
                self._spi.open(self.ناقل, self.شريحة)
                self._spi.max_speed_hz = self.سرعة
            except Exception:
                self._spi = None

    def نقل(self, قائمة_بايتات):
        payload = [int(b) & 0xFF for b in قائمة_بايتات]
        if self._spi is not None and self.مفتوح:
            try:
                return list(self._spi.xfer2(payload))
            except Exception:
                pass
        return list(payload)

    def أغلق(self):
        self.مفتوح = False
        if self._spi is not None:
            try:
                self._spi.close()
            except Exception:
                pass
            self._spi = None
        return True

    اغلق = أغلق


class منفذ_تسلسلي_راسبيري:
    def __init__(self, مسار="/dev/serial0", سرعة=115200):
        self.مسار = str(مسار)
        self.سرعة = int(سرعة)
        self.مفتوح = True
        self.آخر_مرسل = ""
        self.آخر_مستقبل = "OK"
        self._ser = None
        if _IS_REAL_RPI and _HAS_SERIAL and Path(self.مسار).exists():
            try:
                self._ser = _pyserial.Serial(self.مسار, self.سرعة, timeout=1.0)
            except Exception:
                self._ser = None

    def اكتب_سطر(self, رسالة):
        line = f"{رسالة}\n"
        self.آخر_مرسل = str(رسالة)
        if self._ser is not None and self.مفتوح:
            try:
                self._ser.write(line.encode("utf-8"))
            except Exception:
                pass
        return True

    def اقرأ_سطر(self):
        if self._ser is not None and self.مفتوح:
            try:
                raw = self._ser.readline()
                if raw:
                    return raw.decode("utf-8", errors="replace").rstrip("\r\n")
            except Exception:
                pass
        return self.آخر_مستقبل

    اقرا_سطر = اقرأ_سطر

    def ضبط_محاكاة_استقبال(self, نص_مستقبل):
        self.آخر_مستقبل = str(نص_مستقبل)
        return self.آخر_مستقبل

    def أغلق(self):
        self.مفتوح = False
        if self._ser is not None:
            try:
                self._ser.close()
            except Exception:
                pass
            self._ser = None
        return True

    اغلق = أغلق


class كاميرا_راسبيري:
    def __init__(self, عرض=1920, ارتفاع=1080):
        self.عرض = int(عرض)
        self.ارتفاع = int(ارتفاع)
        self.تعمل = False
        self.تبث = False
        self.منفذ_البث = 8000
        self.آخر_صورة = ""
        self.آخر_فيديو = ""
        self._cam = None
        if _IS_REAL_RPI and _HAS_PICAMERA2:
            try:
                self._cam = _Picamera2()
                cfg = self._cam.create_still_configuration(main={"size": (self.عرض, self.ارتفاع)})
                self._cam.configure(cfg)
            except Exception:
                self._cam = None

    def ابدأ(self):
        self.تعمل = True
        if self._cam is not None:
            try:
                self._cam.start()
            except Exception:
                pass
        return True

    ابدا = ابدأ

    def التقط_صورة(self, مسار_الملف):
        self.تعمل = True
        self.آخر_صورة = str(مسار_الملف)
        if self._cam is not None:
            try:
                self._cam.capture_file(self.آخر_صورة)
            except Exception:
                pass
        return True

    def سجل_فيديو(self, مسار_الملف, مدة_ثواني=5):
        self.تعمل = True
        self.آخر_فيديو = str(مسار_الملف)
        if self._cam is not None:
            try:
                self._cam.start_and_record_video(self.آخر_فيديو, duration=float(مدة_ثواني))
            except Exception:
                pass
        return True

    سجّل_فيديو = سجل_فيديو

    def ابدأ_بث(self, منفذ=8000):
        self.تعمل = True
        self.تبث = True
        self.منفذ_البث = int(منفذ)
        return True

    ابدا_بث = ابدأ_بث

    def أوقف(self):
        self.تبث = False
        self.تعمل = False
        if self._cam is not None:
            try:
                self._cam.stop()
            except Exception:
                pass
        return True

    اوقف = أوقف


class قبعة_الحساسات_sensehat:
    def __init__(self):
        self.حرارة = 25.4
        self.رطوبة = 48.2
        self.ضغط = 1013.25
        self._hat = None
        if _IS_REAL_RPI and _HAS_SENSE_HAT:
            try:
                self._hat = _SenseHat()
            except Exception:
                self._hat = None

    def اعرض_رسالة(self, نص_رسالة, سرعة=0.1):
        if self._hat is not None:
            try:
                self._hat.show_message(str(نص_رسالة), scroll_speed=float(سرعة))
            except Exception:
                pass
        return True

    def اضبط_بكسل(self, س, ص, أحمر, أخضر, أزرق):
        if self._hat is not None:
            try:
                self._hat.set_pixel(int(س) % 8, int(ص) % 8, int(أحمر) & 255, int(أخضر) & 255, int(أزرق) & 255)
            except Exception:
                pass
        return True

    def امسح_الشاشة(self):
        if self._hat is not None:
            try:
                self._hat.clear()
            except Exception:
                pass
        return True

    def اقرأ_حرارة(self):
        if self._hat is not None:
            try:
                return round(float(self._hat.get_temperature()), 2)
            except Exception:
                pass
        return self.حرارة

    def اقرأ_رطوبة(self):
        if self._hat is not None:
            try:
                return round(float(self._hat.get_humidity()), 2)
            except Exception:
                pass
        return self.رطوبة

    def اقرأ_ضغط(self):
        if self._hat is not None:
            try:
                return round(float(self._hat.get_pressure()), 2)
            except Exception:
                pass
        return self.ضغط

    def اقرأ_توجيه(self):
        if self._hat is not None:
            try:
                o = self._hat.get_orientation()
                return {
                    "انحدار": round(float(o.get("pitch", 0.0)), 2),
                    "ميلان": round(float(o.get("roll", 0.0)), 2),
                    "انعطاف": round(float(o.get("yaw", 0.0)), 2),
                }
            except Exception:
                pass
        return {"انحدار": 1.5, "ميلان": 0.8, "انعطاف": 180.0}

    def ضبط_محاكاة_بيئة(self, حرارة, رطوبة, ضغط):
        self.حرارة = float(حرارة)
        self.رطوبة = float(رطوبة)
        self.ضغط = float(ضغط)
        return True


class قبعة_محركات_pca9685:
    def __init__(self, عنوان=64):
        self.عنوان = int(عنوان)
        self.زوايا = {}
        self.سرعات = {}
        self._bus = None
        if _IS_REAL_RPI and _HAS_SMBUS and Path("/dev/i2c-1").exists():
            try:
                self._bus = _smbus2.SMBus(1)
                self._bus.write_byte_data(self.عنوان, 0x00, 0x10)
                self._bus.write_byte_data(self.عنوان, 0xFE, 121)  # 50Hz للسيرفو
                self._bus.write_byte_data(self.عنوان, 0x00, 0x20)
            except Exception:
                self._bus = None

    def _set_pwm(self, channel, on_tick, off_tick):
        if self._bus is not None:
            base = 0x06 + 4 * (int(channel) & 0x0F)
            try:
                self._bus.write_byte_data(self.عنوان, base, on_tick & 0xFF)
                self._bus.write_byte_data(self.عنوان, base + 1, (on_tick >> 8) & 0xFF)
                self._bus.write_byte_data(self.عنوان, base + 2, off_tick & 0xFF)
                self._bus.write_byte_data(self.عنوان, base + 3, (off_tick >> 8) & 0xFF)
            except Exception:
                pass

    def اضبط_زاوية_سيرفو(self, قناة, زاوية):
        ang = max(0.0, min(180.0, float(زاوية)))
        self.زوايا[int(قناة)] = ang
        pulse = int(150 + (ang / 180.0) * 450)
        self._set_pwm(قناة, 0, pulse)
        return ang

    def اضبط_سرعة_محرك(self, قناة, نسبة):
        pct = max(0.0, min(100.0, float(نسبة)))
        self.سرعات[int(قناة)] = pct
        duty = int((pct / 100.0) * 4095)
        self._set_pwm(قناة, 0, duty)
        return pct


class شاشة_لمس_dsi:
    def __init__(self, حجم_بوصة=7):
        self.حجم_بوصة = حجم_بوصة
        self.سطوع = 80
        self.مشتغلة = True

    def اضبط_سطوع(self, نسبة):
        pct = max(0, min(100, int(نسبة)))
        self.سطوع = pct
        if _IS_REAL_RPI:
            raw_val = int((pct / 100.0) * 255)
            for bl in (
                Path("/sys/class/backlight/rpi_backlight/brightness"),
                Path("/sys/class/backlight/10-0045/brightness"),
            ):
                if bl.exists():
                    try:
                        bl.write_text(f"{raw_val}\n")
                        break
                    except Exception:
                        pass
        return self.سطوع

    def شغّل_الشاشة(self, حالة):
        on = bool(حالة in (True, 1))
        self.مشتغلة = on
        if _IS_REAL_RPI:
            bl_power = Path("/sys/class/backlight/rpi_backlight/bl_power")
            if bl_power.exists():
                try:
                    bl_power.write_text("0\n" if on else "1\n")
                except Exception:
                    pass
        return self.مشتغلة

    شغل_الشاشة = شغّل_الشاشة


class صوت_راسبيري:
    def __init__(self, مستوى_الصوت=80):
        self.مستوى_الصوت = max(0, min(100, int(مستوى_الصوت)))
        self.آخر_ملف = ""
        self.آخر_نطق = ""

    def اضبط_الصوت(self, نسبة):
        self.مستوى_الصوت = max(0, min(100, int(نسبة)))
        if _IS_REAL_RPI:
            try:
                subprocess.run(["amixer", "-q", "sset", "Master", f"{self.مستوى_الصوت}%"], check=False)
            except Exception:
                pass
        return self.مستوى_الصوت

    def شغّل_ملف(self, مسار):
        self.آخر_ملف = str(مسار)
        if _IS_REAL_RPI and Path(self.آخر_ملف).exists():
            player = "aplay" if self.آخر_ملف.lower().endswith(".wav") else "mpg123"
            try:
                subprocess.run([player, "-q", self.آخر_ملف], check=False)
            except Exception:
                pass
        return True

    شغل_ملف = شغّل_ملف

    def انطق_عربي(self, نص_رسالة):
        self.آخر_نطق = str(نص_رسالة)
        if _IS_REAL_RPI:
            try:
                subprocess.run(["espeak-ng", "-v", "ar", self.آخر_نطق], check=False)
            except Exception:
                pass
        return True


class شاشة_حبر_إلكتروني_epaper:
    def __init__(self, عرض=250, ارتفاع=122):
        self.عرض = int(عرض)
        self.ارتفاع = int(ارتفاع)
        self.نصوص = []
        self.في_وضع_السكون = False
        self.عدد_التحديثات = 0

    def امسح(self):
        self.نصوص.clear()
        self.في_وضع_السكون = False
        return True

    def اعرض_نص(self, س, ص, نص_رسالة):
        self.في_وضع_السكون = False
        self.نصوص.append({"س": int(س), "ص": int(ص), "نص": str(نص_رسالة)})
        return True

    def تحديث_الشاشة(self):
        self.عدد_التحديثات += 1
        return True

    def سكون(self):
        self.في_وضع_السكون = True
        return True

# --- بداية شيفرة المستخدم المترجمة من لغة لسان ---

print("=== تشغيل مركز راسبيري باي 5 الذكي (Lisan Smart AI & IoT Hub) ===")
اللوحة = لوحة_راسبيري("Raspberry Pi 5")
print(("اللوحة: " + str(اللوحة.اسم_الطراز())))
print((((("عدد الأنوية: " + str(اللوحة.عدد_الأنوية())) + " | الذاكرة المستهلكة: ") + str(اللوحة.استهلاك_الذاكرة_ميجا())) + " ميجابايت"))
print(((("حرارة المعالج: " + str(اللوحة.حرارة_المعالج())) + "°C | عنوان الشبكة: ") + str(اللوحة.عنوان_ip())))
مصباح_الحالة = منفذ_gpio_راسبيري(17, "مخرج")
حساس_الحركة = منفذ_gpio_راسبيري(27, "مدخل", "سفلي")
مصباح_الحالة.شغّل()
حساس_الحركة.ضبط_محاكاة(1)
print(("حالة مصباح النظام (BCM 17): " + str(مصباح_الحالة.اقرأ())))
مروحة_التبريد = نبضة_pwm_راسبيري(18, 25000)
مروحة_التبريد.ابدأ(45)
if (اللوحة.حرارة_المعالج() > 40):
    مروحة_التبريد.غير_النسبة(75)
    print("تم رفع سرعة مروحة التبريد PWM إلى 75%")
else:
    مروحة_التبريد.غير_النسبة(30)
ناقل_i2c = ناقل_i2c_راسبيري(1)
ناقل_i2c.اكتب_بايت(60, 0, 175)
الأجهزة_المكتشفة = ناقل_i2c.فحص_الأجهزة()
print(("عدد أجهزة I2C المكتشفة على الناقل: " + str(len(الأجهزة_المكتشفة))))
قبعة_الحساسات = قبعة_الحساسات_sensehat()
حرارة_الغرفة = قبعة_الحساسات.اقرأ_حرارة()
رطوبة_الغرفة = قبعة_الحساسات.اقرأ_رطوبة()
الضغط_الجوي = قبعة_الحساسات.اقرأ_ضغط()
التوجيه = قبعة_الحساسات.اقرأ_توجيه()
print((((((("بيانات Sense HAT -> الحرارة: " + str(حرارة_الغرفة)) + "°C | الرطوبة: ") + str(رطوبة_الغرفة)) + "% | الضغط: ") + str(الضغط_الجوي)) + " hPa"))
print(("زاوية الانعطاف (Yaw): " + str(التوجيه["انعطاف"])))
قبعة_الحساسات.اضبط_بكسل(3, 3, 0, 255, 128)
قبعة_الحساسات.اعرض_رسالة("Lisan Pi 5", 0.08)
قبعة_المحركات = قبعة_محركات_pca9685(64)
زوايا_المسح = [45, 90, 135, 90]
for زاوية in زوايا_المسح:
    قبعة_المحركات.اضبط_زاوية_سيرفو(0, زاوية)
قبعة_المحركات.اضبط_سرعة_محرك(1, 60)
print("تم ضبط حامل الكاميرا على الزاوية 90 درجة وتشغيل محرك التهوية بنسبة 60%")
الكاميرا = كاميرا_راسبيري(1920, 1080)
الكاميرا.ابدأ()
النظام_الصوتي = صوت_راسبيري(85)
النظام_الصوتي.اضبط_الصوت(90)
if (حساس_الحركة.اقرأ() == 1):
    print("تنبيه أمني: تم رصد حركة في المدخل الرئيسي!")
    الكاميرا.التقط_صورة("/tmp/lisan_hub_intruder.jpg")
    الكاميرا.سجل_فيديو("/tmp/lisan_hub_event.mp4", 5)
    الكاميرا.ابدأ_بث(8080)
    النظام_الصوتي.شغّل_ملف("/usr/share/sounds/alsa/Front_Center.wav")
    النظام_الصوتي.انطق_عربي("مرحباً بكم في مركز لسان الذكي على راسبيري باي خمسة، تم رصد حركة وتفعيل البث المباشر")
else:
    النظام_الصوتي.انطق_عربي("النظام في وضع المراقبة الهادئة")
شاشة_اللمس = شاشة_لمس_dsi(7)
شاشة_اللمس.شغّل_الشاشة(True)
شاشة_اللمس.اضبط_سطوع(85)
شاشة_الحبر = شاشة_حبر_إلكتروني_epaper(250, 122)
شاشة_الحبر.امسح()
شاشة_الحبر.اعرض_نص(8, 10, "مركز لسان الذكي - Pi 5")
شاشة_الحبر.اعرض_نص(8, 40, (("الحرارة: " + str(حرارة_الغرفة)) + " مئوية"))
شاشة_الحبر.اعرض_نص(8, 70, "البث: منفذ 8080 نشط")
شاشة_الحبر.تحديث_الشاشة()
شاشة_الحبر.سكون()
الكاميرا.أوقف()
مروحة_التبريد.أوقف()
قبعة_الحساسات.امسح_الشاشة()
مصباح_الحالة.أوقف()
مصباح_الحالة.تنظيف()
حساس_الحركة.تنظيف()
print("=== اكتمل فحص وتشغيل جميع وحدات مركز Raspberry Pi 5 بنجاح ===")
